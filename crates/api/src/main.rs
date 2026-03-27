//! # Meridian REST API Server
//!
//! HTTP API for managing multi-currency stablecoins

mod openapi;

use actix_cors::Cors;
use actix_governor::{Governor, GovernorConfigBuilder};
use actix_web::{middleware::{DefaultHeaders, Logger}, web, App, HttpServer};
use ethers::types::U256;
use meridian_api::{metrics, routes, state::AppState, telemetry, CorrelationIdMiddleware, RateLimitHeadersMiddleware};
use meridian_chains::execution::spawn_confirmation_worker;
use meridian_db::{create_pool, run_migrations};
use openapi::ApiDoc;
use rust_decimal::Decimal;
use std::sync::Arc;
use std::time::Duration;
use tokio::task::JoinHandle;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // CRIT-003: Initialize telemetry with OpenTelemetry support
    let telemetry_config = telemetry::TelemetryConfig::from_env();
    telemetry::init_telemetry(telemetry_config);

    // H.3: Register Prometheus business metrics
    metrics::init_metrics();

    tracing::info!("Starting Meridian API server...");

    // Get configuration from environment
    let host = std::env::var("MERIDIAN_API_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = std::env::var("MERIDIAN_API_PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()
        .expect("Invalid port number");

    // Get database URL from environment
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    // Initialize database connection pool
    tracing::info!("Connecting to database...");
    let db_pool = create_pool(&database_url)
        .await
        .expect("Failed to create database pool");

    // Run database migrations
    tracing::info!("Running database migrations...");
    run_migrations(&db_pool)
        .await
        .expect("Failed to run migrations");

    tracing::info!("Database initialized");

    // Initialize shared application state
    let app_state = Arc::new(AppState::new(db_pool).await);

    tracing::info!("Application state initialized");

    // A.4 + H.4: Spawn background services — store JoinHandles for graceful shutdown
    let mut background_tasks: Vec<JoinHandle<()>> = Vec::new();

    // 1. On-chain confirmation worker (polls PENDING ops and updates to COMPLETED/FAILED)
    if let Some(ref executor) = app_state.evm_executor {
        let handle = spawn_confirmation_worker(
            executor.clone(),
            app_state.db_pool.clone(),
            Duration::from_secs(15),
        );
        background_tasks.push(handle);
        tracing::info!("Confirmation worker spawned (poll interval: 15s)");
    } else {
        tracing::info!("Confirmation worker skipped — EVM executor not configured");
    }

    // 2. Proof of Reserves attestation (every 6h)
    {
        let custody = app_state.custody.clone();
        let executor = app_state.evm_executor.clone();
        let handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(6 * 3600));
            loop {
                interval.tick().await;
                match custody.get_total_value_usd().await {
                    Ok(total_usd) => {
                        tracing::info!(total_usd = %total_usd, "PoR attestation: custody total retrieved");
                        // H.3: Update custody balance metric
                        metrics::set_custody_balance("total", total_usd.to_string().parse::<f64>().unwrap_or(0.0));
                        if let Some(ref exec) = executor {
                            let value_units = (total_usd * Decimal::from(100))
                                .to_string()
                                .parse::<u128>()
                                .unwrap_or(0);
                            match exec.attest_reserves_on_chain(U256::from(value_units)).await {
                                Ok(tx) => tracing::info!(tx_hash = ?tx.tx_hash, "PoR attestation submitted on-chain"),
                                Err(e) => tracing::warn!(error = %e, "PoR attestation submission failed"),
                            }
                        }
                    }
                    Err(e) => tracing::warn!(error = %e, "PoR attestation: custody query failed"),
                }
            }
        });
        background_tasks.push(handle);
        tracing::info!("PoR attestation worker spawned (interval: 6h)");
    }

    // 3. Session cleanup (every 1h — remove expired sessions)
    {
        let db_pool = app_state.db_pool.clone();
        let handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(3600));
            loop {
                interval.tick().await;
                match sqlx::query("DELETE FROM sessions WHERE expires_at < NOW()")
                    .execute(db_pool.as_ref())
                    .await
                {
                    Ok(r) => {
                        if r.rows_affected() > 0 {
                            tracing::info!(expired = r.rows_affected(), "Session cleanup: expired sessions removed");
                        }
                    }
                    Err(e) => tracing::warn!(error = %e, "Session cleanup query failed"),
                }
            }
        });
        background_tasks.push(handle);
        tracing::info!("Session cleanup worker spawned (interval: 1h)");
    }

    tracing::info!("Server starting at http://{}:{}", host, port);

    // Get CORS allowed origins from environment
    let cors_origins = std::env::var("CORS_ALLOWED_ORIGINS")
        .unwrap_or_else(|_| "http://localhost:3000".to_string());

    // Security: Validate CORS origins - reject wildcards in production
    let is_production = std::env::var("ENVIRONMENT")
        .map(|e| e.to_lowercase() == "production")
        .unwrap_or(false);

    if is_production && cors_origins.contains('*') {
        panic!("SECURITY: Wildcard CORS origins (*) are not allowed in production");
    }

    // Validate required production environment variables at startup
    // CRIT-002 & CRIT-004: Validate salts at startup, not on first use
    if is_production {
        // Validate API_KEY_SALT
        match std::env::var("API_KEY_SALT") {
            Ok(salt) if salt.len() < 32 => {
                panic!("SECURITY: API_KEY_SALT must be at least 32 bytes, got {} bytes", salt.len());
            }
            Err(_) => {
                panic!("SECURITY: API_KEY_SALT must be set in production");
            }
            Ok(_) => {}
        }

        // CRIT-002: Validate SESSION_TOKEN_SALT at startup (not on first token hash)
        // CRIT-004: Require minimum 32-byte length for cryptographic security
        match std::env::var("SESSION_TOKEN_SALT") {
            Ok(salt) if salt.len() < 32 => {
                panic!("SECURITY: SESSION_TOKEN_SALT must be at least 32 bytes, got {} bytes", salt.len());
            }
            Err(_) => {
                panic!("SECURITY: SESSION_TOKEN_SALT must be set in production");
            }
            Ok(_) => {}
        }

        if std::env::var("WALLET_SERVICE_URL").is_err() {
            tracing::warn!(
                "WALLET_SERVICE_URL not set - agent wallet creation will fail in production"
            );
        }

        // COMPLIANCE-001: Validate compliance cannot be disabled in production
        if std::env::var("COMPLIANCE_ENABLED")
            .map(|v| v.to_lowercase() == "false")
            .unwrap_or(false)
        {
            panic!("SECURITY: COMPLIANCE_ENABLED=false is not permitted in production. \
                Compliance screening is required for all mint/burn operations.");
        }

        tracing::info!("Production security checks passed (API_KEY_SALT, SESSION_TOKEN_SALT, COMPLIANCE validated)");
    }

    tracing::info!("CORS allowed origins: {}", cors_origins);

    // Configure rate limiting: ~100 requests per minute per IP
    // per_second(2) = 2 tokens/sec = 120/min, burst_size(10) = max burst
    let governor_config = GovernorConfigBuilder::default()
        .per_second(2)
        .burst_size(10)
        .finish()
        .expect("Failed to build rate limiter config");

    tracing::info!("Rate limiting enabled: ~100 requests/minute per IP");

    // Configure request size limits
    let json_limit = std::env::var("MAX_JSON_PAYLOAD_SIZE")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(256 * 1024); // Default 256KB

    tracing::info!("JSON payload limit: {} bytes", json_limit);

    // HIGH-009: Configure HTTP request timeout to prevent hanging requests
    let request_timeout_secs = std::env::var("HTTP_REQUEST_TIMEOUT_SECS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(60u64); // Default 60 seconds

    tracing::info!("HTTP request timeout: {} seconds", request_timeout_secs);

    // H.3: Expose /metrics endpoint — capture db_pool ref before moving into closure
    let metrics_db_pool = app_state.db_pool.clone();

    // Start HTTP server
    let server = HttpServer::new(move || {
        // H.3: Clone for use in metrics handler
        let _metrics_pool = metrics_db_pool.clone();
        let mut cors = Cors::default()
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
            .allowed_headers(vec![
                actix_web::http::header::AUTHORIZATION,
                actix_web::http::header::CONTENT_TYPE,
                actix_web::http::header::ACCEPT,
                "X-Correlation-ID".parse().unwrap(),
                "X-Request-ID".parse().unwrap(),
            ])
            .expose_headers(vec![
                actix_web::http::header::HeaderName::from_static("x-correlation-id"),
            ])
            .max_age(3600);

        // Add allowed origins from environment
        for origin in cors_origins.split(',') {
            let origin = origin.trim();
            if !origin.is_empty() {
                cors = cors.allowed_origin(origin);
            }
        }

        // Configure JSON payload limit
        let json_cfg = web::JsonConfig::default()
            .limit(json_limit)
            .error_handler(|err, _req| {
                actix_web::error::InternalError::from_response(
                    err,
                    actix_web::HttpResponse::PayloadTooLarge()
                        .json(serde_json::json!({
                            "error": "Payload too large"
                        })),
                )
                .into()
            });

        // Security headers middleware
        let security_headers = DefaultHeaders::new()
            .add(("X-Content-Type-Options", "nosniff"))
            .add(("X-Frame-Options", "DENY"))
            .add(("X-XSS-Protection", "1; mode=block"))
            .add(("Referrer-Policy", "strict-origin-when-cross-origin"))
            .add(("Permissions-Policy", "geolocation=(), camera=(), microphone=()"));

        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .app_data(json_cfg)
            .wrap(security_headers)
            // HIGH-010: Add rate limit headers (X-RateLimit-Limit, X-RateLimit-Remaining, X-RateLimit-Reset)
            .wrap(RateLimitHeadersMiddleware::new())
            .wrap(CorrelationIdMiddleware::new())
            .wrap(Governor::new(&governor_config))
            .wrap(Logger::default())
            .wrap(cors)
            // CRIT-002: OpenAPI documentation and Swagger UI
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}")
                    .url("/api-docs/openapi.json", ApiDoc::openapi())
            )
            .configure(routes::configure)
            // H.3: Prometheus metrics scrape endpoint
            .route("/metrics", web::get().to(|| async {
                actix_web::HttpResponse::Ok()
                    .content_type("text/plain; version=0.0.4")
                    .body(telemetry::prometheus_metrics())
            }))
    })
    .bind((host.as_str(), port))?
    // HIGH-009: HTTP timeouts to prevent hanging requests
    .client_request_timeout(Duration::from_secs(request_timeout_secs))
    .client_disconnect_timeout(Duration::from_secs(5))
    .keep_alive(Duration::from_secs(75)) // Keep-alive slightly longer than client timeout
    .run();

    let server_handle = server.handle();

    // H.4: Graceful shutdown — listen for SIGTERM or CTRL-C
    let shutdown_handle = server_handle.clone();
    tokio::spawn(async move {
        #[cfg(unix)]
        {
            use tokio::signal::unix::{signal, SignalKind};
            let mut sigterm = signal(SignalKind::terminate())
                .expect("Failed to install SIGTERM handler");
            tokio::select! {
                _ = sigterm.recv() => tracing::info!("SIGTERM received — initiating graceful shutdown"),
                _ = tokio::signal::ctrl_c() => tracing::info!("CTRL-C received — initiating graceful shutdown"),
            }
        }
        #[cfg(not(unix))]
        {
            tokio::signal::ctrl_c().await.ok();
            tracing::info!("CTRL-C received — initiating graceful shutdown");
        }

        // Stop accepting new requests; wait for in-flight requests to complete
        shutdown_handle.stop(true).await;
    });

    // Run until server stops
    server.await?;

    // H.4: Post-shutdown cleanup
    tracing::info!("HTTP server stopped — aborting background tasks");
    for task in background_tasks {
        task.abort();
    }

    tracing::info!("Flushing telemetry...");
    telemetry::shutdown_telemetry();

    tracing::info!("Shutdown complete");
    Ok(())
}
