//! # Sanctions Screening Module
//!
//! Integration with OFAC, EU, and UN sanctions lists.

use crate::{ComplianceError, ComplianceResult};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Sanction list sources
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SanctionListSource {
    /// US OFAC SDN List
    OfacSdn,
    /// EU Consolidated List
    EuConsolidated,
    /// UN Security Council
    UnSecurityCouncil,
    /// UK HM Treasury
    UkHmTreasury,
}

/// Sanction screening result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreeningResult {
    /// Whether any matches were found
    pub has_match: bool,
    /// Confidence score (0-100)
    pub confidence: u8,
    /// Matched lists
    pub matched_lists: Vec<SanctionListSource>,
    /// Match details
    pub match_details: Vec<ScreeningMatch>,
    /// Timestamp of screening
    pub screened_at: DateTime<Utc>,
}

/// Individual screening match
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreeningMatch {
    /// Sanction list source
    pub source: SanctionListSource,
    /// Matched name
    pub matched_name: String,
    /// Match score (0-100)
    pub score: u8,
    /// Entity type
    pub entity_type: EntityType,
    /// List entry ID
    pub list_id: String,
}

/// Entity types on sanction lists
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntityType {
    Individual,
    Entity,
    Vessel,
    Aircraft,
}

/// Sanctions screening service
pub struct SanctionsService {
    /// API endpoint for screening provider
    api_url: Option<String>,
    /// Last list update timestamp
    last_update: DateTime<Utc>,
}

impl SanctionsService {
    /// Create a new sanctions service
    pub fn new(api_url: Option<String>) -> Self {
        Self {
            api_url,
            last_update: Utc::now(),
        }
    }

    /// Screen a name against sanction lists
    ///
    /// In production, this would call an external provider like:
    /// - Dow Jones Risk & Compliance
    /// - LexisNexis WorldCompliance
    /// - Refinitiv World-Check
    pub async fn screen_name(&self, name: &str) -> ComplianceResult<ScreeningResult> {
        if name.is_empty() {
            return Err(ComplianceError::SanctionCheckFailed(
                "Name cannot be empty".to_string(),
            ));
        }

        // In production: call external screening API
        // For now: return no match (would need actual API integration)
        Ok(ScreeningResult {
            has_match: false,
            confidence: 0,
            matched_lists: vec![],
            match_details: vec![],
            screened_at: Utc::now(),
        })
    }

    /// Screen a wallet address
    pub async fn screen_address(&self, address: &str) -> ComplianceResult<ScreeningResult> {
        if address.is_empty() {
            return Err(ComplianceError::SanctionCheckFailed(
                "Address cannot be empty".to_string(),
            ));
        }

        // In production: check against OFAC SDN crypto address list
        // For now: return no match
        Ok(ScreeningResult {
            has_match: false,
            confidence: 0,
            matched_lists: vec![],
            match_details: vec![],
            screened_at: Utc::now(),
        })
    }

    /// Get last list update timestamp
    pub fn last_update(&self) -> DateTime<Utc> {
        self.last_update
    }

    /// Check if lists need updating (older than 24 hours)
    pub fn needs_update(&self) -> bool {
        let age = Utc::now() - self.last_update;
        age.num_hours() >= 24
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_name_screening() {
        let service = SanctionsService::new(None);
        let result = service.screen_name("John Doe").await;
        assert!(result.is_ok());
        assert!(!result.unwrap().has_match);
    }

    #[tokio::test]
    async fn test_empty_name_screening() {
        let service = SanctionsService::new(None);
        let result = service.screen_name("").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_address_screening() {
        let service = SanctionsService::new(None);
        let result = service
            .screen_address("0x1234567890abcdef1234567890abcdef12345678")
            .await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_needs_update() {
        let service = SanctionsService::new(None);
        assert!(!service.needs_update()); // Just created
    }
}
