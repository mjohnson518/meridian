//! Application state shared across all handlers

use meridian_oracle::ChainlinkOracle;
use rust_decimal::Decimal;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Shared application state
pub struct AppState {
    /// Database connection pool
    pub db_pool: Arc<PgPool>,
    /// Chainlink oracle client (optional, requires RPC URL)
    pub oracle: Arc<RwLock<Option<ChainlinkOracle>>>,
}

impl AppState {
    /// Creates new application state with database pool
    pub async fn new(db_pool: PgPool) -> Self {
        // Try to initialize oracle if RPC URL is provided
        let oracle = if let Ok(rpc_url) = std::env::var("ETHEREUM_RPC_URL") {
            tracing::info!("Initializing Chainlink oracle with RPC URL");
            match ChainlinkOracle::new(&rpc_url, Decimal::new(10, 0)).await {
                Ok(oracle) => {
                    tracing::info!("Chainlink oracle initialized");
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
            db_pool: Arc::new(db_pool),
            oracle: Arc::new(RwLock::new(oracle)),
        }
    }
}
