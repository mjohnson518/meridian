//! Error types for API operations

use actix_web::{http::StatusCode, HttpResponse, ResponseError};
use meridian_basket::BasketError;
use meridian_oracle::OracleError;
use serde::Serialize;
use std::fmt;

/// API error response
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

/// API errors
#[derive(Debug)]
pub enum ApiError {
    BasketError(BasketError),
    OracleError(OracleError),
    NotFound(String),
    BadRequest(String),
    OracleNotConfigured,
    InternalError(String),
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApiError::BasketError(e) => write!(f, "Basket error: {}", e),
            ApiError::OracleError(e) => write!(f, "Oracle error: {}", e),
            ApiError::NotFound(msg) => write!(f, "Not found: {}", msg),
            ApiError::BadRequest(msg) => write!(f, "Bad request: {}", msg),
            ApiError::OracleNotConfigured => write!(f, "Oracle not configured"),
            ApiError::InternalError(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl ResponseError for ApiError {
    fn status_code(&self) -> StatusCode {
        match self {
            ApiError::NotFound(_) => StatusCode::NOT_FOUND,
            ApiError::BadRequest(_) => StatusCode::BAD_REQUEST,
            ApiError::OracleNotConfigured => StatusCode::SERVICE_UNAVAILABLE,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        let error_type = match self {
            ApiError::BasketError(_) => "basket_error",
            ApiError::OracleError(_) => "oracle_error",
            ApiError::NotFound(_) => "not_found",
            ApiError::BadRequest(_) => "bad_request",
            ApiError::OracleNotConfigured => "oracle_not_configured",
            ApiError::InternalError(_) => "internal_error",
        };

        HttpResponse::build(self.status_code()).json(ErrorResponse {
            error: error_type.to_string(),
            message: self.to_string(),
            details: None,
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

