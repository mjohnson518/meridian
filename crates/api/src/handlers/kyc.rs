//! KYC/AML handlers

use crate::error::{ApiError, handle_db_error};
use crate::state::AppState;
use actix_web::{web, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use sha2::{Sha256, Digest};
use std::sync::Arc;

#[derive(Debug, Deserialize)]
pub struct SubmitKycRequest {
    pub user_id: i32,
    pub entity_info: JsonValue,
    pub documents: JsonValue,
    pub compliance: JsonValue,
    pub wallet: JsonValue,
}

#[derive(Debug, Serialize)]
pub struct KycStatusResponse {
    pub application_id: Option<i32>,
    pub status: String,
    pub submitted_at: Option<String>,
    pub reviewed_at: Option<String>,
    pub rejection_reason: Option<String>,
}

/// POST /api/v1/kyc/submit
pub async fn submit_kyc(
    state: web::Data<Arc<AppState>>,
    http_req: HttpRequest,
    req: web::Json<SubmitKycRequest>,
) -> Result<HttpResponse, ApiError> {
    // SECURITY: Verify authenticated user matches the user_id in request
    let auth_user = get_authenticated_user(&state, &http_req).await?;
    if auth_user.user_id != req.user_id {
        tracing::warn!(
            auth_user_id = auth_user.user_id,
            requested_user_id = req.user_id,
            "KYC submission rejected: user_id mismatch"
        );
        return Err(ApiError::Forbidden("Cannot submit KYC for another user".to_string()));
    }

    tracing::info!(user_id = req.user_id, "KYC application submitted");

    // Combine all data into JSONB
    let application_data = serde_json::json!({
        "entity_info": req.entity_info,
        "documents": req.documents,
        "compliance": req.compliance,
        "wallet": req.wallet,
        "submitted_at": chrono::Utc::now().to_rfc3339()
    });

    // Insert application
    let application = sqlx::query!(
        r#"
        INSERT INTO kyc_applications (user_id, application_data, status)
        VALUES ($1, $2, 'PENDING_REVIEW')
        RETURNING id, status, created_at
        "#,
        req.user_id,
        application_data
    )
    .fetch_one(state.db_pool.as_ref())
    .await
    .map_err(|e| {
        tracing::error!("Failed to create KYC application: {}", e);
        ApiError::InternalError("Failed to submit KYC application".to_string())
    })?;

    // Update user KYC status (log error but don't fail submission)
    if let Err(e) = sqlx::query!(
        "UPDATE users SET kyc_status = 'PENDING_REVIEW', updated_at = NOW() WHERE id = $1",
        req.user_id
    )
    .execute(state.db_pool.as_ref())
    .await
    {
        tracing::warn!(user_id = req.user_id, error = %e, "Failed to update user kyc_status");
    }

    tracing::info!(
        application_id = application.id,
        "KYC application created successfully"
    );

    Ok(HttpResponse::Created().json(serde_json::json!({
        "application_id": application.id,
        "status": application.status,
        "submitted_at": application.created_at.to_rfc3339(),
        "message": "KYC application submitted successfully. Review typically takes 24-48 hours."
    })))
}

/// GET /api/v1/kyc/status/{user_id}
pub async fn get_kyc_status(
    state: web::Data<Arc<AppState>>,
    req: HttpRequest,
    user_id: web::Path<i32>,
) -> Result<HttpResponse, ApiError> {
    let user_id = user_id.into_inner();

    // Verify authenticated user matches requested user_id (or is admin)
    let auth_user = get_authenticated_user(&state, &req).await?;
    if auth_user.user_id != user_id && auth_user.role != "ADMIN" {
        return Err(ApiError::Forbidden("Cannot access other user's KYC status".to_string()));
    }

    // Get user's KYC status
    let user_status = sqlx::query!("SELECT kyc_status FROM users WHERE id = $1", user_id)
        .fetch_optional(state.db_pool.as_ref())
        .await
        .map_err(|e| handle_db_error(e, "kyc"))?;

    let user_status = match user_status {
        Some(status) => status.kyc_status,
        None => return Err(ApiError::NotFound("User not found".to_string())),
    };

    // Get latest application if exists
    let application = sqlx::query!(
        r#"
        SELECT id, status, created_at, reviewed_at, rejection_reason
        FROM kyc_applications
        WHERE user_id = $1
        ORDER BY created_at DESC
        LIMIT 1
        "#,
        user_id
    )
    .fetch_optional(state.db_pool.as_ref())
    .await
    .map_err(|e| handle_db_error(e, "kyc"))?;

    let response = if let Some(app) = application {
        KycStatusResponse {
            application_id: Some(app.id),
            status: app.status,
            submitted_at: Some(app.created_at.to_rfc3339()),
            reviewed_at: app.reviewed_at.map(|dt| dt.to_rfc3339()),
            rejection_reason: app.rejection_reason,
        }
    } else {
        KycStatusResponse {
            application_id: None,
            status: user_status,
            submitted_at: None,
            reviewed_at: None,
            rejection_reason: None,
        }
    };

    Ok(HttpResponse::Ok().json(response))
}

/// PUT /api/v1/kyc/approve/{application_id}
pub async fn approve_kyc(
    state: web::Data<Arc<AppState>>,
    req: HttpRequest,
    application_id: web::Path<i32>,
) -> Result<HttpResponse, ApiError> {
    // Verify caller is an admin
    verify_admin(&state, &req).await?;

    let app_id = application_id.into_inner();

    tracing::info!(application_id = app_id, "Approving KYC application");

    // Get application to find user_id
    let application = sqlx::query!("SELECT user_id FROM kyc_applications WHERE id = $1", app_id)
        .fetch_optional(state.db_pool.as_ref())
        .await
        .map_err(|e| handle_db_error(e, "kyc"))?;

    let application = match application {
        Some(app) => app,
        None => return Err(ApiError::NotFound("Application not found".to_string())),
    };

    // Update application status
    sqlx::query!(
        r#"
        UPDATE kyc_applications 
        SET status = 'APPROVED', reviewed_at = NOW(), updated_at = NOW()
        WHERE id = $1
        "#,
        app_id
    )
    .execute(state.db_pool.as_ref())
    .await
    .map_err(|e| handle_db_error(e, "kyc"))?;

    // Update user KYC status
    sqlx::query!(
        "UPDATE users SET kyc_status = 'APPROVED', updated_at = NOW() WHERE id = $1",
        application.user_id
    )
    .execute(state.db_pool.as_ref())
    .await
    .map_err(|e| handle_db_error(e, "kyc"))?;

    tracing::info!(application_id = app_id, user_id = application.user_id, "KYC approved");

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "KYC application approved",
        "application_id": app_id,
        "status": "APPROVED"
    })))
}

/// PUT /api/v1/kyc/reject/{application_id}
pub async fn reject_kyc(
    state: web::Data<Arc<AppState>>,
    req: HttpRequest,
    application_id: web::Path<i32>,
    reason: web::Json<serde_json::Value>,
) -> Result<HttpResponse, ApiError> {
    // Verify caller is an admin
    verify_admin(&state, &req).await?;

    let app_id = application_id.into_inner();
    let rejection_reason = reason
        .get("reason")
        .and_then(|r| r.as_str())
        .unwrap_or("No reason provided");

    tracing::info!(application_id = app_id, "Rejecting KYC application");

    // Get application
    let application = sqlx::query!("SELECT user_id FROM kyc_applications WHERE id = $1", app_id)
        .fetch_optional(state.db_pool.as_ref())
        .await
        .map_err(|e| handle_db_error(e, "kyc"))?;

    let application = match application {
        Some(app) => app,
        None => return Err(ApiError::NotFound("Application not found".to_string())),
    };

    // Update application
    sqlx::query!(
        r#"
        UPDATE kyc_applications 
        SET status = 'REJECTED', rejection_reason = $2, reviewed_at = NOW(), updated_at = NOW()
        WHERE id = $1
        "#,
        app_id,
        rejection_reason
    )
    .execute(state.db_pool.as_ref())
    .await
    .map_err(|e| handle_db_error(e, "kyc"))?;

    // Update user status (log error but don't fail rejection)
    if let Err(e) = sqlx::query!(
        "UPDATE users SET kyc_status = 'REJECTED', updated_at = NOW() WHERE id = $1",
        application.user_id
    )
    .execute(state.db_pool.as_ref())
    .await
    {
        tracing::warn!(user_id = application.user_id, error = %e, "Failed to update user kyc_status to REJECTED");
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "KYC application rejected",
        "application_id": app_id,
        "reason": rejection_reason
    })))
}

struct AuthenticatedUser {
    user_id: i32,
    role: String,
}

/// Get authenticated user info from request token
async fn get_authenticated_user(
    state: &web::Data<Arc<AppState>>,
    req: &HttpRequest,
) -> Result<AuthenticatedUser, ApiError> {
    let token = req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .ok_or_else(|| ApiError::Unauthorized("Missing Authorization header".to_string()))?;

    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    let token_hash = hex::encode(hasher.finalize());

    let session = sqlx::query!(
        r#"
        SELECT s.user_id, u.role
        FROM sessions s
        JOIN users u ON s.user_id = u.id
        WHERE s.access_token = $1 AND s.expires_at > NOW()
        "#,
        token_hash
    )
    .fetch_optional(state.db_pool.as_ref())
    .await
    .map_err(|e| handle_db_error(e, "kyc"))?;

    match session {
        Some(s) => Ok(AuthenticatedUser {
            user_id: s.user_id,
            role: s.role,
        }),
        None => Err(ApiError::Unauthorized("Invalid or expired token".to_string())),
    }
}

/// Verify the caller has ADMIN role
async fn verify_admin(
    state: &web::Data<Arc<AppState>>,
    req: &HttpRequest,
) -> Result<(), ApiError> {
    // Extract token from Authorization header
    let token = req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .ok_or_else(|| ApiError::Unauthorized("Missing Authorization header".to_string()))?;

    // Hash token to compare with stored hash
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    let token_hash = hex::encode(hasher.finalize());

    // Look up session and user role
    let session = sqlx::query!(
        r#"
        SELECT u.role
        FROM sessions s
        JOIN users u ON s.user_id = u.id
        WHERE s.access_token = $1 AND s.expires_at > NOW()
        "#,
        token_hash
    )
    .fetch_optional(state.db_pool.as_ref())
    .await
    .map_err(|e| handle_db_error(e, "kyc"))?;

    match session {
        Some(s) if s.role == "ADMIN" => Ok(()),
        Some(_) => Err(ApiError::Forbidden("Admin access required".to_string())),
        None => Err(ApiError::Unauthorized("Invalid or expired token".to_string())),
    }
}

