//! Mock custody adapter for development and testing.
//!
//! Returns deterministic data based on static bond holdings.
//! All values are realistic EUR-denominated sovereign bonds.

use super::{BondHolding, CustodyAdapter, CustodyProof, CustodyResult, ReserveBalance};
use chrono::{Duration, Utc};
use rust_decimal::Decimal;
use rust_decimal::prelude::FromStr;
use uuid::Uuid;

/// Deterministic mock custody adapter.
///
/// Holdings represent a realistic EUR-denominated portfolio of sovereign bonds
/// from eurozone issuers. Used in dev and test environments.
pub struct MockAdapter {
    /// Override for total USD value (useful in tests)
    total_value_override: Option<Decimal>,
}

impl MockAdapter {
    pub fn new() -> Self {
        Self { total_value_override: None }
    }

    /// Create a mock with a specific total value (for testing reserve ratio scenarios)
    pub fn with_total_value(value: Decimal) -> Self {
        Self { total_value_override: Some(value) }
    }

    fn mock_bonds() -> Vec<BondHolding> {
        let now = Utc::now();
        vec![
            BondHolding {
                id: Uuid::new_v4(),
                isin: "DE0001102580".to_string(),
                name: "German Federal Bond 2.30% 2033".to_string(),
                currency: "EUR".to_string(),
                face_value: Decimal::from_str("5_000_000").unwrap_or_default(),
                market_value: Decimal::from_str("4_875_000").unwrap_or_default(),
                yield_to_maturity: Decimal::from_str("0.0250").unwrap_or_default(),
                maturity_date: now + Duration::days(365 * 9),
                custodian_account_id: "mock-vault-001".to_string(),
                valued_at: now,
            },
            BondHolding {
                id: Uuid::new_v4(),
                isin: "FR0013508470".to_string(),
                name: "OAT France 0.75% 2028".to_string(),
                currency: "EUR".to_string(),
                face_value: Decimal::from_str("3_000_000").unwrap_or_default(),
                market_value: Decimal::from_str("2_820_000").unwrap_or_default(),
                yield_to_maturity: Decimal::from_str("0.0310").unwrap_or_default(),
                maturity_date: now + Duration::days(365 * 4),
                custodian_account_id: "mock-vault-001".to_string(),
                valued_at: now,
            },
            BondHolding {
                id: Uuid::new_v4(),
                isin: "IT0005421703".to_string(),
                name: "BTP Italy 1.65% 2032".to_string(),
                currency: "EUR".to_string(),
                face_value: Decimal::from_str("2_000_000").unwrap_or_default(),
                market_value: Decimal::from_str("1_820_000").unwrap_or_default(),
                yield_to_maturity: Decimal::from_str("0.0420").unwrap_or_default(),
                maturity_date: now + Duration::days(365 * 8),
                custodian_account_id: "mock-vault-002".to_string(),
                valued_at: now,
            },
        ]
    }
}

impl Default for MockAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl CustodyAdapter for MockAdapter {
    fn provider_name(&self) -> &str {
        "MockAdapter"
    }

    async fn get_reserve_balance(&self, currency: &str) -> CustodyResult<ReserveBalance> {
        let total = match currency.to_uppercase().as_str() {
            "EUR" => Decimal::from(9_515_000), // Sum of mock bond market values
            "USD" => Decimal::from(10_000_000),
            "GBP" => Decimal::from(3_000_000),
            _ => Decimal::ZERO,
        };

        Ok(ReserveBalance {
            currency: currency.to_uppercase(),
            total_balance: total,
            available_balance: total,
            locked_balance: Decimal::ZERO,
            snapshot_at: Utc::now(),
            custodian: self.provider_name().to_string(),
        })
    }

    async fn get_bond_holdings(&self) -> CustodyResult<Vec<BondHolding>> {
        Ok(Self::mock_bonds())
    }

    async fn get_total_value_usd(&self) -> CustodyResult<Decimal> {
        if let Some(override_value) = self.total_value_override {
            return Ok(override_value);
        }
        // Sum market values of all bonds and convert EUR -> USD at approximate rate
        let eur_total: Decimal = Self::mock_bonds().iter().map(|b| b.market_value).sum();
        // Approximate EUR/USD = 1.08
        Ok(eur_total * Decimal::from_str("1.08").unwrap_or(Decimal::ONE))
    }

    async fn get_custody_proof(&self) -> CustodyResult<CustodyProof> {
        let now = Utc::now();
        let total_usd = self.get_total_value_usd().await?;
        let covered = Self::mock_bonds().iter().map(|b| b.isin.clone()).collect();

        Ok(CustodyProof {
            id: Uuid::new_v4(),
            custodian: self.provider_name().to_string(),
            total_value_usd: total_usd,
            covered_assets: covered,
            signature: Some("MOCK_SIGNATURE_NOT_VALID_IN_PRODUCTION".to_string()),
            statement_hash: Some("0000000000000000000000000000000000000000000000000000000000000000".to_string()),
            issued_at: now,
            expires_at: now + Duration::hours(24),
        })
    }

    async fn health_check(&self) -> CustodyResult<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_reserve_balance_eur() {
        let adapter = MockAdapter::new();
        let balance = adapter.get_reserve_balance("EUR").await.unwrap();
        assert_eq!(balance.currency, "EUR");
        assert!(balance.total_balance > Decimal::ZERO);
        assert_eq!(balance.total_balance, balance.available_balance + balance.locked_balance);
    }

    #[tokio::test]
    async fn test_mock_bond_holdings_non_empty() {
        let adapter = MockAdapter::new();
        let holdings = adapter.get_bond_holdings().await.unwrap();
        assert!(!holdings.is_empty(), "Mock should have bond holdings");
        // All holdings should have valid ISINs (12 characters)
        for h in &holdings {
            assert_eq!(h.isin.len(), 12, "ISIN should be 12 chars");
        }
    }

    #[tokio::test]
    async fn test_mock_total_value_positive() {
        let adapter = MockAdapter::new();
        let total = adapter.get_total_value_usd().await.unwrap();
        assert!(total > Decimal::ZERO);
    }

    #[tokio::test]
    async fn test_mock_total_value_override() {
        let adapter = MockAdapter::with_total_value(Decimal::from(500_000));
        let total = adapter.get_total_value_usd().await.unwrap();
        assert_eq!(total, Decimal::from(500_000));
    }

    #[tokio::test]
    async fn test_mock_custody_proof() {
        let adapter = MockAdapter::new();
        let proof = adapter.get_custody_proof().await.unwrap();
        assert!(proof.total_value_usd > Decimal::ZERO);
        assert!(!proof.covered_assets.is_empty());
        assert!(proof.expires_at > proof.issued_at);
    }

    #[tokio::test]
    async fn test_mock_health_check() {
        let adapter = MockAdapter::new();
        assert!(adapter.health_check().await.is_ok());
    }

    #[tokio::test]
    async fn test_unknown_currency_returns_zero() {
        let adapter = MockAdapter::new();
        let balance = adapter.get_reserve_balance("XYZ").await.unwrap();
        assert_eq!(balance.total_balance, Decimal::ZERO);
    }
}
