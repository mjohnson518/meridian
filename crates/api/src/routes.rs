//! API route configuration

use crate::handlers;
use actix_web::web;

/// Configure all API routes
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg
        // Health check
        .route("/health", web::get().to(handlers::health_check))
        // Authentication endpoints
        .service(
            web::scope("/api/v1/auth")
                .route("/login", web::post().to(handlers::login))
                .route("/register", web::post().to(handlers::register))
                .route("/verify", web::get().to(handlers::verify)),
        )
        // KYC endpoints
        .service(
            web::scope("/api/v1/kyc")
                .route("/submit", web::post().to(handlers::submit_kyc))
                .route("/status/{user_id}", web::get().to(handlers::get_kyc_status))
                .route("/approve/{application_id}", web::put().to(handlers::approve_kyc))
                .route("/reject/{application_id}", web::put().to(handlers::reject_kyc)),
        )
        // Operations endpoints
        .service(
            web::scope("/api/v1/operations")
                .route("/mint", web::post().to(handlers::mint))
                .route("/burn", web::post().to(handlers::burn))
                .route(
                    "/transactions/{user_id}",
                    web::get().to(handlers::get_transactions),
                ),
        )
        // Agent (x402) endpoints
        .service(
            web::scope("/api/v1/agents")
                .route("/create", web::post().to(handlers::create_agent))
                .route("/pay", web::post().to(handlers::agent_pay))
                .route("/list/{user_id}", web::get().to(handlers::list_agents))
                .route(
                    "/transactions/{agent_id}",
                    web::get().to(handlers::get_agent_transactions),
                ),
        )
        // Basket endpoints
        .service(
            web::scope("/api/v1/baskets")
                .route("", web::get().to(handlers::list_baskets))
                .route(
                    "/single-currency",
                    web::post().to(handlers::create_single_currency_basket),
                )
                .route("/imf-sdr", web::post().to(handlers::create_imf_sdr_basket))
                .route("/custom", web::post().to(handlers::create_custom_basket))
                .route("/{id}", web::get().to(handlers::get_basket))
                .route("/{id}/value", web::get().to(handlers::get_basket_value)),
        )
        // Oracle endpoints
        .service(
            web::scope("/api/v1/oracle")
                .route("/prices", web::get().to(handlers::get_prices))
                .route("/prices/{pair}", web::get().to(handlers::get_price))
                .route(
                    "/prices/{pair}/update",
                    web::post().to(handlers::update_price),
                )
                .route("/feeds", web::post().to(handlers::register_price_feed)),
        );
}
