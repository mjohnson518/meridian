use meridian_oracle::{mainnet_feeds, ChainlinkOracle};
use rust_decimal::Decimal;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Meridian Oracle Multi-Currency Test ===\n");

    let rpc_url =
        std::env::var("ETHEREUM_RPC_URL").expect("ETHEREUM_RPC_URL must be set");

    let oracle = ChainlinkOracle::new(
        &rpc_url,
        Decimal::new(10, 0), // 10% deviation threshold
    )
    .await?;

    let pairs = vec![
        ("EUR/USD", mainnet_feeds::eur_usd()),
        ("GBP/USD", mainnet_feeds::gbp_usd()),
        ("JPY/USD", mainnet_feeds::jpy_usd()),
        ("CNY/USD", mainnet_feeds::cny_usd()),
    ];

    println!("Fetching prices from Chainlink...\n");

    for (pair, address) in pairs {
        print!("Registering {}... ", pair);
        oracle.register_price_feed(pair, address).await?;
        println!("✓");

        print!("Fetching price... ");
        match oracle.update_price(pair).await {
            Ok(price) => {
                println!("${:.4} ✓", price);

                // Get feed info
                let info = oracle.get_feed_info(pair).await?;
                println!("  Decimals: {}", info.decimals);
                println!(
                    "  Updated: {} seconds ago",
                    chrono::Utc::now()
                        .signed_duration_since(info.updated_at)
                        .num_seconds()
                );
                println!("  Stale: {}", if info.is_stale { "YES ⚠️" } else { "NO ✓" });
                println!();
            }
            Err(e) => {
                println!("ERROR: {}", e);
                eprintln!("Failed to fetch {}: {}", pair, e);
            }
        }
    }

    Ok(())
}

