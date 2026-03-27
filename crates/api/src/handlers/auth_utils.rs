//! Shared authentication utilities
//!
//! HIGH-003: Centralized token hashing to eliminate code duplication
//! across handlers (previously duplicated in 6 files)
//!
//! Phase C.4: RBAC — `require_role` and `authenticate_request` consolidate
//! the scattered verify_admin / get_authenticated_user_id helpers.

use crate::error::ApiError;
use actix_web::HttpRequest;
use sha2::{Sha256, Digest};
use std::sync::OnceLock;
use uuid::Uuid;

/// How a request was authenticated
#[derive(Debug, Clone)]
pub enum AuthType {
    /// Standard Bearer token (user session)
    Session,
    /// X-API-Key header (machine-to-machine)
    ApiKey { key_id: Uuid },
}

/// Resolved identity from any supported auth method
#[derive(Debug, Clone)]
pub struct AuthContext {
    pub user_id: Option<i32>,
    pub tenant_id: Option<Uuid>,
    pub role: String,
    pub auth_type: AuthType,
}

impl AuthContext {
    /// Returns true if this identity has at least the given role level.
    ///
    /// Role hierarchy: ADMIN > TREASURY > COMPLIANCE > VIEWER
    pub fn has_role(&self, required: &str) -> bool {
        let level = |r: &str| match r.to_uppercase().as_str() {
            "ADMIN" => 4,
            "TREASURY" => 3,
            "COMPLIANCE" => 2,
            "VIEWER" => 1,
            _ => 0,
        };
        level(&self.role) >= level(required)
    }
}

/// Authenticate a request from either a Bearer session token or `X-API-Key` header.
///
/// Returns the resolved `AuthContext` or an appropriate `ApiError`.
pub async fn authenticate_request(
    pool: &sqlx::PgPool,
    req: &HttpRequest,
) -> Result<AuthContext, ApiError> {
    // Try X-API-Key header first
    if let Some(api_key) = req.headers().get("X-API-Key").and_then(|h| h.to_str().ok()) {
        return authenticate_api_key(pool, api_key).await;
    }

    // Fall back to Bearer token
    let token = req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .ok_or_else(|| ApiError::Unauthorized("Missing Authorization header".to_string()))?;

    authenticate_session(pool, token).await
}

async fn authenticate_session(
    pool: &sqlx::PgPool,
    token: &str,
) -> Result<AuthContext, ApiError> {
    let token_hash = hash_token_for_lookup(token);

    #[derive(sqlx::FromRow)]
    struct SessionRow {
        user_id: i32,
        role: String,
        tenant_id: Option<Uuid>,
    }

    let row: Option<SessionRow> = sqlx::query_as(
        r#"
        SELECT u.id AS user_id, u.role, u.tenant_id
        FROM sessions s
        JOIN users u ON s.user_id = u.id
        WHERE s.access_token = $1 AND s.expires_at > NOW()
        "#,
    )
    .bind(&token_hash)
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        tracing::error!("DB error in session auth: {}", e);
        ApiError::InternalError("Database error".to_string())
    })?;

    match row {
        Some(r) => Ok(AuthContext {
            user_id: Some(r.user_id),
            tenant_id: r.tenant_id,
            role: r.role,
            auth_type: AuthType::Session,
        }),
        None => Err(ApiError::Unauthorized("Invalid or expired token".to_string())),
    }
}

async fn authenticate_api_key(
    pool: &sqlx::PgPool,
    raw_key: &str,
) -> Result<AuthContext, ApiError> {
    let key_hash = hash_api_key(raw_key);

    #[derive(sqlx::FromRow)]
    struct ApiKeyRow {
        id: Uuid,
        tenant_id: Uuid,
        permissions: serde_json::Value,
    }

    let row: Option<ApiKeyRow> = sqlx::query_as(
        r#"
        SELECT id, tenant_id, permissions
        FROM api_keys
        WHERE key_hash = $1
          AND revoked_at IS NULL
          AND (expires_at IS NULL OR expires_at > NOW())
        "#,
    )
    .bind(&key_hash)
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        tracing::error!("DB error in API key auth: {}", e);
        ApiError::InternalError("Database error".to_string())
    })?;

    match row {
        Some(r) => {
            // Update last_used_at (best-effort, don't fail if this errors)
            let _ = sqlx::query("UPDATE api_keys SET last_used_at = NOW() WHERE id = $1")
                .bind(r.id)
                .execute(pool)
                .await;

            // Derive role from permissions: keys with "admin" permission get ADMIN,
            // keys with "mint" or "burn" get TREASURY, others get VIEWER
            let perms: Vec<String> = serde_json::from_value(r.permissions).unwrap_or_default();
            let role = if perms.iter().any(|p| p == "admin") {
                "ADMIN"
            } else if perms.iter().any(|p| p == "mint" || p == "burn") {
                "TREASURY"
            } else if perms.iter().any(|p| p.starts_with("compliance")) {
                "COMPLIANCE"
            } else {
                "VIEWER"
            };

            Ok(AuthContext {
                user_id: None,
                tenant_id: Some(r.tenant_id),
                role: role.to_string(),
                auth_type: AuthType::ApiKey { key_id: r.id },
            })
        }
        None => Err(ApiError::Unauthorized("Invalid or revoked API key".to_string())),
    }
}

/// Require a minimum role level, returning 403 if insufficient.
pub async fn require_role(
    pool: &sqlx::PgPool,
    req: &HttpRequest,
    required_role: &str,
) -> Result<AuthContext, ApiError> {
    let ctx = authenticate_request(pool, req).await?;
    if ctx.has_role(required_role) {
        Ok(ctx)
    } else {
        tracing::warn!(
            role = %ctx.role,
            required = required_role,
            "Access denied: insufficient role"
        );
        Err(ApiError::Forbidden(format!("{} role required", required_role)))
    }
}

/// Hash an API key for storage/lookup.
/// Uses API_KEY_SALT env var (separate from session token salt).
pub fn hash_api_key(raw_key: &str) -> String {
    static API_KEY_SALT: OnceLock<String> = OnceLock::new();
    let salt = API_KEY_SALT.get_or_init(|| {
        std::env::var("API_KEY_SALT").unwrap_or_else(|_| {
            if std::env::var("ENVIRONMENT")
                .map(|e| e.to_lowercase() == "production")
                .unwrap_or(false)
            {
                panic!("API_KEY_SALT must be set in production");
            }
            "dev-api-key-salt-not-for-production".to_string()
        })
    });

    let mut hasher = Sha256::new();
    hasher.update(raw_key.as_bytes());
    hasher.update(salt.as_bytes());
    hex::encode(hasher.finalize())
}

/// Hash token using SHA-256 with salt for database lookup
///
/// This function is used for session token verification across all handlers.
/// Raw token is returned to client, salted hash is stored in database.
///
/// # Security
/// - BE-CRIT-004: Salt prevents rainbow table attacks
/// - Uses OnceLock for thread-safe, one-time initialization
/// - Panics in production if SESSION_TOKEN_SALT is not set
pub fn hash_token_for_lookup(token: &str) -> String {
    // Use static OnceLock to cache the salt for performance
    static TOKEN_SALT: OnceLock<String> = OnceLock::new();

    let salt = TOKEN_SALT.get_or_init(|| {
        std::env::var("SESSION_TOKEN_SALT").unwrap_or_else(|_| {
            // In development, use a default salt with warning
            if std::env::var("ENVIRONMENT")
                .map(|e| e.to_lowercase() == "production")
                .unwrap_or(false)
            {
                // Production MUST have salt configured - panic to prevent insecure operation
                panic!("SESSION_TOKEN_SALT must be set in production environment");
            }
            tracing::warn!("Using default session token salt - set SESSION_TOKEN_SALT in production");
            "dev-session-salt-not-for-production".to_string()
        })
    });

    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    hasher.update(salt.as_bytes());
    hex::encode(hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_token_produces_consistent_output() {
        let token = "test-token-123";
        let hash1 = hash_token_for_lookup(token);
        let hash2 = hash_token_for_lookup(token);

        assert_eq!(hash1, hash2, "Same token should produce same hash");
        assert_eq!(hash1.len(), 64, "SHA-256 hex output should be 64 chars");
    }

    #[test]
    fn test_different_tokens_produce_different_hashes() {
        let hash1 = hash_token_for_lookup("token-a");
        let hash2 = hash_token_for_lookup("token-b");

        assert_ne!(hash1, hash2, "Different tokens should produce different hashes");
    }

    #[test]
    fn test_hash_is_hexadecimal() {
        let hash = hash_token_for_lookup("any-token");

        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()),
                "Hash should only contain hex characters");
    }
}
