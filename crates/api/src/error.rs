//! Error types for API operations

use actix_web::{http::StatusCode, HttpMessage, HttpRequest, HttpResponse, ResponseError};
use meridian_basket::BasketError;
use meridian_db::DbError;
use meridian_oracle::OracleError;
use serde::Serialize;
use std::fmt;

/// API error response
/// HIGH-012: Includes request_id for easier debugging by API consumers
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
    /// Correlation/request ID for tracking this error
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
}

/// API errors
#[derive(Debug)]
#[allow(dead_code)]
pub enum ApiError {
    BasketError(BasketError),
    OracleError(OracleError),
    DatabaseError(DbError),
    NotFound(String),
    BadRequest(String),
    Unauthorized(String),
    Forbidden(String),
    OracleNotConfigured,
    InternalError(String),
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApiError::BasketError(e) => write!(f, "Basket error: {}", e),
            ApiError::OracleError(e) => write!(f, "Oracle error: {}", e),
            ApiError::DatabaseError(e) => write!(f, "Database error: {}", e),
            ApiError::NotFound(msg) => write!(f, "Not found: {}", msg),
            ApiError::BadRequest(msg) => write!(f, "Bad request: {}", msg),
            ApiError::Unauthorized(msg) => write!(f, "Unauthorized: {}", msg),
            ApiError::Forbidden(msg) => write!(f, "Forbidden: {}", msg),
            ApiError::OracleNotConfigured => write!(f, "Oracle not configured"),
            ApiError::InternalError(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

/// Extract correlation ID from request extensions
/// HIGH-012: Used to include request_id in error responses
fn get_correlation_id(req: &HttpRequest) -> Option<String> {
    use crate::middleware::CorrelationId;

    req.extensions()
        .get::<CorrelationId>()
        .map(|c| c.as_str().to_string())
}

impl ApiError {
    /// Create an error response with the request ID included
    /// HIGH-012: This method should be used instead of automatic ResponseError conversion
    /// when you have access to the HttpRequest
    pub fn to_response(&self, req: &HttpRequest) -> HttpResponse {
        let error_type = self.error_type();
        let request_id = get_correlation_id(req);

        HttpResponse::build(self.status_code()).json(ErrorResponse {
            error: error_type.to_string(),
            message: self.to_string(),
            details: None,
            request_id,
        })
    }

    /// Get the error type string for this error
    fn error_type(&self) -> &'static str {
        match self {
            ApiError::BasketError(_) => "basket_error",
            ApiError::OracleError(_) => "oracle_error",
            ApiError::DatabaseError(_) => "database_error",
            ApiError::NotFound(_) => "not_found",
            ApiError::BadRequest(_) => "bad_request",
            ApiError::Unauthorized(_) => "unauthorized",
            ApiError::Forbidden(_) => "forbidden",
            ApiError::OracleNotConfigured => "oracle_not_configured",
            ApiError::InternalError(_) => "internal_error",
        }
    }
}

impl ResponseError for ApiError {
    fn status_code(&self) -> StatusCode {
        match self {
            ApiError::NotFound(_) => StatusCode::NOT_FOUND,
            ApiError::BadRequest(_) => StatusCode::BAD_REQUEST,
            ApiError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            ApiError::Forbidden(_) => StatusCode::FORBIDDEN,
            ApiError::OracleNotConfigured => StatusCode::SERVICE_UNAVAILABLE,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        // Note: ResponseError doesn't have access to HttpRequest
        // For request_id in errors, handlers should use ApiError::to_response(req) instead
        // The request_id is still available in X-Correlation-ID response header
        HttpResponse::build(self.status_code()).json(ErrorResponse {
            error: self.error_type().to_string(),
            message: self.to_string(),
            details: None,
            request_id: None, // Not available without HttpRequest
        })
    }
}

impl From<BasketError> for ApiError {
    fn from(err: BasketError) -> Self {
        ApiError::BasketError(err)
    }
}

impl From<OracleError> for ApiError {
    fn from(err: OracleError) -> Self {
        ApiError::OracleError(err)
    }
}

impl From<DbError> for ApiError {
    fn from(err: DbError) -> Self {
        ApiError::DatabaseError(err)
    }
}

/// Helper function to handle database errors safely.
/// Logs the actual error server-side but returns a generic message to clients.
/// This prevents information disclosure of database structure/constraints.
pub fn handle_db_error<E: std::fmt::Display>(error: E, context: &str) -> ApiError {
    // Log the actual error server-side for debugging
    tracing::error!(
        error = %error,
        context = %context,
        "Database operation failed"
    );
    // Return generic error to client - never expose internal details
    ApiError::InternalError("A database error occurred. Please try again later.".to_string())
}
