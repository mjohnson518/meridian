//! # Meridian Custody Adapter
//!
//! Pluggable custody integration for Proof of Reserves.
//!
//! ## Architecture
//!
//! `CustodyAdapter` is a trait that each custodian implementation satisfies:
//! - `FireblocksAdapter` — production custody via Fireblocks API
//! - `BitGoAdapter`       — alternative via BitGo API
//! - `MockAdapter`        — deterministic mock for testing
//!
//! The automated Proof of Reserves background service in the API calls
//! `get_reserve_balance()` and `get_bond_holdings()` every 6 hours, sums
//! the totals via live oracle FX rates, and submits an `attestReserves()`
//! transaction on-chain if reserves >= min_reserve_ratio.
//!
//! ## Adding a New Custodian
//!
//! Implement `CustodyAdapter` for your type, add it to `CustodyAdapterKind`,
//! and wire it up via the `CUSTODY_PROVIDER` environment variable.

pub mod bitgo;
pub mod fireblocks;
pub mod mock;

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

/// Errors from custody operations
#[derive(Error, Debug)]
pub enum CustodyError {
    #[error("HTTP error communicating with custodian: {0}")]
    Http(String),

    #[error("Authentication failed: {0}")]
    Auth(String),

    #[error("Custodian API error: {status} — {message}")]
    Api { status: u16, message: String },

    #[error("Invalid response from custodian: {0}")]
    InvalidResponse(String),

    #[error("Custody configuration error: {0}")]
    Config(String),

    #[error("Vault not found: {0}")]
    VaultNotFound(String),
}

pub type CustodyResult<T> = Result<T, CustodyError>;

/// Total reserve balance across all vaults for a single currency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReserveBalance {
    /// ISO 4217 currency code (e.g., "EUR", "USD")
    pub currency: String,
    /// Total balance across all accounts/vaults
    pub total_balance: Decimal,
    /// Total balance available (not locked in settlement)
    pub available_balance: Decimal,
    /// Total balance locked/pending
    pub locked_balance: Decimal,
    /// Timestamp of the balance snapshot
    pub snapshot_at: DateTime<Utc>,
    /// Custodian that provided this data
    pub custodian: String,
}

/// A single sovereign bond holding in custody
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BondHolding {
    /// Unique internal holding ID
    pub id: Uuid,
    /// ISIN of the bond (e.g., "DE0001102580" for German Bund)
    pub isin: String,
    /// Human-readable bond name
    pub name: String,
    /// ISO 4217 denomination currency
    pub currency: String,
    /// Face value / nominal amount
    pub face_value: Decimal,
    /// Current market value
    pub market_value: Decimal,
    /// Yield to maturity (as decimal, e.g., 0.0325 = 3.25%)
    pub yield_to_maturity: Decimal,
    /// Bond maturity date
    pub maturity_date: DateTime<Utc>,
    /// Custodian account/vault holding this bond
    pub custodian_account_id: String,
    /// Timestamp of the valuation
    pub valued_at: DateTime<Utc>,
}

/// Cryptographic proof that reserves are held in custody
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustodyProof {
    /// Proof identifier
    pub id: Uuid,
    /// The custodian that issued this proof
    pub custodian: String,
    /// Total attested reserve value in USD (2 decimal places)
    pub total_value_usd: Decimal,
    /// List of asset ISINs/identifiers covered by this proof
    pub covered_assets: Vec<String>,
    /// Cryptographic signature from custodian (format depends on custodian)
    pub signature: Option<String>,
    /// Hash of the underlying statements
    pub statement_hash: Option<String>,
    /// When this proof was generated
    pub issued_at: DateTime<Utc>,
    /// When this proof expires (typically 24h)
    pub expires_at: DateTime<Utc>,
}

/// Core trait that all custody adapters must implement
#[async_trait::async_trait]
pub trait CustodyAdapter: Send + Sync {
    /// Name of this custody provider (e.g., "Fireblocks", "BitGo", "Mock")
    fn provider_name(&self) -> &str;

    /// Get the total reserve balance for a currency across all vaults
    async fn get_reserve_balance(&self, currency: &str) -> CustodyResult<ReserveBalance>;

    /// Get all sovereign bond holdings across all vaults
    async fn get_bond_holdings(&self) -> CustodyResult<Vec<BondHolding>>;

    /// Get the total value of all holdings in USD
    async fn get_total_value_usd(&self) -> CustodyResult<Decimal>;

    /// Generate a proof of reserves (custodian-signed statement of holdings)
    async fn get_custody_proof(&self) -> CustodyResult<CustodyProof>;

    /// Health check — returns Ok if the custodian API is reachable
    async fn health_check(&self) -> CustodyResult<()>;
}

/// Factory function: build the configured custody adapter from environment variables.
///
/// Reads `CUSTODY_PROVIDER` (defaults to "mock"):
/// - `"fireblocks"` — Fireblocks production adapter
/// - `"mock"`       — deterministic mock (dev/test)
pub fn build_adapter_from_env() -> Box<dyn CustodyAdapter> {
    let provider = std::env::var("CUSTODY_PROVIDER")
        .unwrap_or_else(|_| "mock".to_string())
        .to_lowercase();

    match provider.as_str() {
        "fireblocks" => {
            let api_key = std::env::var("FIREBLOCKS_API_KEY")
                .expect("FIREBLOCKS_API_KEY required when CUSTODY_PROVIDER=fireblocks");
            let api_secret = std::env::var("FIREBLOCKS_API_SECRET")
                .expect("FIREBLOCKS_API_SECRET required when CUSTODY_PROVIDER=fireblocks");
            let base_url = std::env::var("FIREBLOCKS_BASE_URL")
                .unwrap_or_else(|_| "https://api.fireblocks.io".to_string());

            tracing::info!("Initializing Fireblocks custody adapter");
            Box::new(fireblocks::FireblocksAdapter::new(api_key, api_secret, base_url))
        }
        "bitgo" => {
            let api_key = std::env::var("BITGO_API_KEY")
                .expect("BITGO_API_KEY required when CUSTODY_PROVIDER=bitgo");
            let wallet_id = std::env::var("BITGO_WALLET_ID")
                .expect("BITGO_WALLET_ID required when CUSTODY_PROVIDER=bitgo");
            let base_url = std::env::var("BITGO_BASE_URL")
                .unwrap_or_else(|_| "https://app.bitgo.com".to_string());

            tracing::info!("Initializing BitGo custody adapter");
            Box::new(bitgo::BitGoAdapter::new(api_key, wallet_id, base_url))
        }
        _ => {
            tracing::info!("Using mock custody adapter (CUSTODY_PROVIDER={})", provider);
            Box::new(mock::MockAdapter::new())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reserve_balance_fields() {
        let balance = ReserveBalance {
            currency: "EUR".to_string(),
            total_balance: Decimal::from(1_000_000),
            available_balance: Decimal::from(950_000),
            locked_balance: Decimal::from(50_000),
            snapshot_at: Utc::now(),
            custodian: "MockAdapter".to_string(),
        };
        assert_eq!(balance.total_balance, balance.available_balance + balance.locked_balance);
    }
}
