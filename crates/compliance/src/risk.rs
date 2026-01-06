//! # Risk Assessment Module
//!
//! Customer and transaction risk scoring per FATF guidelines.

use crate::RiskLevel;
use serde::{Deserialize, Serialize};

/// Risk factors for assessment
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskFactor {
    /// Geographic risk (high-risk jurisdiction)
    Geographic,
    /// Customer type risk (PEP, high-net-worth)
    CustomerType,
    /// Transaction pattern risk (structuring, velocity)
    TransactionPattern,
    /// Product risk (complex products)
    Product,
    /// Channel risk (non-face-to-face)
    Channel,
}

/// Risk score breakdown
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessment {
    /// Overall risk score (0-100)
    pub total_score: u8,
    /// Risk level classification
    pub risk_level: RiskLevel,
    /// Geographic risk component (0-25)
    pub geographic_score: u8,
    /// Customer type risk component (0-25)
    pub customer_type_score: u8,
    /// Transaction pattern risk component (0-25)
    pub transaction_pattern_score: u8,
    /// Product/channel risk component (0-25)
    pub product_channel_score: u8,
    /// Flags that contributed to score
    pub risk_factors: Vec<RiskFactor>,
}

impl Default for RiskAssessment {
    fn default() -> Self {
        Self {
            total_score: 25, // Medium-low default
            risk_level: RiskLevel::Low,
            geographic_score: 5,
            customer_type_score: 5,
            transaction_pattern_score: 5,
            product_channel_score: 10,
            risk_factors: vec![],
        }
    }
}

/// Risk scoring engine
pub struct RiskEngine {
    /// High-risk countries (ISO 3166-1 alpha-2)
    high_risk_countries: Vec<String>,
    /// Prohibited countries
    prohibited_countries: Vec<String>,
}

impl Default for RiskEngine {
    fn default() -> Self {
        Self {
            high_risk_countries: vec![
                "RU".to_string(),
                "BY".to_string(),
                "MM".to_string(),
                "VE".to_string(),
                "AF".to_string(),
                "YE".to_string(),
            ],
            prohibited_countries: vec![
                "KP".to_string(),
                "IR".to_string(),
                "SY".to_string(),
                "CU".to_string(),
            ],
        }
    }
}

impl RiskEngine {
    /// Create a new risk engine
    pub fn new() -> Self {
        Self::default()
    }

    /// Calculate geographic risk score (0-25)
    pub fn calculate_geographic_score(&self, country_code: &str) -> u8 {
        let code = country_code.to_uppercase();

        if self.prohibited_countries.contains(&code) {
            25 // Maximum risk
        } else if self.high_risk_countries.contains(&code) {
            20 // High risk
        } else {
            5 // Standard risk
        }
    }

    /// Calculate customer type risk score (0-25)
    pub fn calculate_customer_type_score(&self, is_pep: bool, is_high_net_worth: bool) -> u8 {
        let mut score = 5u8;

        if is_pep {
            score = score.saturating_add(15);
        }

        if is_high_net_worth {
            score = score.saturating_add(5);
        }

        score.min(25)
    }

    /// Calculate transaction pattern risk score (0-25)
    pub fn calculate_transaction_pattern_score(
        &self,
        daily_transaction_count: u32,
        avg_transaction_size: u64,
        has_structuring_pattern: bool,
    ) -> u8 {
        let mut score = 5u8;

        // High velocity
        if daily_transaction_count > 10 {
            score = score.saturating_add(5);
        }
        if daily_transaction_count > 50 {
            score = score.saturating_add(5);
        }

        // Large transactions
        if avg_transaction_size > 10_000_000 {
            // > $100k
            score = score.saturating_add(5);
        }

        // Structuring detected
        if has_structuring_pattern {
            score = score.saturating_add(10);
        }

        score.min(25)
    }

    /// Perform full risk assessment
    pub fn assess_risk(
        &self,
        country_code: &str,
        is_pep: bool,
        is_high_net_worth: bool,
        daily_transaction_count: u32,
        avg_transaction_size: u64,
        has_structuring_pattern: bool,
    ) -> RiskAssessment {
        let geographic_score = self.calculate_geographic_score(country_code);
        let customer_type_score = self.calculate_customer_type_score(is_pep, is_high_net_worth);
        let transaction_pattern_score = self.calculate_transaction_pattern_score(
            daily_transaction_count,
            avg_transaction_size,
            has_structuring_pattern,
        );
        let product_channel_score = 10; // Default for stablecoin product

        let total_score = geographic_score
            .saturating_add(customer_type_score)
            .saturating_add(transaction_pattern_score)
            .saturating_add(product_channel_score);

        let risk_level = match total_score {
            0..=25 => RiskLevel::Low,
            26..=50 => RiskLevel::Medium,
            51..=75 => RiskLevel::High,
            _ => RiskLevel::Prohibited,
        };

        let mut risk_factors = Vec::new();
        if geographic_score >= 20 {
            risk_factors.push(RiskFactor::Geographic);
        }
        if customer_type_score >= 15 {
            risk_factors.push(RiskFactor::CustomerType);
        }
        if transaction_pattern_score >= 15 {
            risk_factors.push(RiskFactor::TransactionPattern);
        }

        RiskAssessment {
            total_score,
            risk_level,
            geographic_score,
            customer_type_score,
            transaction_pattern_score,
            product_channel_score,
            risk_factors,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_geographic_scoring() {
        let engine = RiskEngine::new();

        assert_eq!(engine.calculate_geographic_score("KP"), 25); // Prohibited
        assert_eq!(engine.calculate_geographic_score("RU"), 20); // High risk
        assert_eq!(engine.calculate_geographic_score("US"), 5); // Standard
    }

    #[test]
    fn test_customer_type_scoring() {
        let engine = RiskEngine::new();

        assert_eq!(engine.calculate_customer_type_score(false, false), 5);
        assert_eq!(engine.calculate_customer_type_score(true, false), 20);
        assert_eq!(engine.calculate_customer_type_score(true, true), 25);
    }

    #[test]
    fn test_full_risk_assessment() {
        let engine = RiskEngine::new();

        // Low risk customer
        let assessment = engine.assess_risk("US", false, false, 5, 1000_00, false);
        assert_eq!(assessment.risk_level, RiskLevel::Low);

        // High risk customer (PEP in high-risk country)
        let assessment = engine.assess_risk("RU", true, true, 100, 500_000_00, true);
        assert_eq!(assessment.risk_level, RiskLevel::Prohibited);
    }
}
