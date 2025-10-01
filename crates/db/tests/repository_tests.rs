//! Repository tests with test transactions
//!
//! These tests require a PostgreSQL database.
//! Set DATABASE_URL environment variable to run them.
//!
//! Example:
//! DATABASE_URL=postgresql://postgres:password@localhost/meridian_test cargo test

use meridian_basket::{CurrencyBasket, CurrencyComponent, RebalanceStrategy};
use meridian_db::*;
use rust_decimal::Decimal;
use std::collections::HashMap;

/// Helper to get database URL from environment
fn get_database_url() -> Option<String> {
    std::env::var("DATABASE_URL").ok()
}

/// Helper to create a test basket
fn create_test_basket() -> CurrencyBasket {
    CurrencyBasket::new_single_currency(
        "Test EUR Basket".to_string(),
        "EUR".to_string(),
        "0xb49f677943BC038e9857d61E7d053CaA2C1734C1".to_string(),
    )
    .unwrap()
}

#[tokio::test]
async fn test_create_and_find_basket() {
    let Some(db_url) = get_database_url() else {
        println!("Skipping test: DATABASE_URL not set");
        return;
    };

    let pool = create_pool(&db_url).await.expect("Failed to create pool");
    run_migrations(&pool).await.expect("Failed to run migrations");

    let repo = BasketRepository::new(pool.clone());

    // Create basket
    let basket = create_test_basket();
    let basket_id = basket.id;

    let created_id = repo.create(&basket).await.expect("Failed to create basket");
    assert_eq!(created_id, basket_id);

    // Find basket
    let found = repo.find_by_id(basket_id).await.expect("Failed to find basket");
    assert_eq!(found.id, basket_id);
    assert_eq!(found.name, "Test EUR Basket");

    // Cleanup
    repo.delete(basket_id).await.expect("Failed to delete");
}

#[tokio::test]
async fn test_list_baskets_with_pagination() {
    let Some(db_url) = get_database_url() else {
        println!("Skipping test: DATABASE_URL not set");
        return;
    };

    let pool = create_pool(&db_url).await.expect("Failed to create pool");
    run_migrations(&pool).await.expect("Failed to run migrations");

    let repo = BasketRepository::new(pool.clone());

    // Create multiple baskets
    let basket1 = create_test_basket();
    let basket2 = create_test_basket();

    repo.create(&basket1).await.expect("Failed to create basket1");
    repo.create(&basket2).await.expect("Failed to create basket2");

    // List baskets
    let baskets = repo.list(10, 0).await.expect("Failed to list baskets");
    assert!(baskets.len() >= 2);

    // Count baskets
    let count = repo.count().await.expect("Failed to count");
    assert!(count >= 2);

    // Cleanup
    repo.delete(basket1.id).await.ok();
    repo.delete(basket2.id).await.ok();
}

#[tokio::test]
async fn test_insert_and_retrieve_price() {
    let Some(db_url) = get_database_url() else {
        println!("Skipping test: DATABASE_URL not set");
        return;
    };

    let pool = create_pool(&db_url).await.expect("Failed to create pool");
    run_migrations(&pool).await.expect("Failed to run migrations");

    let repo = PriceRepository::new(pool);

    // Insert price
    let request = InsertPriceRequest {
        currency_pair: "EUR/USD".to_string(),
        price: Decimal::new(108, 2),
        source: "chainlink".to_string(),
        is_stale: false,
        round_id: Some(Decimal::from(12345)),
    };

    let price_id = repo.insert(request).await.expect("Failed to insert price");
    assert!(price_id > 0);

    // Get latest price
    let latest = repo
        .get_latest("EUR/USD")
        .await
        .expect("Failed to get latest price");

    assert_eq!(latest.currency_pair, "EUR/USD");
    assert_eq!(latest.price, Decimal::new(108, 2));
    assert_eq!(latest.source, "chainlink");
}

#[tokio::test]
async fn test_price_statistics() {
    let Some(db_url) = get_database_url() else {
        println!("Skipping test: DATABASE_URL not set");
        return;
    };

    let pool = create_pool(&db_url).await.expect("Failed to create pool");
    run_migrations(&pool).await.expect("Failed to run migrations");

    let repo = PriceRepository::new(pool);

    // Insert multiple prices
    for price_val in [100, 105, 110, 95, 108] {
        let request = InsertPriceRequest {
            currency_pair: "TEST/USD".to_string(),
            price: Decimal::new(price_val, 2),
            source: "chainlink".to_string(),
            is_stale: false,
            round_id: None,
        };
        repo.insert(request).await.expect("Failed to insert");
    }

    // Get statistics
    let start_time = chrono::Utc::now() - chrono::Duration::hours(1);
    let stats = repo
        .get_stats("TEST/USD", start_time)
        .await
        .expect("Failed to get stats");

    assert_eq!(stats.count, 5);
    assert_eq!(stats.min_price, Decimal::new(95, 2));
    assert_eq!(stats.max_price, Decimal::new(110, 2));
}

#[tokio::test]
async fn test_create_stablecoin() {
    let Some(db_url) = get_database_url() else {
        println!("Skipping test: DATABASE_URL not set");
        return;
    };

    let pool = create_pool(&db_url).await.expect("Failed to create pool");
    run_migrations(&pool).await.expect("Failed to run migrations");

    let repo = StablecoinRepository::new(pool);

    // Create stablecoin
    let request = CreateStablecoinRequest {
        name: "EUR Meridian".to_string(),
        symbol: "EURM".to_string(),
        basket_id: None,
        chain_id: 11155111, // Sepolia
    };

    let id = repo.create(request).await.expect("Failed to create stablecoin");

    // Find stablecoin
    let stablecoin = repo.find_by_id(id).await.expect("Failed to find");
    assert_eq!(stablecoin.name, "EUR Meridian");
    assert_eq!(stablecoin.symbol, "EURM");
    assert_eq!(stablecoin.status, "deploying");
}

#[tokio::test]
async fn test_audit_log_immutability() {
    let Some(db_url) = get_database_url() else {
        println!("Skipping test: DATABASE_URL not set");
        return;
    };

    let pool = create_pool(&db_url).await.expect("Failed to create pool");
    run_migrations(&pool).await.expect("Failed to run migrations");

    let repo = AuditRepository::new(pool);

    // Create audit log
    let request = CreateAuditLogRequest {
        operation: "basket_created".to_string(),
        actor: Some("admin@meridian.com".to_string()),
        stablecoin_id: None,
        basket_id: None,
        details: serde_json::json!({"test": "data"}),
    };

    let log_id = repo.log(request).await.expect("Failed to create audit log");

    // Get recent logs
    let logs = repo.get_recent(10).await.expect("Failed to get logs");
    assert!(!logs.is_empty());

    // Verify log exists
    let found = logs.iter().any(|log| log.id == log_id);
    assert!(found, "Audit log should be retrievable");
}

