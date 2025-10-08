//! Oracle price feed handlers

use crate::error::ApiError;
use crate::models::*;
use crate::state::AppState;
use actix_web::{web, HttpResponse};
use chrono::Utc;
use ethers::types::Address;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

/// Get all current prices
///
/// GET /api/v1/oracle/prices
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
pub async fn update_price(
    state: web::Data<Arc<AppState>>,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let pair = path.into_inner();

    tracing::info!(pair = %pair, "Updating price from blockchain");

    let oracle_guard = state.oracle.read().await;
    let oracle = oracle_guard.as_ref().ok_or(ApiError::OracleNotConfigured)?;

    let price = oracle.update_price(&pair).await?;

    let feed = oracle.get_feed_info(&pair).await?;

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
pub async fn register_price_feed(
    state: web::Data<Arc<AppState>>,
    req: web::Json<RegisterFeedRequest>,
) -> Result<HttpResponse, ApiError> {
    tracing::info!(
        pair = %req.pair,
        address = %req.chainlink_address,
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
