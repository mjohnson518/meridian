//! Audit log repository for immutable audit trail

use crate::error::DbError;
use crate::models::{AuditLogRow, CreateAuditLogRequest};
use crate::Pool;
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Repository for audit log operations
pub struct AuditRepository {
    pool: Pool,
}

impl AuditRepository {
    /// Creates a new audit repository
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }

    /// Inserts a new audit log entry (immutable)
    pub async fn log(&self, request: CreateAuditLogRequest) -> Result<i64, DbError> {
        let result = sqlx::query!(
            r#"
            INSERT INTO audit_logs (operation, actor, stablecoin_id, basket_id, details)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id
            "#,
            request.operation,
            request.actor,
            request.stablecoin_id,
            request.basket_id,
            request.details,
        )
        .fetch_one(&self.pool)
        .await?;

        tracing::info!(
            audit_id = %result.id,
            operation = %request.operation,
            "Audit log entry created"
        );

        Ok(result.id)
    }

    /// Retrieves audit logs for a specific stablecoin
    pub async fn get_stablecoin_logs(
        &self,
        stablecoin_id: Uuid,
        limit: i64,
    ) -> Result<Vec<AuditLogRow>, DbError> {
        let rows = sqlx::query_as!(
            AuditLogRow,
            r#"
            SELECT id, operation, actor, stablecoin_id, basket_id, details, timestamp
            FROM audit_logs
            WHERE stablecoin_id = $1
            ORDER BY timestamp DESC
            LIMIT $2
            "#,
            stablecoin_id,
            limit
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    /// Retrieves audit logs for a specific basket
    pub async fn get_basket_logs(
        &self,
        basket_id: Uuid,
        limit: i64,
    ) -> Result<Vec<AuditLogRow>, DbError> {
        let rows = sqlx::query_as!(
            AuditLogRow,
            r#"
            SELECT id, operation, actor, stablecoin_id, basket_id, details, timestamp
            FROM audit_logs
            WHERE basket_id = $1
            ORDER BY timestamp DESC
            LIMIT $2
            "#,
            basket_id,
            limit
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    /// Retrieves recent audit logs
    pub async fn get_recent(&self, limit: i64) -> Result<Vec<AuditLogRow>, DbError> {
        let rows = sqlx::query_as!(
            AuditLogRow,
            r#"
            SELECT id, operation, actor, stablecoin_id, basket_id, details, timestamp
            FROM audit_logs
            ORDER BY timestamp DESC
            LIMIT $1
            "#,
            limit
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    /// Gets audit logs by operation type
    pub async fn get_by_operation(
        &self,
        operation: &str,
        start_time: DateTime<Utc>,
        limit: i64,
    ) -> Result<Vec<AuditLogRow>, DbError> {
        let rows = sqlx::query_as!(
            AuditLogRow,
            r#"
            SELECT id, operation, actor, stablecoin_id, basket_id, details, timestamp
            FROM audit_logs
            WHERE operation = $1 AND timestamp >= $2
            ORDER BY timestamp DESC
            LIMIT $3
            "#,
            operation,
            start_time,
            limit
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    /// Counts total audit log entries
    pub async fn count(&self) -> Result<i64, DbError> {
        let result = sqlx::query!("SELECT COUNT(*) as count FROM audit_logs")
            .fetch_one(&self.pool)
            .await?;

        Ok(result.count.unwrap_or(0))
    }
}

