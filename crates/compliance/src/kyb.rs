//! # KYB (Know Your Business) Module
//!
//! Corporate entity verification for institutional onboarding.
//! Required for MiCA-regulated stablecoin issuance — individual KYC is
//! insufficient for institutional clients; the legal entity itself must be
//! verified against incorporation records, UBO declarations, and LEI codes.
//!
//! ## Required Documents
//!
//! - Certificate of Incorporation
//! - Articles of Association / Memorandum
//! - UBO Declaration (≥25% beneficial owners)
//! - Board Resolution authorizing account opening
//! - LEI Code (ISO 17442) — mandatory for MiCA Art. 45

use crate::{ComplianceError, ComplianceResult, ComplianceStatus};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Corporate document types required for KYB
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum KybDocumentType {
    /// Certificate of Incorporation issued by registry authority
    CertificateOfIncorporation,
    /// Articles of Association / Memorandum of Association
    ArticlesOfAssociation,
    /// UBO Declaration listing all beneficial owners ≥25%
    UboDeclaration,
    /// Board Resolution authorizing account/service use
    BoardResolution,
    /// Register of Members / Shareholders
    ShareRegister,
    /// Latest filed financial statements (past 2 years)
    FinancialStatements,
    /// Source of Funds declaration
    SourceOfFunds,
    /// Regulatory license (if applicable, e.g., MiFID, e-money license)
    RegulatoryLicense,
}

/// Beneficial owner (UBO) record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UboRecord {
    /// Full legal name
    pub full_name: String,
    /// Date of birth (YYYY-MM-DD)
    pub date_of_birth: String,
    /// Nationality (ISO 3166-1 alpha-2)
    pub nationality: String,
    /// Ownership percentage (0–100)
    pub ownership_pct: Decimal,
    /// Whether this UBO controls via direct shareholding
    pub is_direct: bool,
    /// KYC status of this individual
    pub kyc_status: ComplianceStatus,
}

/// KYB application status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KybStatus {
    /// Documents pending submission
    PendingDocuments,
    /// All documents submitted, under review
    UnderReview,
    /// Additional information requested
    AdditionalInfoRequired,
    /// Approved — entity can proceed
    Approved,
    /// Rejected — see rejection_reason
    Rejected,
    /// Suspended — previously approved but suspended
    Suspended,
}

/// Submitted corporate document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KybDocument {
    pub id: Uuid,
    pub document_type: KybDocumentType,
    /// SHA-256 hash of document bytes (for integrity verification)
    pub content_hash: String,
    /// Issuing authority (e.g., "Companies House", "Kammergericht Berlin")
    pub issuing_authority: Option<String>,
    /// Document date (when issued/signed)
    pub document_date: Option<String>,
    pub submitted_at: DateTime<Utc>,
    pub verified: bool,
}

/// KYB application for a corporate entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KybApplication {
    pub id: Uuid,
    /// Tenant ID this application belongs to
    pub tenant_id: Uuid,
    /// Legal entity name (as registered)
    pub legal_name: String,
    /// Trading name (if different)
    pub trading_name: Option<String>,
    /// Country of incorporation (ISO 3166-1 alpha-2)
    pub jurisdiction: String,
    /// Company registration number
    pub registration_number: String,
    /// LEI Code (ISO 17442 — 20 alphanumeric chars, MiCA Art. 45 mandatory)
    pub lei_code: Option<String>,
    /// Registered address
    pub registered_address: String,
    /// Industry / business description
    pub business_description: String,
    /// Ultimate Beneficial Owners (≥25% threshold)
    pub ubos: Vec<UboRecord>,
    /// Submitted corporate documents
    pub documents: Vec<KybDocument>,
    pub status: KybStatus,
    pub rejection_reason: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Validation error for a KYB application
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KybValidationError {
    pub field: String,
    pub message: String,
}

/// KYB verification service
pub struct KybService;

impl KybService {
    pub fn new() -> Self {
        Self
    }

    /// Validate a KYB application for completeness before submission.
    pub fn validate(&self, app: &KybApplication) -> Vec<KybValidationError> {
        let mut errors = Vec::new();

        if app.legal_name.trim().is_empty() {
            errors.push(KybValidationError {
                field: "legal_name".to_string(),
                message: "Legal name is required".to_string(),
            });
        }

        if app.registration_number.trim().is_empty() {
            errors.push(KybValidationError {
                field: "registration_number".to_string(),
                message: "Company registration number is required".to_string(),
            });
        }

        // Validate LEI format: 20 alphanumeric characters
        if let Some(lei) = &app.lei_code {
            if lei.len() != 20 || !lei.chars().all(|c| c.is_alphanumeric()) {
                errors.push(KybValidationError {
                    field: "lei_code".to_string(),
                    message: "LEI must be exactly 20 alphanumeric characters (ISO 17442)".to_string(),
                });
            }
        }

        // Validate UBOs: at least one required, total ownership must be accountable
        if app.ubos.is_empty() {
            errors.push(KybValidationError {
                field: "ubos".to_string(),
                message: "At least one UBO (≥25% ownership) is required".to_string(),
            });
        }

        for (i, ubo) in app.ubos.iter().enumerate() {
            if ubo.ownership_pct < Decimal::from(25) {
                errors.push(KybValidationError {
                    field: format!("ubos[{}].ownership_pct", i),
                    message: "UBO ownership must be ≥25%".to_string(),
                });
            }
            if ubo.full_name.trim().is_empty() {
                errors.push(KybValidationError {
                    field: format!("ubos[{}].full_name", i),
                    message: "UBO full name is required".to_string(),
                });
            }
        }

        // Required document types
        let submitted_types: std::collections::HashSet<String> = app.documents.iter()
            .map(|d| format!("{:?}", d.document_type))
            .collect();

        for required in &[
            KybDocumentType::CertificateOfIncorporation,
            KybDocumentType::ArticlesOfAssociation,
            KybDocumentType::UboDeclaration,
            KybDocumentType::BoardResolution,
        ] {
            let key = format!("{:?}", required);
            if !submitted_types.contains(&key) {
                errors.push(KybValidationError {
                    field: "documents".to_string(),
                    message: format!("{:?} document is required", required),
                });
            }
        }

        errors
    }

    /// Submit a KYB application.
    ///
    /// In production, this would:
    /// 1. Call a KYB provider (e.g., Comply Advantage, Onfido Business)
    /// 2. Cross-reference the LEI code with GLEIF database
    /// 3. Verify company registry entries via Companies House API (UK), Handelsregister (DE), etc.
    pub async fn submit(
        &self,
        app: &KybApplication,
    ) -> ComplianceResult<KybStatus> {
        let errors = self.validate(app);
        if !errors.is_empty() {
            return Err(ComplianceError::KycFailed(
                errors.iter().map(|e| e.message.as_str()).collect::<Vec<_>>().join("; ")
            ));
        }

        // If LEI provided, validate with GLEIF (production: call https://api.gleif.org/api/v1/lei-records)
        if let Some(lei) = &app.lei_code {
            tracing::info!(lei, legal_name = %app.legal_name, "KYB submitted with LEI");
        } else {
            tracing::warn!(legal_name = %app.legal_name, "KYB submitted without LEI code — required for MiCA Art. 45");
        }

        // Return UnderReview — a webhook from the KYB provider will update to Approved/Rejected
        Ok(KybStatus::UnderReview)
    }
}

impl Default for KybService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_app(ubos: Vec<UboRecord>, docs: Vec<KybDocument>) -> KybApplication {
        KybApplication {
            id: Uuid::new_v4(),
            tenant_id: Uuid::new_v4(),
            legal_name: "Meridian Issuer GmbH".to_string(),
            trading_name: None,
            jurisdiction: "DE".to_string(),
            registration_number: "HRB123456".to_string(),
            lei_code: Some("5493001KJTIIGC8Y1R12".to_string()),
            registered_address: "Unter den Linden 1, Berlin".to_string(),
            business_description: "Stablecoin issuance and management".to_string(),
            ubos,
            documents: docs,
            status: KybStatus::PendingDocuments,
            rejection_reason: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn test_lei_validation() {
        let service = KybService::new();
        let mut app = make_app(
            vec![UboRecord {
                full_name: "Hans Mueller".to_string(),
                date_of_birth: "1975-06-15".to_string(),
                nationality: "DE".to_string(),
                ownership_pct: Decimal::from(100),
                is_direct: true,
                kyc_status: ComplianceStatus::Approved,
            }],
            vec![],
        );

        // Invalid LEI — too short
        app.lei_code = Some("TOOSHORT".to_string());
        let errors = service.validate(&app);
        assert!(errors.iter().any(|e| e.field == "lei_code"));
    }

    #[test]
    fn test_missing_required_docs() {
        let service = KybService::new();
        let app = make_app(
            vec![UboRecord {
                full_name: "Test Owner".to_string(),
                date_of_birth: "1980-01-01".to_string(),
                nationality: "GB".to_string(),
                ownership_pct: Decimal::from(100),
                is_direct: true,
                kyc_status: ComplianceStatus::Approved,
            }],
            vec![],
        );
        let errors = service.validate(&app);
        // Should flag all 4 required documents
        let doc_errors: Vec<_> = errors.iter().filter(|e| e.field == "documents").collect();
        assert_eq!(doc_errors.len(), 4);
    }
}
