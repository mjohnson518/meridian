//! Health check handler

use crate::models::HealthResponse;
use crate::state::AppState;
use actix_web::{web, HttpResponse};
use std::sync::Arc;

/// Health check endpoint
///
/// GET /health
pub async fn health_check(state: web::Data<Arc<AppState>>) -> HttpResponse {
    let oracle_enabled = {
        let oracle_guard = state.oracle.read().await;
        oracle_guard.is_some()
    };

    let baskets_count = {
        let baskets = state.baskets.read().await;
        baskets.len()
    };

    let response = HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        oracle_enabled,
        baskets_count,
    };

    HttpResponse::Ok().json(response)
}

