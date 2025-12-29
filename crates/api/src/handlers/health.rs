//! Health check and metrics handlers

use crate::models::HealthResponse;
use crate::state::AppState;
use actix_web::{web, HttpResponse};
use meridian_db::BasketRepository;
use std::sync::Arc;
use std::time::Instant;

/// Health check endpoint with database verification
///
/// GET /health
pub async fn health_check(state: web::Data<Arc<AppState>>) -> HttpResponse {
    let start = Instant::now();

    // Verify database connectivity
    let db_healthy = sqlx::query("SELECT 1")
        .fetch_one(state.db_pool.as_ref())
        .await
        .is_ok();

    if !db_healthy {
        return HttpResponse::ServiceUnavailable().json(serde_json::json!({
            "status": "unhealthy",
            "error": "database connection failed",
            "version": env!("CARGO_PKG_VERSION")
        }));
    }

    let oracle_enabled = {
        let oracle_guard = state.oracle.read().await;
        oracle_guard.is_some()
    };

    let basket_repo = BasketRepository::new((*state.db_pool).clone());
    let baskets_count = basket_repo.count().await.unwrap_or(0) as usize;

    let response_time_ms = start.elapsed().as_millis();

    let response = HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        oracle_enabled,
        baskets_count,
    };

    HttpResponse::Ok()
        .insert_header(("X-Response-Time-Ms", response_time_ms.to_string()))
        .json(response)
}

/// Prometheus-compatible metrics endpoint
///
/// GET /metrics
pub async fn metrics(state: web::Data<Arc<AppState>>) -> HttpResponse {
    let mut output = String::new();

    // Service info
    output.push_str("# HELP meridian_info Service information\n");
    output.push_str("# TYPE meridian_info gauge\n");
    output.push_str(&format!(
        "meridian_info{{version=\"{}\"}} 1\n",
        env!("CARGO_PKG_VERSION")
    ));

    // Database pool stats
    let pool_size = state.db_pool.size();
    let pool_idle = state.db_pool.num_idle();

    output.push_str("# HELP meridian_db_pool_size Database connection pool size\n");
    output.push_str("# TYPE meridian_db_pool_size gauge\n");
    output.push_str(&format!("meridian_db_pool_size {}\n", pool_size));

    output.push_str("# HELP meridian_db_pool_idle Idle database connections\n");
    output.push_str("# TYPE meridian_db_pool_idle gauge\n");
    output.push_str(&format!("meridian_db_pool_idle {}\n", pool_idle));

    // Oracle status
    let oracle_enabled = {
        let oracle_guard = state.oracle.read().await;
        if oracle_guard.is_some() { 1 } else { 0 }
    };
    output.push_str("# HELP meridian_oracle_enabled Oracle integration status\n");
    output.push_str("# TYPE meridian_oracle_enabled gauge\n");
    output.push_str(&format!("meridian_oracle_enabled {}\n", oracle_enabled));

    // Basket count
    let basket_repo = BasketRepository::new((*state.db_pool).clone());
    let baskets_count = basket_repo.count().await.unwrap_or(0);
    output.push_str("# HELP meridian_baskets_total Total number of baskets\n");
    output.push_str("# TYPE meridian_baskets_total gauge\n");
    output.push_str(&format!("meridian_baskets_total {}\n", baskets_count));

    // User count
    let user_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
        .fetch_one(state.db_pool.as_ref())
        .await
        .unwrap_or(0);
    output.push_str("# HELP meridian_users_total Total registered users\n");
    output.push_str("# TYPE meridian_users_total gauge\n");
    output.push_str(&format!("meridian_users_total {}\n", user_count));

    // Operations count by type
    let operations: Vec<(String, i64)> = sqlx::query_as(
        "SELECT operation_type, COUNT(*) as count FROM operations GROUP BY operation_type"
    )
    .fetch_all(state.db_pool.as_ref())
    .await
    .unwrap_or_default();

    output.push_str("# HELP meridian_operations_total Total operations by type\n");
    output.push_str("# TYPE meridian_operations_total counter\n");
    for (op_type, count) in operations {
        output.push_str(&format!(
            "meridian_operations_total{{type=\"{}\"}} {}\n",
            op_type, count
        ));
    }

    HttpResponse::Ok()
        .content_type("text/plain; version=0.0.4")
        .body(output)
}
