//! # Meridian REST API Server
//!
//! HTTP API for managing multi-currency stablecoins

mod middleware;
mod openapi;

use actix_cors::Cors;
use actix_governor::{Governor, GovernorConfigBuilder};
use actix_web::{middleware::{DefaultHeaders, Logger}, web, App, HttpServer};
use meridian_api::{routes, state::AppState, telemetry};
use middleware::{CorrelationIdMiddleware, RateLimitHeadersMiddleware};
use meridian_db::{create_pool, run_migrations};
use openapi::ApiDoc;
use std::sync::Arc;
use std::time::Duration;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // CRIT-003: Initialize telemetry with OpenTelemetry support
    let telemetry_config = telemetry::TelemetryConfig::from_env();
    telemetry::init_telemetry(telemetry_config);

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
        tracing::info!("Production security checks passed (API_KEY_SALT, SESSION_TOKEN_SALT validated)");
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

    // Start HTTP server
    HttpServer::new(move || {
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
    })
    .bind((host.as_str(), port))?
    // HIGH-009: HTTP timeouts to prevent hanging requests
    .client_request_timeout(Duration::from_secs(request_timeout_secs))
    .client_disconnect_timeout(Duration::from_secs(5))
    .keep_alive(Duration::from_secs(75)) // Keep-alive slightly longer than client timeout
    .run()
    .await
}
