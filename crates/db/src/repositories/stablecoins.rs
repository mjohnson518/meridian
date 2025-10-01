//! Stablecoin repository

use crate::error::DbError;
use crate::models::{CreateStablecoinRequest, StablecoinRow};
use crate::Pool;
use rust_decimal::Decimal;
use uuid::Uuid;

/// Repository for stablecoin operations
pub struct StablecoinRepository {
    pool: Pool,
}

impl StablecoinRepository {
    /// Creates a new stablecoin repository
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }

    /// Creates a new stablecoin record
    pub async fn create(&self, request: CreateStablecoinRequest) -> Result<Uuid, DbError> {
        let id = Uuid::new_v4();

        sqlx::query!(
            r#"
            INSERT INTO stablecoins (id, name, symbol, basket_id, chain_id)
            VALUES ($1, $2, $3, $4, $5)
            "#,
            id,
            request.name,
            request.symbol,
            request.basket_id,
            request.chain_id,
        )
        .execute(&self.pool)
        .await?;

        tracing::info!(stablecoin_id = %id, "Stablecoin created in database");

        Ok(id)
    }

    /// Finds a stablecoin by ID
    pub async fn find_by_id(&self, id: Uuid) -> Result<StablecoinRow, DbError> {
        let row = sqlx::query_as!(
            StablecoinRow,
            r#"
            SELECT id, name, symbol, contract_address, basket_id, chain_id, 
                   total_supply, total_reserve_value, status, deployed_at, created_at, updated_at
            FROM stablecoins
            WHERE id = $1
            "#,
            id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(row)
    }

    /// Finds a stablecoin by contract address
    pub async fn find_by_contract_address(
        &self,
        contract_address: &str,
    ) -> Result<StablecoinRow, DbError> {
        let row = sqlx::query_as!(
            StablecoinRow,
            r#"
            SELECT id, name, symbol, contract_address, basket_id, chain_id,
                   total_supply, total_reserve_value, status, deployed_at, created_at, updated_at
            FROM stablecoins
            WHERE contract_address = $1
            "#,
            contract_address
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(row)
    }

    /// Updates contract address after deployment
    pub async fn set_contract_address(
        &self,
        id: Uuid,
        contract_address: &str,
    ) -> Result<(), DbError> {
        sqlx::query!(
            r#"
            UPDATE stablecoins
            SET contract_address = $1, 
                status = 'active',
                deployed_at = NOW(),
                updated_at = NOW()
            WHERE id = $2
            "#,
            contract_address,
            id
        )
        .execute(&self.pool)
        .await?;

        tracing::info!(
            stablecoin_id = %id,
            contract_address = %contract_address,
            "Contract address set"
        );

        Ok(())
    }

    /// Updates total supply and reserve value
    pub async fn update_balances(
        &self,
        id: Uuid,
        total_supply: Decimal,
        total_reserve_value: Decimal,
    ) -> Result<(), DbError> {
        sqlx::query!(
            r#"
            UPDATE stablecoins
            SET total_supply = $1,
                total_reserve_value = $2,
                updated_at = NOW()
            WHERE id = $3
            "#,
            total_supply,
            total_reserve_value,
            id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Updates stablecoin status
    pub async fn update_status(&self, id: Uuid, status: &str) -> Result<(), DbError> {
        sqlx::query!(
            r#"
            UPDATE stablecoins
            SET status = $1, updated_at = NOW()
            WHERE id = $2
            "#,
            status,
            id
        )
        .execute(&self.pool)
        .await?;

        tracing::info!(stablecoin_id = %id, status = %status, "Status updated");

        Ok(())
    }

    /// Lists all stablecoins with pagination
    pub async fn list(&self, limit: i64, offset: i64) -> Result<Vec<StablecoinRow>, DbError> {
        let rows = sqlx::query_as!(
            StablecoinRow,
            r#"
            SELECT id, name, symbol, contract_address, basket_id, chain_id,
                   total_supply, total_reserve_value, status, deployed_at, created_at, updated_at
            FROM stablecoins
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    /// Lists stablecoins by chain ID
    pub async fn find_by_chain(
        &self,
        chain_id: i32,
        limit: i64,
    ) -> Result<Vec<StablecoinRow>, DbError> {
        let rows = sqlx::query_as!(
            StablecoinRow,
            r#"
            SELECT id, name, symbol, contract_address, basket_id, chain_id,
                   total_supply, total_reserve_value, status, deployed_at, created_at, updated_at
            FROM stablecoins
            WHERE chain_id = $1
            ORDER BY created_at DESC
            LIMIT $2
            "#,
            chain_id,
            limit
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    /// Counts total number of stablecoins
    pub async fn count(&self) -> Result<i64, DbError> {
        let result = sqlx::query!("SELECT COUNT(*) as count FROM stablecoins")
            .fetch_one(&self.pool)
            .await?;

        Ok(result.count.unwrap_or(0))
    }
}

