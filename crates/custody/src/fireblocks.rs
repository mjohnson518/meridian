//! Fireblocks custody adapter.
//!
//! Integrates with the Fireblocks REST API to retrieve vault balances,
//! supported assets, and transaction history for Proof of Reserves.
//!
//! ## Authentication
//!
//! Fireblocks uses API key + RSA private key JWT signing.
//! In production, the private key should come from HSM / secrets manager.
//!
//! ## Environment Variables
//!
//! ```text
//! FIREBLOCKS_API_KEY     — Your Fireblocks API key (UUID)
//! FIREBLOCKS_API_SECRET  — RSA private key PEM for request signing
//! FIREBLOCKS_BASE_URL    — API base URL (default: https://api.fireblocks.io)
//! FIREBLOCKS_VAULT_IDS   — Comma-separated vault IDs to include in reserve calculation
//! ```

use super::{BondHolding, CustodyAdapter, CustodyError, CustodyProof, CustodyResult, ReserveBalance};
use chrono::{Duration, Utc};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use rust_decimal::Decimal;
use rust_decimal::prelude::FromStr;
use serde::Deserialize;
use uuid::Uuid;

/// Fireblocks vault balance response (subset of API response)
#[derive(Debug, Deserialize)]
struct FireblocksVaultBalance {
    id: String,
    #[serde(default)]
    assets: Vec<FireblocksAsset>,
}

/// Asset balance within a vault
#[derive(Debug, Deserialize)]
struct FireblocksAsset {
    id: String,
    total: String,
    available: String,
    #[serde(default)]
    locked_amount: String,
}

/// Fireblocks API client for custody operations
pub struct FireblocksAdapter {
    api_key: String,
    /// RSA private key PEM for JWT signing (used in build_auth_header)
    #[allow(dead_code)]
    api_secret: String,
    base_url: String,
    http: reqwest::Client,
    /// Vault IDs to include in reserve totals (empty = all vaults)
    vault_ids: Vec<String>,
}

impl FireblocksAdapter {
    pub fn new(api_key: String, api_secret: String, base_url: String) -> Self {
        let vault_ids: Vec<String> = std::env::var("FIREBLOCKS_VAULT_IDS")
            .unwrap_or_default()
            .split(',')
            .filter(|s| !s.is_empty())
            .map(|s| s.trim().to_string())
            .collect();

        let http = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("Failed to build HTTP client");

        Self { api_key, api_secret, base_url, http, vault_ids }
    }

    /// Build a signed JWT for Fireblocks API authentication.
    ///
    /// In production, use a proper JWT library with RS256 signing.
    /// This is a placeholder — a real implementation should sign using
    /// the RSA private key from `api_secret`.
    fn build_auth_header(&self, _path: &str, _body: &str) -> CustodyResult<String> {
        // TODO(custody): Implement Fireblocks JWT signing with RS256.
        // The JWT requires: sub=api_key, nonce=uuid, iat=now, exp=now+30s,
        //   uri=path, bodyHash=sha256(body).
        // For now, return a placeholder that will be replaced with real signing.
        Err(CustodyError::Config(
            "Fireblocks JWT signing not yet implemented — requires RS256 library. \
             Set CUSTODY_PROVIDER=mock for testing.".to_string()
        ))
    }

    async fn get_vault_balances(&self) -> CustodyResult<Vec<FireblocksVaultBalance>> {
        let auth = self.build_auth_header("/v1/vault/accounts_paged", "")?;

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", auth))
                .map_err(|e| CustodyError::Auth(e.to_string()))?,
        );

        let url = format!("{}/v1/vault/accounts_paged", self.base_url);
        let response = self.http.get(&url).headers(headers).send().await
            .map_err(|e| CustodyError::Http(e.to_string()))?;

        if !response.status().is_success() {
            return Err(CustodyError::Api {
                status: response.status().as_u16(),
                message: response.text().await.unwrap_or_default(),
            });
        }

        #[derive(Deserialize)]
        struct PagedResponse {
            accounts: Vec<FireblocksVaultBalance>,
        }

        let body: PagedResponse = response.json().await
            .map_err(|e| CustodyError::InvalidResponse(e.to_string()))?;

        // Filter by configured vault IDs if specified
        if self.vault_ids.is_empty() {
            Ok(body.accounts)
        } else {
            Ok(body.accounts.into_iter()
                .filter(|v| self.vault_ids.contains(&v.id))
                .collect())
        }
    }
}

#[async_trait::async_trait]
impl CustodyAdapter for FireblocksAdapter {
    fn provider_name(&self) -> &str {
        "Fireblocks"
    }

    async fn get_reserve_balance(&self, currency: &str) -> CustodyResult<ReserveBalance> {
        let vaults = self.get_vault_balances().await?;

        // Map Fireblocks asset IDs to ISO currency codes
        // e.g., "EUR_TEST" -> "EUR", "USDC" -> "USD"
        let target_asset = format!("{}_TEST", currency.to_uppercase());
        let target_asset_prod = currency.to_uppercase();

        let mut total = Decimal::ZERO;
        let mut available = Decimal::ZERO;
        let mut locked = Decimal::ZERO;

        for vault in &vaults {
            for asset in &vault.assets {
                if asset.id == target_asset || asset.id == target_asset_prod {
                    total += Decimal::from_str(&asset.total).unwrap_or_default();
                    available += Decimal::from_str(&asset.available).unwrap_or_default();
                    locked += Decimal::from_str(&asset.locked_amount).unwrap_or_default();
                }
            }
        }

        Ok(ReserveBalance {
            currency: currency.to_uppercase(),
            total_balance: total,
            available_balance: available,
            locked_balance: locked,
            snapshot_at: Utc::now(),
            custodian: self.provider_name().to_string(),
        })
    }

    async fn get_bond_holdings(&self) -> CustodyResult<Vec<BondHolding>> {
        // Fireblocks is primarily a digital asset custodian, not a securities custodian.
        // Bond holdings would come from an integrated securities custodian (e.g., Clearstream).
        // This returns an empty list until securities custody integration is complete.
        tracing::warn!(
            "Fireblocks bond holdings not yet implemented — \
             integrate with securities custodian (Clearstream/Euroclear)"
        );
        Ok(vec![])
    }

    async fn get_total_value_usd(&self) -> CustodyResult<Decimal> {
        let vaults = self.get_vault_balances().await?;

        let mut total_usd = Decimal::ZERO;
        for vault in &vaults {
            for asset in &vault.assets {
                // This requires a price oracle to convert asset amounts to USD.
                // For now, only handle stablecoin-like assets where value ≈ face value.
                let balance = Decimal::from_str(&asset.total).unwrap_or_default();
                // USD-denominated assets: USDC, USDT, USD_TEST
                if asset.id.starts_with("USD") {
                    total_usd += balance;
                }
                // EUR assets at approximate EUR/USD = 1.08
                if asset.id.starts_with("EUR") {
                    total_usd += balance * Decimal::from_str("1.08").unwrap_or(Decimal::ONE);
                }
            }
        }

        Ok(total_usd)
    }

    async fn get_custody_proof(&self) -> CustodyResult<CustodyProof> {
        let now = Utc::now();
        let total_usd = self.get_total_value_usd().await?;

        Ok(CustodyProof {
            id: Uuid::new_v4(),
            custodian: self.provider_name().to_string(),
            total_value_usd: total_usd,
            covered_assets: vec![],
            signature: None, // Real implementation would include Fireblocks co-sign
            statement_hash: None,
            issued_at: now,
            expires_at: now + Duration::hours(24),
        })
    }

    async fn health_check(&self) -> CustodyResult<()> {
        // Check the Fireblocks API is reachable
        let url = format!("{}/v1/supported_assets", self.base_url);
        let response = self.http.get(&url)
            .header("X-API-Key", &self.api_key)
            .send()
            .await
            .map_err(|e| CustodyError::Http(e.to_string()))?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(CustodyError::Api {
                status: response.status().as_u16(),
                message: "Health check failed".to_string(),
            })
        }
    }
}
