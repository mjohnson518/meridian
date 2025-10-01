//! Request and response models for the API

use meridian_basket::{BasketType, CurrencyBasket, RebalanceStrategy};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

// ============ Basket Models ============

/// Request to create a new single-currency basket
#[derive(Debug, Deserialize, Serialize)]
pub struct CreateSingleCurrencyBasketRequest {
    pub name: String,
    pub currency_code: String,
    pub chainlink_feed: String,
}

/// Request to create an IMF SDR basket
#[derive(Debug, Deserialize, Serialize)]
pub struct CreateImfSdrBasketRequest {
    pub name: String,
    pub chainlink_feeds: HashMap<String, String>,
}

/// Request to create a custom basket
#[derive(Debug, Deserialize, Serialize)]
pub struct CreateCustomBasketRequest {
    pub name: String,
    pub components: Vec<ComponentRequest>,
    pub rebalance_strategy: RebalanceStrategyRequest,
}

/// Currency component in a basket
#[derive(Debug, Deserialize, Serialize)]
pub struct ComponentRequest {
    pub currency_code: String,
    pub target_weight: Decimal,
    pub min_weight: Decimal,
    pub max_weight: Decimal,
    pub chainlink_feed: String,
}

/// Rebalancing strategy
#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RebalanceStrategyRequest {
    None,
    Fixed {
        interval_days: u32,
    },
    ThresholdBased {
        max_deviation_percent: Decimal,
    },
}

impl From<RebalanceStrategyRequest> for RebalanceStrategy {
    fn from(req: RebalanceStrategyRequest) -> Self {
        match req {
            RebalanceStrategyRequest::None => RebalanceStrategy::None,
            RebalanceStrategyRequest::Fixed { interval_days } => {
                RebalanceStrategy::Fixed { interval_days }
            }
            RebalanceStrategyRequest::ThresholdBased {
                max_deviation_percent,
            } => RebalanceStrategy::ThresholdBased {
                max_deviation_percent,
            },
        }
    }
}

/// Response for basket operations
#[derive(Debug, Serialize)]
pub struct BasketResponse {
    pub id: Uuid,
    pub name: String,
    pub basket_type: String,
    pub components: Vec<ComponentResponse>,
    pub rebalance_strategy: String,
    pub created_at: String,
}

impl From<CurrencyBasket> for BasketResponse {
    fn from(basket: CurrencyBasket) -> Self {
        let basket_type = match basket.basket_type {
            BasketType::SingleCurrency => "single_currency".to_string(),
            BasketType::ImfSdr => "imf_sdr".to_string(),
            BasketType::CustomBasket => "custom_basket".to_string(),
        };

        let components = basket
            .components
            .into_iter()
            .map(ComponentResponse::from)
            .collect();

        let rebalance_strategy = match basket.rebalance_strategy {
            RebalanceStrategy::None => "none".to_string(),
            RebalanceStrategy::Fixed { interval_days } => {
                format!("fixed (every {} days)", interval_days)
            }
            RebalanceStrategy::ThresholdBased {
                max_deviation_percent,
            } => format!("threshold_based ({}%)", max_deviation_percent),
            RebalanceStrategy::Scheduled { .. } => "scheduled".to_string(),
        };

        Self {
            id: basket.id,
            name: basket.name,
            basket_type,
            components,
            rebalance_strategy,
            created_at: basket.created_at.to_rfc3339(),
        }
    }
}

/// Currency component response
#[derive(Debug, Serialize)]
pub struct ComponentResponse {
    pub currency_code: String,
    pub target_weight: Decimal,
    pub min_weight: Decimal,
    pub max_weight: Decimal,
    pub chainlink_feed: String,
}

impl From<meridian_basket::CurrencyComponent> for ComponentResponse {
    fn from(component: meridian_basket::CurrencyComponent) -> Self {
        Self {
            currency_code: component.currency_code,
            target_weight: component.target_weight,
            min_weight: component.min_weight,
            max_weight: component.max_weight,
            chainlink_feed: component.chainlink_feed,
        }
    }
}

/// Response for basket valuation
#[derive(Debug, Serialize)]
pub struct BasketValueResponse {
    pub basket_id: Uuid,
    pub value_usd: Decimal,
    pub prices_used: HashMap<String, Decimal>,
    pub needs_rebalancing: bool,
    pub calculated_at: String,
}

// ============ Oracle Models ============

/// Response for price queries
#[derive(Debug, Serialize)]
pub struct PriceResponse {
    pub pair: String,
    pub price_usd: Decimal,
    pub is_stale: bool,
    pub updated_at: String,
}

/// Response for multiple prices
#[derive(Debug, Serialize)]
pub struct PricesResponse {
    pub prices: HashMap<String, PriceData>,
}

#[derive(Debug, Serialize)]
pub struct PriceData {
    pub price_usd: Decimal,
    pub is_stale: bool,
    pub updated_at: String,
}

/// Request to register a price feed
#[derive(Debug, Deserialize)]
pub struct RegisterFeedRequest {
    pub pair: String,
    pub chainlink_address: String,
}

// ============ Health Check ============

/// Health check response
#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub oracle_enabled: bool,
    pub baskets_count: usize,
}

