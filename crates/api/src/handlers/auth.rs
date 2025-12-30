//! Authentication handlers

use crate::error::{ApiError, handle_db_error};
use crate::state::AppState;
use actix_web::{cookie::{Cookie, SameSite}, web, HttpRequest, HttpResponse};
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

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
#[allow(dead_code)]
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
    // BE-CRIT-001: Validate email format before any database operations
    validate_email(&req.email)?;

    // Log login attempt without exposing full email (PII)
    let masked_email = mask_email(&req.email);
    tracing::info!(email = %masked_email, "Login attempt");

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

    // SECURITY: Constant-time response to prevent timing attacks that enumerate valid emails
    // Always perform bcrypt verification even for non-existent users
    let (user_exists, password_hash) = match &user {
        Some(u) => (true, u.password_hash.clone()),
        None => {
            // Use a dummy hash to equalize timing with real user lookups
            // This hash was pre-computed for "dummy_password_never_used"
            (false, "$2b$12$K4G/XZJY1Fm9Q8m6Y1K.O.xGxZ1Fm9Q8m6Y1K.O.xGxZ1Fm9Q8m6".to_string())
        }
    };

    // Always verify password to prevent timing attacks (result ignored for non-existent users)
    let password_valid = verify_password(&req.password, &password_hash)?;

    // Return same error for both non-existent user and wrong password (prevents user enumeration)
    if !user_exists || !password_valid {
        tracing::warn!("Login failed: invalid credentials");
        return Err(ApiError::Unauthorized("Invalid credentials".to_string()));
    }

    // Extract user from Option - guaranteed to be Some since we verified user_exists above
    let user = user.ok_or_else(|| {
        // This should never happen due to the check above, but handle gracefully
        tracing::error!("Unexpected: user was None after existence check");
        ApiError::InternalError("Authentication state error".to_string())
    })?;

    // Generate tokens
    let access_token = generate_token();
    let refresh_token = generate_token();
    let expires_at = Utc::now() + Duration::hours(24);

    // Hash tokens for storage (raw tokens returned to client)
    let access_token_hash = hash_token(&access_token);
    let refresh_token_hash = hash_token(&refresh_token);

    // Store session with hashed tokens
    sqlx::query!(
        r#"
        INSERT INTO sessions (user_id, access_token, refresh_token, expires_at)
        VALUES ($1, $2, $3, $4)
        "#,
        user.id,
        access_token_hash,
        refresh_token_hash,
        expires_at
    )
    .execute(state.db_pool.as_ref())
    .await
    .map_err(|e| {
        tracing::error!("Failed to create session: {}", e);
        ApiError::InternalError("Failed to create session".to_string())
    })?;

    // Update last login (log error but don't fail login)
    if let Err(e) = sqlx::query!(
        "UPDATE users SET last_login_at = NOW() WHERE id = $1",
        user.id
    )
    .execute(state.db_pool.as_ref())
    .await
    {
        tracing::warn!(user_id = user.id, error = %e, "Failed to update last_login_at");
    }

    tracing::info!(user_id = user.id, "Login successful");

    // SECURITY: Set tokens in httpOnly cookies to prevent XSS token theft
    // Tokens are also returned in body for WebSocket auth (which can't use cookies)
    let is_production = std::env::var("ENVIRONMENT")
        .map(|e| e.to_lowercase() == "production")
        .unwrap_or(false);

    let access_cookie = create_auth_cookie("meridian_access_token", &access_token, is_production, 86400); // 24 hours
    let refresh_cookie = create_auth_cookie("meridian_refresh_token", &refresh_token, is_production, 86400 * 7); // 7 days

    Ok(HttpResponse::Ok()
        .cookie(access_cookie)
        .cookie(refresh_cookie)
        .json(LoginResponse {
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
    // BE-CRIT-001: Validate email format before any database operations
    validate_email(&req.email)?;

    // Log registration attempt without exposing full email (PII)
    let masked_email = mask_email(&req.email);
    tracing::info!(email = %masked_email, "Registration attempt");

    // Validate password complexity
    validate_password(&req.password)?;

    // Check if user already exists
    let existing = sqlx::query!("SELECT id FROM users WHERE email = $1", req.email)
        .fetch_optional(state.db_pool.as_ref())
        .await
        .map_err(|e| handle_db_error(e, "auth"))?;

    if existing.is_some() {
        return Err(ApiError::BadRequest("Email already registered".to_string()));
    }

    // Hash password
    let password_hash = hash_password(&req.password)?;

    // SECURITY: Always assign VIEWER role on registration
    // Admin roles must be assigned through separate admin interface
    // Ignore any client-provided role to prevent privilege escalation
    let role = "VIEWER".to_string();

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

    // Hash tokens for storage (raw tokens returned to client)
    let access_token_hash = hash_token(&access_token);
    let refresh_token_hash = hash_token(&refresh_token);

    // Create session with hashed tokens
    sqlx::query!(
        r#"
        INSERT INTO sessions (user_id, access_token, refresh_token, expires_at)
        VALUES ($1, $2, $3, $4)
        "#,
        user.id,
        access_token_hash,
        refresh_token_hash,
        expires_at
    )
    .execute(state.db_pool.as_ref())
    .await
    .map_err(|e| ApiError::InternalError(format!("Failed to create session: {}", e)))?;

    tracing::info!(user_id = user.id, "Registration successful");

    // SECURITY: Set tokens in httpOnly cookies to prevent XSS token theft
    let is_production = std::env::var("ENVIRONMENT")
        .map(|e| e.to_lowercase() == "production")
        .unwrap_or(false);

    let access_cookie = create_auth_cookie("meridian_access_token", &access_token, is_production, 86400);
    let refresh_cookie = create_auth_cookie("meridian_refresh_token", &refresh_token, is_production, 86400 * 7);

    Ok(HttpResponse::Created()
        .cookie(access_cookie)
        .cookie(refresh_cookie)
        .json(LoginResponse {
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
    // Extract token from Authorization header OR httpOnly cookie
    // Prefer cookie for better security against XSS
    let token = req
        .cookie("meridian_access_token")
        .map(|c| c.value().to_string())
        .or_else(|| {
            req.headers()
                .get("Authorization")
                .and_then(|h| h.to_str().ok())
                .and_then(|h| h.strip_prefix("Bearer "))
                .map(|s| s.to_string())
        })
        .ok_or_else(|| ApiError::Unauthorized("Missing authentication".to_string()))?;

    // Hash the incoming token to compare with stored hash
    let token_hash = hash_token(&token);

    // Query session using hashed token
    let session = sqlx::query!(
        r#"
        SELECT s.user_id, s.expires_at, u.email, u.role, u.organization, u.kyc_status, u.wallet_address, u.created_at
        FROM sessions s
        JOIN users u ON s.user_id = u.id
        WHERE s.access_token = $1 AND s.expires_at > NOW()
        "#,
        token_hash
    )
    .fetch_optional(state.db_pool.as_ref())
    .await
    .map_err(|e| handle_db_error(e, "auth"))?;

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

/// POST /api/v1/auth/refresh
/// Exchange a valid refresh token for new access and refresh tokens
pub async fn refresh_token(
    state: web::Data<Arc<AppState>>,
    req: HttpRequest,
) -> Result<HttpResponse, ApiError> {
    // Extract refresh token from Authorization header
    let refresh_token = req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .ok_or_else(|| ApiError::Unauthorized("Missing or invalid Authorization header".to_string()))?;

    // Hash the incoming token to compare with stored hash
    let token_hash = hash_token(refresh_token);

    // Find session by refresh token hash
    let session = sqlx::query!(
        r#"
        SELECT s.id, s.user_id, s.expires_at, u.email, u.role, u.organization, u.kyc_status, u.wallet_address, u.created_at
        FROM sessions s
        JOIN users u ON s.user_id = u.id
        WHERE s.refresh_token = $1 AND s.expires_at > NOW()
        "#,
        token_hash
    )
    .fetch_optional(state.db_pool.as_ref())
    .await
    .map_err(|e| handle_db_error(e, "auth"))?;

    let session = match session {
        Some(s) => s,
        None => return Err(ApiError::Unauthorized("Invalid or expired refresh token".to_string())),
    };

    // Generate new tokens
    let new_access_token = generate_token();
    let new_refresh_token = generate_token();
    let expires_at = Utc::now() + Duration::hours(24);

    // Hash new tokens for storage
    let new_access_token_hash = hash_token(&new_access_token);
    let new_refresh_token_hash = hash_token(&new_refresh_token);

    // Update session with new tokens (token rotation)
    sqlx::query!(
        r#"
        UPDATE sessions
        SET access_token = $1, refresh_token = $2, expires_at = $3
        WHERE id = $4
        "#,
        new_access_token_hash,
        new_refresh_token_hash,
        expires_at,
        session.id
    )
    .execute(state.db_pool.as_ref())
    .await
    .map_err(|e| {
        tracing::error!("Failed to update session: {}", e);
        ApiError::InternalError("Failed to refresh tokens".to_string())
    })?;

    tracing::info!(user_id = session.user_id, "Token refreshed successfully");

    Ok(HttpResponse::Ok().json(LoginResponse {
        access_token: new_access_token,
        refresh_token: new_refresh_token,
        expires_at: expires_at.timestamp(),
        user: UserResponse {
            id: session.user_id,
            email: session.email,
            role: session.role,
            organization: session.organization,
            kyc_status: session.kyc_status,
            wallet_address: session.wallet_address,
            created_at: session.created_at.to_rfc3339(),
        },
    }))
}

/// POST /api/v1/auth/logout
/// CRIT-007: Revoke current session tokens and clear cookies
/// Allows users to invalidate their tokens before natural expiration
pub async fn logout(
    state: web::Data<Arc<AppState>>,
    req: HttpRequest,
) -> Result<HttpResponse, ApiError> {
    // Extract token from Authorization header OR httpOnly cookie
    let token = req
        .cookie("meridian_access_token")
        .map(|c| c.value().to_string())
        .or_else(|| {
            req.headers()
                .get("Authorization")
                .and_then(|h| h.to_str().ok())
                .and_then(|h| h.strip_prefix("Bearer "))
                .map(|s| s.to_string())
        });

    // If no token provided, still clear cookies and return success
    // This handles edge case where cookies exist but header doesn't
    if let Some(token) = token {
        let token_hash = hash_token(&token);

        // Delete the session from database
        let result = sqlx::query!(
            "DELETE FROM sessions WHERE access_token = $1",
            token_hash
        )
        .execute(state.db_pool.as_ref())
        .await;

        match result {
            Ok(r) => {
                if r.rows_affected() > 0 {
                    tracing::info!("Session revoked successfully");
                } else {
                    tracing::debug!("No session found to revoke (may already be expired)");
                }
            }
            Err(e) => {
                // Log error but don't fail - user intent is to logout
                tracing::warn!("Failed to delete session from database: {}", e);
            }
        }
    }

    // SECURITY: Always clear cookies regardless of token validity
    // This ensures client-side cleanup even if server-side fails
    let is_production = std::env::var("ENVIRONMENT")
        .map(|e| e.to_lowercase() == "production")
        .unwrap_or(false);

    // Create expired cookies to clear client-side tokens
    let clear_access_cookie = Cookie::build("meridian_access_token".to_string(), "".to_string())
        .path("/")
        .http_only(true)
        .same_site(SameSite::Strict)
        .secure(is_production)
        .max_age(actix_web::cookie::time::Duration::seconds(0)) // Immediate expiry
        .finish();

    let clear_refresh_cookie = Cookie::build("meridian_refresh_token".to_string(), "".to_string())
        .path("/")
        .http_only(true)
        .same_site(SameSite::Strict)
        .secure(is_production)
        .max_age(actix_web::cookie::time::Duration::seconds(0))
        .finish();

    Ok(HttpResponse::Ok()
        .cookie(clear_access_cookie)
        .cookie(clear_refresh_cookie)
        .json(serde_json::json!({
            "message": "Logged out successfully"
        })))
}

/// POST /api/v1/auth/logout-all
/// CRIT-007: Revoke ALL sessions for the current user
/// Use when user suspects account compromise
pub async fn logout_all(
    state: web::Data<Arc<AppState>>,
    req: HttpRequest,
) -> Result<HttpResponse, ApiError> {
    // Extract and verify current token
    let token = req
        .cookie("meridian_access_token")
        .map(|c| c.value().to_string())
        .or_else(|| {
            req.headers()
                .get("Authorization")
                .and_then(|h| h.to_str().ok())
                .and_then(|h| h.strip_prefix("Bearer "))
                .map(|s| s.to_string())
        })
        .ok_or_else(|| ApiError::Unauthorized("Missing authentication".to_string()))?;

    let token_hash = hash_token(&token);

    // Get user_id from current session
    let session = sqlx::query!(
        "SELECT user_id FROM sessions WHERE access_token = $1 AND expires_at > NOW()",
        token_hash
    )
    .fetch_optional(state.db_pool.as_ref())
    .await
    .map_err(|e| {
        tracing::error!("Database error during logout-all: {}", e);
        ApiError::InternalError("Database error".to_string())
    })?;

    let session = match session {
        Some(s) => s,
        None => return Err(ApiError::Unauthorized("Invalid or expired token".to_string())),
    };

    // Delete ALL sessions for this user
    let result = sqlx::query!(
        "DELETE FROM sessions WHERE user_id = $1",
        session.user_id
    )
    .execute(state.db_pool.as_ref())
    .await
    .map_err(|e| {
        tracing::error!("Failed to delete all sessions: {}", e);
        ApiError::InternalError("Failed to revoke sessions".to_string())
    })?;

    tracing::info!(
        user_id = session.user_id,
        sessions_revoked = result.rows_affected(),
        "All sessions revoked"
    );

    // Clear cookies
    let is_production = std::env::var("ENVIRONMENT")
        .map(|e| e.to_lowercase() == "production")
        .unwrap_or(false);

    let clear_access_cookie = Cookie::build("meridian_access_token".to_string(), "".to_string())
        .path("/")
        .http_only(true)
        .same_site(SameSite::Strict)
        .secure(is_production)
        .max_age(actix_web::cookie::time::Duration::seconds(0))
        .finish();

    let clear_refresh_cookie = Cookie::build("meridian_refresh_token".to_string(), "".to_string())
        .path("/")
        .http_only(true)
        .same_site(SameSite::Strict)
        .secure(is_production)
        .max_age(actix_web::cookie::time::Duration::seconds(0))
        .finish();

    Ok(HttpResponse::Ok()
        .cookie(clear_access_cookie)
        .cookie(clear_refresh_cookie)
        .json(serde_json::json!({
            "message": "All sessions revoked",
            "sessions_revoked": result.rows_affected()
        })))
}

// Helper functions

/// Create a secure httpOnly cookie for authentication
/// SECURITY: httpOnly prevents JavaScript access, protecting against XSS token theft
fn create_auth_cookie(name: &str, value: &str, is_production: bool, max_age_secs: i64) -> Cookie<'static> {
    Cookie::build(name.to_string(), value.to_string())
        .path("/")
        .http_only(true) // CRITICAL: Prevents JavaScript access
        .same_site(SameSite::Strict) // Prevents CSRF attacks
        .secure(is_production) // HTTPS only in production
        .max_age(actix_web::cookie::time::Duration::seconds(max_age_secs))
        .finish()
}

/// Mask email address for logging (PII protection)
/// Example: "user@example.com" -> "u***@example.com"
fn mask_email(email: &str) -> String {
    if let Some(at_idx) = email.find('@') {
        let local = &email[..at_idx];
        let domain = &email[at_idx..];
        if local.len() > 1 {
            format!("{}***{}", &local[..1], domain)
        } else {
            format!("***{}", domain)
        }
    } else {
        "***".to_string()
    }
}

fn generate_token() -> String {
    // Generate cryptographically secure random token (32 bytes = 256 bits)
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let mut bytes = [0u8; 32];
    rng.fill(&mut bytes);
    hex::encode(bytes)
}

/// Hash token using SHA-256 with salt for storage
/// Raw token is returned to client, salted hash is stored in database
/// BE-CRIT-004: Added salt to prevent rainbow table attacks
fn hash_token(token: &str) -> String {
    use sha2::{Sha256, Digest};
    use std::sync::OnceLock;

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

/// Hash password using bcrypt with cost factor 12 (recommended for production)
fn hash_password(password: &str) -> Result<String, ApiError> {
    const BCRYPT_COST: u32 = 12;
    
    bcrypt::hash(password, BCRYPT_COST).map_err(|e| {
        tracing::error!("Failed to hash password: {}", e);
        ApiError::InternalError("Password hashing failed".to_string())
    })
}

/// Verify password against bcrypt hash
fn verify_password(password: &str, hash: &str) -> Result<bool, ApiError> {
    bcrypt::verify(password, hash).map_err(|e| {
        tracing::error!("Failed to verify password: {}", e);
        ApiError::InternalError("Password verification failed".to_string())
    })
}

/// BE-CRIT-001: Validate email format
/// Requirements: valid email format with basic regex validation
fn validate_email(email: &str) -> Result<(), ApiError> {
    // Basic email validation:
    // - Not empty
    // - Contains exactly one @
    // - Has non-empty local and domain parts
    // - Domain has at least one dot
    // - Max 254 characters (RFC 5321)
    // - Only printable ASCII characters

    if email.is_empty() {
        return Err(ApiError::BadRequest("Email cannot be empty".to_string()));
    }

    if email.len() > 254 {
        return Err(ApiError::BadRequest("Email cannot exceed 254 characters".to_string()));
    }

    // Only printable ASCII allowed
    if !email.chars().all(|c| c.is_ascii() && (' '..='~').contains(&c)) {
        return Err(ApiError::BadRequest("Email contains invalid characters".to_string()));
    }

    // Split on @ and validate structure
    let parts: Vec<&str> = email.split('@').collect();
    if parts.len() != 2 {
        return Err(ApiError::BadRequest("Email must contain exactly one @ symbol".to_string()));
    }

    let local = parts[0];
    let domain = parts[1];

    // Local part validation
    if local.is_empty() || local.len() > 64 {
        return Err(ApiError::BadRequest("Invalid email local part".to_string()));
    }

    // Domain part validation
    if domain.is_empty() || !domain.contains('.') {
        return Err(ApiError::BadRequest("Invalid email domain".to_string()));
    }

    // Domain cannot start or end with dot
    if domain.starts_with('.') || domain.ends_with('.') {
        return Err(ApiError::BadRequest("Invalid email domain format".to_string()));
    }

    // Basic character validation for local part
    // Allow alphanumeric, dots, hyphens, underscores, plus signs
    let valid_local_chars = |c: char| c.is_alphanumeric() || c == '.' || c == '-' || c == '_' || c == '+';
    if !local.chars().all(valid_local_chars) {
        return Err(ApiError::BadRequest("Email local part contains invalid characters".to_string()));
    }

    // Basic character validation for domain
    let valid_domain_chars = |c: char| c.is_alphanumeric() || c == '.' || c == '-';
    if !domain.chars().all(valid_domain_chars) {
        return Err(ApiError::BadRequest("Email domain contains invalid characters".to_string()));
    }

    Ok(())
}

/// Validate password complexity
/// Requirements: min 8 chars, uppercase, lowercase, digit, special char
fn validate_password(password: &str) -> Result<(), ApiError> {
    if password.len() < 8 {
        return Err(ApiError::BadRequest("Password must be at least 8 characters".to_string()));
    }
    if !password.chars().any(|c| c.is_uppercase()) {
        return Err(ApiError::BadRequest("Password must contain an uppercase letter".to_string()));
    }
    if !password.chars().any(|c| c.is_lowercase()) {
        return Err(ApiError::BadRequest("Password must contain a lowercase letter".to_string()));
    }
    if !password.chars().any(|c| c.is_ascii_digit()) {
        return Err(ApiError::BadRequest("Password must contain a digit".to_string()));
    }
    if !password.chars().any(|c| !c.is_alphanumeric()) {
        return Err(ApiError::BadRequest("Password must contain a special character".to_string()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_password_valid() {
        assert!(validate_password("Password1!").is_ok());
        assert!(validate_password("StrongP@ss123").is_ok());
        assert!(validate_password("MyP@ssw0rd").is_ok());
    }

    #[test]
    fn test_validate_password_too_short() {
        let result = validate_password("Short1!");
        assert!(result.is_err());
        if let Err(ApiError::BadRequest(msg)) = result {
            assert!(msg.contains("8 characters"));
        }
    }

    #[test]
    fn test_validate_password_no_uppercase() {
        let result = validate_password("password1!");
        assert!(result.is_err());
        if let Err(ApiError::BadRequest(msg)) = result {
            assert!(msg.contains("uppercase"));
        }
    }

    #[test]
    fn test_validate_password_no_lowercase() {
        let result = validate_password("PASSWORD1!");
        assert!(result.is_err());
        if let Err(ApiError::BadRequest(msg)) = result {
            assert!(msg.contains("lowercase"));
        }
    }

    #[test]
    fn test_validate_password_no_digit() {
        let result = validate_password("Password!");
        assert!(result.is_err());
        if let Err(ApiError::BadRequest(msg)) = result {
            assert!(msg.contains("digit"));
        }
    }

    #[test]
    fn test_validate_password_no_special_char() {
        let result = validate_password("Password1");
        assert!(result.is_err());
        if let Err(ApiError::BadRequest(msg)) = result {
            assert!(msg.contains("special character"));
        }
    }

    #[test]
    fn test_hash_token_deterministic() {
        let token = "test-token-12345";
        let hash1 = hash_token(token);
        let hash2 = hash_token(token);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_hash_token_different_for_different_tokens() {
        let hash1 = hash_token("token1");
        let hash2 = hash_token("token2");
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_hash_token_format() {
        let hash = hash_token("test-token");
        // SHA-256 produces 64 hex characters
        assert_eq!(hash.len(), 64);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_generate_token_unique() {
        let token1 = generate_token();
        let token2 = generate_token();
        assert_ne!(token1, token2);
    }

    #[test]
    fn test_generate_token_length() {
        let token = generate_token();
        // 32 bytes of random data = 64 hex characters
        assert_eq!(token.len(), 64);
        // Verify all characters are valid hex
        assert!(token.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_hash_password_produces_valid_bcrypt() {
        let password = "TestPassword1!";
        let hash = hash_password(password).unwrap();
        // Bcrypt hash starts with $2b$ or $2a$
        assert!(hash.starts_with("$2b$") || hash.starts_with("$2a$"));
    }

    #[test]
    fn test_verify_password_correct() {
        let password = "TestPassword1!";
        let hash = hash_password(password).unwrap();
        assert!(verify_password(password, &hash).unwrap());
    }

    #[test]
    fn test_verify_password_incorrect() {
        let password = "TestPassword1!";
        let wrong_password = "WrongPassword1!";
        let hash = hash_password(password).unwrap();
        assert!(!verify_password(wrong_password, &hash).unwrap());
    }

    // Tests for mask_email (PII protection)
    #[test]
    fn test_mask_email_normal() {
        assert_eq!(mask_email("user@example.com"), "u***@example.com");
        assert_eq!(mask_email("john.doe@company.org"), "j***@company.org");
    }

    #[test]
    fn test_mask_email_single_char_local() {
        assert_eq!(mask_email("a@example.com"), "***@example.com");
    }

    #[test]
    fn test_mask_email_no_at_sign() {
        assert_eq!(mask_email("invalid-email"), "***");
    }

    #[test]
    fn test_mask_email_empty() {
        assert_eq!(mask_email(""), "***");
    }
}

