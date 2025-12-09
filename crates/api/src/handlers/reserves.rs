//! Reserves and Attestation handlers

use crate::error::ApiError;
use crate::state::AppState;
use actix_web::{web, HttpResponse};
use chrono::{Duration, Utc};
use serde::Serialize;
use std::sync::Arc;

#[derive(Debug, Serialize)]
pub struct BondHolding {
    pub isin: String,
    pub name: String,
    pub maturity: String,
    pub quantity: f64,
    pub price: f64,
    pub value: f64,
    pub r#yield: f64, // 'yield' is a keyword in Rust
    pub rating: String,
}

#[derive(Debug, Serialize)]
pub struct CurrencyBreakdown {
    pub currency: String,
    pub value: f64,
    pub percentage: f64,
}

#[derive(Debug, Serialize)]
pub struct HistoryPoint {
    pub timestamp: i64,
    pub ratio: f64,
    pub total_value: f64,
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
}

#[derive(Debug, Serialize)]
pub struct AttestationStatus {
    pub timestamp: String,
    pub status: String,
    pub next_attestation: String,
}

/// GET /api/v1/reserves/{currency}
pub async fn get_reserves(
    state: web::Data<Arc<AppState>>,
    currency: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let currency_code = currency.into_inner();
    
    tracing::info!("Fetching reserves for {}", currency_code);

    // In a real implementation, we would:
    // 1. Query total minted supply from operations/ledger
    // 2. Query total assets in custody from reserves table/oracle
    // 3. Calculate ratio

    // For now, we'll simulate a healthy reserve based on the requested currency
    
    // Example: Fetch total minted from DB
    // let minted = sqlx::query!("SELECT SUM(usd_value) as total FROM operations WHERE operation_type = 'MINT' AND status = 'COMPLETED'")...

    // Mock response to satisfy frontend contract
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
                quantity: 10050.0,
                price: 99.50,
                value: 10004750.00,
                r#yield: 2.65,
                rating: "AAA".to_string(),
            }
        ],
        history: (0..30).map(|i| {
            HistoryPoint {
                timestamp: (Utc::now() - Duration::days(29 - i)).timestamp() * 1000,
                ratio: 100.0 + (i as f64 / 10.0).sin(),
                total_value: 10000000.0 + (i as f64 * 1000.0),
            }
        }).collect(),
        currencies: vec![
            CurrencyBreakdown {
                currency: currency_code.clone(),
                value: 10042250.00,
                percentage: 100.0,
            }
        ],
    };

    Ok(HttpResponse::Ok().json(response))
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
