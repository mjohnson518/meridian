//! BitGo custody adapter.
//!
//! Integrates with the BitGo REST API to retrieve wallet balances and
//! bond holdings for Proof of Reserves.
//!
//! ## Authentication
//!
//! BitGo uses long-lived access tokens passed as Bearer tokens.
//!
//! ## Environment Variables
//!
//! ```text
//! BITGO_API_KEY         — Long-lived access token
//! BITGO_WALLET_ID       — Primary wallet ID for reserve tracking
//! BITGO_ENTERPRISE_ID   — Enterprise ID (optional, for multi-wallet)
//! BITGO_BASE_URL        — API base URL (default: https://app.bitgo.com)
//! ```

use super::{BondHolding, CustodyAdapter, CustodyError, CustodyProof, CustodyResult, ReserveBalance};
use chrono::{Duration, Utc};
use reqwest::header::{HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use rust_decimal::Decimal;
use rust_decimal::prelude::FromStr;
use serde::Deserialize;
use uuid::Uuid;

/// BitGo wallet balance response
#[derive(Debug, Deserialize)]
struct BitGoWallet {
    #[serde(rename = "balanceString", default)]
    balance_string: String,
    #[serde(rename = "confirmedBalanceString", default)]
    confirmed_balance_string: String,
    coin: String,
}

/// BitGo API client
pub struct BitGoAdapter {
    api_key: String,
    wallet_id: String,
    base_url: String,
    http: reqwest::Client,
}

impl BitGoAdapter {
    pub fn new(api_key: String, wallet_id: String, base_url: String) -> Self {
        let http = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("Failed to build HTTP client");

        Self { api_key, wallet_id, base_url, http }
    }

    fn auth_header(&self) -> CustodyResult<HeaderValue> {
        HeaderValue::from_str(&format!("Bearer {}", self.api_key))
            .map_err(|e| CustodyError::Auth(e.to_string()))
    }

    async fn get_wallet(&self, wallet_id: &str) -> CustodyResult<BitGoWallet> {
        let url = format!("{}/api/v2/wallet/{}", self.base_url, wallet_id);
        let response = self.http
            .get(&url)
            .header(AUTHORIZATION, self.auth_header()?)
            .header(CONTENT_TYPE, "application/json")
            .send()
            .await
            .map_err(|e| CustodyError::Http(e.to_string()))?;

        if !response.status().is_success() {
            return Err(CustodyError::Api {
                status: response.status().as_u16(),
                message: response.text().await.unwrap_or_default(),
            });
        }

        response.json::<BitGoWallet>().await
            .map_err(|e| CustodyError::InvalidResponse(e.to_string()))
    }
}

#[async_trait::async_trait]
impl CustodyAdapter for BitGoAdapter {
    fn provider_name(&self) -> &str {
        "BitGo"
    }

    async fn get_reserve_balance(&self, currency: &str) -> CustodyResult<ReserveBalance> {
        let wallet = self.get_wallet(&self.wallet_id).await?;

        // BitGo returns balances in smallest unit (satoshis for BTC, wei for ETH, etc.)
        // For fiat-backed stablecoins, balance is in micro-units (6 decimal places)
        let raw_balance = Decimal::from_str(&wallet.balance_string).unwrap_or_default();
        let divisor = Decimal::from(1_000_000); // 6 decimal places
        let total = raw_balance / divisor;

        let raw_confirmed = Decimal::from_str(&wallet.confirmed_balance_string).unwrap_or_default();
        let available = raw_confirmed / divisor;
        let locked = total - available;

        Ok(ReserveBalance {
            currency: currency.to_uppercase(),
            total_balance: total,
            available_balance: available,
            locked_balance: locked.max(Decimal::ZERO),
            snapshot_at: Utc::now(),
            custodian: self.provider_name().to_string(),
        })
    }

    async fn get_bond_holdings(&self) -> CustodyResult<Vec<BondHolding>> {
        // BitGo is a digital asset custodian, not a securities custodian.
        // Bond holdings integration requires Clearstream/Euroclear connectivity.
        tracing::warn!(
            "BitGo bond holdings not yet implemented — \
             integrate with securities custodian (Clearstream/Euroclear)"
        );
        Ok(vec![])
    }

    async fn get_total_value_usd(&self) -> CustodyResult<Decimal> {
        let wallet = self.get_wallet(&self.wallet_id).await?;

        let raw_balance = Decimal::from_str(&wallet.balance_string).unwrap_or_default();
        let divisor = Decimal::from(1_000_000);
        let balance = raw_balance / divisor;

        // Approximate conversion for common coins
        let usd_value = match wallet.coin.to_uppercase().as_str() {
            "USDC" | "USDT" | "BUSD" => balance,
            // EUR stablecoins at approximate EUR/USD = 1.08
            c if c.starts_with("EUR") => balance * Decimal::from_str("1.08").unwrap_or(Decimal::ONE),
            // GBP at approximate GBP/USD = 1.27
            c if c.starts_with("GBP") => balance * Decimal::from_str("1.27").unwrap_or(Decimal::ONE),
            _ => balance, // Default: treat as USD-equivalent
        };

        Ok(usd_value)
    }

    async fn get_custody_proof(&self) -> CustodyResult<CustodyProof> {
        let now = Utc::now();
        let total_usd = self.get_total_value_usd().await?;

        Ok(CustodyProof {
            id: Uuid::new_v4(),
            custodian: self.provider_name().to_string(),
            total_value_usd: total_usd,
            covered_assets: vec![self.wallet_id.clone()],
            signature: None,
            statement_hash: None,
            issued_at: now,
            expires_at: now + Duration::hours(24),
        })
    }

    async fn health_check(&self) -> CustodyResult<()> {
        let url = format!("{}/api/v2/ping", self.base_url);
        let response = self.http
            .get(&url)
            .header(AUTHORIZATION, self.auth_header()?)
            .send()
            .await
            .map_err(|e| CustodyError::Http(e.to_string()))?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(CustodyError::Api {
                status: response.status().as_u16(),
                message: "BitGo health check failed".to_string(),
            })
        }
    }
}
