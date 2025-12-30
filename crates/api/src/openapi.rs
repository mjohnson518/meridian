//! OpenAPI specification for Meridian API
//! CRIT-002: API contract documentation

use utoipa::OpenApi;

use meridian_api::handlers::{baskets, health, oracle, reserves};
use meridian_api::models::{
    BasketResponse, BasketValueResponse, ComponentRequest, ComponentResponse,
    CreateCustomBasketRequest, CreateImfSdrBasketRequest, CreateSingleCurrencyBasketRequest,
    HealthResponse, PaginationQuery, PriceData, PriceResponse, PricesResponse,
    RebalanceStrategyRequest, RegisterFeedRequest,
};

/// Meridian API OpenAPI specification
#[derive(OpenApi)]
#[openapi(
    info(
        title = "Meridian Stablecoin API",
        version = "0.1.0",
        description = "Multi-currency stablecoin platform providing turnkey infrastructure for launching stablecoins backed by sovereign bonds. Supports EUR, GBP, JPY, and other non-USD currencies.",
        license(
            name = "MIT",
            url = "https://opensource.org/licenses/MIT"
        ),
        contact(
            name = "Meridian Team",
            email = "support@meridian.finance"
        )
    ),
    servers(
        (url = "http://localhost:8080", description = "Local development server"),
        (url = "https://api.meridian.finance", description = "Production server")
    ),
    tags(
        (name = "health", description = "Health check and metrics endpoints"),
        (name = "auth", description = "Authentication and session management"),
        (name = "baskets", description = "Currency basket management"),
        (name = "oracle", description = "Price feed and oracle operations"),
        (name = "reserves", description = "Reserve attestation and verification"),
        (name = "operations", description = "Mint and burn operations"),
        (name = "kyc", description = "KYC/AML compliance endpoints"),
        (name = "agents", description = "AI agent (x402) wallet management")
    ),
    paths(
        // Health
        health::health_check,
        health::metrics,
        // Baskets
        baskets::list_baskets,
        baskets::get_basket,
        baskets::get_basket_value,
        baskets::create_single_currency_basket,
        baskets::create_imf_sdr_basket,
        baskets::create_custom_basket,
        // Oracle
        oracle::get_prices,
        oracle::get_price,
        oracle::update_price,
        oracle::register_price_feed,
        // Reserves
        reserves::get_reserves,
        reserves::get_attestation_status,
    ),
    components(
        schemas(
            // Basket models
            CreateSingleCurrencyBasketRequest,
            CreateImfSdrBasketRequest,
            CreateCustomBasketRequest,
            ComponentRequest,
            RebalanceStrategyRequest,
            BasketResponse,
            ComponentResponse,
            BasketValueResponse,
            // Oracle models
            PriceResponse,
            PricesResponse,
            PriceData,
            RegisterFeedRequest,
            // Health models
            HealthResponse,
            // Pagination
            PaginationQuery,
            // Reserve models
            reserves::ReserveData,
            reserves::BondHolding,
            reserves::CurrencyBreakdown,
            reserves::HistoryPoint,
            reserves::AttestationStatus,
            // Error response
            ErrorResponse,
        )
    ),
    security(
        ("bearer_auth" = [])
    ),
    modifiers(&SecurityAddon)
)]
pub struct ApiDoc;

/// Error response schema
#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct ErrorResponse {
    /// Error message
    #[schema(example = "Invalid request parameters")]
    pub error: String,
    /// Error code (if applicable)
    #[schema(example = "VALIDATION_ERROR")]
    pub code: Option<String>,
    /// Additional details
    pub details: Option<serde_json::Value>,
}

/// Security scheme modifier to add Bearer auth
struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer_auth",
                utoipa::openapi::security::SecurityScheme::Http(
                    utoipa::openapi::security::Http::new(
                        utoipa::openapi::security::HttpAuthScheme::Bearer,
                    ),
                ),
            );
        }
    }
}
