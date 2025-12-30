//! Request and response models for the API

use meridian_basket::{BasketType, CurrencyBasket, RebalanceStrategy};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

// ============ Basket Models ============

/// Request to create a new single-currency basket
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct CreateSingleCurrencyBasketRequest {
    /// Name of the basket (e.g., "EUR Stablecoin")
    #[schema(example = "EUR Stablecoin")]
    pub name: String,
    /// ISO currency code (e.g., "EUR", "GBP", "JPY")
    #[schema(example = "EUR")]
    pub currency_code: String,
    /// Chainlink price feed address on Ethereum
    #[schema(example = "0x1a81afB8146aeFfCFc5E50e8479e826E7D55b910")]
    pub chainlink_feed: String,
}

/// Request to create an IMF SDR basket
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct CreateImfSdrBasketRequest {
    /// Name of the basket (e.g., "IMF SDR Basket")
    #[schema(example = "IMF SDR Basket")]
    pub name: String,
    /// Map of currency codes to Chainlink feed addresses
    pub chainlink_feeds: HashMap<String, String>,
}

/// Request to create a custom basket
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct CreateCustomBasketRequest {
    /// Name of the custom basket
    #[schema(example = "European Trade Basket")]
    pub name: String,
    /// Currency components with weights
    pub components: Vec<ComponentRequest>,
    /// Rebalancing strategy for the basket
    pub rebalance_strategy: RebalanceStrategyRequest,
}

/// Currency component in a basket
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct ComponentRequest {
    /// ISO currency code
    #[schema(example = "EUR")]
    pub currency_code: String,
    /// Target weight in the basket (0.0-1.0)
    #[schema(example = "0.50", value_type = String)]
    pub target_weight: Decimal,
    /// Minimum allowed weight (0.0-1.0)
    #[schema(example = "0.40", value_type = String)]
    pub min_weight: Decimal,
    /// Maximum allowed weight (0.0-1.0)
    #[schema(example = "0.60", value_type = String)]
    pub max_weight: Decimal,
    /// Chainlink price feed address
    #[schema(example = "0x1a81afB8146aeFfCFc5E50e8479e826E7D55b910")]
    pub chainlink_feed: String,
}

/// Rebalancing strategy
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RebalanceStrategyRequest {
    /// No automatic rebalancing
    None,
    /// Rebalance on fixed interval
    Fixed {
        /// Days between rebalancing
        interval_days: u32,
    },
    /// Rebalance when deviation exceeds threshold
    ThresholdBased {
        /// Maximum allowed deviation percentage
        #[schema(value_type = String)]
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
#[derive(Debug, Serialize, ToSchema)]
pub struct BasketResponse {
    /// Unique basket identifier
    pub id: Uuid,
    /// Basket name
    #[schema(example = "EUR Stablecoin")]
    pub name: String,
    /// Type of basket (single_currency, imf_sdr, custom_basket)
    #[schema(example = "single_currency")]
    pub basket_type: String,
    /// Currency components in the basket
    pub components: Vec<ComponentResponse>,
    /// Rebalancing strategy description
    #[schema(example = "none")]
    pub rebalance_strategy: String,
    /// ISO 8601 creation timestamp
    #[schema(example = "2025-01-01T12:00:00Z")]
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
#[derive(Debug, Serialize, ToSchema)]
pub struct ComponentResponse {
    /// ISO currency code
    #[schema(example = "EUR")]
    pub currency_code: String,
    /// Target weight (0.0-1.0)
    #[schema(value_type = String)]
    pub target_weight: Decimal,
    /// Minimum weight (0.0-1.0)
    #[schema(value_type = String)]
    pub min_weight: Decimal,
    /// Maximum weight (0.0-1.0)
    #[schema(value_type = String)]
    pub max_weight: Decimal,
    /// Chainlink price feed address
    #[schema(example = "0x1a81afB8146aeFfCFc5E50e8479e826E7D55b910")]
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
#[derive(Debug, Serialize, ToSchema)]
pub struct BasketValueResponse {
    /// Basket identifier
    pub basket_id: Uuid,
    /// Current basket value in USD
    #[schema(value_type = String)]
    pub value_usd: Decimal,
    /// Prices used for calculation (currency -> price)
    #[schema(value_type = Object)]
    pub prices_used: HashMap<String, Decimal>,
    /// Whether the basket needs rebalancing
    pub needs_rebalancing: bool,
    /// ISO 8601 calculation timestamp
    #[schema(example = "2025-01-01T12:00:00Z")]
    pub calculated_at: String,
}

// ============ Oracle Models ============

/// Response for price queries
#[derive(Debug, Serialize, ToSchema)]
pub struct PriceResponse {
    /// Currency pair (e.g., "EUR/USD")
    #[schema(example = "EUR/USD")]
    pub pair: String,
    /// Price in USD
    #[schema(value_type = String, example = "1.0842")]
    pub price_usd: Decimal,
    /// Whether the price data is stale
    pub is_stale: bool,
    /// ISO 8601 timestamp of last update
    #[schema(example = "2025-01-01T12:00:00Z")]
    pub updated_at: String,
}

/// Response for multiple prices
#[derive(Debug, Serialize, ToSchema)]
pub struct PricesResponse {
    /// Map of currency pairs to price data
    #[schema(value_type = Object)]
    pub prices: HashMap<String, PriceData>,
}

/// Individual price data
#[derive(Debug, Serialize, ToSchema)]
pub struct PriceData {
    /// Price in USD
    #[schema(value_type = String, example = "1.0842")]
    pub price_usd: Decimal,
    /// Whether the price is stale
    pub is_stale: bool,
    /// ISO 8601 timestamp
    #[schema(example = "2025-01-01T12:00:00Z")]
    pub updated_at: String,
}

/// Request to register a price feed
#[derive(Debug, Deserialize, ToSchema)]
pub struct RegisterFeedRequest {
    /// Currency pair (e.g., "EUR/USD")
    #[schema(example = "EUR/USD")]
    pub pair: String,
    /// Chainlink price feed contract address
    #[schema(example = "0x1a81afB8146aeFfCFc5E50e8479e826E7D55b910")]
    pub chainlink_address: String,
}

// ============ Health Check ============

/// Health check response
#[derive(Debug, Serialize, ToSchema)]
pub struct HealthResponse {
    /// Service health status ("healthy" or "degraded")
    #[schema(example = "healthy")]
    pub status: String,
    /// API version
    #[schema(example = "0.1.0")]
    pub version: String,
    /// Whether the oracle is configured
    pub oracle_enabled: bool,
    /// Number of active baskets
    pub baskets_count: usize,
}

// ============ Pagination ============

/// CRIT-013: Pagination query parameters with safe defaults
#[derive(Debug, Deserialize, ToSchema, IntoParams)]
pub struct PaginationQuery {
    /// Maximum items to return (default: 20, max: 100)
    #[serde(default = "default_limit")]
    #[schema(default = 20, maximum = 100)]
    pub limit: u32,
    /// Offset for pagination (default: 0)
    #[serde(default)]
    #[schema(default = 0)]
    pub offset: u32,
}

fn default_limit() -> u32 {
    20
}

impl PaginationQuery {
    /// Get safe limit (clamped to max 100)
    pub fn safe_limit(&self) -> i64 {
        self.limit.min(100) as i64
    }

    /// Get offset as i64
    pub fn offset(&self) -> i64 {
        self.offset as i64
    }
}

/// Paginated list response
#[derive(Debug, Serialize, ToSchema)]
#[aliases(PaginatedBasketResponse = PaginatedResponse<BasketResponse>)]
pub struct PaginatedResponse<T> {
    /// Items in this page
    pub items: Vec<T>,
    /// Items per page
    pub limit: u32,
    /// Current offset
    pub offset: u32,
    /// Total item count (if available)
    pub total: Option<i64>,
}
