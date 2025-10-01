//! Error types for oracle operations

use rust_decimal::Decimal;
use thiserror::Error;

/// Errors that can occur during oracle operations
#[derive(Error, Debug)]
pub enum OracleError {
    #[error("Price feed not found: {0}")]
    PriceFeedNotFound(String),

    #[error("Stale price for {0}: last updated {1} seconds ago")]
    StalePrice(String, u64),

    #[error("Price deviation detected for {pair}: old={old_price}, new={new_price}, deviation={deviation}%")]
    PriceDeviation {
        pair: String,
        old_price: Decimal,
        new_price: Decimal,
        deviation: Decimal,
    },

    #[error("Invalid price data: {0}")]
    InvalidPrice(String),

    #[error("Invalid address: {0}")]
    InvalidAddress(String),

    #[error("Provider error: {0}")]
    ProviderError(String),

    #[error("Contract call failed: {0}")]
    ContractError(String),

    #[error("Decimal conversion error: {0}")]
    DecimalConversion(String),
}

// Convert ethers provider errors
impl From<ethers::providers::ProviderError> for OracleError {
    fn from(err: ethers::providers::ProviderError) -> Self {
        OracleError::ProviderError(err.to_string())
    }
}

