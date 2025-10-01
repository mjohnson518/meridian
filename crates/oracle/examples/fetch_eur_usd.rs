//! Example: Fetch EUR/USD price from Chainlink
//!
//! This example demonstrates how to connect to Chainlink and fetch the EUR/USD price.
//!
//! Usage:
//! ```bash
//! ETHEREUM_RPC_URL=https://eth-mainnet.g.alchemy.com/v2/YOUR_KEY \
//!   cargo run --example fetch_eur_usd
//! ```

use meridian_oracle::{mainnet_feeds, ChainlinkOracle};
use rust_decimal::Decimal;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing for logging
    tracing_subscriber::fmt::init();

    // Get RPC URL from environment
    let rpc_url = std::env::var("ETHEREUM_RPC_URL")
        .expect("ETHEREUM_RPC_URL environment variable not set");

    println!("🔗 Connecting to Ethereum mainnet...");

    // Create oracle with 10% deviation threshold
    let oracle = ChainlinkOracle::new(&rpc_url, Decimal::new(10, 0)).await?;

    println!("✅ Connected!");

    // Register EUR/USD price feed
    println!("\n📊 Registering EUR/USD price feed...");
    let eur_usd_address = mainnet_feeds::eur_usd();
    println!("   Address: {:?}", eur_usd_address);

    oracle
        .register_price_feed("EUR/USD", eur_usd_address)
        .await?;

    // Get feed information
    let feed_info = oracle.get_feed_info("EUR/USD").await?;
    println!("   Description: {}", feed_info.description);
    println!("   Decimals: {}", feed_info.decimals);

    // Update price from blockchain
    println!("\n💰 Fetching latest EUR/USD price...");
    let price = oracle.update_price("EUR/USD").await?;

    println!("\n✨ EUR/USD Price: ${}", price);
    println!("   Round: {}", feed_info.latest_round);
    println!("   Updated: {}", feed_info.updated_at);

    // Get cached price (should be fast)
    println!("\n🚀 Getting cached price...");
    let cached_price = oracle.get_price("EUR/USD").await?;
    println!("   Cached: ${}", cached_price);
    assert_eq!(price, cached_price);

    println!("\n✅ Example complete!");

    Ok(())
}

