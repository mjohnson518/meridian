//! Database error types

use thiserror::Error;

/// Errors that can occur during database operations
#[derive(Error, Debug)]
pub enum DbError {
    #[error("Database connection error: {0}")]
    ConnectionError(String),

    #[error("Migration error: {0}")]
    MigrationError(String),

    #[error("Query error: {0}")]
    QueryError(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Duplicate entry: {0}")]
    DuplicateEntry(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Transaction error: {0}")]
    TransactionError(String),
}

// Convert SQLx errors
impl From<sqlx::Error> for DbError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => DbError::NotFound("Record not found".to_string()),
            sqlx::Error::Database(db_err) => {
                if let Some(constraint) = db_err.constraint() {
                    DbError::DuplicateEntry(format!("Constraint violation: {}", constraint))
                } else {
                    DbError::QueryError(db_err.to_string())
                }
            }
            _ => DbError::QueryError(err.to_string()),
        }
    }
}

// Convert serde JSON errors
impl From<serde_json::Error> for DbError {
    fn from(err: serde_json::Error) -> Self {
        DbError::SerializationError(err.to_string())
    }
}

