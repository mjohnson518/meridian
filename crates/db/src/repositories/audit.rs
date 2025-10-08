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
        let result: (i64,) = sqlx::query_as(
            r#"
            INSERT INTO audit_logs (operation, actor, stablecoin_id, basket_id, details)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id
            "#,
        )
        .bind(&request.operation)
        .bind(&request.actor)
        .bind(request.stablecoin_id)
        .bind(request.basket_id)
        .bind(&request.details)
        .fetch_one(&self.pool)
        .await?;

        tracing::info!(
            audit_id = %result.0,
            operation = %request.operation,
            "Audit log entry created"
        );

        Ok(result.0)
    }

    /// Retrieves audit logs for a specific stablecoin
    pub async fn get_stablecoin_logs(
        &self,
        stablecoin_id: Uuid,
        limit: i64,
    ) -> Result<Vec<AuditLogRow>, DbError> {
        let rows = sqlx::query_as::<_, AuditLogRow>(
            r#"
            SELECT id, operation, actor, stablecoin_id, basket_id, details, timestamp
            FROM audit_logs
            WHERE stablecoin_id = $1
            ORDER BY timestamp DESC
            LIMIT $2
            "#,
        )
        .bind(stablecoin_id)
        .bind(limit)
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
        let rows = sqlx::query_as::<_, AuditLogRow>(
            r#"
            SELECT id, operation, actor, stablecoin_id, basket_id, details, timestamp
            FROM audit_logs
            WHERE basket_id = $1
            ORDER BY timestamp DESC
            LIMIT $2
            "#,
        )
        .bind(basket_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    /// Retrieves recent audit logs
    pub async fn get_recent(&self, limit: i64) -> Result<Vec<AuditLogRow>, DbError> {
        let rows = sqlx::query_as::<_, AuditLogRow>(
            r#"
            SELECT id, operation, actor, stablecoin_id, basket_id, details, timestamp
            FROM audit_logs
            ORDER BY timestamp DESC
            LIMIT $1
            "#,
        )
        .bind(limit)
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
        let rows = sqlx::query_as::<_, AuditLogRow>(
            r#"
            SELECT id, operation, actor, stablecoin_id, basket_id, details, timestamp
            FROM audit_logs
            WHERE operation = $1 AND timestamp >= $2
            ORDER BY timestamp DESC
            LIMIT $3
            "#,
        )
        .bind(operation)
        .bind(start_time)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    /// Counts total audit log entries
    pub async fn count(&self) -> Result<i64, DbError> {
        let result: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM audit_logs")
            .fetch_one(&self.pool)
            .await?;

        Ok(result.0)
    }
}
