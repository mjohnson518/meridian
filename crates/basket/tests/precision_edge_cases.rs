#[cfg(test)]
mod precision_tests {
    use rust_decimal::Decimal;
    use std::str::FromStr;

    #[test]
    fn test_maximum_precision_28_digits() {
        // Test that rust_decimal maintains full precision
        let val = Decimal::from_str("1.1234567890123456789012345678").unwrap();
        let text = val.to_string();
        assert_eq!(text, "1.1234567890123456789012345678");
        println!("28-digit precision maintained: {}", text);
    }

    #[test]
    fn test_very_large_basket_value() {
        // Test $1 trillion basket value
        let val = Decimal::from_str("1000000000000.00").unwrap();
        assert!(val > Decimal::ZERO);
        println!("Large value test passed: ${}", val);
    }

    #[test]
    fn test_very_small_weight() {
        // Test 0.01% weight (1 basis point)
        let weight = Decimal::from_str("0.0001").unwrap();
        assert!(weight > Decimal::ZERO);
        assert!(weight < Decimal::ONE);
        println!(
            "Small weight test passed: {}%",
            weight * Decimal::from(100)
        );
    }

    #[test]
    fn test_weight_sum_precision() {
        // Test that 3 weights of 33.33% each sum correctly
        let w1 = Decimal::from_str("33.33").unwrap();
        let w2 = Decimal::from_str("33.33").unwrap();
        let w3 = Decimal::from_str("33.34").unwrap();
        let sum = w1 + w2 + w3;
        assert_eq!(sum, Decimal::from(100));
        println!(
            "Weight sum test passed: {} + {} + {} = {}",
            w1, w2, w3, sum
        );
    }
}

