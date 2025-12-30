//! # Meridian REST API
//!
//! HTTP API service for stablecoin management and oracle integration

pub mod error;
pub mod handlers;
pub mod middleware;
pub mod models;
pub mod routes;
pub mod state;
pub mod telemetry;

pub use error::ApiError;
pub use middleware::{CorrelationId, CorrelationIdMiddleware, RateLimitHeadersMiddleware};
pub use state::AppState;
