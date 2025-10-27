//! Authentication handlers

use crate::error::ApiError;
use crate::state::AppState;
use actix_web::{web, HttpRequest, HttpResponse};
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: i64,
    pub user: UserResponse,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: i32,
    pub email: String,
    pub role: String,
    pub organization: String,
    pub kyc_status: String,
    pub wallet_address: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub organization: String,
    pub role: Option<String>,
}

/// POST /api/v1/auth/login
pub async fn login(
    state: web::Data<Arc<AppState>>,
    req: web::Json<LoginRequest>,
) -> Result<HttpResponse, ApiError> {
    tracing::info!(email = %req.email, "Login attempt");

    // Query user from database
    let user = sqlx::query!(
        r#"
        SELECT id, email, password_hash, role, organization, kyc_status, wallet_address, created_at
        FROM users
        WHERE email = $1
        "#,
        req.email
    )
    .fetch_optional(state.db_pool.as_ref())
    .await
    .map_err(|e| {
        tracing::error!("Database error during login: {}", e);
        ApiError::InternalError("Failed to query user".to_string())
    })?;

    let user = match user {
        Some(u) => u,
        None => {
            tracing::warn!("Login failed: user not found");
            return Err(ApiError::Unauthorized("Invalid credentials".to_string()));
        }
    };

    // Verify password
    let password_valid = verify_password(&req.password, &user.password_hash)?;
    if !password_valid {
        tracing::warn!("Login failed: invalid password");
        return Err(ApiError::Unauthorized("Invalid credentials".to_string()));
    }

    // Generate tokens
    let access_token = generate_token();
    let refresh_token = generate_token();
    let expires_at = Utc::now() + Duration::hours(24);

    // Store session
    sqlx::query!(
        r#"
        INSERT INTO sessions (user_id, access_token, refresh_token, expires_at)
        VALUES ($1, $2, $3, $4)
        "#,
        user.id,
        access_token,
        refresh_token,
        expires_at
    )
    .execute(state.db_pool.as_ref())
    .await
    .map_err(|e| {
        tracing::error!("Failed to create session: {}", e);
        ApiError::InternalError("Failed to create session".to_string())
    })?;

    // Update last login
    sqlx::query!(
        "UPDATE users SET last_login_at = NOW() WHERE id = $1",
        user.id
    )
    .execute(state.db_pool.as_ref())
    .await
    .ok();

    tracing::info!(user_id = user.id, "Login successful");

    Ok(HttpResponse::Ok().json(LoginResponse {
        access_token,
        refresh_token,
        expires_at: expires_at.timestamp(),
        user: UserResponse {
            id: user.id,
            email: user.email,
            role: user.role,
            organization: user.organization,
            kyc_status: user.kyc_status,
            wallet_address: user.wallet_address,
            created_at: user.created_at.to_rfc3339(),
        },
    }))
}

/// POST /api/v1/auth/register
pub async fn register(
    state: web::Data<Arc<AppState>>,
    req: web::Json<RegisterRequest>,
) -> Result<HttpResponse, ApiError> {
    tracing::info!(email = %req.email, "Registration attempt");

    // Check if user already exists
    let existing = sqlx::query!("SELECT id FROM users WHERE email = $1", req.email)
        .fetch_optional(state.db_pool.as_ref())
        .await
        .map_err(|e| ApiError::InternalError(format!("Database error: {}", e)))?;

    if existing.is_some() {
        return Err(ApiError::BadRequest("Email already registered".to_string()));
    }

    // Hash password
    let password_hash = hash_password(&req.password)?;
    let role = req.role.clone().unwrap_or_else(|| "VIEWER".to_string());

    // Insert user
    let user = sqlx::query!(
        r#"
        INSERT INTO users (email, password_hash, role, organization, kyc_status)
        VALUES ($1, $2, $3, $4, 'NOT_STARTED')
        RETURNING id, email, role, organization, kyc_status, wallet_address, created_at
        "#,
        req.email,
        password_hash,
        role,
        req.organization
    )
    .fetch_one(state.db_pool.as_ref())
    .await
    .map_err(|e| {
        tracing::error!("Failed to create user: {}", e);
        ApiError::InternalError("Failed to create user".to_string())
    })?;

    // Generate tokens
    let access_token = generate_token();
    let refresh_token = generate_token();
    let expires_at = Utc::now() + Duration::hours(24);

    // Create session
    sqlx::query!(
        r#"
        INSERT INTO sessions (user_id, access_token, refresh_token, expires_at)
        VALUES ($1, $2, $3, $4)
        "#,
        user.id,
        access_token,
        refresh_token,
        expires_at
    )
    .execute(state.db_pool.as_ref())
    .await
    .map_err(|e| ApiError::InternalError(format!("Failed to create session: {}", e)))?;

    tracing::info!(user_id = user.id, "Registration successful");

    Ok(HttpResponse::Created().json(LoginResponse {
        access_token,
        refresh_token,
        expires_at: expires_at.timestamp(),
        user: UserResponse {
            id: user.id,
            email: user.email,
            role: user.role,
            organization: user.organization,
            kyc_status: user.kyc_status,
            wallet_address: user.wallet_address,
            created_at: user.created_at.to_rfc3339(),
        },
    }))
}

/// GET /api/v1/auth/verify
pub async fn verify(
    state: web::Data<Arc<AppState>>,
    req: HttpRequest,
) -> Result<HttpResponse, ApiError> {
    // Extract Authorization header
    let token = req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .ok_or_else(|| ApiError::Unauthorized("Missing or invalid Authorization header".to_string()))?;

    // Query session
    let session = sqlx::query!(
        r#"
        SELECT s.user_id, s.expires_at, u.email, u.role, u.organization, u.kyc_status, u.wallet_address, u.created_at
        FROM sessions s
        JOIN users u ON s.user_id = u.id
        WHERE s.access_token = $1 AND s.expires_at > NOW()
        "#,
        token
    )
    .fetch_optional(state.db_pool.as_ref())
    .await
    .map_err(|e| ApiError::InternalError(format!("Database error: {}", e)))?;

    let session = match session {
        Some(s) => s,
        None => return Err(ApiError::Unauthorized("Invalid or expired token".to_string())),
    };

    Ok(HttpResponse::Ok().json(UserResponse {
        id: session.user_id,
        email: session.email,
        role: session.role,
        organization: session.organization,
        kyc_status: session.kyc_status,
        wallet_address: session.wallet_address,
        created_at: session.created_at.to_rfc3339(),
    }))
}

// Helper functions
fn generate_token() -> String {
    // Generate secure random token
    Uuid::new_v4().to_string() + &Uuid::new_v4().to_string()
}

fn hash_password(password: &str) -> Result<String, ApiError> {
    // Use bcrypt for password hashing
    // For now, simplified version - in production use bcrypt crate
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    hasher.update(b"meridian_salt_change_in_production");
    let result = hasher.finalize();
    Ok(format!("{:x}", result))
}

fn verify_password(password: &str, hash: &str) -> Result<bool, ApiError> {
    let computed_hash = hash_password(password)?;
    Ok(computed_hash == hash)
}

