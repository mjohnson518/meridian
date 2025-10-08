//! Database models

use chrono::{DateTime, Utc};
use meridian_basket::{BasketType, CurrencyBasket};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

// ============ Basket Models ============

/// Database representation of a currency basket
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct BasketRow {
    pub id: Uuid,
    pub name: String,
    pub basket_type: String,
    pub components: serde_json::Value,
    pub rebalance_strategy: serde_json::Value,
    pub last_rebalanced: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl BasketRow {
    /// Converts a CurrencyBasket to a database row
    pub fn from_basket(basket: &CurrencyBasket) -> Result<Self, serde_json::Error> {
        let basket_type = match basket.basket_type {
            BasketType::SingleCurrency => "single_currency",
            BasketType::ImfSdr => "imf_sdr",
            BasketType::CustomBasket => "custom_basket",
        };

        Ok(Self {
            id: basket.id,
            name: basket.name.clone(),
            basket_type: basket_type.to_string(),
            components: serde_json::to_value(&basket.components)?,
            rebalance_strategy: serde_json::to_value(&basket.rebalance_strategy)?,
            last_rebalanced: basket.last_rebalanced,
            created_at: basket.created_at,
            updated_at: Utc::now(),
        })
    }

    /// Converts a database row to a CurrencyBasket
    pub fn to_basket(&self) -> Result<CurrencyBasket, serde_json::Error> {
        let basket_type = match self.basket_type.as_str() {
            "single_currency" => BasketType::SingleCurrency,
            "imf_sdr" => BasketType::ImfSdr,
            "custom_basket" => BasketType::CustomBasket,
            _ => BasketType::CustomBasket, // Default fallback
        };

        Ok(CurrencyBasket {
            id: self.id,
            name: self.name.clone(),
            basket_type,
            components: serde_json::from_value(self.components.clone())?,
            rebalance_strategy: serde_json::from_value(self.rebalance_strategy.clone())?,
            last_rebalanced: self.last_rebalanced,
            created_at: self.created_at,
        })
    }
}

// ============ Price History Models ============

/// Database representation of a price record
/// Note: price and round_id stored as TEXT due to SQLx-Decimal compatibility
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct PriceHistoryRow {
    pub id: i64,
    pub currency_pair: String,
    pub price: String, // TEXT storage for Decimal
    pub source: String,
    pub is_stale: bool,
    pub round_id: Option<String>, // TEXT storage for Decimal
    pub timestamp: DateTime<Utc>,
}

/// Request to insert a new price record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsertPriceRequest {
    pub currency_pair: String,
    pub price: Decimal,
    pub source: String,
    pub is_stale: bool,
    pub round_id: Option<Decimal>,
}

// ============ Stablecoin Models ============

/// Database representation of a stablecoin
/// Note: total_supply and total_reserve_value stored as TEXT due to SQLx-Decimal compatibility
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct StablecoinRow {
    pub id: Uuid,
    pub name: String,
    pub symbol: String,
    pub contract_address: Option<String>,
    pub basket_id: Option<Uuid>,
    pub chain_id: i32,
    pub total_supply: String,        // TEXT storage for Decimal
    pub total_reserve_value: String, // TEXT storage for Decimal
    pub status: String,
    pub deployed_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Request to create a new stablecoin record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateStablecoinRequest {
    pub name: String,
    pub symbol: String,
    pub basket_id: Option<Uuid>,
    pub chain_id: i32,
}

// ============ Audit Log Models ============

/// Database representation of an audit log entry
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct AuditLogRow {
    pub id: i64,
    pub operation: String,
    pub actor: Option<String>,
    pub stablecoin_id: Option<Uuid>,
    pub basket_id: Option<Uuid>,
    pub details: serde_json::Value,
    pub timestamp: DateTime<Utc>,
}

/// Request to create an audit log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAuditLogRequest {
    pub operation: String,
    pub actor: Option<String>,
    pub stablecoin_id: Option<Uuid>,
    pub basket_id: Option<Uuid>,
    pub details: serde_json::Value,
}
