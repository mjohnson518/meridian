//! # Meridian Database Layer
//!
//! PostgreSQL database access layer with SQLx
//!
//! ## Features
//!
//! - Repository pattern for data access
//! - Connection pooling with PgPool
//! - Transaction support
//! - Type-safe queries with SQLx
//! - Migration support
//!
//! ## Note on Decimal Storage
//!
//! Currently using TEXT storage for rust_decimal::Decimal due to SQLx-Decimal
//! compatibility issues. This is a temporary workaround.
//! TODO: Migrate to native NUMERIC when SQLx-Decimal is compatible

mod decimal_helpers;
mod error;
mod models;
mod repositories;

pub use decimal_helpers::{decimal_to_text, opt_text_to_decimal, text_to_decimal};
pub use error::DbError;
pub use models::*;
pub use repositories::*;

use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::PgPool;
use std::time::Duration;

/// Database connection pool
pub type Pool = PgPool;

/// Creates a new database connection pool
///
/// # Arguments
///
/// * `database_url` - PostgreSQL connection string
///
/// # Example
///
/// ```rust,no_run
/// use meridian_db::create_pool;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let pool = create_pool("postgresql://user:pass@localhost/meridian").await?;
/// # Ok(())
/// # }
/// ```
pub async fn create_pool(database_url: &str) -> Result<Pool, DbError> {
    let options = database_url
        .parse::<PgConnectOptions>()
        .map_err(|e| DbError::ConnectionError(e.to_string()))?;

    // Get max connections from environment or use default
    let max_connections: u32 = std::env::var("DATABASE_MAX_CONNECTIONS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(20);

    let pool = PgPoolOptions::new()
        .max_connections(max_connections)
        .min_connections(2)
        .acquire_timeout(Duration::from_secs(30))
        .idle_timeout(Some(Duration::from_secs(600)))
        .connect_with(options)
        .await
        .map_err(|e| DbError::ConnectionError(e.to_string()))?;

    tracing::info!("Database pool created with max {} connections", max_connections);

    Ok(pool)
}

/// Runs all pending database migrations
///
/// # Example
///
/// ```rust,no_run
/// use meridian_db::{create_pool, run_migrations};
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let pool = create_pool("postgresql://user:pass@localhost/meridian").await?;
/// run_migrations(&pool).await?;
/// # Ok(())
/// # }
/// ```
pub async fn run_migrations(pool: &Pool) -> Result<(), DbError> {
    tracing::info!("Running database migrations...");

    sqlx::migrate!("./migrations")
        .run(pool)
        .await
        .map_err(|e| DbError::MigrationError(e.to_string()))?;

    tracing::info!("✅ Migrations completed successfully");

    Ok(())
}
