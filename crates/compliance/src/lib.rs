//! # Meridian Compliance Module
//!
//! Regulatory compliance for GENIUS Act, MiCA, and multi-jurisdiction support.
//! This module provides the foundation for:
//! - Customer identification (KYC)
//! - Risk assessment and scoring
//! - Sanction screening (OFAC, EU, UN)
//! - Transaction monitoring
//! - Regulatory reporting

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

pub mod kyc;
pub mod risk;
pub mod sanctions;
pub mod monitoring;

/// Compliance-related errors
#[derive(Error, Debug)]
pub enum ComplianceError {
    #[error("KYC verification failed: {0}")]
    KycFailed(String),

    #[error("Sanction screening failed: {0}")]
    SanctionCheckFailed(String),

    #[error("Risk assessment failed: {0}")]
    RiskAssessmentFailed(String),

    #[error("Transaction blocked: {0}")]
    TransactionBlocked(String),

    #[error("Jurisdiction not supported: {0}")]
    UnsupportedJurisdiction(String),

    #[error("Document expired: {0}")]
    DocumentExpired(String),

    #[error("External service error: {0}")]
    ExternalServiceError(String),
}

/// Result type for compliance operations
pub type ComplianceResult<T> = Result<T, ComplianceError>;

/// Supported regulatory frameworks
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RegulatoryFramework {
    /// US GENIUS Act (Guiding and Establishing National Innovation for US Stablecoins)
    GeniusAct,
    /// EU Markets in Crypto-Assets Regulation
    MiCA,
    /// UK Financial Conduct Authority
    FcaUk,
    /// Switzerland FINMA
    FinmaCh,
    /// Singapore MAS
    MasSg,
    /// Japan FSA
    FsaJp,
}

/// Customer compliance status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComplianceStatus {
    /// Initial state, no verification started
    NotStarted,
    /// KYC documents submitted, awaiting review
    Pending,
    /// KYC approved, customer can transact
    Approved,
    /// KYC rejected, customer cannot transact
    Rejected,
    /// Account suspended due to compliance concerns
    Suspended,
    /// Periodic review required
    ReviewRequired,
}

/// Risk level classification per FATF guidelines
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum RiskLevel {
    Low = 1,
    Medium = 2,
    High = 3,
    Prohibited = 4,
}

/// Customer compliance record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerCompliance {
    /// Unique customer identifier
    pub customer_id: Uuid,
    /// Current compliance status
    pub status: ComplianceStatus,
    /// Applicable regulatory frameworks
    pub frameworks: Vec<RegulatoryFramework>,
    /// Overall risk score (0-100)
    pub risk_score: u8,
    /// Risk level classification
    pub risk_level: RiskLevel,
    /// KYC verification timestamp
    pub kyc_verified_at: Option<DateTime<Utc>>,
    /// KYC expiration timestamp
    pub kyc_expires_at: Option<DateTime<Utc>>,
    /// Country of residence (ISO 3166-1 alpha-2)
    pub country_code: String,
    /// Whether enhanced due diligence is required
    pub edd_required: bool,
    /// Last review timestamp
    pub last_review_at: DateTime<Utc>,
    /// Next scheduled review
    pub next_review_at: DateTime<Utc>,
}

impl CustomerCompliance {
    /// Create a new customer compliance record
    pub fn new(customer_id: Uuid, country_code: String) -> Self {
        let now = Utc::now();
        Self {
            customer_id,
            status: ComplianceStatus::NotStarted,
            frameworks: vec![],
            risk_score: 0,
            risk_level: RiskLevel::Medium, // Default to medium until assessed
            kyc_verified_at: None,
            kyc_expires_at: None,
            country_code,
            edd_required: false,
            last_review_at: now,
            next_review_at: now + chrono::Duration::days(365), // Annual review default
        }
    }

    /// Check if KYC verification has expired
    pub fn is_kyc_expired(&self) -> bool {
        match self.kyc_expires_at {
            Some(expires) => Utc::now() > expires,
            None => true, // No KYC means it's effectively expired
        }
    }

    /// Check if customer can transact
    pub fn can_transact(&self) -> bool {
        self.status == ComplianceStatus::Approved && !self.is_kyc_expired()
    }

    /// Check if periodic review is due
    pub fn is_review_due(&self) -> bool {
        Utc::now() > self.next_review_at
    }
}

/// Transaction compliance check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionCheck {
    /// Transaction identifier
    pub transaction_id: String,
    /// Whether transaction is approved
    pub approved: bool,
    /// Risk score for this transaction (0-100)
    pub risk_score: u8,
    /// Flags raised during screening
    pub flags: Vec<ComplianceFlag>,
    /// Timestamp of check
    pub checked_at: DateTime<Utc>,
    /// Required actions if any
    pub required_actions: Vec<String>,
}

/// Compliance flags that can be raised
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComplianceFlag {
    /// Transaction exceeds daily limit
    DailyLimitExceeded,
    /// Transaction exceeds single transaction limit
    SingleTransactionLimitExceeded,
    /// High risk jurisdiction involved
    HighRiskJurisdiction,
    /// Potential structuring detected
    StructuringDetected,
    /// PEP (Politically Exposed Person) involved
    PepInvolved,
    /// Sanction list match
    SanctionMatch,
    /// Unusual activity pattern
    UnusualPattern,
    /// Velocity limit exceeded
    VelocityExceeded,
}

/// Compliance service configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceConfig {
    /// Enable/disable compliance checks (for testing)
    pub enabled: bool,
    /// Sanction list provider URL
    pub sanctions_api_url: Option<String>,
    /// KYC provider URL
    pub kyc_api_url: Option<String>,
    /// Default transaction limit (in base currency units)
    pub default_daily_limit: u64,
    /// Default single transaction limit
    pub default_single_limit: u64,
    /// Countries on prohibited list
    pub prohibited_countries: Vec<String>,
    /// Countries requiring enhanced due diligence
    pub high_risk_countries: Vec<String>,
}

impl Default for ComplianceConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            sanctions_api_url: None,
            kyc_api_url: None,
            default_daily_limit: 1_000_000, // $10,000.00 in cents
            default_single_limit: 300_000,  // $3,000.00 in cents
            prohibited_countries: vec![
                "KP".to_string(), // North Korea
                "IR".to_string(), // Iran
                "SY".to_string(), // Syria
                "CU".to_string(), // Cuba (OFAC)
            ],
            high_risk_countries: vec![
                "RU".to_string(), // Russia
                "BY".to_string(), // Belarus
                "MM".to_string(), // Myanmar
                "VE".to_string(), // Venezuela
            ],
        }
    }
}

/// Main compliance service
pub struct ComplianceService {
    config: ComplianceConfig,
}

impl ComplianceService {
    /// Create a new compliance service
    pub fn new(config: ComplianceConfig) -> Self {
        Self { config }
    }

    /// Create with default configuration
    pub fn default_service() -> Self {
        Self::new(ComplianceConfig::default())
    }

    /// Check if compliance is enabled
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// Check if a country is prohibited
    pub fn is_country_prohibited(&self, country_code: &str) -> bool {
        self.config.prohibited_countries.contains(&country_code.to_uppercase())
    }

    /// Check if a country requires enhanced due diligence
    pub fn requires_edd(&self, country_code: &str) -> bool {
        self.config.high_risk_countries.contains(&country_code.to_uppercase())
    }

    /// Perform pre-transaction compliance check
    pub fn check_transaction(
        &self,
        customer: &CustomerCompliance,
        amount_cents: u64,
        transaction_id: &str,
    ) -> ComplianceResult<TransactionCheck> {
        if !self.config.enabled {
            return Ok(TransactionCheck {
                transaction_id: transaction_id.to_string(),
                approved: true,
                risk_score: 0,
                flags: vec![],
                checked_at: Utc::now(),
                required_actions: vec![],
            });
        }

        let mut flags = Vec::new();
        let mut risk_score: u8 = customer.risk_score;

        // Check customer can transact
        if !customer.can_transact() {
            return Err(ComplianceError::TransactionBlocked(
                "Customer not approved for transactions".to_string(),
            ));
        }

        // Check country
        if self.is_country_prohibited(&customer.country_code) {
            return Err(ComplianceError::TransactionBlocked(format!(
                "Country {} is prohibited",
                customer.country_code
            )));
        }

        // Check transaction limits
        if amount_cents > self.config.default_single_limit {
            flags.push(ComplianceFlag::SingleTransactionLimitExceeded);
            risk_score = risk_score.saturating_add(20);
        }

        // Check EDD requirement
        if customer.edd_required {
            risk_score = risk_score.saturating_add(15);
        }

        // High risk jurisdiction
        if self.requires_edd(&customer.country_code) {
            flags.push(ComplianceFlag::HighRiskJurisdiction);
            risk_score = risk_score.saturating_add(25);
        }

        // Determine if transaction should be blocked
        let approved = risk_score < 80 && flags.iter().all(|f| *f != ComplianceFlag::SanctionMatch);

        let required_actions = if risk_score >= 60 {
            vec!["Manual review required".to_string()]
        } else {
            vec![]
        };

        Ok(TransactionCheck {
            transaction_id: transaction_id.to_string(),
            approved,
            risk_score: risk_score.min(100),
            flags,
            checked_at: Utc::now(),
            required_actions,
        })
    }

    /// Get applicable regulatory frameworks for a jurisdiction
    pub fn get_frameworks(&self, country_code: &str) -> Vec<RegulatoryFramework> {
        match country_code.to_uppercase().as_str() {
            "US" => vec![RegulatoryFramework::GeniusAct],
            // EU member states
            "DE" | "FR" | "IT" | "ES" | "NL" | "BE" | "AT" | "PT" | "IE" | "GR" | "FI" | "SE"
            | "PL" | "CZ" | "RO" | "HU" | "BG" | "SK" | "HR" | "SI" | "LT" | "LV" | "EE" | "CY"
            | "LU" | "MT" | "DK" => vec![RegulatoryFramework::MiCA],
            "GB" => vec![RegulatoryFramework::FcaUk],
            "CH" => vec![RegulatoryFramework::FinmaCh],
            "SG" => vec![RegulatoryFramework::MasSg],
            "JP" => vec![RegulatoryFramework::FsaJp],
            _ => vec![], // No specific framework
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_customer_compliance_creation() {
        let customer = CustomerCompliance::new(Uuid::new_v4(), "US".to_string());
        assert_eq!(customer.status, ComplianceStatus::NotStarted);
        assert!(!customer.can_transact());
    }

    #[test]
    fn test_kyc_expiration() {
        let mut customer = CustomerCompliance::new(Uuid::new_v4(), "US".to_string());
        assert!(customer.is_kyc_expired()); // No KYC means expired

        customer.status = ComplianceStatus::Approved;
        customer.kyc_verified_at = Some(Utc::now());
        customer.kyc_expires_at = Some(Utc::now() + chrono::Duration::days(365));
        assert!(!customer.is_kyc_expired());
    }

    #[test]
    fn test_prohibited_countries() {
        let service = ComplianceService::default_service();
        assert!(service.is_country_prohibited("KP"));
        assert!(service.is_country_prohibited("IR"));
        assert!(!service.is_country_prohibited("US"));
        assert!(!service.is_country_prohibited("DE"));
    }

    #[test]
    fn test_edd_countries() {
        let service = ComplianceService::default_service();
        assert!(service.requires_edd("RU"));
        assert!(service.requires_edd("BY"));
        assert!(!service.requires_edd("US"));
    }

    #[test]
    fn test_regulatory_frameworks() {
        let service = ComplianceService::default_service();
        assert_eq!(service.get_frameworks("US"), vec![RegulatoryFramework::GeniusAct]);
        assert_eq!(service.get_frameworks("DE"), vec![RegulatoryFramework::MiCA]);
        assert_eq!(service.get_frameworks("GB"), vec![RegulatoryFramework::FcaUk]);
    }

    #[test]
    fn test_transaction_check_approved() {
        let service = ComplianceService::default_service();
        let mut customer = CustomerCompliance::new(Uuid::new_v4(), "US".to_string());
        customer.status = ComplianceStatus::Approved;
        customer.kyc_verified_at = Some(Utc::now());
        customer.kyc_expires_at = Some(Utc::now() + chrono::Duration::days(365));

        let result = service.check_transaction(&customer, 100_00, "tx_123");
        assert!(result.is_ok());
        let check = result.unwrap();
        assert!(check.approved);
    }

    #[test]
    fn test_transaction_check_prohibited_country() {
        let service = ComplianceService::default_service();
        let mut customer = CustomerCompliance::new(Uuid::new_v4(), "KP".to_string());
        customer.status = ComplianceStatus::Approved;
        customer.kyc_verified_at = Some(Utc::now());
        customer.kyc_expires_at = Some(Utc::now() + chrono::Duration::days(365));

        let result = service.check_transaction(&customer, 100_00, "tx_123");
        assert!(result.is_err());
    }

    #[test]
    fn test_disabled_compliance() {
        let config = ComplianceConfig {
            enabled: false,
            ..Default::default()
        };
        let service = ComplianceService::new(config);
        let customer = CustomerCompliance::new(Uuid::new_v4(), "KP".to_string());

        // Even prohibited country passes when compliance is disabled
        let result = service.check_transaction(&customer, 100_00, "tx_123");
        assert!(result.is_ok());
        assert!(result.unwrap().approved);
    }
}
