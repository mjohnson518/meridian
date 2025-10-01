//! Basket repository for database operations

use crate::error::DbError;
use crate::models::BasketRow;
use crate::Pool;
use meridian_basket::CurrencyBasket;
use uuid::Uuid;

/// Repository for basket operations
pub struct BasketRepository {
    pool: Pool,
}

impl BasketRepository {
    /// Creates a new basket repository
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }

    /// Inserts a new basket into the database
    pub async fn create(&self, basket: &CurrencyBasket) -> Result<Uuid, DbError> {
        let row = BasketRow::from_basket(basket)?;

        sqlx::query!(
            r#"
            INSERT INTO baskets (id, name, basket_type, components, rebalance_strategy, last_rebalanced, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
            row.id,
            row.name,
            row.basket_type,
            row.components,
            row.rebalance_strategy,
            row.last_rebalanced,
            row.created_at,
            row.updated_at,
        )
        .execute(&self.pool)
        .await?;

        tracing::info!(basket_id = %row.id, "Basket created in database");

        Ok(row.id)
    }

    /// Retrieves a basket by ID
    pub async fn find_by_id(&self, id: Uuid) -> Result<CurrencyBasket, DbError> {
        let row = sqlx::query_as!(
            BasketRow,
            r#"
            SELECT id, name, basket_type, components, rebalance_strategy, last_rebalanced, created_at, updated_at
            FROM baskets
            WHERE id = $1
            "#,
            id
        )
        .fetch_one(&self.pool)
        .await?;

        row.to_basket().map_err(DbError::from)
    }

    /// Lists all baskets with pagination
    pub async fn list(&self, limit: i64, offset: i64) -> Result<Vec<CurrencyBasket>, DbError> {
        let rows = sqlx::query_as!(
            BasketRow,
            r#"
            SELECT id, name, basket_type, components, rebalance_strategy, last_rebalanced, created_at, updated_at
            FROM baskets
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(|row| row.to_basket().map_err(DbError::from))
            .collect()
    }

    /// Counts total number of baskets
    pub async fn count(&self) -> Result<i64, DbError> {
        let result = sqlx::query!("SELECT COUNT(*) as count FROM baskets")
            .fetch_one(&self.pool)
            .await?;

        Ok(result.count.unwrap_or(0))
    }

    /// Updates basket's last_rebalanced timestamp
    pub async fn mark_rebalanced(&self, id: Uuid) -> Result<(), DbError> {
        sqlx::query!(
            r#"
            UPDATE baskets
            SET last_rebalanced = NOW(), updated_at = NOW()
            WHERE id = $1
            "#,
            id
        )
        .execute(&self.pool)
        .await?;

        tracing::info!(basket_id = %id, "Basket marked as rebalanced");

        Ok(())
    }

    /// Deletes a basket by ID
    pub async fn delete(&self, id: Uuid) -> Result<(), DbError> {
        let result = sqlx::query!(
            r#"
            DELETE FROM baskets
            WHERE id = $1
            "#,
            id
        )
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(DbError::NotFound(format!("Basket {} not found", id)));
        }

        tracing::info!(basket_id = %id, "Basket deleted");

        Ok(())
    }

    /// Lists baskets by type
    pub async fn find_by_type(
        &self,
        basket_type: &str,
        limit: i64,
    ) -> Result<Vec<CurrencyBasket>, DbError> {
        let rows = sqlx::query_as!(
            BasketRow,
            r#"
            SELECT id, name, basket_type, components, rebalance_strategy, last_rebalanced, created_at, updated_at
            FROM baskets
            WHERE basket_type = $1
            ORDER BY created_at DESC
            LIMIT $2
            "#,
            basket_type,
            limit
        )
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(|row| row.to_basket().map_err(DbError::from))
            .collect()
    }
}

