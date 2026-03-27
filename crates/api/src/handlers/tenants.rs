//! Tenant, API key, and webhook management handlers
//!
//! Phase C: Multi-tenancy — each institutional client is a tenant with isolated
//! data, API keys, and webhook subscriptions.
//!
//! All endpoints require ADMIN role (session or API key with "admin" permission).

use crate::error::ApiError;
use crate::handlers::auth_utils::{hash_api_key, require_role};
use crate::state::AppState;
use actix_web::{web, HttpRequest, HttpResponse};
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::sync::Arc;
use uuid::Uuid;

// ─── Tenant CRUD ────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CreateTenantRequest {
    pub name: String,
    pub legal_entity: String,
    pub jurisdiction: String,
    #[serde(default)]
    pub custody_config: serde_json::Value,
    #[serde(default)]
    pub chain_config: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct TenantResponse {
    pub id: Uuid,
    pub name: String,
    pub legal_entity: String,
    pub jurisdiction: String,
    pub status: String,
    pub created_at: String,
}

/// POST /api/v1/tenants
pub async fn create_tenant(
    state: web::Data<Arc<AppState>>,
    req: HttpRequest,
    body: web::Json<CreateTenantRequest>,
) -> Result<HttpResponse, ApiError> {
    require_role(state.db_pool.as_ref(), &req, "ADMIN").await?;

    #[derive(sqlx::FromRow)]
    struct Row {
        id: Uuid,
        name: String,
        legal_entity: String,
        jurisdiction: String,
        status: String,
        created_at: chrono::DateTime<chrono::Utc>,
    }

    let row: Row = sqlx::query_as(
        r#"
        INSERT INTO tenants (name, legal_entity, jurisdiction, custody_config, chain_config)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, name, legal_entity, jurisdiction, status, created_at
        "#,
    )
    .bind(&body.name)
    .bind(&body.legal_entity)
    .bind(&body.jurisdiction)
    .bind(&body.custody_config)
    .bind(&body.chain_config)
    .fetch_one(state.db_pool.as_ref())
    .await
    .map_err(|e| {
        tracing::error!("Failed to create tenant: {}", e);
        ApiError::InternalError("Failed to create tenant".to_string())
    })?;

    tracing::info!(tenant_id = %row.id, name = %row.name, "Tenant created");

    Ok(HttpResponse::Created().json(TenantResponse {
        id: row.id,
        name: row.name,
        legal_entity: row.legal_entity,
        jurisdiction: row.jurisdiction,
        status: row.status,
        created_at: row.created_at.to_rfc3339(),
    }))
}

/// GET /api/v1/tenants
pub async fn list_tenants(
    state: web::Data<Arc<AppState>>,
    req: HttpRequest,
) -> Result<HttpResponse, ApiError> {
    require_role(state.db_pool.as_ref(), &req, "ADMIN").await?;

    #[derive(sqlx::FromRow, Serialize)]
    struct Row {
        id: Uuid,
        name: String,
        legal_entity: String,
        jurisdiction: String,
        status: String,
        created_at: chrono::DateTime<chrono::Utc>,
    }

    let rows: Vec<Row> = sqlx::query_as(
        "SELECT id, name, legal_entity, jurisdiction, status, created_at FROM tenants ORDER BY created_at DESC"
    )
    .fetch_all(state.db_pool.as_ref())
    .await
    .map_err(|e| {
        tracing::error!("Failed to list tenants: {}", e);
        ApiError::InternalError("Failed to list tenants".to_string())
    })?;

    Ok(HttpResponse::Ok().json(serde_json::json!({ "tenants": rows })))
}

/// GET /api/v1/tenants/{id}
pub async fn get_tenant(
    state: web::Data<Arc<AppState>>,
    req: HttpRequest,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let ctx = require_role(state.db_pool.as_ref(), &req, "VIEWER").await?;
    let tenant_id = path.into_inner();

    // ADMIN can see any tenant; others can only see their own
    if !ctx.has_role("ADMIN") && ctx.tenant_id != Some(tenant_id) {
        return Err(ApiError::Forbidden("Cannot access another tenant's data".to_string()));
    }

    #[derive(sqlx::FromRow, Serialize)]
    struct Row {
        id: Uuid,
        name: String,
        legal_entity: String,
        jurisdiction: String,
        status: String,
        custody_config: serde_json::Value,
        chain_config: serde_json::Value,
        created_at: chrono::DateTime<chrono::Utc>,
    }

    let row: Option<Row> = sqlx::query_as(
        "SELECT id, name, legal_entity, jurisdiction, status, custody_config, chain_config, created_at FROM tenants WHERE id = $1"
    )
    .bind(tenant_id)
    .fetch_optional(state.db_pool.as_ref())
    .await
    .map_err(|e| {
        tracing::error!("Failed to get tenant: {}", e);
        ApiError::InternalError("Failed to get tenant".to_string())
    })?;

    match row {
        Some(r) => Ok(HttpResponse::Ok().json(r)),
        None => Err(ApiError::NotFound("Tenant not found".to_string())),
    }
}

// ─── API Key Management ──────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CreateApiKeyRequest {
    pub tenant_id: Uuid,
    pub name: String,
    #[serde(default)]
    pub permissions: Vec<String>,
    pub rate_limit_per_minute: Option<i32>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// POST /api/v1/auth/api-keys
///
/// Creates a new API key. The raw key is returned ONCE — it cannot be retrieved again.
pub async fn create_api_key(
    state: web::Data<Arc<AppState>>,
    req: HttpRequest,
    body: web::Json<CreateApiKeyRequest>,
) -> Result<HttpResponse, ApiError> {
    let ctx = require_role(state.db_pool.as_ref(), &req, "ADMIN").await?;

    // Generate raw key: mk_ + 32 random bytes as hex
    let random_bytes: Vec<u8> = rand::thread_rng().sample_iter(&rand::distributions::Standard).take(32).collect();
    let raw_key = format!("mk_{}", hex::encode(&random_bytes));
    let key_prefix = &raw_key[..12.min(raw_key.len())];
    let key_hash = hash_api_key(&raw_key);

    let permissions_json = serde_json::to_value(&body.permissions)
        .unwrap_or(serde_json::json!([]));

    let key_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO api_keys (tenant_id, name, key_hash, key_prefix, permissions, rate_limit_per_minute, expires_at, created_by)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING id
        "#,
    )
    .bind(body.tenant_id)
    .bind(&body.name)
    .bind(&key_hash)
    .bind(key_prefix)
    .bind(&permissions_json)
    .bind(body.rate_limit_per_minute.unwrap_or(60))
    .bind(body.expires_at)
    .bind(ctx.user_id)
    .fetch_one(state.db_pool.as_ref())
    .await
    .map_err(|e| {
        tracing::error!("Failed to create API key: {}", e);
        ApiError::InternalError("Failed to create API key".to_string())
    })?;

    tracing::info!(key_id = %key_id, tenant_id = %body.tenant_id, name = %body.name, "API key created");

    Ok(HttpResponse::Created().json(serde_json::json!({
        "id": key_id,
        "name": body.name,
        "key": raw_key,           // Shown ONCE — not stored
        "key_prefix": key_prefix,
        "permissions": body.permissions,
        "tenant_id": body.tenant_id,
        "expires_at": body.expires_at,
        "warning": "Store this key securely — it will not be shown again"
    })))
}

/// GET /api/v1/auth/api-keys?tenant_id={uuid}
pub async fn list_api_keys(
    state: web::Data<Arc<AppState>>,
    req: HttpRequest,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> Result<HttpResponse, ApiError> {
    let ctx = require_role(state.db_pool.as_ref(), &req, "ADMIN").await?;

    // Determine which tenant to list keys for
    let tenant_id: Option<Uuid> = query.get("tenant_id")
        .and_then(|s| s.parse().ok())
        .or(ctx.tenant_id);

    #[derive(sqlx::FromRow, Serialize)]
    struct Row {
        id: Uuid,
        name: String,
        key_prefix: String,
        permissions: serde_json::Value,
        rate_limit_per_minute: i32,
        last_used_at: Option<chrono::DateTime<chrono::Utc>>,
        expires_at: Option<chrono::DateTime<chrono::Utc>>,
        revoked_at: Option<chrono::DateTime<chrono::Utc>>,
        created_at: chrono::DateTime<chrono::Utc>,
    }

    let rows: Vec<Row> = sqlx::query_as(
        r#"
        SELECT id, name, key_prefix, permissions, rate_limit_per_minute,
               last_used_at, expires_at, revoked_at, created_at
        FROM api_keys
        WHERE ($1::uuid IS NULL OR tenant_id = $1)
        ORDER BY created_at DESC
        "#,
    )
    .bind(tenant_id)
    .fetch_all(state.db_pool.as_ref())
    .await
    .map_err(|e| {
        tracing::error!("Failed to list API keys: {}", e);
        ApiError::InternalError("Failed to list API keys".to_string())
    })?;

    Ok(HttpResponse::Ok().json(serde_json::json!({ "api_keys": rows })))
}

/// DELETE /api/v1/auth/api-keys/{id}
pub async fn revoke_api_key(
    state: web::Data<Arc<AppState>>,
    req: HttpRequest,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    require_role(state.db_pool.as_ref(), &req, "ADMIN").await?;
    let key_id = path.into_inner();

    let rows_affected = sqlx::query(
        "UPDATE api_keys SET revoked_at = NOW() WHERE id = $1 AND revoked_at IS NULL"
    )
    .bind(key_id)
    .execute(state.db_pool.as_ref())
    .await
    .map_err(|e| {
        tracing::error!("Failed to revoke API key: {}", e);
        ApiError::InternalError("Failed to revoke API key".to_string())
    })?
    .rows_affected();

    if rows_affected == 0 {
        return Err(ApiError::NotFound("API key not found or already revoked".to_string()));
    }

    tracing::info!(key_id = %key_id, "API key revoked");
    Ok(HttpResponse::Ok().json(serde_json::json!({ "revoked": true, "id": key_id })))
}

// ─── Webhook Management ──────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CreateWebhookRequest {
    pub tenant_id: Uuid,
    pub url: String,
    pub events: Vec<String>,
    pub timeout_secs: Option<i32>,
}

const VALID_EVENTS: &[&str] = &[
    "operation.completed",
    "operation.failed",
    "reserve.attestation",
    "compliance.alert",
    "kyc.approved",
    "kyc.rejected",
];

/// POST /api/v1/webhooks
pub async fn create_webhook(
    state: web::Data<Arc<AppState>>,
    req: HttpRequest,
    body: web::Json<CreateWebhookRequest>,
) -> Result<HttpResponse, ApiError> {
    require_role(state.db_pool.as_ref(), &req, "ADMIN").await?;

    // Validate event types
    for event in &body.events {
        if !VALID_EVENTS.contains(&event.as_str()) {
            return Err(ApiError::BadRequest(format!(
                "Unknown event type '{}'. Valid: {}",
                event,
                VALID_EVENTS.join(", ")
            )));
        }
    }

    // Validate URL scheme
    if !body.url.starts_with("https://") {
        return Err(ApiError::BadRequest(
            "Webhook URL must use HTTPS".to_string()
        ));
    }

    // Generate signing secret (32 random bytes → hex)
    let secret_bytes: Vec<u8> = rand::thread_rng().sample_iter(&rand::distributions::Standard).take(32).collect();
    let raw_secret = hex::encode(&secret_bytes);
    let mut hasher = Sha256::new();
    hasher.update(raw_secret.as_bytes());
    let secret_hash = hex::encode(hasher.finalize());

    let webhook_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO webhooks (tenant_id, url, events, secret_hash, timeout_secs)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id
        "#,
    )
    .bind(body.tenant_id)
    .bind(&body.url)
    .bind(&body.events)
    .bind(&secret_hash)
    .bind(body.timeout_secs.unwrap_or(10))
    .fetch_one(state.db_pool.as_ref())
    .await
    .map_err(|e| {
        tracing::error!("Failed to create webhook: {}", e);
        ApiError::InternalError("Failed to create webhook".to_string())
    })?;

    tracing::info!(webhook_id = %webhook_id, url = %body.url, "Webhook registered");

    Ok(HttpResponse::Created().json(serde_json::json!({
        "id": webhook_id,
        "url": body.url,
        "events": body.events,
        "secret": raw_secret,   // Shown ONCE — use to verify X-Meridian-Signature header
        "warning": "Store this secret securely — it will not be shown again"
    })))
}

/// GET /api/v1/webhooks?tenant_id={uuid}
pub async fn list_webhooks(
    state: web::Data<Arc<AppState>>,
    req: HttpRequest,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> Result<HttpResponse, ApiError> {
    let ctx = require_role(state.db_pool.as_ref(), &req, "ADMIN").await?;

    let tenant_id: Option<Uuid> = query.get("tenant_id")
        .and_then(|s| s.parse().ok())
        .or(ctx.tenant_id);

    #[derive(sqlx::FromRow, Serialize)]
    struct Row {
        id: Uuid,
        url: String,
        events: Vec<String>,
        is_active: bool,
        timeout_secs: i32,
        created_at: chrono::DateTime<chrono::Utc>,
    }

    let rows: Vec<Row> = sqlx::query_as(
        r#"
        SELECT id, url, events, is_active, timeout_secs, created_at
        FROM webhooks
        WHERE ($1::uuid IS NULL OR tenant_id = $1)
        ORDER BY created_at DESC
        "#,
    )
    .bind(tenant_id)
    .fetch_all(state.db_pool.as_ref())
    .await
    .map_err(|e| {
        tracing::error!("Failed to list webhooks: {}", e);
        ApiError::InternalError("Failed to list webhooks".to_string())
    })?;

    Ok(HttpResponse::Ok().json(serde_json::json!({ "webhooks": rows })))
}

/// DELETE /api/v1/webhooks/{id}
pub async fn delete_webhook(
    state: web::Data<Arc<AppState>>,
    req: HttpRequest,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    require_role(state.db_pool.as_ref(), &req, "ADMIN").await?;
    let webhook_id = path.into_inner();

    let rows_affected = sqlx::query(
        "UPDATE webhooks SET is_active = FALSE WHERE id = $1 AND is_active = TRUE"
    )
    .bind(webhook_id)
    .execute(state.db_pool.as_ref())
    .await
    .map_err(|e| {
        tracing::error!("Failed to deactivate webhook: {}", e);
        ApiError::InternalError("Failed to deactivate webhook".to_string())
    })?
    .rows_affected();

    if rows_affected == 0 {
        return Err(ApiError::NotFound("Webhook not found or already inactive".to_string()));
    }

    tracing::info!(webhook_id = %webhook_id, "Webhook deactivated");
    Ok(HttpResponse::Ok().json(serde_json::json!({ "deactivated": true, "id": webhook_id })))
}

/// POST /api/v1/webhooks/test
///
/// Sends a test payload to all active webhooks for a tenant.
pub async fn test_webhook(
    state: web::Data<Arc<AppState>>,
    req: HttpRequest,
    body: web::Json<serde_json::Value>,
) -> Result<HttpResponse, ApiError> {
    let ctx = require_role(state.db_pool.as_ref(), &req, "ADMIN").await?;

    let tenant_id = ctx.tenant_id.ok_or_else(|| {
        ApiError::BadRequest("Cannot test webhooks without a tenant context".to_string())
    })?;

    // Queue test delivery for all active webhooks
    let webhook_ids: Vec<Uuid> = sqlx::query_scalar(
        "SELECT id FROM webhooks WHERE tenant_id = $1 AND is_active = TRUE"
    )
    .bind(tenant_id)
    .fetch_all(state.db_pool.as_ref())
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch webhooks: {}", e);
        ApiError::InternalError("Failed to fetch webhooks".to_string())
    })?;

    let test_payload = serde_json::json!({
        "event": "webhook.test",
        "tenant_id": tenant_id,
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "data": body.0,
    });

    for webhook_id in &webhook_ids {
        let _ = sqlx::query(
            r#"
            INSERT INTO webhook_deliveries (webhook_id, event_type, payload, next_attempt_at)
            VALUES ($1, 'webhook.test', $2, NOW())
            "#,
        )
        .bind(webhook_id)
        .bind(&test_payload)
        .execute(state.db_pool.as_ref())
        .await;
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "queued": webhook_ids.len(),
        "webhook_ids": webhook_ids,
        "note": "Deliveries queued — check webhook_deliveries table for status"
    })))
}
