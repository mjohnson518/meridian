//! # KYC (Know Your Customer) Module
//!
//! Customer identification and verification functionality.

use crate::{ComplianceError, ComplianceResult, ComplianceStatus};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Document types accepted for KYC verification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DocumentType {
    /// Government-issued passport
    Passport,
    /// National ID card
    NationalId,
    /// Driver's license
    DriversLicense,
    /// Residence permit
    ResidencePermit,
    /// Utility bill (for address verification)
    UtilityBill,
    /// Bank statement (for address verification)
    BankStatement,
}

/// KYC verification level
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum KycLevel {
    /// Basic verification (email + phone)
    Basic,
    /// Standard verification (ID document)
    Standard,
    /// Enhanced verification (ID + proof of address + selfie)
    Enhanced,
    /// Institutional verification (corporate documents)
    Institutional,
}

/// KYC document submission
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KycDocument {
    /// Document ID
    pub id: Uuid,
    /// Customer ID
    pub customer_id: Uuid,
    /// Document type
    pub document_type: DocumentType,
    /// Document number (encrypted)
    pub document_number_hash: String,
    /// Issuing country (ISO 3166-1 alpha-2)
    pub issuing_country: String,
    /// Document expiration date
    pub expires_at: Option<DateTime<Utc>>,
    /// Verification status
    pub verified: bool,
    /// Verification timestamp
    pub verified_at: Option<DateTime<Utc>>,
    /// Submission timestamp
    pub submitted_at: DateTime<Utc>,
}

/// KYC verification request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KycVerificationRequest {
    /// Customer ID
    pub customer_id: Uuid,
    /// Requested verification level
    pub level: KycLevel,
    /// First name
    pub first_name: String,
    /// Last name
    pub last_name: String,
    /// Date of birth (YYYY-MM-DD)
    pub date_of_birth: String,
    /// Nationality (ISO 3166-1 alpha-2)
    pub nationality: String,
    /// Country of residence (ISO 3166-1 alpha-2)
    pub country_of_residence: String,
    /// Address line 1
    pub address_line1: String,
    /// Address line 2 (optional)
    pub address_line2: Option<String>,
    /// City
    pub city: String,
    /// Postal code
    pub postal_code: String,
}

/// KYC verification result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KycVerificationResult {
    /// Customer ID
    pub customer_id: Uuid,
    /// Resulting status
    pub status: ComplianceStatus,
    /// Achieved verification level
    pub level: KycLevel,
    /// Rejection reason if applicable
    pub rejection_reason: Option<String>,
    /// Verification timestamp
    pub verified_at: DateTime<Utc>,
    /// Expiration timestamp (typically 1 year)
    pub expires_at: DateTime<Utc>,
}

/// KYC service interface
pub struct KycService {
    /// External KYC provider URL (optional)
    provider_url: Option<String>,
}

impl KycService {
    /// Create a new KYC service
    pub fn new(provider_url: Option<String>) -> Self {
        Self { provider_url }
    }

    /// Submit KYC verification request
    ///
    /// In production, this would call an external KYC provider like:
    /// - Jumio
    /// - Onfido
    /// - Veriff
    /// - Sumsub
    pub async fn submit_verification(
        &self,
        request: &KycVerificationRequest,
    ) -> ComplianceResult<KycVerificationResult> {
        // Validate basic fields
        if request.first_name.is_empty() || request.last_name.is_empty() {
            return Err(ComplianceError::KycFailed("Name fields required".to_string()));
        }

        // In production: call external KYC provider
        // For now: return pending status
        let now = Utc::now();
        Ok(KycVerificationResult {
            customer_id: request.customer_id,
            status: ComplianceStatus::Pending,
            level: request.level,
            rejection_reason: None,
            verified_at: now,
            expires_at: now + chrono::Duration::days(365),
        })
    }

    /// Check verification status
    pub async fn check_status(&self, customer_id: Uuid) -> ComplianceResult<ComplianceStatus> {
        // In production: check with KYC provider
        // For now: return pending
        let _ = customer_id;
        Ok(ComplianceStatus::Pending)
    }

    /// Validate document expiration
    pub fn is_document_valid(&self, document: &KycDocument) -> bool {
        if !document.verified {
            return false;
        }

        match document.expires_at {
            Some(expires) => Utc::now() < expires,
            None => true, // No expiration means valid
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kyc_levels_ordering() {
        assert!(KycLevel::Basic < KycLevel::Standard);
        assert!(KycLevel::Standard < KycLevel::Enhanced);
        assert!(KycLevel::Enhanced < KycLevel::Institutional);
    }

    #[test]
    fn test_document_validity() {
        let service = KycService::new(None);

        let valid_doc = KycDocument {
            id: Uuid::new_v4(),
            customer_id: Uuid::new_v4(),
            document_type: DocumentType::Passport,
            document_number_hash: "hash".to_string(),
            issuing_country: "US".to_string(),
            expires_at: Some(Utc::now() + chrono::Duration::days(365)),
            verified: true,
            verified_at: Some(Utc::now()),
            submitted_at: Utc::now(),
        };

        assert!(service.is_document_valid(&valid_doc));

        let expired_doc = KycDocument {
            expires_at: Some(Utc::now() - chrono::Duration::days(1)),
            ..valid_doc.clone()
        };

        assert!(!service.is_document_valid(&expired_doc));

        let unverified_doc = KycDocument {
            verified: false,
            ..valid_doc
        };

        assert!(!service.is_document_valid(&unverified_doc));
    }
}
