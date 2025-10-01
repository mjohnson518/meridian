//! Price history repository

use crate::error::DbError;
use crate::models::{InsertPriceRequest, PriceHistoryRow};
use crate::Pool;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;

/// Repository for price history operations
pub struct PriceRepository {
    pool: Pool,
}

impl PriceRepository {
    /// Creates a new price repository
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }

    /// Inserts a new price record
    pub async fn insert(&self, request: InsertPriceRequest) -> Result<i64, DbError> {
        let result = sqlx::query!(
            r#"
            INSERT INTO price_history (currency_pair, price, source, is_stale, round_id)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id
            "#,
            request.currency_pair,
            request.price,
            request.source,
            request.is_stale,
            request.round_id,
        )
        .fetch_one(&self.pool)
        .await?;

        tracing::debug!(
            pair = %request.currency_pair,
            price = %request.price,
            "Price inserted into history"
        );

        Ok(result.id)
    }

    /// Gets the latest price for a currency pair
    pub async fn get_latest(&self, currency_pair: &str) -> Result<PriceHistoryRow, DbError> {
        let row = sqlx::query_as!(
            PriceHistoryRow,
            r#"
            SELECT id, currency_pair, price, source, is_stale, round_id, timestamp
            FROM price_history
            WHERE currency_pair = $1
            ORDER BY timestamp DESC
            LIMIT 1
            "#,
            currency_pair
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(row)
    }

    /// Gets price history for a currency pair within a time range
    pub async fn get_history(
        &self,
        currency_pair: &str,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
        limit: i64,
    ) -> Result<Vec<PriceHistoryRow>, DbError> {
        let rows = sqlx::query_as!(
            PriceHistoryRow,
            r#"
            SELECT id, currency_pair, price, source, is_stale, round_id, timestamp
            FROM price_history
            WHERE currency_pair = $1
                AND timestamp >= $2
                AND timestamp <= $3
            ORDER BY timestamp DESC
            LIMIT $4
            "#,
            currency_pair,
            start_time,
            end_time,
            limit
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    /// Gets all unique currency pairs with price data
    pub async fn get_all_pairs(&self) -> Result<Vec<String>, DbError> {
        let rows = sqlx::query!(
            r#"
            SELECT DISTINCT currency_pair
            FROM price_history
            ORDER BY currency_pair
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.currency_pair).collect())
    }

    /// Gets statistics for a currency pair
    pub async fn get_stats(
        &self,
        currency_pair: &str,
        start_time: DateTime<Utc>,
    ) -> Result<PriceStats, DbError> {
        let result = sqlx::query!(
            r#"
            SELECT 
                MIN(price) as min_price,
                MAX(price) as max_price,
                AVG(price) as avg_price,
                COUNT(*) as count
            FROM price_history
            WHERE currency_pair = $1
                AND timestamp >= $2
            "#,
            currency_pair,
            start_time
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(PriceStats {
            currency_pair: currency_pair.to_string(),
            min_price: result.min_price.unwrap_or(Decimal::ZERO),
            max_price: result.max_price.unwrap_or(Decimal::ZERO),
            avg_price: result.avg_price.map(|d| Decimal::try_from(d).unwrap_or(Decimal::ZERO)).unwrap_or(Decimal::ZERO),
            count: result.count.unwrap_or(0),
        })
    }

    /// Deletes old price records (cleanup)
    pub async fn delete_older_than(&self, cutoff_time: DateTime<Utc>) -> Result<u64, DbError> {
        let result = sqlx::query!(
            r#"
            DELETE FROM price_history
            WHERE timestamp < $1
            "#,
            cutoff_time
        )
        .execute(&self.pool)
        .await?;

        let deleted = result.rows_affected();
        tracing::info!(deleted = %deleted, "Old price records deleted");

        Ok(deleted)
    }
}

/// Price statistics for a currency pair
#[derive(Debug, Clone, serde::Serialize)]
pub struct PriceStats {
    pub currency_pair: String,
    pub min_price: Decimal,
    pub max_price: Decimal,
    pub avg_price: Decimal,
    pub count: i64,
}

