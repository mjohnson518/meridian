//! KYC/AML handlers

use crate::error::ApiError;
use crate::state::AppState;
use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
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
    req: web::Json<SubmitKycRequest>,
) -> Result<HttpResponse, ApiError> {
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

    // Update user KYC status
    sqlx::query!(
        "UPDATE users SET kyc_status = 'PENDING_REVIEW', updated_at = NOW() WHERE id = $1",
        req.user_id
    )
    .execute(state.db_pool.as_ref())
    .await
    .ok();

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
    user_id: web::Path<i32>,
) -> Result<HttpResponse, ApiError> {
    let user_id = user_id.into_inner();

    // Get user's KYC status
    let user_status = sqlx::query!("SELECT kyc_status FROM users WHERE id = $1", user_id)
        .fetch_optional(state.db_pool.as_ref())
        .await
        .map_err(|e| ApiError::InternalError(format!("Database error: {}", e)))?;

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
    .map_err(|e| ApiError::InternalError(format!("Database error: {}", e)))?;

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
    application_id: web::Path<i32>,
) -> Result<HttpResponse, ApiError> {
    let app_id = application_id.into_inner();

    tracing::info!(application_id = app_id, "Approving KYC application");

    // Get application to find user_id
    let application = sqlx::query!("SELECT user_id FROM kyc_applications WHERE id = $1", app_id)
        .fetch_optional(state.db_pool.as_ref())
        .await
        .map_err(|e| ApiError::InternalError(format!("Database error: {}", e)))?;

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
    .map_err(|e| ApiError::InternalError(format!("Database error: {}", e)))?;

    // Update user KYC status
    sqlx::query!(
        "UPDATE users SET kyc_status = 'APPROVED', updated_at = NOW() WHERE id = $1",
        application.user_id
    )
    .execute(state.db_pool.as_ref())
    .await
    .map_err(|e| ApiError::InternalError(format!("Database error: {}", e)))?;

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
    application_id: web::Path<i32>,
    reason: web::Json<serde_json::Value>,
) -> Result<HttpResponse, ApiError> {
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
        .map_err(|e| ApiError::InternalError(format!("Database error: {}", e)))?;

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
    .map_err(|e| ApiError::InternalError(format!("Database error: {}", e)))?;

    // Update user status
    sqlx::query!(
        "UPDATE users SET kyc_status = 'REJECTED', updated_at = NOW() WHERE id = $1",
        application.user_id
    )
    .execute(state.db_pool.as_ref())
    .await
    .ok();

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "KYC application rejected",
        "application_id": app_id,
        "reason": rejection_reason
    })))
}

