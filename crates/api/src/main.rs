//! # Meridian REST API Server
//!
//! HTTP API for managing multi-currency stablecoins

mod error;
mod handlers;
mod models;
mod routes;
mod state;

use actix_cors::Cors;
use actix_web::{middleware::Logger, web, App, HttpServer};
use meridian_db::{create_pool, run_migrations};
use state::AppState;
use std::sync::Arc;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    tracing::info!("Starting Meridian API server...");

    // Get configuration from environment
    let host = std::env::var("MERIDIAN_API_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = std::env::var("MERIDIAN_API_PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()
        .expect("Invalid port number");

    // Get database URL from environment
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    // Initialize database connection pool
    tracing::info!("Connecting to database...");
    let db_pool = create_pool(&database_url)
        .await
        .expect("Failed to create database pool");

    // Run database migrations
    tracing::info!("Running database migrations...");
    run_migrations(&db_pool)
        .await
        .expect("Failed to run migrations");

    tracing::info!("Database initialized");

    // Initialize shared application state
    let app_state = Arc::new(AppState::new(db_pool).await);

    tracing::info!("Application state initialized");
    tracing::info!("Server starting at http://{}:{}", host, port);

    // Start HTTP server
    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .wrap(Logger::default())
            .wrap(cors)
            .configure(routes::configure)
    })
    .bind((host.as_str(), port))?
    .run()
    .await
}
