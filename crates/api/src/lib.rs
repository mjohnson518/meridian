//! # Meridian REST API
//!
//! HTTP API service for stablecoin management and oracle integration

pub mod error;
pub mod handlers;
pub mod models;
pub mod routes;
pub mod state;

pub use error::ApiError;
pub use state::AppState;
