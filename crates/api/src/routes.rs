//! API route configuration

use crate::handlers;
use actix_web::web;

/// Configure all API routes
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg
        // Health check
        .route("/health", web::get().to(handlers::health_check))
        
        // Basket endpoints
        .service(
            web::scope("/api/v1/baskets")
                .route("", web::get().to(handlers::list_baskets))
                .route(
                    "/single-currency",
                    web::post().to(handlers::create_single_currency_basket),
                )
                .route(
                    "/imf-sdr",
                    web::post().to(handlers::create_imf_sdr_basket),
                )
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

