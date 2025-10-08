#[cfg(test)]
mod load_tests {
    use sqlx::PgPool;
    use std::env;
    use std::time::Instant;

    #[tokio::test]
    #[ignore] // Mark as ignored to not run in standard test suite
    async fn test_bulk_price_insert_10k() {
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

        let pool = PgPool::connect(&database_url)
            .await
            .expect("Failed to connect to database");

        println!("Starting bulk insert of 10,000 price records...");

        let start = Instant::now();

        for i in 0..10000 {
            let price = format!("1.{:018}", i); // e.g., "1.000000000000000001"

            sqlx::query(
                "INSERT INTO price_history (currency_pair, price, source, timestamp)
                 VALUES ($1, $2, $3, NOW())",
            )
            .bind("LOAD/TEST")
            .bind(&price)
            .bind("load_test")
            .execute(&pool)
            .await
            .unwrap_or_else(|e| panic!("Failed to insert record {}: {}", i, e));

            if i % 1000 == 0 && i > 0 {
                println!("  Inserted {} records...", i);
            }
        }

        let duration = start.elapsed();

        println!("✓ Inserted 10,000 records in {:?}", duration);
        println!(
            "  Average: {:.2} ms per insert",
            duration.as_millis() as f64 / 10000.0
        );
        println!(
            "  Throughput: {:.0} inserts/sec",
            10000.0 / duration.as_secs_f64()
        );

        // Cleanup
        let deleted = sqlx::query("DELETE FROM price_history WHERE currency_pair = 'LOAD/TEST'")
            .execute(&pool)
            .await
            .expect("Failed to cleanup");

        println!("  Cleaned up {} records", deleted.rows_affected());

        // Should complete in reasonable time (target: <10 seconds)
        assert!(
            duration.as_secs() < 30,
            "Bulk insert too slow: {:?}",
            duration
        );

        if duration.as_secs() < 10 {
            println!("✓ Performance: EXCELLENT (<10 seconds)");
        } else if duration.as_secs() < 20 {
            println!("⚠ Performance: ACCEPTABLE (10-20 seconds)");
        } else {
            println!("⚠ Performance: SLOW (20-30 seconds)");
        }
    }
}

