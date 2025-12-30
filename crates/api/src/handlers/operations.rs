//! Mint/Burn operation handlers

use crate::error::{ApiError, handle_db_error};
use crate::state::AppState;
use actix_web::{web, HttpRequest, HttpResponse};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

/// CRIT-001: Retry configuration for oracle calls
const MAX_RETRIES: u32 = 3;
const INITIAL_BACKOFF_MS: u64 = 100;
const MAX_BACKOFF_MS: u64 = 2000;

/// CRIT-001: Generate random jitter (0.0 to 0.5) for backoff
/// Uses simple time-based pseudo-randomness to avoid adding rand crate dependency
fn rand_jitter() -> f64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or(0);
    // Convert to 0.0-0.5 range
    (nanos as f64 % 500.0) / 1000.0
}

/// CRIT-003: Idempotency key for preventing duplicate operations
/// Client must provide a unique key for each distinct operation
const IDEMPOTENCY_KEY_TTL_HOURS: i64 = 24;

#[derive(Debug, Deserialize)]
pub struct MintRequest {
    pub user_id: i32,
    pub currency: String,
    pub amount: String, // TEXT decimal
    /// CRIT-003: Unique idempotency key to prevent duplicate operations
    /// Must be unique per user+operation. Recommended: UUID v4
    pub idempotency_key: Option<String>,
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

// SECURITY: Amount validation bounds
// Max transaction: 10 billion units (prevents overflow and unrealistic requests)
const MAX_TRANSACTION_AMOUNT: &str = "10000000000";
// Min FX rate to prevent division issues (0.0000001)
const MIN_FX_RATE: &str = "0.0000001";

/// Validate amount is positive and within reasonable bounds
/// Returns Ok(()) if valid, Err(ApiError) if not
fn validate_amount(amount: &Decimal, context: &str) -> Result<(), ApiError> {
    // BACKEND-CRIT-001: Amount must be greater than zero
    if *amount <= Decimal::ZERO {
        tracing::warn!(amount = %amount, context = context, "Invalid amount: must be greater than zero");
        return Err(ApiError::BadRequest("Amount must be greater than zero".to_string()));
    }

    // BACKEND-HIGH-002: Amount must not exceed max (prevents overflow, unrealistic requests)
    let max_amount = Decimal::from_str(MAX_TRANSACTION_AMOUNT)
        .expect("MAX_TRANSACTION_AMOUNT is a valid constant");
    if *amount > max_amount {
        tracing::warn!(amount = %amount, max = %max_amount, context = context, "Amount exceeds maximum");
        return Err(ApiError::BadRequest(format!(
            "Amount exceeds maximum allowed: {}",
            MAX_TRANSACTION_AMOUNT
        )));
    }

    Ok(())
}

/// Validate FX rate is positive and reasonable
fn validate_fx_rate(rate: &Decimal, currency: &str) -> Result<(), ApiError> {
    // BACKEND-CRIT-003: FX rate must be greater than zero to prevent division errors
    let min_rate = Decimal::from_str(MIN_FX_RATE)
        .expect("MIN_FX_RATE is a valid constant");

    if *rate <= Decimal::ZERO {
        tracing::error!(rate = %rate, currency = currency, "Invalid FX rate: zero or negative");
        return Err(ApiError::InternalError("Invalid FX rate received from oracle".to_string()));
    }

    if *rate < min_rate {
        tracing::warn!(rate = %rate, currency = currency, min = %min_rate, "FX rate suspiciously low");
        return Err(ApiError::InternalError("FX rate below minimum threshold".to_string()));
    }

    Ok(())
}

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

/// CRIT-003: Idempotency key record for database row mapping
#[derive(sqlx::FromRow)]
struct IdempotencyRecord {
    id: i32,
    currency: String,
    amount: String,
    usd_value: String,
    bond_requirement: Option<String>,
    fees_charged: String,
    settlement_date: Option<chrono::DateTime<chrono::Utc>>,
    status: String,
}

/// CRIT-003: Check for existing operation with same idempotency key
/// Uses runtime query (query_as) to avoid compile-time DB dependency
async fn check_idempotency(
    pool: &sqlx::PgPool,
    user_id: i32,
    idempotency_key: &str,
    operation_type: &str,
) -> Result<Option<MintResponse>, ApiError> {
    let cutoff = chrono::Utc::now() - chrono::Duration::hours(IDEMPOTENCY_KEY_TTL_HOURS);

    let existing: Option<IdempotencyRecord> = sqlx::query_as(
        r#"
        SELECT id, currency, amount, usd_value, bond_requirement, fees_charged,
               settlement_date, status
        FROM operations
        WHERE user_id = $1
          AND idempotency_key = $2
          AND operation_type = $3
          AND created_at > $4
        "#
    )
    .bind(user_id)
    .bind(idempotency_key)
    .bind(operation_type)
    .bind(cutoff)
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        let err_str = e.to_string();
        // Graceful degradation if migration not applied
        if err_str.contains("idempotency_key") || err_str.contains("column") {
            tracing::warn!("Idempotency check failed - migration 20251230000001 required");
            return ApiError::InternalError(
                "Idempotency feature requires database migration".to_string()
            );
        }
        tracing::error!("Failed to check idempotency key: {}", e);
        ApiError::InternalError("Database error".to_string())
    })?;

    if let Some(op) = existing {
        tracing::info!(
            idempotency_key = idempotency_key,
            operation_id = op.id,
            "Returning cached result for idempotent request"
        );

        return Ok(Some(MintResponse {
            transaction_id: op.id,
            currency: op.currency,
            amount: op.amount,
            usd_value: op.usd_value,
            bond_requirement: op.bond_requirement.unwrap_or_default(),
            fees_charged: op.fees_charged,
            settlement_date: op.settlement_date.map(|d| d.to_rfc3339()).unwrap_or_default(),
            status: op.status,
        }));
    }

    Ok(None)
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

    // CRIT-003: Check idempotency key if provided
    if let Some(ref idem_key) = req.idempotency_key {
        if let Some(cached_response) = check_idempotency(
            state.db_pool.as_ref(),
            req.user_id,
            idem_key,
            "MINT",
        ).await? {
            return Ok(HttpResponse::Ok().json(cached_response));
        }
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

    // BACKEND-CRIT-001: Validate amount is positive and within bounds
    validate_amount(&amount_decimal, "mint")?;

    // Get FX rate (from oracle or fallback)
    let fx_rate = get_fx_rate(&state, &req.currency).await?;

    // BACKEND-CRIT-003: Validate FX rate before division
    validate_fx_rate(&fx_rate, &req.currency)?;

    let usd_value = amount_decimal / fx_rate;

    // Calculate fees and requirements
    let fees = (usd_value * Decimal::from(FEE_ISSUANCE_BPS)) / Decimal::from(10000);
    let bond_requirement = usd_value * (Decimal::from(100 + RESERVE_BUFFER_PERCENT)) / Decimal::from(100);

    // Calculate settlement date (T+1)
    let settlement_date = chrono::Utc::now() + chrono::Duration::days(1);

    // CRIT-003: Insert operation with idempotency key using runtime query
    #[derive(sqlx::FromRow)]
    struct InsertResult {
        id: i32,
        status: String,
    }

    let operation: InsertResult = sqlx::query_as(
        r#"
        INSERT INTO operations (
            user_id, operation_type, currency, amount, usd_value,
            bond_requirement, fees_charged, status, settlement_date, idempotency_key
        )
        VALUES ($1, 'MINT', $2, $3, $4, $5, $6, 'PENDING', $7, $8)
        RETURNING id, status
        "#
    )
    .bind(req.user_id)
    .bind(&req.currency)
    .bind(&req.amount)
    .bind(usd_value.to_string())
    .bind(bond_requirement.to_string())
    .bind(fees.to_string())
    .bind(settlement_date)
    .bind(&req.idempotency_key)
    .fetch_one(state.db_pool.as_ref())
    .await
    .map_err(|e| {
        let err_str = e.to_string();
        // Graceful degradation if migration not applied
        if err_str.contains("idempotency_key") {
            tracing::warn!("Insert failed due to missing column - migration 20251230000001 required");
        }
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

    // CRIT-003: Check idempotency key if provided
    if let Some(ref idem_key) = req.idempotency_key {
        if let Some(cached_response) = check_idempotency(
            state.db_pool.as_ref(),
            req.user_id,
            idem_key,
            "BURN",
        ).await? {
            return Ok(HttpResponse::Ok().json(cached_response));
        }
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

    // BACKEND-CRIT-001: Validate amount is positive and within bounds
    validate_amount(&amount_decimal, "burn")?;

    // Get FX rate
    let fx_rate = get_fx_rate(&state, &req.currency).await?;

    // BACKEND-CRIT-003: Validate FX rate before division
    validate_fx_rate(&fx_rate, &req.currency)?;

    let usd_value = amount_decimal / fx_rate;

    // Calculate redemption fee
    let fees = (usd_value * Decimal::from(FEE_REDEMPTION_BPS)) / Decimal::from(10000);
    let net_proceeds = usd_value - fees;

    // Settlement date
    let settlement_date = chrono::Utc::now() + chrono::Duration::days(2); // T+2 for bond sales

    // CRIT-003: Insert burn operation with idempotency key using runtime query
    #[derive(sqlx::FromRow)]
    struct BurnResult {
        id: i32,
        status: String,
    }

    let operation: BurnResult = sqlx::query_as(
        r#"
        INSERT INTO operations (
            user_id, operation_type, currency, amount, usd_value,
            fees_charged, status, settlement_date, idempotency_key
        )
        VALUES ($1, 'BURN', $2, $3, $4, $5, 'PENDING', $6, $7)
        RETURNING id, status
        "#
    )
    .bind(req.user_id)
    .bind(&req.currency)
    .bind(&req.amount)
    .bind(net_proceeds.to_string())
    .bind(fees.to_string())
    .bind(settlement_date)
    .bind(&req.idempotency_key)
    .fetch_one(state.db_pool.as_ref())
    .await
    .map_err(|e| {
        let err_str = e.to_string();
        if err_str.contains("idempotency_key") {
            tracing::warn!("Insert failed due to missing column - migration 20251230000001 required");
        }
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

/// CRIT-001 + CRIT-002: Get FX rate with circuit breaker and exponential backoff retry
/// Uses circuit breaker to fast-fail when oracle is unavailable
/// Retries oracle calls before falling back to static rates
async fn get_fx_rate(
    state: &Arc<AppState>,
    currency: &str,
) -> Result<Decimal, ApiError> {
    use crate::state::CircuitState;

    let pair = format!("{}/USD", currency);

    // CRIT-002: Check circuit breaker first
    let circuit_state = state.oracle_circuit_breaker.state();
    if circuit_state == CircuitState::Open {
        tracing::warn!(
            pair = %pair,
            "Circuit breaker OPEN - skipping oracle, using fallback rates"
        );
        // Fast-fail to fallback - don't even try oracle
        return get_fallback_rate(currency);
    }

    // 1. Try to get authentic price from Oracle with retry logic
    let oracle_guard = state.oracle.read().await;

    if let Some(oracle) = oracle_guard.as_ref() {
        let mut last_error: Option<String> = None;

        for attempt in 0..MAX_RETRIES {
            match oracle.get_price(&pair).await {
                Ok(price) => {
                    // CRIT-002: Record success for circuit breaker
                    state.oracle_circuit_breaker.record_success();

                    if attempt > 0 {
                        tracing::info!(
                            pair = %pair,
                            attempt = attempt + 1,
                            "Oracle succeeded after retry"
                        );
                    }
                    return Ok(price);
                }
                Err(e) => {
                    last_error = Some(e.to_string());

                    if attempt < MAX_RETRIES - 1 {
                        // CRIT-001: Exponential backoff with jitter
                        let backoff_ms = (INITIAL_BACKOFF_MS * 2u64.pow(attempt))
                            .min(MAX_BACKOFF_MS);
                        // Add 0-50% jitter to prevent thundering herd
                        let jitter = (backoff_ms as f64 * rand_jitter()) as u64;
                        let wait_time = Duration::from_millis(backoff_ms + jitter);

                        tracing::warn!(
                            pair = %pair,
                            attempt = attempt + 1,
                            backoff_ms = wait_time.as_millis(),
                            error = %e,
                            "Oracle call failed, retrying with backoff"
                        );

                        sleep(wait_time).await;
                    } else {
                        // CRIT-002: Record failure for circuit breaker after all retries exhausted
                        state.oracle_circuit_breaker.record_failure();

                        tracing::error!(
                            pair = %pair,
                            attempts = MAX_RETRIES,
                            error = %e,
                            circuit_state = ?state.oracle_circuit_breaker.state(),
                            "Oracle failed after all retries, falling back to static rates"
                        );
                    }
                }
            }
        }

        // Log that we're falling back after exhausting retries
        if let Some(err) = last_error {
            tracing::warn!(
                pair = %pair,
                last_error = %err,
                "Oracle exhausted {} retries, using fallback rates",
                MAX_RETRIES
            );
        }
    } else {
        tracing::debug!("Oracle not configured, using static rates for {}", currency);
    }

    // Drop the oracle guard before async operations
    drop(oracle_guard);

    get_fallback_rate(currency)
}

/// Get fallback FX rate (used when oracle is unavailable)
fn get_fallback_rate(currency: &str) -> Result<Decimal, ApiError> {

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

    // HIGH-011: Updated fallback rates as of 2025-12-29
    tracing::warn!(
        currency = currency,
        "Using FALLBACK FX rates - these may be stale. Last updated: 2025-12-29"
    );

    let rate = match currency {
        "EUR" => "1.04",  // HIGH-011: Updated from 1.09
        "GBP" => "1.25",  // HIGH-011: Updated from 1.22
        "JPY" => "0.0063", // HIGH-011: Updated from 0.0067
        "MXN" => "0.049", // HIGH-011: Updated from 0.058
        "BRL" => "0.16",  // HIGH-011: Updated from 0.20
        "ARS" => "0.00098", // HIGH-011: Updated from 0.0011
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

    // BE-MED-001 FIX: Use salted hash matching auth.rs to find session
    let token_hash = hash_token_for_lookup(token);

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

// HIGH-003: Use centralized token hashing from auth_utils
use super::auth_utils::hash_token_for_lookup;

#[cfg(test)]
mod tests {
    use super::*;

    // ========================
    // validate_amount tests
    // ========================

    #[test]
    fn test_validate_amount_zero() {
        let amount = Decimal::ZERO;
        let result = validate_amount(&amount, "test");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("greater than zero"));
    }

    #[test]
    fn test_validate_amount_negative() {
        let amount = Decimal::from_str("-100").unwrap();
        let result = validate_amount(&amount, "test");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("greater than zero"));
    }

    #[test]
    fn test_validate_amount_valid() {
        let amount = Decimal::from_str("100.50").unwrap();
        let result = validate_amount(&amount, "test");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_amount_maximum_boundary() {
        // Just at the limit - should pass
        let amount = Decimal::from_str(MAX_TRANSACTION_AMOUNT).unwrap();
        let result = validate_amount(&amount, "test");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_amount_exceeds_maximum() {
        // One more than max - should fail
        let amount = Decimal::from_str("10000000001").unwrap();
        let result = validate_amount(&amount, "test");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("exceeds maximum"));
    }

    #[test]
    fn test_validate_amount_small_positive() {
        let amount = Decimal::from_str("0.000001").unwrap();
        let result = validate_amount(&amount, "test");
        assert!(result.is_ok());
    }

    // ========================
    // validate_fx_rate tests
    // ========================

    #[test]
    fn test_validate_fx_rate_zero() {
        let rate = Decimal::ZERO;
        let result = validate_fx_rate(&rate, "EUR");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid FX rate"));
    }

    #[test]
    fn test_validate_fx_rate_negative() {
        let rate = Decimal::from_str("-1.05").unwrap();
        let result = validate_fx_rate(&rate, "EUR");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_fx_rate_below_minimum() {
        let rate = Decimal::from_str("0.00000001").unwrap();
        let result = validate_fx_rate(&rate, "JPY");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("below minimum"));
    }

    #[test]
    fn test_validate_fx_rate_at_minimum() {
        let rate = Decimal::from_str(MIN_FX_RATE).unwrap();
        let result = validate_fx_rate(&rate, "EUR");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_fx_rate_valid_eur() {
        let rate = Decimal::from_str("1.04").unwrap();
        let result = validate_fx_rate(&rate, "EUR");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_fx_rate_valid_jpy() {
        let rate = Decimal::from_str("0.0063").unwrap();
        let result = validate_fx_rate(&rate, "JPY");
        assert!(result.is_ok());
    }

    // ========================
    // validate_currency tests
    // ========================

    #[test]
    fn test_validate_currency_eur() {
        assert!(validate_currency("EUR").is_ok());
    }

    #[test]
    fn test_validate_currency_gbp() {
        assert!(validate_currency("GBP").is_ok());
    }

    #[test]
    fn test_validate_currency_jpy() {
        assert!(validate_currency("JPY").is_ok());
    }

    #[test]
    fn test_validate_currency_mxn() {
        assert!(validate_currency("MXN").is_ok());
    }

    #[test]
    fn test_validate_currency_brl() {
        assert!(validate_currency("BRL").is_ok());
    }

    #[test]
    fn test_validate_currency_ars() {
        assert!(validate_currency("ARS").is_ok());
    }

    #[test]
    fn test_validate_currency_lowercase() {
        // Should work with lowercase
        assert!(validate_currency("eur").is_ok());
        assert!(validate_currency("gbp").is_ok());
    }

    #[test]
    fn test_validate_currency_unsupported_usd() {
        let result = validate_currency("USD");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unsupported currency"));
    }

    #[test]
    fn test_validate_currency_unsupported_random() {
        let result = validate_currency("XYZ");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_currency_empty() {
        let result = validate_currency("");
        assert!(result.is_err());
    }

    // ========================
    // Fee calculation tests
    // ========================

    #[test]
    fn test_fee_constants() {
        // Verify fee constants are reasonable (25 basis points = 0.25%)
        assert_eq!(FEE_ISSUANCE_BPS, 25);
        assert_eq!(FEE_REDEMPTION_BPS, 25);
        assert_eq!(RESERVE_BUFFER_PERCENT, 2);
    }

    // ========================
    // hash_token_for_lookup tests
    // ========================

    #[test]
    fn test_hash_token_for_lookup_consistent() {
        let token = "test-token-12345";
        let hash1 = hash_token_for_lookup(token);
        let hash2 = hash_token_for_lookup(token);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_hash_token_for_lookup_different_tokens() {
        let hash1 = hash_token_for_lookup("token1");
        let hash2 = hash_token_for_lookup("token2");
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_hash_token_for_lookup_hex_format() {
        let hash = hash_token_for_lookup("test");
        // SHA-256 produces 64 hex characters
        assert_eq!(hash.len(), 64);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }
}

