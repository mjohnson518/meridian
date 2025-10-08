//! # Meridian Oracle Integration
//!
//! This crate provides real-time FX price feeds for multi-currency stablecoins
//! using Chainlink's decentralized oracle network.
//!
//! ## Features
//!
//! - Connect to Chainlink price feeds on Ethereum mainnet
//! - Query real-time FX rates for 20+ currency pairs
//! - Automatic staleness detection (>1 hour)
//! - Deviation threshold monitoring
//! - Support for multiple price feed sources (Chainlink primary)
//!
//! ## Example
//!
//! ```rust,no_run
//! use meridian_oracle::{ChainlinkOracle, PriceFeedConfig};
//! use rust_decimal::Decimal;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Connect to Ethereum mainnet via Alchemy
//! let rpc_url = "https://eth-mainnet.g.alchemy.com/v2/YOUR_KEY";
//! let oracle = ChainlinkOracle::new(rpc_url, Decimal::new(10, 0)).await?;
//!
//! // Get EUR/USD price
//! let eur_usd_price = oracle.get_price("EUR/USD").await?;
//! println!("EUR/USD: {}", eur_usd_price);
//! # Ok(())
//! # }
//! ```

mod error;
mod feeds;
mod oracle;

pub use error::OracleError;
pub use feeds::mainnet_feeds;
pub use oracle::{ChainlinkOracle, PriceFeed, PriceFeedConfig};
