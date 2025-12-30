//! Basket management handlers

use crate::error::{ApiError, handle_db_error};
use crate::models::{
    BasketResponse, BasketValueResponse, CreateCustomBasketRequest, CreateImfSdrBasketRequest,
    CreateSingleCurrencyBasketRequest, PaginatedResponse, PaginationQuery,
};
use crate::state::AppState;
use actix_web::{web, HttpRequest, HttpResponse};
use chrono::Utc;
use meridian_basket::{CurrencyBasket, CurrencyComponent};
use meridian_db::{BasketRepository, DbError};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

/// Create a new single-currency basket
///
/// POST /api/v1/baskets/single-currency
/// MED-001: Requires authentication
#[utoipa::path(
    post,
    path = "/api/v1/baskets/single-currency",
    tag = "baskets",
    security(("bearer_auth" = [])),
    request_body = CreateSingleCurrencyBasketRequest,
    responses(
        (status = 201, description = "Basket created successfully", body = BasketResponse),
        (status = 400, description = "Invalid request"),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn create_single_currency_basket(
    state: web::Data<Arc<AppState>>,
    http_req: HttpRequest,
    req: web::Json<CreateSingleCurrencyBasketRequest>,
) -> Result<HttpResponse, ApiError> {
    // MED-001: Verify user is authenticated before allowing basket creation
    let _user_id = get_authenticated_user_id(state.db_pool.as_ref(), &http_req).await?;

    tracing::info!(
        name = %req.name,
        currency = %req.currency_code,
        "Creating single-currency basket"
    );

    let basket = CurrencyBasket::new_single_currency(
        req.name.clone(),
        req.currency_code.clone(),
        req.chainlink_feed.clone(),
    )?;

    // Persist basket to database
    let basket_repo = BasketRepository::new((*state.db_pool).clone());
    basket_repo.create(&basket).await.map_err(|e| {
        tracing::error!("Failed to persist basket: {}", e);
        ApiError::InternalError("Failed to persist basket".to_string())
    })?;

    tracing::info!(id = %basket.id, "Basket created and persisted to database");

    Ok(HttpResponse::Created().json(BasketResponse::from(basket)))
}

/// Create an IMF SDR basket
///
/// POST /api/v1/baskets/imf-sdr
/// MED-001: Requires authentication
#[utoipa::path(
    post,
    path = "/api/v1/baskets/imf-sdr",
    tag = "baskets",
    security(("bearer_auth" = [])),
    request_body = CreateImfSdrBasketRequest,
    responses(
        (status = 201, description = "IMF SDR basket created", body = BasketResponse),
        (status = 400, description = "Invalid request"),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn create_imf_sdr_basket(
    state: web::Data<Arc<AppState>>,
    http_req: HttpRequest,
    req: web::Json<CreateImfSdrBasketRequest>,
) -> Result<HttpResponse, ApiError> {
    // MED-001: Verify user is authenticated before allowing basket creation
    let _user_id = get_authenticated_user_id(state.db_pool.as_ref(), &http_req).await?;

    tracing::info!(name = %req.name, "Creating IMF SDR basket");

    let basket = CurrencyBasket::new_imf_sdr(req.name.clone(), req.chainlink_feeds.clone())?;

    // Persist basket to database
    let basket_repo = BasketRepository::new((*state.db_pool).clone());
    basket_repo.create(&basket).await.map_err(|e| {
        tracing::error!("Failed to persist basket: {}", e);
        ApiError::InternalError("Failed to persist basket".to_string())
    })?;

    tracing::info!(id = %basket.id, "IMF SDR basket created and persisted to database");

    Ok(HttpResponse::Created().json(BasketResponse::from(basket)))
}

/// Create a custom basket
///
/// POST /api/v1/baskets/custom
/// MED-001: Requires authentication
#[utoipa::path(
    post,
    path = "/api/v1/baskets/custom",
    tag = "baskets",
    security(("bearer_auth" = [])),
    request_body = CreateCustomBasketRequest,
    responses(
        (status = 201, description = "Custom basket created", body = BasketResponse),
        (status = 400, description = "Invalid request"),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn create_custom_basket(
    state: web::Data<Arc<AppState>>,
    http_req: HttpRequest,
    req: web::Json<CreateCustomBasketRequest>,
) -> Result<HttpResponse, ApiError> {
    // MED-001: Verify user is authenticated before allowing basket creation
    let _user_id = get_authenticated_user_id(state.db_pool.as_ref(), &http_req).await?;

    tracing::info!(
        name = %req.name,
        components = req.components.len(),
        "Creating custom basket"
    );

    // Convert request components to basket components
    let components: Result<Vec<CurrencyComponent>, _> = req
        .components
        .iter()
        .map(|c| {
            CurrencyComponent::new(
                c.currency_code.clone(),
                c.target_weight,
                c.min_weight,
                c.max_weight,
                c.chainlink_feed.clone(),
            )
        })
        .collect();

    let components = components?;

    let basket = CurrencyBasket::new_custom_basket(
        req.name.clone(),
        components,
        req.rebalance_strategy.clone().into(),
    )?;

    // Persist basket to database
    let basket_repo = BasketRepository::new((*state.db_pool).clone());
    basket_repo.create(&basket).await.map_err(|e| {
        tracing::error!("Failed to persist basket: {}", e);
        ApiError::InternalError("Failed to persist basket".to_string())
    })?;

    tracing::info!(id = %basket.id, "Custom basket created and persisted to database");

    Ok(HttpResponse::Created().json(BasketResponse::from(basket)))
}

/// Get basket by ID
///
/// GET /api/v1/baskets/{id}
/// CRIT-005: Requires authentication
#[utoipa::path(
    get,
    path = "/api/v1/baskets/{id}",
    tag = "baskets",
    security(("bearer_auth" = [])),
    params(
        ("id" = Uuid, Path, description = "Basket UUID")
    ),
    responses(
        (status = 200, description = "Basket details", body = BasketResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Basket not found")
    )
)]
pub async fn get_basket(
    state: web::Data<Arc<AppState>>,
    http_req: HttpRequest,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    // CRIT-005: Verify user is authenticated before allowing basket access
    let _user_id = get_authenticated_user_id(state.db_pool.as_ref(), &http_req).await?;

    let basket_id = path.into_inner();

    // HIGH-011: Use info level for significant API operations
    tracing::info!(id = %basket_id, "Fetching basket");

    let basket_repo = BasketRepository::new((*state.db_pool).clone());
    let basket = basket_repo
        .find_by_id(basket_id)
        .await
        .map_err(|e| match e {
            DbError::NotFound(_) => ApiError::NotFound(format!("Basket {} not found", basket_id)),
            _ => {
                tracing::error!("Failed to fetch basket: {}", e);
                ApiError::InternalError("Database error".to_string())
            }
        })?;

    Ok(HttpResponse::Ok().json(BasketResponse::from(basket)))
}

/// List all baskets with pagination
///
/// GET /api/v1/baskets?limit=20&offset=0
/// CRIT-005: Requires authentication
/// CRIT-013: Safe pagination with max limit of 100
#[utoipa::path(
    get,
    path = "/api/v1/baskets",
    tag = "baskets",
    security(("bearer_auth" = [])),
    params(PaginationQuery),
    responses(
        (status = 200, description = "List of baskets", body = PaginatedBasketResponse),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn list_baskets(
    state: web::Data<Arc<AppState>>,
    http_req: HttpRequest,
    query: web::Query<PaginationQuery>,
) -> Result<HttpResponse, ApiError> {
    // CRIT-005: Verify user is authenticated before allowing basket listing
    let _user_id = get_authenticated_user_id(state.db_pool.as_ref(), &http_req).await?;

    let pagination = query.into_inner();
    // HIGH-011: Use info level for significant API operations
    tracing::info!(
        limit = pagination.safe_limit(),
        offset = pagination.offset(),
        "Listing baskets with pagination"
    );

    let basket_repo = BasketRepository::new((*state.db_pool).clone());
    // CRIT-013: Use safe pagination (max 100 enforced)
    let baskets = basket_repo
        .list(pagination.safe_limit(), pagination.offset())
        .await
        .map_err(|e| {
            tracing::error!("Failed to list baskets: {}", e);
            ApiError::InternalError("Database error".to_string())
        })?;

    let items: Vec<BasketResponse> = baskets.into_iter().map(BasketResponse::from).collect();

    let response = PaginatedResponse {
        items,
        limit: pagination.limit.min(100),
        offset: pagination.offset,
        total: None, // Could add count query if needed
    };

    Ok(HttpResponse::Ok().json(response))
}

/// Calculate basket value
///
/// GET /api/v1/baskets/{id}/value
/// CRIT-018 FIX: Requires authentication to prevent information disclosure
#[utoipa::path(
    get,
    path = "/api/v1/baskets/{id}/value",
    tag = "baskets",
    security(("bearer_auth" = [])),
    params(
        ("id" = Uuid, Path, description = "Basket UUID")
    ),
    responses(
        (status = 200, description = "Basket value calculation", body = BasketValueResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Basket not found"),
        (status = 503, description = "Oracle not configured")
    )
)]
pub async fn get_basket_value(
    state: web::Data<Arc<AppState>>,
    http_req: HttpRequest,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    // CRIT-018: Verify user is authenticated before returning basket value with FX rates
    let _user_id = get_authenticated_user_id(state.db_pool.as_ref(), &http_req).await?;

    let basket_id = path.into_inner();

    // HIGH-011: Use info level for significant API operations
    tracing::info!(id = %basket_id, "Calculating basket value");

    let basket_repo = BasketRepository::new((*state.db_pool).clone());
    let basket = basket_repo
        .find_by_id(basket_id)
        .await
        .map_err(|e| match e {
            DbError::NotFound(_) => ApiError::NotFound(format!("Basket {} not found", basket_id)),
            _ => {
                tracing::error!("Failed to fetch basket: {}", e);
                ApiError::InternalError("Database error".to_string())
            }
        })?;

    // Get oracle
    let oracle_guard = state.oracle.read().await;
    let oracle = oracle_guard.as_ref().ok_or(ApiError::OracleNotConfigured)?;

    // Fetch prices for all components
    let mut prices = HashMap::new();
    for component in &basket.components {
        let price = oracle.update_price(&component.currency_code).await?;
        prices.insert(component.currency_code.clone(), price);
    }

    // Calculate value
    let value = basket.calculate_value(&prices)?;
    let needs_rebalancing = basket.needs_rebalancing(&prices)?;

    let response = BasketValueResponse {
        basket_id: basket.id,
        value_usd: value,
        prices_used: prices,
        needs_rebalancing,
        calculated_at: Utc::now().to_rfc3339(),
    };

    Ok(HttpResponse::Ok().json(response))
}

/// Extract authenticated user ID from request token
/// MED-001: Helper function for authentication checks
async fn get_authenticated_user_id(
    pool: &sqlx::PgPool,
    req: &HttpRequest,
) -> Result<i32, ApiError> {
    let token = req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .ok_or_else(|| ApiError::Unauthorized("Missing Authorization header".to_string()))?;

    // CRIT-001 FIX: Use salted hash matching auth.rs for session lookup
    let token_hash = hash_token_for_lookup(token);

    let session = sqlx::query!(
        r#"
        SELECT user_id
        FROM sessions
        WHERE access_token = $1 AND expires_at > NOW()
        "#,
        token_hash
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| handle_db_error(e, "baskets"))?;

    match session {
        Some(s) => Ok(s.user_id),
        None => Err(ApiError::Unauthorized("Invalid or expired token".to_string())),
    }
}

// HIGH-003: Use centralized token hashing from auth_utils
use super::auth_utils::hash_token_for_lookup;
