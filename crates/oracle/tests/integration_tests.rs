//! Integration tests for Chainlink oracle
//!
//! These tests require an Ethereum RPC endpoint to run.
//! Set the ETHEREUM_RPC_URL environment variable to enable them.
//!
//! Example: ETHEREUM_RPC_URL=https://eth-mainnet.g.alchemy.com/v2/YOUR_KEY cargo test

use meridian_oracle::{mainnet_feeds, ChainlinkOracle, OracleError};
use rust_decimal::Decimal;

/// Helper to get RPC URL from environment
fn get_rpc_url() -> Option<String> {
    std::env::var("ETHEREUM_RPC_URL").ok()
}

#[tokio::test]
async fn test_eur_usd_feed_registration() {
    let Some(rpc_url) = get_rpc_url() else {
        println!("Skipping integration test: ETHEREUM_RPC_URL not set");
        return;
    };

    let oracle = ChainlinkOracle::new(&rpc_url, Decimal::new(10, 0))
        .await
        .expect("Failed to create oracle");

    let eur_usd_address = mainnet_feeds::eur_usd();

    // Register EUR/USD feed
    oracle
        .register_price_feed("EUR/USD", eur_usd_address)
        .await
        .expect("Failed to register EUR/USD feed");

    // Verify feed is registered
    let feeds = oracle.list_feeds().await;
    assert!(feeds.contains(&"EUR/USD".to_string()));

    // Get feed info
    let feed_info = oracle
        .get_feed_info("EUR/USD")
        .await
        .expect("Failed to get feed info");

    assert_eq!(feed_info.pair, "EUR/USD");
    assert_eq!(feed_info.decimals, 8); // EUR/USD uses 8 decimals
    assert!(feed_info.description.contains("EUR"));
}

#[tokio::test]
async fn test_update_and_get_eur_usd_price() {
    let Some(rpc_url) = get_rpc_url() else {
        println!("Skipping integration test: ETHEREUM_RPC_URL not set");
        return;
    };

    let oracle = ChainlinkOracle::new(&rpc_url, Decimal::new(10, 0))
        .await
        .expect("Failed to create oracle");

    let eur_usd_address = mainnet_feeds::eur_usd();

    // Register and update EUR/USD feed
    oracle
        .register_price_feed("EUR/USD", eur_usd_address)
        .await
        .expect("Failed to register EUR/USD feed");

    let price = oracle
        .update_price("EUR/USD")
        .await
        .expect("Failed to update EUR/USD price");

    println!("EUR/USD price: ${}", price);

    // Verify price is reasonable (between 0.50 and 2.00)
    assert!(price > Decimal::new(50, 2)); // > 0.50
    assert!(price < Decimal::new(200, 2)); // < 2.00

    // Get cached price (should not be stale)
    let cached_price = oracle
        .get_price("EUR/USD")
        .await
        .expect("Failed to get cached price");

    assert_eq!(price, cached_price);
}

#[tokio::test]
async fn test_multiple_feeds() {
    let Some(rpc_url) = get_rpc_url() else {
        println!("Skipping integration test: ETHEREUM_RPC_URL not set");
        return;
    };

    let oracle = ChainlinkOracle::new(&rpc_url, Decimal::new(10, 0))
        .await
        .expect("Failed to create oracle");

    // Register EUR/USD
    oracle
        .register_price_feed("EUR/USD", mainnet_feeds::eur_usd())
        .await
        .expect("Failed to register EUR/USD");

    // Register GBP/USD
    oracle
        .register_price_feed("GBP/USD", mainnet_feeds::gbp_usd())
        .await
        .expect("Failed to register GBP/USD");

    // Register JPY/USD
    oracle
        .register_price_feed("JPY/USD", mainnet_feeds::jpy_usd())
        .await
        .expect("Failed to register JPY/USD");

    // Update all feeds
    let eur_price = oracle
        .update_price("EUR/USD")
        .await
        .expect("EUR update failed");
    let gbp_price = oracle
        .update_price("GBP/USD")
        .await
        .expect("GBP update failed");
    let jpy_price = oracle
        .update_price("JPY/USD")
        .await
        .expect("JPY update failed");

    println!("EUR/USD: ${}", eur_price);
    println!("GBP/USD: ${}", gbp_price);
    println!("JPY/USD: ${}", jpy_price);

    // Sanity checks
    assert!(eur_price > Decimal::ZERO);
    assert!(gbp_price > Decimal::ZERO);
    assert!(jpy_price > Decimal::ZERO);

    // GBP should typically be worth more than EUR
    assert!(gbp_price > eur_price);

    // JPY should be much smaller than EUR/GBP
    assert!(jpy_price < eur_price);
}

#[tokio::test]
async fn test_stale_price_detection() {
    let Some(rpc_url) = get_rpc_url() else {
        println!("Skipping integration test: ETHEREUM_RPC_URL not set");
        return;
    };

    let oracle = ChainlinkOracle::new(&rpc_url, Decimal::new(10, 0))
        .await
        .expect("Failed to create oracle");

    let eur_usd_address = mainnet_feeds::eur_usd();

    // Register EUR/USD feed
    oracle
        .register_price_feed("EUR/USD", eur_usd_address)
        .await
        .expect("Failed to register EUR/USD feed");

    // Try to get price before updating (should be stale)
    let result = oracle.get_price("EUR/USD").await;
    assert!(result.is_err());

    match result.unwrap_err() {
        OracleError::StalePrice(pair, age) => {
            assert_eq!(pair, "EUR/USD");
            println!("Correctly detected stale price (age: {}s)", age);
        }
        other => panic!("Expected StalePrice error, got: {:?}", other),
    }
}

#[tokio::test]
async fn test_feed_not_found() {
    let Some(rpc_url) = get_rpc_url() else {
        println!("Skipping integration test: ETHEREUM_RPC_URL not set");
        return;
    };

    let oracle = ChainlinkOracle::new(&rpc_url, Decimal::new(10, 0))
        .await
        .expect("Failed to create oracle");

    // Try to get price for unregistered feed
    let result = oracle.get_price("XYZ/USD").await;
    assert!(result.is_err());

    match result.unwrap_err() {
        OracleError::PriceFeedNotFound(pair) => {
            assert_eq!(pair, "XYZ/USD");
        }
        other => panic!("Expected PriceFeedNotFound error, got: {:?}", other),
    }
}
