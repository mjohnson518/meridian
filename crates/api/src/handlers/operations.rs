//! Mint/Burn operation handlers

use crate::error::{ApiError, handle_db_error};
use crate::state::AppState;
use actix_web::{web, HttpRequest, HttpResponse};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use std::str::FromStr;
use std::sync::Arc;

#[derive(Debug, Deserialize)]
pub struct MintRequest {
    pub user_id: i32,
    pub currency: String,
    pub amount: String, // TEXT decimal
}

#[derive(Debug, Serialize)]
pub struct MintResponse {
    pub transaction_id: i32,
    pub currency: String,
    pub amount: String,
    pub usd_value: String,
    pub bond_requirement: String,
    pub fees_charged: String,
    pub settlement_date: String,
    pub status: String,
}

#[derive(Debug, Serialize)]
pub struct TransactionResponse {
    pub id: i32,
    pub operation_type: String,
    pub currency: String,
    pub amount: String,
    pub usd_value: String,
    pub status: String,
    pub transaction_hash: Option<String>,
    pub created_at: String,
    pub settlement_date: Option<String>,
}

const FEE_ISSUANCE_BPS: i64 = 25; // 25 basis points
const FEE_REDEMPTION_BPS: i64 = 25;
const RESERVE_BUFFER_PERCENT: i64 = 2; // 2% over-collateralization

/// Supported currency codes (ISO 4217)
/// Only these currencies can be minted/burned on the platform
const SUPPORTED_CURRENCIES: &[&str] = &["EUR", "GBP", "JPY", "MXN", "BRL", "ARS"];

/// Validate currency code against whitelist
fn validate_currency(currency: &str) -> Result<(), ApiError> {
    let normalized = currency.to_uppercase();
    if !SUPPORTED_CURRENCIES.contains(&normalized.as_str()) {
        return Err(ApiError::BadRequest(format!(
            "Unsupported currency: {}. Supported: {}",
            currency,
            SUPPORTED_CURRENCIES.join(", ")
        )));
    }
    Ok(())
}

/// POST /api/v1/operations/mint
pub async fn mint(
    state: web::Data<Arc<AppState>>,
    http_req: HttpRequest,
    req: web::Json<MintRequest>,
) -> Result<HttpResponse, ApiError> {
    // SECURITY: Verify authenticated user matches the user_id in request
    let auth_user_id = get_authenticated_user_id(state.db_pool.as_ref(), &http_req).await?;
    if auth_user_id != req.user_id {
        tracing::warn!(
            auth_user_id = auth_user_id,
            requested_user_id = req.user_id,
            "Mint request rejected: user_id mismatch"
        );
        return Err(ApiError::Forbidden("Cannot mint for another user".to_string()));
    }

    // Validate currency is on the supported whitelist
    validate_currency(&req.currency)?;

    tracing::info!(
        user_id = req.user_id,
        currency = %req.currency,
        amount = %req.amount,
        "Mint request received"
    );

    // Verify user is KYC approved
    let user = sqlx::query!("SELECT kyc_status FROM users WHERE id = $1", req.user_id)
        .fetch_optional(state.db_pool.as_ref())
        .await
        .map_err(|e| handle_db_error(e, "operations"))?;

    let user = match user {
        Some(u) => u,
        None => return Err(ApiError::NotFound("User not found".to_string())),
    };

    if user.kyc_status != "APPROVED" {
        return Err(ApiError::Forbidden(
            "KYC approval required for mint operations".to_string(),
        ));
    }

    // Parse amount
    let amount_decimal = Decimal::from_str(&req.amount)
        .map_err(|_| ApiError::BadRequest("Invalid amount format".to_string()))?;

    // Get FX rate (from oracle or fallback)
    let fx_rate = get_fx_rate(&state, &req.currency).await?;
    let usd_value = amount_decimal / fx_rate;

    // Calculate fees and requirements
    let fees = (usd_value * Decimal::from(FEE_ISSUANCE_BPS)) / Decimal::from(10000);
    let bond_requirement = usd_value * (Decimal::from(100 + RESERVE_BUFFER_PERCENT)) / Decimal::from(100);

    // Calculate settlement date (T+1)
    let settlement_date = chrono::Utc::now() + chrono::Duration::days(1);

    // Insert operation
    let operation = sqlx::query!(
        r#"
        INSERT INTO operations (
            user_id, operation_type, currency, amount, usd_value, 
            bond_requirement, fees_charged, status, settlement_date
        )
        VALUES ($1, 'MINT', $2, $3, $4, $5, $6, 'PENDING', $7)
        RETURNING id, status, created_at
        "#,
        req.user_id,
        req.currency,
        req.amount,
        usd_value.to_string(),
        bond_requirement.to_string(),
        fees.to_string(),
        settlement_date
    )
    .fetch_one(state.db_pool.as_ref())
    .await
    .map_err(|e| {
        tracing::error!("Failed to create mint operation: {}", e);
        ApiError::InternalError("Failed to create mint operation".to_string())
    })?;

    tracing::info!(
        transaction_id = operation.id,
        usd_value = %usd_value,
        "Mint operation created"
    );

    Ok(HttpResponse::Created().json(MintResponse {
        transaction_id: operation.id,
        currency: req.currency.clone(),
        amount: req.amount.clone(),
        usd_value: usd_value.to_string(),
        bond_requirement: bond_requirement.to_string(),
        fees_charged: fees.to_string(),
        settlement_date: settlement_date.to_rfc3339(),
        status: operation.status,
    }))
}

/// POST /api/v1/operations/burn
pub async fn burn(
    state: web::Data<Arc<AppState>>,
    http_req: HttpRequest,
    req: web::Json<MintRequest>, // Same structure as mint
) -> Result<HttpResponse, ApiError> {
    // SECURITY: Verify authenticated user matches the user_id in request
    let auth_user_id = get_authenticated_user_id(state.db_pool.as_ref(), &http_req).await?;
    if auth_user_id != req.user_id {
        tracing::warn!(
            auth_user_id = auth_user_id,
            requested_user_id = req.user_id,
            "Burn request rejected: user_id mismatch"
        );
        return Err(ApiError::Forbidden("Cannot burn for another user".to_string()));
    }

    // Validate currency is on the supported whitelist
    validate_currency(&req.currency)?;

    tracing::info!(
        user_id = req.user_id,
        currency = %req.currency,
        amount = %req.amount,
        "Burn request received"
    );

    // Verify KYC
    let user = sqlx::query!("SELECT kyc_status FROM users WHERE id = $1", req.user_id)
        .fetch_optional(state.db_pool.as_ref())
        .await
        .map_err(|e| handle_db_error(e, "operations"))?;

    let user = match user {
        Some(u) => u,
        None => return Err(ApiError::NotFound("User not found".to_string())),
    };

    if user.kyc_status != "APPROVED" {
        return Err(ApiError::Forbidden(
            "KYC approval required for burn operations".to_string(),
        ));
    }

    // Parse amount
    let amount_decimal = Decimal::from_str(&req.amount)
        .map_err(|_| ApiError::BadRequest("Invalid amount format".to_string()))?;

    // Get FX rate
    let fx_rate = get_fx_rate(&state, &req.currency).await?;
    let usd_value = amount_decimal / fx_rate;

    // Calculate redemption fee
    let fees = (usd_value * Decimal::from(FEE_REDEMPTION_BPS)) / Decimal::from(10000);
    let net_proceeds = usd_value - fees;

    // Settlement date
    let settlement_date = chrono::Utc::now() + chrono::Duration::days(2); // T+2 for bond sales

    // Insert burn operation
    let operation = sqlx::query!(
        r#"
        INSERT INTO operations (
            user_id, operation_type, currency, amount, usd_value, 
            fees_charged, status, settlement_date
        )
        VALUES ($1, 'BURN', $2, $3, $4, $5, 'PENDING', $6)
        RETURNING id, status, created_at
        "#,
        req.user_id,
        req.currency,
        req.amount,
        net_proceeds.to_string(),
        fees.to_string(),
        settlement_date
    )
    .fetch_one(state.db_pool.as_ref())
    .await
    .map_err(|e| {
        tracing::error!("Failed to create burn operation: {}", e);
        ApiError::InternalError("Failed to create burn operation".to_string())
    })?;

    tracing::info!(
        transaction_id = operation.id,
        net_proceeds = %net_proceeds,
        "Burn operation created"
    );

    Ok(HttpResponse::Created().json(serde_json::json!({
        "transaction_id": operation.id,
        "currency": req.currency,
        "amount_burned": req.amount,
        "usd_value": usd_value.to_string(),
        "fees_charged": fees.to_string(),
        "net_proceeds": net_proceeds.to_string(),
        "settlement_date": settlement_date.to_rfc3339(),
        "status": operation.status
    })))
}

/// GET /api/v1/operations/transactions/{user_id}
pub async fn get_transactions(
    state: web::Data<Arc<AppState>>,
    req: HttpRequest,
    user_id: web::Path<i32>,
) -> Result<HttpResponse, ApiError> {
    let user_id = user_id.into_inner();

    // Verify authenticated user matches requested user_id
    let auth_user_id = get_authenticated_user_id(state.db_pool.as_ref(), &req).await?;
    if auth_user_id != user_id {
        return Err(ApiError::Forbidden("Cannot access other user's transactions".to_string()));
    }

    let transactions = sqlx::query!(
        r#"
        SELECT id, operation_type, currency, amount, usd_value, status, 
               transaction_hash, created_at, settlement_date
        FROM operations
        WHERE user_id = $1
        ORDER BY created_at DESC
        LIMIT 100
        "#,
        user_id
    )
    .fetch_all(state.db_pool.as_ref())
    .await
    .map_err(|e| handle_db_error(e, "operations"))?;

    let responses: Vec<TransactionResponse> = transactions
        .into_iter()
        .map(|tx| TransactionResponse {
            id: tx.id,
            operation_type: tx.operation_type,
            currency: tx.currency,
            amount: tx.amount,
            usd_value: tx.usd_value,
            status: tx.status,
            transaction_hash: tx.transaction_hash,
            created_at: tx.created_at.to_rfc3339(),
            settlement_date: tx.settlement_date.map(|dt| dt.to_rfc3339()),
        })
        .collect();

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "transactions": responses,
        "count": responses.len()
    })))
}

// Helper function to get FX rates (simplified - should use oracle)
// Helper function to get FX rates (uses Oracle with fallback)
async fn get_fx_rate(
    state: &Arc<AppState>,
    currency: &str,
) -> Result<Decimal, ApiError> {
    // 1. Try to get authentic price from Oracle
    let oracle_guard = state.oracle.read().await;
    
    if let Some(oracle) = oracle_guard.as_ref() {
        let pair = format!("{}/USD", currency);
        match oracle.get_price(&pair).await {
            Ok(price) => return Ok(price),
            Err(e) => {
                tracing::warn!("Oracle failed for {}: {}, falling back to static rates", pair, e);
            }
        }
    } else {
        tracing::debug!("Oracle not configured, using static rates for {}", currency);
    }

    // 2. Fallback to hardcoded rates (for dev or if oracle fails)
    // SECURITY: These rates are potentially stale and should not be used in production
    let is_production = std::env::var("ENVIRONMENT")
        .map(|e| e.to_lowercase() == "production")
        .unwrap_or(false);

    if is_production {
        tracing::error!(
            currency = currency,
            "CRITICAL: Using fallback FX rates in production! Oracle is unavailable."
        );
        // In production, STRICT_FX_RATES defaults to TRUE for safety
        // Only disable if explicitly set to "false" (dangerous - requires explicit opt-out)
        let strict_mode = std::env::var("STRICT_FX_RATES")
            .map(|v| v.to_lowercase() != "false")
            .unwrap_or(true); // Default to strict in production

        if strict_mode {
            return Err(ApiError::InternalError(
                "FX rate oracle unavailable. Cannot use fallback rates in production. \
                Set STRICT_FX_RATES=false to allow stale rates (NOT RECOMMENDED).".to_string()
            ));
        }
        tracing::warn!("STRICT_FX_RATES=false - allowing stale fallback rates in production (DANGEROUS)");
    }

    tracing::warn!(
        currency = currency,
        "Using FALLBACK FX rates - these may be stale. Last updated: 2024-01-01"
    );

    let rate = match currency {
        "EUR" => "1.09",
        "GBP" => "1.22",
        "JPY" => "0.0067",
        "MXN" => "0.058",
        "BRL" => "0.20",
        "ARS" => "0.0011",
        _ => return Err(ApiError::BadRequest(format!("Unsupported currency: {}", currency))),
    };

    Decimal::from_str(rate)
        .map_err(|_| ApiError::InternalError("Invalid FX rate".to_string()))
}

/// Extract authenticated user ID from request token
async fn get_authenticated_user_id(
    pool: &sqlx::PgPool,
    req: &HttpRequest,
) -> Result<i32, ApiError> {
    let token = req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .ok_or_else(|| ApiError::Unauthorized("Missing Authorization header".to_string()))?;

    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    let token_hash = hex::encode(hasher.finalize());

    let session = sqlx::query!(
        r#"
        SELECT user_id
        FROM sessions
        WHERE access_token = $1 AND expires_at > NOW()
        "#,
        token_hash
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| handle_db_error(e, "operations"))?;

    match session {
        Some(s) => Ok(s.user_id),
        None => Err(ApiError::Unauthorized("Invalid or expired token".to_string())),
    }
}

