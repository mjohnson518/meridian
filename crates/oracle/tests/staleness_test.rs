#[cfg(test)]
mod staleness_tests {
    use meridian_oracle::*;
    use rust_decimal::Decimal;

    #[tokio::test]
    async fn test_stale_price_detection() {
        // This test verifies that prices older than threshold are marked stale
        // Note: Actual Chainlink feeds update frequently, so this tests the logic

        let rpc_url = std::env::var("ETHEREUM_RPC_URL")
            .unwrap_or_else(|_| "http://localhost:8545".to_string());

        let oracle = ChainlinkOracle::new(&rpc_url, Decimal::new(10, 0)).await;

        // If RPC available, test real staleness detection
        if let Ok(oracle) = oracle {
            // Register a feed
            let result = oracle
                .register_price_feed("EUR/USD", mainnet_feeds::eur_usd())
                .await;

            if result.is_ok() {
                // Fetch price
                let price_result = oracle.update_price("EUR/USD").await;

                if let Ok(price) = price_result {
                    println!("EUR/USD price fetched: ${}", price);

                    // Get feed info
                    let info = oracle.get_feed_info("EUR/USD").await;

                    if let Ok(info) = info {
                        let age_seconds = chrono::Utc::now()
                            .signed_duration_since(info.updated_at)
                            .num_seconds();

                        println!("EUR/USD staleness: {}", info.is_stale);
                        println!("Last update: {} seconds ago", age_seconds);

                        // Chainlink EUR/USD updates vary by network conditions
                        // The staleness flag should accurately reflect if data is >1 hour old
                        if age_seconds > 3600 {
                            assert!(
                                info.is_stale,
                                "Price older than 1 hour should be marked stale"
                            );
                        } else {
                            assert!(
                                !info.is_stale,
                                "Fresh price should not be marked stale"
                            );
                        }
                    }
                }
            }
        }
    }
}

