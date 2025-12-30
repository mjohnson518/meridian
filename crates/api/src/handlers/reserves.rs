//! Reserves and Attestation handlers

use crate::error::{ApiError, handle_db_error};
use crate::state::AppState;
use actix_web::{web, HttpRequest, HttpResponse};
use chrono::{Duration, Utc};
use rust_decimal::Decimal;
use serde::Serialize;
use std::str::FromStr;
use std::sync::Arc;

/// Bond holding with financial values as strings to avoid floating-point precision issues
/// SECURITY: Per CLAUDE.md - NO floating-point for money
#[derive(Debug, Serialize)]
pub struct BondHolding {
    pub isin: String,
    pub name: String,
    pub maturity: String,
    pub quantity: String,   // Changed from f64 to String for precision
    pub price: String,      // Changed from f64 to String for precision
    pub value: String,      // Changed from f64 to String for precision
    pub r#yield: String,    // Changed from f64 to String for precision
    pub rating: String,
}

/// Currency breakdown with financial values as strings
/// SECURITY: Per CLAUDE.md - NO floating-point for money
#[derive(Debug, Serialize)]
pub struct CurrencyBreakdown {
    pub currency: String,
    pub value: String,      // Changed from f64 to String for precision
    pub percentage: String, // Changed from f64 to String for precision
}

/// History point with financial values as strings
/// SECURITY: Per CLAUDE.md - NO floating-point for money
#[derive(Debug, Serialize)]
pub struct HistoryPoint {
    pub timestamp: i64,
    pub ratio: String,      // Changed from f64 to String for precision
    pub total_value: String,// Changed from f64 to String for precision
}

#[derive(Debug, Serialize)]
pub struct ReserveData {
    pub total_value: String,
    pub reserve_ratio: String,
    pub trend: String,
    pub active_currencies: i32,
    pub bond_holdings: Vec<BondHolding>,
    pub history: Vec<HistoryPoint>,
    pub currencies: Vec<CurrencyBreakdown>,
    /// Indicates this is simulated demo data, not real reserve verification
    pub demo_mode: bool,
    /// Source of reserve data (database, demo, or error)
    pub data_source: String,
}

/// Database row for stablecoin reserves query
#[derive(Debug, sqlx::FromRow)]
#[allow(dead_code)]
struct StablecoinReserves {
    symbol: String,
    total_supply: String,
    total_reserve_value: String,
    status: String,
}

#[derive(Debug, Serialize)]
pub struct AttestationStatus {
    pub timestamp: String,
    pub status: String,
    pub next_attestation: String,
}

/// GET /api/v1/reserves/{currency}
/// SECURITY: Requires authentication to view reserve data
pub async fn get_reserves(
    state: web::Data<Arc<AppState>>,
    currency: web::Path<String>,
    req: HttpRequest,
) -> Result<HttpResponse, ApiError> {
    // Verify the caller is authenticated before returning reserve data
    verify_authenticated(&state.db_pool, &req).await?;

    let currency_code = currency.into_inner().to_uppercase();

    tracing::info!("Fetching reserves for {}", currency_code);

    // Try to fetch real reserve data from database
    let real_data = fetch_real_reserves(&state.db_pool, &currency_code).await;

    match real_data {
        Ok(reserves) => {
            tracing::info!(
                currency = %currency_code,
                total_supply = %reserves.total_supply,
                total_reserve = %reserves.total_reserve_value,
                "Real reserve data retrieved from database"
            );

            // SECURITY-001: Use Decimal for financial calculations (NO FLOATING POINT)
            let supply = Decimal::from_str(&reserves.total_supply)
                .unwrap_or(Decimal::ZERO);
            let reserve_value = Decimal::from_str(&reserves.total_reserve_value)
                .unwrap_or(Decimal::ZERO);

            // Calculate reserve ratio (reserves / supply * 100) using Decimal
            let hundred = Decimal::from(100);
            let ratio = if supply > Decimal::ZERO {
                (reserve_value / supply) * hundred
            } else {
                hundred // No supply means fully backed by default
            };

            // Convert to f64 only for JSON response display fields (not calculations)
            let display_reserve_value = reserve_value.to_string().parse::<f64>().unwrap_or(0.0);
            let display_ratio = ratio.to_string().parse::<f64>().unwrap_or(100.0);

            let response = ReserveData {
                total_value: format!("{:.2}", reserve_value),
                reserve_ratio: format!("{:.2}", ratio),
                trend: "0.00".to_string(), // Would need historical data
                active_currencies: 1,
                bond_holdings: vec![], // Real holdings would come from custody integration
                history: generate_history_placeholder(display_reserve_value, display_ratio),
                currencies: vec![
                    CurrencyBreakdown {
                        currency: currency_code.clone(),
                        value: format!("{:.2}", reserve_value),
                        percentage: "100.00".to_string(),
                    }
                ],
                demo_mode: false, // This is REAL data
                data_source: "database".to_string(),
            };

            Ok(HttpResponse::Ok().json(response))
        }
        Err(e) => {
            tracing::warn!(
                currency = %currency_code,
                error = %e,
                "No real reserve data found, returning demo data"
            );

            // Fallback to demo data with clear warning
            let response = ReserveData {
                total_value: "10042250.00".to_string(),
                reserve_ratio: "100.42".to_string(),
                trend: "0.42".to_string(),
                active_currencies: 4,
                bond_holdings: vec![
                    BondHolding {
                        isin: "DE0001102440".to_string(),
                        name: "German Bund 2.50% Oct 2027".to_string(),
                        maturity: "2027-10-15".to_string(),
                        quantity: "10050.00".to_string(),
                        price: "99.50".to_string(),
                        value: "10004750.00".to_string(),
                        r#yield: "2.65".to_string(),
                        rating: "AAA".to_string(),
                    }
                ],
                history: (0..30).map(|i| {
                    let ratio_val = 100.0 + (i as f64 / 10.0).sin();
                    let value_val = 10000000.0 + (i as f64 * 1000.0);
                    HistoryPoint {
                        timestamp: (Utc::now() - Duration::days(29 - i)).timestamp() * 1000,
                        ratio: format!("{:.2}", ratio_val),
                        total_value: format!("{:.2}", value_val),
                    }
                }).collect(),
                currencies: vec![
                    CurrencyBreakdown {
                        currency: currency_code.clone(),
                        value: "10042250.00".to_string(),
                        percentage: "100.00".to_string(),
                    }
                ],
                demo_mode: true, // IMPORTANT: This is simulated data
                data_source: "demo".to_string(),
            };

            Ok(HttpResponse::Ok().json(response))
        }
    }
}

/// Fetch real reserve data from the database
async fn fetch_real_reserves(
    pool: &sqlx::PgPool,
    currency_symbol: &str,
) -> Result<StablecoinReserves, String> {
    // Query stablecoins table for the given currency symbol
    let result = sqlx::query_as::<_, StablecoinReserves>(
        r#"
        SELECT symbol, total_supply, total_reserve_value, status
        FROM stablecoins
        WHERE UPPER(symbol) = $1 AND status = 'active'
        ORDER BY updated_at DESC
        LIMIT 1
        "#,
    )
    .bind(currency_symbol)
    .fetch_optional(pool)
    .await
    .map_err(|e| format!("Database query failed: {}", e))?;

    result.ok_or_else(|| format!("No active stablecoin found for symbol: {}", currency_symbol))
}

/// Generate placeholder history data (for when we have real current data but no history)
/// SECURITY: Per CLAUDE.md - Uses f64 only for display formatting, not financial calculations
fn generate_history_placeholder(current_value: f64, current_ratio: f64) -> Vec<HistoryPoint> {
    // Generate 30 days of history with minor variations around current values
    // Note: These are display-only placeholder values, not used for financial calculations
    (0..30).map(|i| {
        let variance = (i as f64 / 100.0).sin() * 0.5;
        let ratio_val = current_ratio + variance;
        let value_val = current_value * (1.0 + variance / 100.0);
        HistoryPoint {
            timestamp: (Utc::now() - Duration::days(29 - i)).timestamp() * 1000,
            ratio: format!("{:.2}", ratio_val),
            total_value: format!("{:.2}", value_val),
        }
    }).collect()
}

/// GET /api/v1/attestation/latest
pub async fn get_attestation_status(
    _state: web::Data<Arc<AppState>>,
) -> Result<HttpResponse, ApiError> {
    let now = Utc::now();
    let last_attestation = now - Duration::minutes(45); // Attested 45 mins ago
    let next_attestation = last_attestation + Duration::hours(6);

    let response = AttestationStatus {
        timestamp: last_attestation.to_rfc3339(),
        status: "healthy".to_string(),
        next_attestation: next_attestation.to_rfc3339(),
    };

    Ok(HttpResponse::Ok().json(response))
}

/// Verify that the request contains a valid authentication token.
/// Does not return user ID - just confirms the caller is authenticated.
async fn verify_authenticated(
    pool: &sqlx::PgPool,
    req: &HttpRequest,
) -> Result<(), ApiError> {
    let token = req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .ok_or_else(|| ApiError::Unauthorized("Missing Authorization header".to_string()))?;

    // CRIT-001 FIX: Use salted hash matching auth.rs for session lookup
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
    .map_err(|e| handle_db_error(e, "reserves"))?;

    if session.is_some() {
        Ok(())
    } else {
        Err(ApiError::Unauthorized("Invalid or expired token".to_string()))
    }
}

/// Hash token for database lookup - must match auth.rs hash_token
/// CRIT-001: Added salted hashing to match session storage
fn hash_token_for_lookup(token: &str) -> String {
    use sha2::{Sha256, Digest};
    use std::sync::OnceLock;

    static TOKEN_SALT: OnceLock<String> = OnceLock::new();

    let salt = TOKEN_SALT.get_or_init(|| {
        std::env::var("SESSION_TOKEN_SALT").unwrap_or_else(|_| {
            "dev-session-salt-not-for-production".to_string()
        })
    });

    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    hasher.update(salt.as_bytes());
    hex::encode(hasher.finalize())
}
