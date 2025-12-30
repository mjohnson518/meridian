//! Chainlink oracle client implementation

use crate::error::OracleError;
use chrono::{DateTime, Utc};
use ethers::{
    contract::abigen,
    providers::{Http, Middleware, Provider},
    types::{Address, I256, U256},
};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::timeout;

/// Configuration for a price feed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceFeedConfig {
    /// Currency pair (e.g., "EUR/USD")
    pub pair: String,
    /// Chainlink aggregator contract address
    pub address: Address,
    /// Human-readable description
    pub description: Option<String>,
}

/// Current state of a price feed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceFeed {
    /// Currency pair (e.g., "EUR/USD")
    pub pair: String,
    /// Chainlink aggregator address
    pub address: Address,
    /// Number of decimals in the price
    pub decimals: u8,
    /// Latest price in USD
    pub latest_price: Decimal,
    /// Latest round ID
    pub latest_round: U256,
    /// Timestamp of last update
    pub updated_at: DateTime<Utc>,
    /// Whether the price is stale (>1 hour old)
    pub is_stale: bool,
    /// Human-readable description from contract
    pub description: String,
}

// Generate Chainlink AggregatorV3Interface bindings
abigen!(
    ChainlinkAggregatorV3,
    r#"[
        function latestRoundData() external view returns (uint80 roundId, int256 answer, uint256 startedAt, uint256 updatedAt, uint80 answeredInRound)
        function decimals() external view returns (uint8)
        function description() external view returns (string memory)
        function version() external view returns (uint256)
    ]"#
);

/// Default timeout for RPC calls (30 seconds)
const RPC_TIMEOUT_SECS: u64 = 30;

/// Chainlink oracle client for querying FX price feeds
///
/// Connects to Ethereum mainnet and queries Chainlink price feed aggregators
/// for real-time foreign exchange rates.
pub struct ChainlinkOracle {
    /// HTTP provider for Ethereum RPC calls
    provider: Arc<Provider<Http>>,
    /// Registered price feeds
    price_feeds: Arc<RwLock<HashMap<String, PriceFeed>>>,
    /// Maximum allowed price deviation (as percentage)
    deviation_threshold: Decimal,
    /// Staleness threshold in seconds (default: 3600 = 1 hour)
    stale_threshold_seconds: u64,
}

impl ChainlinkOracle {
    /// Creates a new Chainlink oracle client
    ///
    /// # Arguments
    ///
    /// * `rpc_url` - Ethereum RPC endpoint (e.g., Alchemy, Infura)
    /// * `deviation_threshold` - Maximum allowed price change percentage (e.g., 10.0 for 10%)
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use meridian_oracle::ChainlinkOracle;
    /// use rust_decimal::Decimal;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let oracle = ChainlinkOracle::new(
    ///     "https://eth-mainnet.g.alchemy.com/v2/YOUR_KEY",
    ///     Decimal::new(10, 0) // 10% deviation threshold
    /// ).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn new(rpc_url: &str, deviation_threshold: Decimal) -> Result<Self, OracleError> {
        let provider = Provider::<Http>::try_from(rpc_url)
            .map_err(|e| OracleError::ProviderError(e.to_string()))?;

        // Verify connection by getting chain ID (with timeout)
        let chain_id = timeout(
            Duration::from_secs(RPC_TIMEOUT_SECS),
            provider.get_chainid(),
        )
        .await
        .map_err(|_| OracleError::ProviderError("RPC connection timeout".to_string()))?
        .map_err(|e| OracleError::ProviderError(format!("Failed to connect: {}", e)))?;

        tracing::info!(
            chain_id = %chain_id,
            "Connected to Ethereum network"
        );

        Ok(Self {
            provider: Arc::new(provider),
            price_feeds: Arc::new(RwLock::new(HashMap::new())),
            deviation_threshold,
            stale_threshold_seconds: 3600, // 1 hour
        })
    }

    /// Registers a new price feed for a currency pair
    ///
    /// # Arguments
    ///
    /// * `pair` - Currency pair identifier (e.g., "EUR/USD")
    /// * `address` - Chainlink aggregator contract address
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use meridian_oracle::ChainlinkOracle;
    /// use rust_decimal::Decimal;
    /// use ethers::types::Address;
    /// use std::str::FromStr;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let oracle = ChainlinkOracle::new(
    ///     "https://eth-mainnet.g.alchemy.com/v2/YOUR_KEY",
    ///     Decimal::new(10, 0)
    /// ).await?;
    ///
    /// // Register EUR/USD feed
    /// let eur_usd_address = Address::from_str("0xb49f677943BC038e9857d61E7d053CaA2C1734C1")?;
    /// oracle.register_price_feed("EUR/USD", eur_usd_address).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn register_price_feed(
        &self,
        pair: &str,
        address: Address,
    ) -> Result<(), OracleError> {
        tracing::info!(
            pair = %pair,
            address = %address,
            "Registering price feed"
        );

        // Create contract instance
        let aggregator = ChainlinkAggregatorV3::new(address, Arc::clone(&self.provider));

        // Query contract metadata (with timeout)
        let decimals = timeout(
            Duration::from_secs(RPC_TIMEOUT_SECS),
            aggregator.decimals().call(),
        )
        .await
        .map_err(|_| OracleError::ContractError("RPC timeout getting decimals".to_string()))?
        .map_err(|e| OracleError::ContractError(format!("Failed to get decimals: {}", e)))?;

        let description = timeout(
            Duration::from_secs(RPC_TIMEOUT_SECS),
            aggregator.description().call(),
        )
        .await
        .map_err(|_| OracleError::ContractError("RPC timeout getting description".to_string()))?
        .map_err(|e| OracleError::ContractError(format!("Failed to get description: {}", e)))?;

        tracing::info!(
            pair = %pair,
            decimals = %decimals,
            description = %description,
            "Price feed metadata retrieved"
        );

        // Create initial feed entry (marked as stale until first update)
        let feed = PriceFeed {
            pair: pair.to_string(),
            address,
            decimals,
            latest_price: Decimal::ZERO,
            latest_round: U256::zero(),
            updated_at: Utc::now(),
            is_stale: true,
            description,
        };

        // Store in registry
        let mut feeds = self.price_feeds.write().await;
        feeds.insert(pair.to_string(), feed);

        Ok(())
    }

    /// Gets the current price for a currency pair
    ///
    /// Returns cached price if available and not stale. Use `update_price()`
    /// to force a refresh from the blockchain.
    ///
    /// # Arguments
    ///
    /// * `pair` - Currency pair (e.g., "EUR/USD")
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Price feed is not registered
    /// - Price is stale (>1 hour old)
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use meridian_oracle::ChainlinkOracle;
    /// # use rust_decimal::Decimal;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let oracle = ChainlinkOracle::new("http://localhost:8545", Decimal::new(10, 0)).await?;
    /// let price = oracle.get_price("EUR/USD").await?;
    /// println!("EUR/USD: ${}", price);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_price(&self, pair: &str) -> Result<Decimal, OracleError> {
        let feeds = self.price_feeds.read().await;

        let feed = feeds
            .get(pair)
            .ok_or_else(|| OracleError::PriceFeedNotFound(pair.to_string()))?;

        if feed.is_stale {
            let age = (Utc::now() - feed.updated_at).num_seconds() as u64;
            return Err(OracleError::StalePrice(pair.to_string(), age));
        }

        Ok(feed.latest_price)
    }

    /// Updates the price for a currency pair from the blockchain
    ///
    /// Queries the Chainlink aggregator contract and updates the cached price.
    /// Performs staleness detection and deviation checks.
    ///
    /// # Arguments
    ///
    /// * `pair` - Currency pair to update
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use meridian_oracle::ChainlinkOracle;
    /// # use rust_decimal::Decimal;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let oracle = ChainlinkOracle::new("http://localhost:8545", Decimal::new(10, 0)).await?;
    /// let price = oracle.update_price("EUR/USD").await?;
    /// println!("Updated EUR/USD: ${}", price);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn update_price(&self, pair: &str) -> Result<Decimal, OracleError> {
        // Get feed info (need to release lock before contract call)
        let (address, decimals, old_price, old_is_stale) = {
            let feeds = self.price_feeds.read().await;
            let feed = feeds
                .get(pair)
                .ok_or_else(|| OracleError::PriceFeedNotFound(pair.to_string()))?;
            (
                feed.address,
                feed.decimals,
                feed.latest_price,
                feed.is_stale,
            )
        };

        // Create contract instance
        let aggregator = ChainlinkAggregatorV3::new(address, Arc::clone(&self.provider));

        // Query latest round data (with timeout)
        let (round_id, answer, _started_at, updated_at, _answered_in_round) = timeout(
            Duration::from_secs(RPC_TIMEOUT_SECS),
            aggregator.latest_round_data().call(),
        )
        .await
        .map_err(|_| OracleError::ContractError("RPC timeout getting latest round data".to_string()))?
        .map_err(|e| {
            OracleError::ContractError(format!("Failed to get latest round data: {}", e))
        })?;

        tracing::debug!(
            pair = %pair,
            round_id = %round_id,
            answer = %answer,
            updated_at = %updated_at,
            "Retrieved latest round data"
        );

        // Convert Chainlink answer to Decimal
        let price = self.chainlink_answer_to_decimal(answer, decimals)?;

        // Check staleness
        let now = Utc::now().timestamp() as u64;
        let price_age = now.saturating_sub(updated_at.as_u64());
        let is_stale = price_age > self.stale_threshold_seconds;

        if is_stale {
            tracing::warn!(
                pair = %pair,
                age_seconds = %price_age,
                threshold = %self.stale_threshold_seconds,
                "Price is stale"
            );
        }

        // Check for excessive price deviation (if not first update)
        if !old_is_stale && old_price != Decimal::ZERO {
            let deviation = ((price - old_price) / old_price * Decimal::new(100, 0)).abs();

            if deviation > self.deviation_threshold {
                tracing::warn!(
                    pair = %pair,
                    old_price = %old_price,
                    new_price = %price,
                    deviation = %deviation,
                    threshold = %self.deviation_threshold,
                    "Large price deviation detected"
                );

                return Err(OracleError::PriceDeviation {
                    pair: pair.to_string(),
                    old_price,
                    new_price: price,
                    deviation,
                });
            }
        }

        // Update stored feed
        let mut feeds = self.price_feeds.write().await;
        if let Some(feed) = feeds.get_mut(pair) {
            feed.latest_price = price;
            feed.latest_round = round_id.into();
            feed.updated_at =
                DateTime::from_timestamp(updated_at.as_u64() as i64, 0).unwrap_or_else(Utc::now);
            feed.is_stale = is_stale;

            tracing::info!(
                pair = %pair,
                price = %price,
                round = %round_id,
                is_stale = %is_stale,
                "Price updated"
            );
        }

        Ok(price)
    }

    /// Gets information about a registered price feed
    ///
    /// # Arguments
    ///
    /// * `pair` - Currency pair
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use meridian_oracle::ChainlinkOracle;
    /// # use rust_decimal::Decimal;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let oracle = ChainlinkOracle::new("http://localhost:8545", Decimal::new(10, 0)).await?;
    /// let feed = oracle.get_feed_info("EUR/USD").await?;
    /// println!("Feed: {} ({})", feed.description, feed.address);
    /// println!("Decimals: {}", feed.decimals);
    /// println!("Latest price: ${}", feed.latest_price);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_feed_info(&self, pair: &str) -> Result<PriceFeed, OracleError> {
        let feeds = self.price_feeds.read().await;
        feeds
            .get(pair)
            .cloned()
            .ok_or_else(|| OracleError::PriceFeedNotFound(pair.to_string()))
    }

    /// Lists all registered price feeds
    pub async fn list_feeds(&self) -> Vec<String> {
        let feeds = self.price_feeds.read().await;
        feeds.keys().cloned().collect()
    }

    /// Converts Chainlink's int256 answer to Decimal
    ///
    /// Chainlink returns prices as int256 with a specified number of decimals.
    /// For example, EUR/USD with 8 decimals: 108000000 = 1.08
    fn chainlink_answer_to_decimal(
        &self,
        answer: I256,
        decimals: u8,
    ) -> Result<Decimal, OracleError> {
        // Convert I256 to string (handles negative numbers)
        let answer_str = answer.to_string();

        // Parse as Decimal
        let answer_decimal = Decimal::from_str(&answer_str)
            .map_err(|e| OracleError::DecimalConversion(format!("Invalid answer: {}", e)))?;

        // Divide by 10^decimals
        let divisor = Decimal::from(10_u64.pow(decimals as u32));
        let price = answer_decimal / divisor;

        Ok(price)
    }

    /// Gets the staleness threshold in seconds
    pub fn stale_threshold(&self) -> u64 {
        self.stale_threshold_seconds
    }

    /// Sets the staleness threshold in seconds
    pub fn set_stale_threshold(&mut self, seconds: u64) {
        self.stale_threshold_seconds = seconds;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chainlink_answer_conversion() {
        let oracle = ChainlinkOracle {
            provider: Arc::new(Provider::<Http>::try_from("http://localhost:8545").unwrap()),
            price_feeds: Arc::new(RwLock::new(HashMap::new())),
            deviation_threshold: Decimal::new(10, 0),
            stale_threshold_seconds: 3600,
        };

        // EUR/USD: 1.08 with 8 decimals = 108000000
        let answer = I256::from(108000000);
        let price = oracle.chainlink_answer_to_decimal(answer, 8).unwrap();
        assert_eq!(price, Decimal::new(108, 2)); // 1.08

        // GBP/USD: 1.27 with 8 decimals = 127000000
        let answer = I256::from(127000000);
        let price = oracle.chainlink_answer_to_decimal(answer, 8).unwrap();
        assert_eq!(price, Decimal::new(127, 2)); // 1.27

        // JPY/USD: 0.0067 with 8 decimals = 670000
        let answer = I256::from(670000);
        let price = oracle.chainlink_answer_to_decimal(answer, 8).unwrap();
        assert_eq!(price, Decimal::new(67, 4)); // 0.0067
    }

    #[tokio::test]
    async fn test_oracle_creation_invalid_url() {
        let result = ChainlinkOracle::new("invalid://url", Decimal::new(10, 0)).await;
        assert!(result.is_err());
    }
}
