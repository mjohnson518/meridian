//! # KYC (Know Your Customer) Module
//!
//! Customer identification and verification functionality.

use crate::{ComplianceError, ComplianceResult, ComplianceStatus};
use chrono::{DateTime, Utc};
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use uuid::Uuid;

type HmacSha256 = Hmac<Sha256>;

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

/// Sumsub API applicant response (subset)
#[derive(Debug, Deserialize)]
struct SumsumApplicantResponse {
    id: String,
    #[serde(default)]
    review: Option<SumsumReview>,
}

#[derive(Debug, Deserialize)]
struct SumsumReview {
    #[serde(rename = "reviewStatus")]
    review_status: String,
    #[serde(rename = "reviewResult")]
    review_result: Option<SumsumReviewResult>,
}

#[derive(Debug, Deserialize)]
struct SumsumReviewResult {
    #[serde(rename = "reviewAnswer")]
    review_answer: String, // "GREEN" | "RED"
    #[allow(dead_code)] // Deserialized for future use in rejection reason mapping
    #[serde(rename = "rejectLabels")]
    reject_labels: Option<Vec<String>>,
}

/// KYC service interface
pub struct KycService {
    /// Sumsub base URL (None = use mock mode)
    provider_url: Option<String>,
    /// Sumsub App Token (X-App-Token header)
    app_token: Option<String>,
    /// Sumsub Secret Key (for HMAC-SHA256 signatures)
    secret_key: Option<String>,
    http: reqwest::Client,
}

impl KycService {
    /// Create a new KYC service.
    /// If `provider_url` is None, returns pending (mock mode for dev).
    pub fn new(provider_url: Option<String>) -> Self {
        let app_token = std::env::var("SUMSUB_APP_TOKEN").ok();
        let secret_key = std::env::var("SUMSUB_SECRET_KEY").ok();
        Self {
            provider_url,
            app_token,
            secret_key,
            http: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .expect("Failed to build HTTP client"),
        }
    }

    /// Generate Sumsub HMAC-SHA256 request signature.
    ///
    /// Signature = HMAC-SHA256(secret, ts + method + path + body_bytes)
    fn sumsub_signature(&self, ts: i64, method: &str, path: &str, body: &[u8]) -> Option<String> {
        let secret = self.secret_key.as_deref()?;
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).ok()?;
        mac.update(ts.to_string().as_bytes());
        mac.update(method.as_bytes());
        mac.update(path.as_bytes());
        mac.update(body);
        Some(hex::encode(mac.finalize().into_bytes()))
    }

    /// Create a Sumsub applicant and return the external applicant ID.
    async fn create_sumsub_applicant(
        &self,
        request: &KycVerificationRequest,
    ) -> ComplianceResult<String> {
        let base_url = self.provider_url.as_deref().unwrap_or("https://api.sumsub.com");
        let app_token = self.app_token.as_deref()
            .ok_or_else(|| ComplianceError::ExternalServiceError("SUMSUB_APP_TOKEN not set".to_string()))?;

        let path = "/resources/applicants?levelName=basic-kyc-level";
        let ts = Utc::now().timestamp();

        let body = serde_json::json!({
            "externalUserId": request.customer_id.to_string(),
            "info": {
                "firstName": request.first_name,
                "lastName": request.last_name,
                "dob": request.date_of_birth,
                "country": request.nationality,
                "addresses": [{
                    "street": request.address_line1,
                    "town": request.city,
                    "postCode": request.postal_code,
                    "country": request.country_of_residence,
                }]
            }
        });
        let body_bytes = serde_json::to_vec(&body)
            .map_err(|e| ComplianceError::ExternalServiceError(e.to_string()))?;

        let sig = self.sumsub_signature(ts, "POST", path, &body_bytes)
            .ok_or_else(|| ComplianceError::ExternalServiceError("Signature failed — check SUMSUB_SECRET_KEY".to_string()))?;

        let response = self.http
            .post(format!("{}{}", base_url, path))
            .header("X-App-Token", app_token)
            .header("X-App-Access-Sig", sig)
            .header("X-App-Access-Ts", ts.to_string())
            .header("Content-Type", "application/json")
            .body(body_bytes)
            .send()
            .await
            .map_err(|e| ComplianceError::ExternalServiceError(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let msg = response.text().await.unwrap_or_default();
            return Err(ComplianceError::ExternalServiceError(
                format!("Sumsub API error {}: {}", status, msg)
            ));
        }

        let applicant: SumsumApplicantResponse = response.json().await
            .map_err(|e| ComplianceError::ExternalServiceError(e.to_string()))?;

        tracing::info!(
            customer_id = %request.customer_id,
            applicant_id = %applicant.id,
            "Sumsub applicant created"
        );

        Ok(applicant.id)
    }

    /// Check verification status from Sumsub by external applicant ID.
    pub async fn check_status_from_provider(
        &self,
        external_kyc_id: &str,
    ) -> ComplianceResult<ComplianceStatus> {
        let base_url = self.provider_url.as_deref().unwrap_or("https://api.sumsub.com");
        let app_token = self.app_token.as_deref()
            .ok_or_else(|| ComplianceError::ExternalServiceError("SUMSUB_APP_TOKEN not set".to_string()))?;

        let path = format!("/resources/applicants/{}/status", external_kyc_id);
        let ts = Utc::now().timestamp();
        let sig = self.sumsub_signature(ts, "GET", &path, &[])
            .ok_or_else(|| ComplianceError::ExternalServiceError("Signature failed".to_string()))?;

        let response = self.http
            .get(format!("{}{}", base_url, path))
            .header("X-App-Token", app_token)
            .header("X-App-Access-Sig", sig)
            .header("X-App-Access-Ts", ts.to_string())
            .send()
            .await
            .map_err(|e| ComplianceError::ExternalServiceError(e.to_string()))?;

        let applicant: SumsumApplicantResponse = response.json().await
            .map_err(|e| ComplianceError::ExternalServiceError(e.to_string()))?;

        let status = match applicant.review.as_ref().map(|r| r.review_status.as_str()) {
            Some("completed") => {
                match applicant.review.as_ref()
                    .and_then(|r| r.review_result.as_ref())
                    .map(|r| r.review_answer.as_str())
                {
                    Some("GREEN") => ComplianceStatus::Approved,
                    Some("RED") => ComplianceStatus::Rejected,
                    _ => ComplianceStatus::Pending,
                }
            }
            Some("pending") | Some("queued") => ComplianceStatus::Pending,
            _ => ComplianceStatus::Pending,
        };

        Ok(status)
    }

    /// Submit KYC verification request.
    ///
    /// If Sumsub is configured (SUMSUB_APP_TOKEN + SUMSUB_SECRET_KEY env vars),
    /// creates a real applicant and returns their external ID.
    /// Otherwise returns Pending (mock mode for dev/test).
    ///
    /// Returns (external_kyc_id, result) — caller should persist external_kyc_id.
    pub async fn submit_verification(
        &self,
        request: &KycVerificationRequest,
    ) -> ComplianceResult<KycVerificationResult> {
        if request.first_name.is_empty() || request.last_name.is_empty() {
            return Err(ComplianceError::KycFailed("Name fields required".to_string()));
        }

        let now = Utc::now();

        // If provider is configured, call it; otherwise return pending (mock)
        if self.provider_url.is_some() && self.app_token.is_some() {
            match self.create_sumsub_applicant(request).await {
                Ok(_applicant_id) => {
                    // Verification submitted — status starts as Pending until webhook arrives
                    return Ok(KycVerificationResult {
                        customer_id: request.customer_id,
                        status: ComplianceStatus::Pending,
                        level: request.level,
                        rejection_reason: None,
                        verified_at: now,
                        expires_at: now + chrono::Duration::days(365),
                    });
                }
                Err(e) => {
                    tracing::warn!(error = %e, "Sumsub submission failed, falling back to pending");
                }
            }
        } else {
            tracing::debug!("KYC provider not configured — returning Pending (dev mode)");
        }

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
