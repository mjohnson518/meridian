//! Application state shared across all handlers

use meridian_basket::CurrencyBasket;
use meridian_oracle::ChainlinkOracle;
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Shared application state
pub struct AppState {
    /// Registry of all currency baskets
    pub baskets: Arc<RwLock<HashMap<Uuid, CurrencyBasket>>>,
    /// Chainlink oracle client (optional, requires RPC URL)
    pub oracle: Arc<RwLock<Option<ChainlinkOracle>>>,
}

impl AppState {
    /// Creates new application state
    pub async fn new() -> Self {
        // Try to initialize oracle if RPC URL is provided
        let oracle = if let Ok(rpc_url) = std::env::var("ETHEREUM_RPC_URL") {
            tracing::info!("Initializing Chainlink oracle with RPC URL");
            match ChainlinkOracle::new(&rpc_url, Decimal::new(10, 0)).await {
                Ok(oracle) => {
                    tracing::info!("✅ Chainlink oracle initialized");
                    Some(oracle)
                }
                Err(e) => {
                    tracing::warn!("Failed to initialize oracle: {}", e);
                    None
                }
            }
        } else {
            tracing::info!("No ETHEREUM_RPC_URL provided, oracle features disabled");
            None
        };

        Self {
            baskets: Arc::new(RwLock::new(HashMap::new())),
            oracle: Arc::new(RwLock::new(oracle)),
        }
    }

    /// Gets a basket by ID
    pub async fn get_basket(&self, id: &Uuid) -> Option<CurrencyBasket> {
        let baskets = self.baskets.read().await;
        baskets.get(id).cloned()
    }

    /// Adds a new basket
    pub async fn add_basket(&self, basket: CurrencyBasket) -> Uuid {
        let id = basket.id;
        let mut baskets = self.baskets.write().await;
        baskets.insert(id, basket);
        id
    }

    /// Lists all baskets
    pub async fn list_baskets(&self) -> Vec<CurrencyBasket> {
        let baskets = self.baskets.read().await;
        baskets.values().cloned().collect()
    }
}
