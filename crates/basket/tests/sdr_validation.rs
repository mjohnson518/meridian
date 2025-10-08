#[test]
fn test_imf_sdr_weights_exactly_100_percent() {
    use meridian_basket::CurrencyBasket;
    use rust_decimal::Decimal;
    use std::collections::HashMap;

    let mut feeds = HashMap::new();
    feeds.insert(
        "USD".to_string(),
        "0x0000000000000000000000000000000000000001".to_string(),
    );
    feeds.insert(
        "EUR".to_string(),
        "0x0000000000000000000000000000000000000002".to_string(),
    );
    feeds.insert(
        "CNY".to_string(),
        "0x0000000000000000000000000000000000000003".to_string(),
    );
    feeds.insert(
        "JPY".to_string(),
        "0x0000000000000000000000000000000000000004".to_string(),
    );
    feeds.insert(
        "GBP".to_string(),
        "0x0000000000000000000000000000000000000005".to_string(),
    );

    let basket = CurrencyBasket::new_imf_sdr("Test SDR".to_string(), feeds).unwrap();

    let total_weight: Decimal = basket.components.iter().map(|c| c.target_weight).sum();

    println!("SDR component weights:");
    for component in &basket.components {
        println!(
            "  {}: {}%",
            component.currency_code, component.target_weight
        );
    }
    println!("Total weight: {}%", total_weight);

    assert_eq!(
        total_weight,
        Decimal::from(100),
        "SDR weights must sum to exactly 100%"
    );
}

