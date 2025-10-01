//! Basket management handlers

use crate::error::ApiError;
use crate::models::*;
use crate::state::AppState;
use actix_web::{web, HttpResponse};
use chrono::Utc;
use meridian_basket::{CurrencyBasket, CurrencyComponent};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

/// Create a new single-currency basket
///
/// POST /api/v1/baskets/single-currency
pub async fn create_single_currency_basket(
    state: web::Data<Arc<AppState>>,
    req: web::Json<CreateSingleCurrencyBasketRequest>,
) -> Result<HttpResponse, ApiError> {
    tracing::info!(
        name = %req.name,
        currency = %req.currency_code,
        "Creating single-currency basket"
    );

    let basket = CurrencyBasket::new_single_currency(
        req.name.clone(),
        req.currency_code.clone(),
        req.chainlink_feed.clone(),
    )?;

    let id = state.add_basket(basket.clone()).await;

    tracing::info!(id = %id, "Basket created successfully");

    Ok(HttpResponse::Created().json(BasketResponse::from(basket)))
}

/// Create an IMF SDR basket
///
/// POST /api/v1/baskets/imf-sdr
pub async fn create_imf_sdr_basket(
    state: web::Data<Arc<AppState>>,
    req: web::Json<CreateImfSdrBasketRequest>,
) -> Result<HttpResponse, ApiError> {
    tracing::info!(name = %req.name, "Creating IMF SDR basket");

    let basket = CurrencyBasket::new_imf_sdr(req.name.clone(), req.chainlink_feeds.clone())?;

    let id = state.add_basket(basket.clone()).await;

    tracing::info!(id = %id, "IMF SDR basket created successfully");

    Ok(HttpResponse::Created().json(BasketResponse::from(basket)))
}

/// Create a custom basket
///
/// POST /api/v1/baskets/custom
pub async fn create_custom_basket(
    state: web::Data<Arc<AppState>>,
    req: web::Json<CreateCustomBasketRequest>,
) -> Result<HttpResponse, ApiError> {
    tracing::info!(
        name = %req.name,
        components = req.components.len(),
        "Creating custom basket"
    );

    // Convert request components to basket components
    let components: Result<Vec<CurrencyComponent>, _> = req
        .components
        .iter()
        .map(|c| {
            CurrencyComponent::new(
                c.currency_code.clone(),
                c.target_weight,
                c.min_weight,
                c.max_weight,
                c.chainlink_feed.clone(),
            )
        })
        .collect();

    let components = components?;

    let basket = CurrencyBasket::new_custom_basket(
        req.name.clone(),
        components,
        req.rebalance_strategy.clone().into(),
    )?;

    let id = state.add_basket(basket.clone()).await;

    tracing::info!(id = %id, "Custom basket created successfully");

    Ok(HttpResponse::Created().json(BasketResponse::from(basket)))
}

/// Get basket by ID
///
/// GET /api/v1/baskets/{id}
pub async fn get_basket(
    state: web::Data<Arc<AppState>>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let basket_id = path.into_inner();

    tracing::debug!(id = %basket_id, "Fetching basket");

    let basket = state
        .get_basket(&basket_id)
        .await
        .ok_or_else(|| ApiError::NotFound(format!("Basket {} not found", basket_id)))?;

    Ok(HttpResponse::Ok().json(BasketResponse::from(basket)))
}

/// List all baskets
///
/// GET /api/v1/baskets
pub async fn list_baskets(state: web::Data<Arc<AppState>>) -> Result<HttpResponse, ApiError> {
    tracing::debug!("Listing all baskets");

    let baskets = state.list_baskets().await;
    let responses: Vec<BasketResponse> = baskets.into_iter().map(BasketResponse::from).collect();

    Ok(HttpResponse::Ok().json(responses))
}

/// Calculate basket value
///
/// GET /api/v1/baskets/{id}/value
pub async fn get_basket_value(
    state: web::Data<Arc<AppState>>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let basket_id = path.into_inner();

    tracing::debug!(id = %basket_id, "Calculating basket value");

    let basket = state
        .get_basket(&basket_id)
        .await
        .ok_or_else(|| ApiError::NotFound(format!("Basket {} not found", basket_id)))?;

    // Get oracle
    let oracle_guard = state.oracle.read().await;
    let oracle = oracle_guard
        .as_ref()
        .ok_or(ApiError::OracleNotConfigured)?;

    // Fetch prices for all components
    let mut prices = HashMap::new();
    for component in &basket.components {
        let price = oracle.update_price(&component.currency_code).await?;
        prices.insert(component.currency_code.clone(), price);
    }

    // Calculate value
    let value = basket.calculate_value(&prices)?;
    let needs_rebalancing = basket.needs_rebalancing(&prices)?;

    let response = BasketValueResponse {
        basket_id: basket.id,
        value_usd: value,
        prices_used: prices,
        needs_rebalancing,
        calculated_at: Utc::now().to_rfc3339(),
    };

    Ok(HttpResponse::Ok().json(response))
}

