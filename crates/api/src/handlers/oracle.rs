//! Oracle price feed handlers

use crate::error::{ApiError, handle_db_error};
use crate::models::*;
use crate::state::AppState;
use actix_web::{web, HttpRequest, HttpResponse};
use chrono::Utc;
use ethers::types::Address;
use meridian_db::{InsertPriceRequest, PriceRepository};
use rust_decimal::Decimal;
use sha2::{Sha256, Digest};
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::{Arc, OnceLock};

/// Get all current prices
///
/// GET /api/v1/oracle/prices
#[utoipa::path(
    get,
    path = "/api/v1/oracle/prices",
    tag = "oracle",
    responses(
        (status = 200, description = "All current prices", body = PricesResponse),
        (status = 503, description = "Oracle not configured")
    )
)]
pub async fn get_prices(state: web::Data<Arc<AppState>>) -> Result<HttpResponse, ApiError> {
    tracing::debug!("Fetching all oracle prices");

    let oracle_guard = state.oracle.read().await;
    let oracle = oracle_guard.as_ref().ok_or(ApiError::OracleNotConfigured)?;

    // Get all registered feeds
    let feeds = oracle.list_feeds().await;
    let mut prices_map = HashMap::new();

    for pair in feeds {
        match oracle.get_feed_info(&pair).await {
            Ok(feed) => {
                prices_map.insert(
                    pair.clone(),
                    PriceData {
                        price_usd: feed.latest_price,
                        is_stale: feed.is_stale,
                        updated_at: feed.updated_at.to_rfc3339(),
                    },
                );
            }
            Err(e) => {
                tracing::warn!(pair = %pair, error = %e, "Failed to get price");
            }
        }
    }

    Ok(HttpResponse::Ok().json(PricesResponse { prices: prices_map }))
}

/// Get price for a specific currency pair
///
/// GET /api/v1/oracle/prices/{pair}
#[utoipa::path(
    get,
    path = "/api/v1/oracle/prices/{pair}",
    tag = "oracle",
    params(
        ("pair" = String, Path, description = "Currency pair (e.g., EUR/USD)")
    ),
    responses(
        (status = 200, description = "Price for the pair", body = PriceResponse),
        (status = 404, description = "Price feed not found"),
        (status = 503, description = "Oracle not configured")
    )
)]
pub async fn get_price(
    state: web::Data<Arc<AppState>>,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let pair = path.into_inner();

    tracing::debug!(pair = %pair, "Fetching price");

    let oracle_guard = state.oracle.read().await;
    let oracle = oracle_guard.as_ref().ok_or(ApiError::OracleNotConfigured)?;

    let feed = oracle.get_feed_info(&pair).await?;

    let response = PriceResponse {
        pair: feed.pair,
        price_usd: feed.latest_price,
        is_stale: feed.is_stale,
        updated_at: feed.updated_at.to_rfc3339(),
    };

    Ok(HttpResponse::Ok().json(response))
}

/// Update price for a specific currency pair
///
/// POST /api/v1/oracle/prices/{pair}/update
/// CRIT-008: Requires admin role (not just authentication)
#[utoipa::path(
    post,
    path = "/api/v1/oracle/prices/{pair}/update",
    tag = "oracle",
    security(("bearer_auth" = [])),
    params(
        ("pair" = String, Path, description = "Currency pair to update")
    ),
    responses(
        (status = 200, description = "Price updated", body = PriceResponse),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Admin role required"),
        (status = 503, description = "Oracle not configured")
    )
)]
pub async fn update_price(
    state: web::Data<Arc<AppState>>,
    http_req: HttpRequest,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    // CRIT-008: Verify user is authenticated AND has admin role
    let user = get_authenticated_user_with_role(state.db_pool.as_ref(), &http_req).await?;
    // HIGH-012: Case-insensitive role check
    if user.role.to_uppercase() != "ADMIN" {
        tracing::warn!(user_id = user.id, role = %user.role, "Unauthorized price update attempt");
        return Err(ApiError::Forbidden("Admin role required to update prices".to_string()));
    }

    let pair = path.into_inner();

    tracing::info!(pair = %pair, "Updating price from blockchain");

    let oracle_guard = state.oracle.read().await;
    let oracle = oracle_guard.as_ref().ok_or(ApiError::OracleNotConfigured)?;

    let price = oracle.update_price(&pair).await?;

    let feed = oracle.get_feed_info(&pair).await?;

    // Persist price to database
    let price_repo = PriceRepository::new((*state.db_pool).clone());
    
    // Convert round_id safely - skip if conversion would overflow
    let round_id = if feed.latest_round.bits() <= 64 {
        Some(Decimal::from(feed.latest_round.as_u64()))
    } else {
        tracing::warn!(pair = %pair, "Round ID too large for Decimal, skipping");
        None
    };
    
    let insert_request = InsertPriceRequest {
        currency_pair: pair.clone(),
        price,
        source: "chainlink".to_string(),
        is_stale: feed.is_stale,
        round_id,
    };
    price_repo.insert(insert_request).await.map_err(|e| {
        tracing::error!("Failed to persist price: {}", e);
        ApiError::InternalError("Failed to persist price".to_string())
    })?;

    tracing::info!(pair = %pair, price = %price, "Price updated and persisted to database");

    let response = PriceResponse {
        pair: feed.pair,
        price_usd: price,
        is_stale: feed.is_stale,
        updated_at: Utc::now().to_rfc3339(),
    };

    Ok(HttpResponse::Ok().json(response))
}

/// Register a new price feed
///
/// POST /api/v1/oracle/feeds
/// MED-003: Requires admin role
#[utoipa::path(
    post,
    path = "/api/v1/oracle/feeds",
    tag = "oracle",
    security(("bearer_auth" = [])),
    request_body = RegisterFeedRequest,
    responses(
        (status = 201, description = "Price feed registered"),
        (status = 400, description = "Invalid address"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Admin role required"),
        (status = 503, description = "Oracle not configured")
    )
)]
pub async fn register_price_feed(
    state: web::Data<Arc<AppState>>,
    http_req: HttpRequest,
    req: web::Json<RegisterFeedRequest>,
) -> Result<HttpResponse, ApiError> {
    // MED-003: Verify user is authenticated AND has admin role
    let user = get_authenticated_user_with_role(state.db_pool.as_ref(), &http_req).await?;
    // HIGH-012: Case-insensitive role check
    if user.role.to_uppercase() != "ADMIN" {
        tracing::warn!(user_id = user.id, role = %user.role, "Unauthorized price feed registration attempt");
        return Err(ApiError::Forbidden("Admin role required to register price feeds".to_string()));
    }

    tracing::info!(
        pair = %req.pair,
        address = %req.chainlink_address,
        admin_id = user.id,
        "Registering price feed"
    );

    let oracle_guard = state.oracle.read().await;
    let oracle = oracle_guard.as_ref().ok_or(ApiError::OracleNotConfigured)?;

    let address = Address::from_str(&req.chainlink_address)
        .map_err(|e| ApiError::BadRequest(format!("Invalid address: {}", e)))?;

    oracle.register_price_feed(&req.pair, address).await?;

    Ok(HttpResponse::Created().json(serde_json::json!({
        "success": true,
        "pair": req.pair,
        "address": req.chainlink_address
    })))
}

/// User info returned from authentication
struct AuthenticatedUser {
    id: i32,
    role: String,
}

/// Extract authenticated user ID from request token
/// MED-002: Helper function for authentication checks
#[allow(dead_code)]
async fn get_authenticated_user_id(
    pool: &sqlx::PgPool,
    req: &HttpRequest,
) -> Result<i32, ApiError> {
    let user = get_authenticated_user_with_role(pool, req).await?;
    Ok(user.id)
}

/// Extract authenticated user with role from request token
/// MED-003: Helper function for role-based access control
/// CRIT-001 FIX: Added SESSION_TOKEN_SALT for consistent token hashing
async fn get_authenticated_user_with_role(
    pool: &sqlx::PgPool,
    req: &HttpRequest,
) -> Result<AuthenticatedUser, ApiError> {
    let token = req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .ok_or_else(|| ApiError::Unauthorized("Missing Authorization header".to_string()))?;

    // CRIT-001 FIX: Use salted token hashing consistent with auth.rs
    static TOKEN_SALT: OnceLock<String> = OnceLock::new();

    let salt = TOKEN_SALT.get_or_init(|| {
        std::env::var("SESSION_TOKEN_SALT").unwrap_or_else(|_| {
            // In development, use a default salt with warning
            if std::env::var("ENVIRONMENT")
                .map(|e| e.to_lowercase() == "production")
                .unwrap_or(false)
            {
                // Production MUST have salt configured - panic to prevent insecure operation
                panic!("SESSION_TOKEN_SALT must be set in production environment");
            }
            tracing::warn!("Using default session token salt - set SESSION_TOKEN_SALT in production");
            "dev-session-salt-not-for-production".to_string()
        })
    });

    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    hasher.update(salt.as_bytes());
    let token_hash = hex::encode(hasher.finalize());

    let session = sqlx::query!(
        r#"
        SELECT s.user_id, u.role
        FROM sessions s
        JOIN users u ON s.user_id = u.id
        WHERE s.access_token = $1 AND s.expires_at > NOW()
        "#,
        token_hash
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| handle_db_error(e, "oracle"))?;

    match session {
        Some(s) => Ok(AuthenticatedUser {
            id: s.user_id,
            role: s.role,
        }),
        None => Err(ApiError::Unauthorized("Invalid or expired token".to_string())),
    }
}
