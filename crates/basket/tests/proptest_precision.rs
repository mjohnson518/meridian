/// Proptest-based precision and conservation tests for the basket engine.
///
/// These tests verify that basket math remains correct under arbitrary inputs:
/// - Weight sums are conserved through basket construction
/// - calculate_value for a single-currency basket equals the price (100% weight)
/// - Two-component baskets with complementary weights always succeed
use meridian_basket::{BasketError, CurrencyBasket, CurrencyComponent, RebalanceStrategy};
use proptest::prelude::*;
use rust_decimal::Decimal;
use std::collections::HashMap;

fn feed() -> String {
    "0xb49f677943BC038e9857d61E7d053CaA2C1734C1".to_string()
}

/// Strategy: integer price in 1..=100_000 (representing cents, i.e. 0.01..=1000.00 USD)
fn arb_price_cents() -> impl Strategy<Value = i64> {
    1i64..=100_000i64
}

proptest! {
    /// Single-currency basket calculate_value must equal the price exactly.
    ///
    /// When weight = 100%, value = (100/100) * price = price.
    #[test]
    fn prop_single_currency_value_equals_price(price_cents in arb_price_cents()) {
        let basket = CurrencyBasket::new_single_currency(
            "EUR Basket".to_string(),
            "EUR".to_string(),
            feed(),
        ).unwrap();

        let price = Decimal::new(price_cents, 2); // e.g. 10800 cents = 108.00
        let mut prices = HashMap::new();
        prices.insert("EUR".to_string(), price);

        let value = basket.calculate_value(&prices).unwrap();
        prop_assert_eq!(value, price, "single-currency value must equal price");
    }

    /// Weight sum of a valid two-component basket is always exactly 100.
    ///
    /// Given eur_pct in 1..=99, gbp_pct = 100 - eur_pct, both are valid and sum to 100.
    #[test]
    fn prop_two_component_weight_sum_is_100(eur_pct in 1u32..=99u32) {
        let gbp_pct = 100 - eur_pct;

        let eur = CurrencyComponent::new(
            "EUR".to_string(),
            Decimal::from(eur_pct),
            Decimal::from(eur_pct.saturating_sub(1).max(1)),
            Decimal::from((eur_pct + 1).min(100)),
            feed(),
        ).unwrap();

        let gbp = CurrencyComponent::new(
            "GBP".to_string(),
            Decimal::from(gbp_pct),
            Decimal::from(gbp_pct.saturating_sub(1).max(1)),
            Decimal::from((gbp_pct + 1).min(100)),
            feed(),
        ).unwrap();

        let basket = CurrencyBasket::new_custom_basket(
            "EUR-GBP".to_string(),
            vec![eur, gbp],
            RebalanceStrategy::None,
        ).unwrap();

        let total: Decimal = basket.components.iter().map(|c| c.target_weight).sum();
        prop_assert_eq!(total, Decimal::new(100, 0), "weights must sum to 100");
    }

    /// A basket with weights that don't sum to 100 must be rejected.
    ///
    /// Given two components with weights that intentionally don't sum to 100,
    /// new_custom_basket must return InvalidWeights.
    #[test]
    fn prop_invalid_weight_sum_rejected(eur_pct in 1u32..=49u32) {
        // eur_pct + eur_pct < 100 always (max is 49+49=98)
        let eur = CurrencyComponent::new(
            "EUR".to_string(),
            Decimal::from(eur_pct),
            Decimal::from(eur_pct.saturating_sub(1).max(1)),
            Decimal::from((eur_pct + 1).min(100)),
            feed(),
        ).unwrap();

        let gbp = CurrencyComponent::new(
            "GBP".to_string(),
            Decimal::from(eur_pct), // same value → sum < 100
            Decimal::from(eur_pct.saturating_sub(1).max(1)),
            Decimal::from((eur_pct + 1).min(100)),
            feed(),
        ).unwrap();

        let result = CurrencyBasket::new_custom_basket(
            "Bad Basket".to_string(),
            vec![eur, gbp],
            RebalanceStrategy::None,
        );

        prop_assert!(
            matches!(result, Err(BasketError::InvalidWeights { .. })),
            "basket with weights != 100 must be rejected"
        );
    }

    /// calculate_value scales linearly with price for single-currency basket.
    ///
    /// double_price = 2 * price → double_value = 2 * value
    #[test]
    fn prop_value_scales_linearly(price_cents in 1i64..=50_000i64) {
        let basket = CurrencyBasket::new_single_currency(
            "GBP Basket".to_string(),
            "GBP".to_string(),
            feed(),
        ).unwrap();

        let price = Decimal::new(price_cents, 2);
        let double_price = price * Decimal::new(2, 0);

        let mut prices = HashMap::new();
        prices.insert("GBP".to_string(), price);
        let value = basket.calculate_value(&prices).unwrap();

        let mut prices2 = HashMap::new();
        prices2.insert("GBP".to_string(), double_price);
        let double_value = basket.calculate_value(&prices2).unwrap();

        prop_assert_eq!(double_value, value * Decimal::new(2, 0), "value must scale linearly with price");
    }
}
