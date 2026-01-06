//! # Transaction Monitoring Module
//!
//! Real-time and batch transaction monitoring for suspicious activity.

use crate::ComplianceFlag;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Transaction for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoredTransaction {
    /// Transaction ID
    pub id: String,
    /// Customer ID
    pub customer_id: Uuid,
    /// Transaction type
    pub transaction_type: TransactionType,
    /// Amount (in smallest unit)
    pub amount: Decimal,
    /// Currency code
    pub currency: String,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Source address (if applicable)
    pub source_address: Option<String>,
    /// Destination address (if applicable)
    pub destination_address: Option<String>,
}

/// Transaction types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionType {
    /// Mint stablecoin
    Mint,
    /// Burn/redeem stablecoin
    Burn,
    /// Transfer between addresses
    Transfer,
    /// Deposit fiat
    Deposit,
    /// Withdraw fiat
    Withdrawal,
}

/// Alert severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum AlertSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Compliance alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceAlert {
    /// Alert ID
    pub id: Uuid,
    /// Customer ID
    pub customer_id: Uuid,
    /// Related transaction ID
    pub transaction_id: Option<String>,
    /// Alert severity
    pub severity: AlertSeverity,
    /// Compliance flags triggered
    pub flags: Vec<ComplianceFlag>,
    /// Alert description
    pub description: String,
    /// Created timestamp
    pub created_at: DateTime<Utc>,
    /// Whether alert has been reviewed
    pub reviewed: bool,
    /// Review notes
    pub review_notes: Option<String>,
}

/// Transaction monitoring service
pub struct MonitoringService {
    /// Daily transaction threshold for alerts
    daily_threshold: Decimal,
    /// Single transaction threshold for alerts
    single_threshold: Decimal,
    /// Structuring detection window (transactions)
    structuring_window: usize,
    /// Structuring threshold (just under reporting threshold)
    structuring_threshold: Decimal,
}

impl Default for MonitoringService {
    fn default() -> Self {
        Self {
            daily_threshold: Decimal::new(1_000_000, 2),    // $10,000
            single_threshold: Decimal::new(300_000, 2),    // $3,000
            structuring_window: 10,
            structuring_threshold: Decimal::new(950_000, 2), // $9,500 (just under $10k)
        }
    }
}

impl MonitoringService {
    /// Create a new monitoring service
    pub fn new() -> Self {
        Self::default()
    }

    /// Analyze a single transaction for suspicious patterns
    pub fn analyze_transaction(&self, tx: &MonitoredTransaction) -> Vec<ComplianceFlag> {
        let mut flags = Vec::new();

        // Large transaction check
        if tx.amount > self.single_threshold {
            flags.push(ComplianceFlag::SingleTransactionLimitExceeded);
        }

        flags
    }

    /// Analyze transaction history for patterns
    pub fn analyze_pattern(&self, transactions: &[MonitoredTransaction]) -> Vec<ComplianceFlag> {
        let mut flags = Vec::new();

        if transactions.is_empty() {
            return flags;
        }

        // Calculate daily total
        let today = Utc::now().date_naive();
        let daily_total: Decimal = transactions
            .iter()
            .filter(|tx| tx.timestamp.date_naive() == today)
            .map(|tx| tx.amount)
            .sum();

        if daily_total > self.daily_threshold {
            flags.push(ComplianceFlag::DailyLimitExceeded);
        }

        // Check for structuring (multiple transactions just under threshold)
        let recent_transactions: Vec<_> = transactions
            .iter()
            .rev()
            .take(self.structuring_window)
            .collect();

        let structuring_count = recent_transactions
            .iter()
            .filter(|tx| {
                tx.amount >= self.structuring_threshold && tx.amount < self.daily_threshold
            })
            .count();

        if structuring_count >= 3 {
            flags.push(ComplianceFlag::StructuringDetected);
        }

        // Velocity check (too many transactions in short time)
        if transactions.len() > 20 {
            let one_hour_ago = Utc::now() - chrono::Duration::hours(1);
            let hourly_count = transactions
                .iter()
                .filter(|tx| tx.timestamp > one_hour_ago)
                .count();

            if hourly_count > 10 {
                flags.push(ComplianceFlag::VelocityExceeded);
            }
        }

        flags
    }

    /// Create alert from flags
    pub fn create_alert(
        &self,
        customer_id: Uuid,
        transaction_id: Option<String>,
        flags: Vec<ComplianceFlag>,
    ) -> Option<ComplianceAlert> {
        if flags.is_empty() {
            return None;
        }

        let severity = if flags.contains(&ComplianceFlag::SanctionMatch) {
            AlertSeverity::Critical
        } else if flags.contains(&ComplianceFlag::StructuringDetected)
            || flags.contains(&ComplianceFlag::PepInvolved)
        {
            AlertSeverity::High
        } else if flags.contains(&ComplianceFlag::DailyLimitExceeded) {
            AlertSeverity::Medium
        } else {
            AlertSeverity::Low
        };

        let description = flags
            .iter()
            .map(|f| format!("{:?}", f))
            .collect::<Vec<_>>()
            .join(", ");

        Some(ComplianceAlert {
            id: Uuid::new_v4(),
            customer_id,
            transaction_id,
            severity,
            flags,
            description,
            created_at: Utc::now(),
            reviewed: false,
            review_notes: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_transaction(amount: Decimal) -> MonitoredTransaction {
        MonitoredTransaction {
            id: "tx_test".to_string(),
            customer_id: Uuid::new_v4(),
            transaction_type: TransactionType::Transfer,
            amount,
            currency: "USD".to_string(),
            timestamp: Utc::now(),
            source_address: None,
            destination_address: None,
        }
    }

    #[test]
    fn test_large_transaction_detection() {
        let service = MonitoringService::new();
        let tx = create_test_transaction(Decimal::new(5_000_00, 2)); // $5,000

        let flags = service.analyze_transaction(&tx);
        assert!(flags.contains(&ComplianceFlag::SingleTransactionLimitExceeded));
    }

    #[test]
    fn test_normal_transaction() {
        let service = MonitoringService::new();
        let tx = create_test_transaction(Decimal::new(1_000_00, 2)); // $1,000

        let flags = service.analyze_transaction(&tx);
        assert!(flags.is_empty());
    }

    #[test]
    fn test_alert_creation() {
        let service = MonitoringService::new();
        let flags = vec![ComplianceFlag::StructuringDetected];

        let alert = service.create_alert(Uuid::new_v4(), Some("tx_123".to_string()), flags);
        assert!(alert.is_some());

        let alert = alert.unwrap();
        assert_eq!(alert.severity, AlertSeverity::High);
    }

    #[test]
    fn test_no_alert_for_empty_flags() {
        let service = MonitoringService::new();
        let alert = service.create_alert(Uuid::new_v4(), None, vec![]);
        assert!(alert.is_none());
    }
}
