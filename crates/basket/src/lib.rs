//! # Meridian Currency Basket Engine
//!
//! This crate provides the core business logic for managing multi-currency stablecoins
//! and their underlying currency baskets. It supports:
//!
//! - Single-currency stablecoins (e.g., EUR, GBP, JPY)
//! - IMF SDR basket replication
//! - Custom multi-currency baskets with configurable weights
//! - Automatic rebalancing based on deviation thresholds
//!
//! ## Safety
//!
//! All financial calculations use [`rust_decimal::Decimal`] for precise arithmetic
//! without floating-point errors. This is critical for production financial infrastructure.
//!
//! ## Example
//!
//! ```rust
//! use meridian_basket::{CurrencyBasket, BasketType, CurrencyComponent};
//! use rust_decimal::Decimal;
//! use std::collections::HashMap;
//!
//! // Create a single-currency EUR basket
//! let eur_basket = CurrencyBasket::new_single_currency(
//!     "EUR Basket".to_string(),
//!     "EUR".to_string(),
//!     "0xb49f677943BC038e9857d61E7d053CaA2C1734C1".to_string(),
//! ).unwrap();
//!
//! // Calculate value with current price
//! let mut prices = HashMap::new();
//! prices.insert("EUR".to_string(), Decimal::new(108, 2)); // 1.08 EUR/USD
//!
//! let value = eur_basket.calculate_value(&prices).unwrap();
//! ```

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use uuid::Uuid;

/// Errors that can occur during basket operations
#[derive(Error, Debug)]
pub enum BasketError {
    #[error("Currency component not found: {0}")]
    ComponentNotFound(String),

    #[error("Invalid basket weights: sum is {actual}, expected 100.00%")]
    InvalidWeights { actual: Decimal },

    #[error("Price not available for currency: {0}")]
    PriceNotAvailable(String),

    #[error("Rebalancing not applicable for basket type: {0:?}")]
    RebalancingNotApplicable(BasketType),

    #[error("Invalid weight range: min={min}, max={max}, target={target}")]
    InvalidWeightRange {
        min: Decimal,
        max: Decimal,
        target: Decimal,
    },

    #[error("Empty basket: at least one currency component required")]
    EmptyBasket,

    #[error("Invalid currency code: {0}")]
    InvalidCurrencyCode(String),

    #[error("Calculation error: {0}")]
    CalculationError(String),
}

/// Type of currency basket
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BasketType {
    /// Single currency stablecoin (e.g., just EUR or GBP)
    SingleCurrency,
    /// IMF Special Drawing Rights basket (USD, EUR, CNY, JPY, GBP)
    ImfSdr,
    /// Custom multi-currency basket with user-defined weights
    CustomBasket,
}

/// Rebalancing strategy for currency baskets
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RebalanceStrategy {
    /// No automatic rebalancing
    None,
    /// Fixed schedule (e.g., monthly)
    Fixed {
        interval_days: u32,
    },
    /// Rebalance when any component deviates beyond threshold
    ThresholdBased {
        /// Maximum deviation percentage (e.g., 5.0 means 5%)
        max_deviation_percent: Decimal,
    },
    /// Scheduled rebalancing on specific dates
    Scheduled {
        /// Specific timestamps for rebalancing
        schedule: Vec<DateTime<Utc>>,
    },
}

/// Individual currency component within a basket
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CurrencyComponent {
    /// Unique identifier
    pub id: Uuid,
    /// ISO 4217 currency code (e.g., "EUR", "GBP", "JPY")
    pub currency_code: String,
    /// Target weight as a percentage (e.g., 43.38 for 43.38%)
    pub target_weight: Decimal,
    /// Minimum allowed weight before rebalancing triggers
    pub min_weight: Decimal,
    /// Maximum allowed weight before rebalancing triggers
    pub max_weight: Decimal,
    /// Chainlink price feed contract address
    pub chainlink_feed: String,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
}

impl CurrencyComponent {
    /// Creates a new currency component with validation
    ///
    /// # Arguments
    ///
    /// * `currency_code` - ISO 4217 code (must be 3 uppercase letters)
    /// * `target_weight` - Target percentage (0-100)
    /// * `min_weight` - Minimum percentage (0-100, must be < target)
    /// * `max_weight` - Maximum percentage (0-100, must be > target)
    /// * `chainlink_feed` - Ethereum address of Chainlink price feed
    ///
    /// # Errors
    ///
    /// Returns error if weights are invalid or currency code is malformed
    pub fn new(
        currency_code: String,
        target_weight: Decimal,
        min_weight: Decimal,
        max_weight: Decimal,
        chainlink_feed: String,
    ) -> Result<Self, BasketError> {
        // Validate currency code (must be 3 uppercase letters)
        if currency_code.len() != 3 || !currency_code.chars().all(|c| c.is_ascii_uppercase()) {
            return Err(BasketError::InvalidCurrencyCode(currency_code));
        }

        // Validate weight ranges
        if min_weight > target_weight || target_weight > max_weight {
            return Err(BasketError::InvalidWeightRange {
                min: min_weight,
                max: max_weight,
                target: target_weight,
            });
        }

        Ok(Self {
            id: Uuid::new_v4(),
            currency_code,
            target_weight,
            min_weight,
            max_weight,
            chainlink_feed,
            created_at: Utc::now(),
        })
    }

    /// Checks if the current weight is within acceptable bounds
    pub fn is_within_bounds(&self, current_weight: Decimal) -> bool {
        current_weight >= self.min_weight && current_weight <= self.max_weight
    }
}

/// Multi-currency basket configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CurrencyBasket {
    /// Unique identifier
    pub id: Uuid,
    /// Human-readable name
    pub name: String,
    /// Type of basket
    pub basket_type: BasketType,
    /// Currency components in this basket
    pub components: Vec<CurrencyComponent>,
    /// Rebalancing strategy
    pub rebalance_strategy: RebalanceStrategy,
    /// Last rebalance timestamp
    pub last_rebalanced: Option<DateTime<Utc>>,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
}

impl CurrencyBasket {
    /// Creates a new single-currency basket
    ///
    /// This is the simplest basket type, suitable for traditional stablecoins
    /// like EURM (EUR-backed) or GBPM (GBP-backed).
    ///
    /// # Arguments
    ///
    /// * `name` - Descriptive name (e.g., "EUR Basket")
    /// * `currency_code` - ISO 4217 code (e.g., "EUR")
    /// * `chainlink_feed` - Chainlink price feed address
    ///
    /// # Example
    ///
    /// ```rust
    /// use meridian_basket::CurrencyBasket;
    ///
    /// let basket = CurrencyBasket::new_single_currency(
    ///     "EUR Basket".to_string(),
    ///     "EUR".to_string(),
    ///     "0xb49f677943BC038e9857d61E7d053CaA2C1734C1".to_string(),
    /// ).unwrap();
    /// ```
    pub fn new_single_currency(
        name: String,
        currency_code: String,
        chainlink_feed: String,
    ) -> Result<Self, BasketError> {
        let hundred = Decimal::new(100, 0);

        let component = CurrencyComponent::new(
            currency_code,
            hundred, // 100% weight
            hundred, // No rebalancing for single currency
            hundred,
            chainlink_feed,
        )?;

        Ok(Self {
            id: Uuid::new_v4(),
            name,
            basket_type: BasketType::SingleCurrency,
            components: vec![component],
            rebalance_strategy: RebalanceStrategy::None,
            last_rebalanced: None,
            created_at: Utc::now(),
        })
    }

    /// Creates a new IMF SDR basket with standard weights
    ///
    /// The IMF Special Drawing Rights basket consists of:
    /// - USD: 43.38%
    /// - EUR: 29.31%
    /// - CNY: 12.28%
    /// - JPY: 7.59%
    /// - GBP: 7.44%
    ///
    /// # Arguments
    ///
    /// * `name` - Descriptive name (e.g., "IMF SDR Basket")
    /// * `feeds` - Map of currency codes to Chainlink feed addresses
    ///
    /// # Example
    ///
    /// ```rust
    /// use meridian_basket::CurrencyBasket;
    /// use std::collections::HashMap;
    ///
    /// let mut feeds = HashMap::new();
    /// feeds.insert("USD".to_string(), "0x0000000000000000000000000000000000000001".to_string());
    /// feeds.insert("EUR".to_string(), "0xb49f677943BC038e9857d61E7d053CaA2C1734C1".to_string());
    /// feeds.insert("CNY".to_string(), "0xeF8A4aF35cd47424672E3C590aBD37FBB7A7759a".to_string());
    /// feeds.insert("JPY".to_string(), "0xBcE206caE7f0ec07b545EddE332A47C2F75bbeb3".to_string());
    /// feeds.insert("GBP".to_string(), "0x5c0Ab2d9b5a7ed9f470386e82BB36A3613cDd4b5".to_string());
    ///
    /// let basket = CurrencyBasket::new_imf_sdr("IMF SDR".to_string(), feeds).unwrap();
    /// ```
    pub fn new_imf_sdr(
        name: String,
        feeds: HashMap<String, String>,
    ) -> Result<Self, BasketError> {
        // IMF SDR weights as of 2024 (reviewed every 5 years)
        let sdr_weights = [
            ("USD", "43.38", "41.21", "45.55"),
            ("EUR", "29.31", "27.84", "30.78"),
            ("CNY", "12.28", "11.67", "12.89"),
            ("JPY", "7.59", "7.21", "7.97"),
            ("GBP", "7.44", "7.07", "7.81"),
        ];

        let mut components = Vec::new();

        for (code, target, min, max) in sdr_weights {
            let feed = feeds
                .get(code)
                .ok_or_else(|| BasketError::ComponentNotFound(code.to_string()))?;

            let component = CurrencyComponent::new(
                code.to_string(),
                Decimal::from_str_exact(target).map_err(|e| {
                    BasketError::CalculationError(format!("Invalid weight: {}", e))
                })?,
                Decimal::from_str_exact(min).map_err(|e| {
                    BasketError::CalculationError(format!("Invalid weight: {}", e))
                })?,
                Decimal::from_str_exact(max).map_err(|e| {
                    BasketError::CalculationError(format!("Invalid weight: {}", e))
                })?,
                feed.clone(),
            )?;

            components.push(component);
        }

        Ok(Self {
            id: Uuid::new_v4(),
            name,
            basket_type: BasketType::ImfSdr,
            components,
            rebalance_strategy: RebalanceStrategy::ThresholdBased {
                max_deviation_percent: Decimal::new(5, 0), // 5% deviation
            },
            last_rebalanced: None,
            created_at: Utc::now(),
        })
    }

    /// Creates a custom multi-currency basket
    ///
    /// Allows full customization of currency weights and rebalancing strategy.
    ///
    /// # Arguments
    ///
    /// * `name` - Descriptive name
    /// * `components` - Vector of currency components
    /// * `rebalance_strategy` - Rebalancing approach
    ///
    /// # Errors
    ///
    /// Returns error if weights don't sum to 100%
    ///
    /// # Example
    ///
    /// ```rust
    /// use meridian_basket::{CurrencyBasket, CurrencyComponent, RebalanceStrategy};
    /// use rust_decimal::Decimal;
    ///
    /// let eur = CurrencyComponent::new(
    ///     "EUR".to_string(),
    ///     Decimal::new(60, 0), // 60%
    ///     Decimal::new(55, 0),
    ///     Decimal::new(65, 0),
    ///     "0xb49f677943BC038e9857d61E7d053CaA2C1734C1".to_string(),
    /// ).unwrap();
    ///
    /// let gbp = CurrencyComponent::new(
    ///     "GBP".to_string(),
    ///     Decimal::new(40, 0), // 40%
    ///     Decimal::new(35, 0),
    ///     Decimal::new(45, 0),
    ///     "0x5c0Ab2d9b5a7ed9f470386e82BB36A3613cDd4b5".to_string(),
    /// ).unwrap();
    ///
    /// let basket = CurrencyBasket::new_custom_basket(
    ///     "EUR-GBP Basket".to_string(),
    ///     vec![eur, gbp],
    ///     RebalanceStrategy::ThresholdBased {
    ///         max_deviation_percent: Decimal::new(3, 0),
    ///     },
    /// ).unwrap();
    /// ```
    pub fn new_custom_basket(
        name: String,
        components: Vec<CurrencyComponent>,
        rebalance_strategy: RebalanceStrategy,
    ) -> Result<Self, BasketError> {
        if components.is_empty() {
            return Err(BasketError::EmptyBasket);
        }

        // Validate that weights sum to 100%
        let total_weight: Decimal = components.iter().map(|c| c.target_weight).sum();
        let hundred = Decimal::new(100, 0);

        if (total_weight - hundred).abs() > Decimal::new(1, 2) {
            // Allow 0.01% tolerance
            return Err(BasketError::InvalidWeights {
                actual: total_weight,
            });
        }

        Ok(Self {
            id: Uuid::new_v4(),
            name,
            basket_type: BasketType::CustomBasket,
            components,
            rebalance_strategy,
            last_rebalanced: None,
            created_at: Utc::now(),
        })
    }

    /// Calculates the current value of the basket in USD
    ///
    /// This method takes a map of currency prices (in USD) and computes
    /// the weighted average value of the basket.
    ///
    /// # Arguments
    ///
    /// * `prices` - Map of currency codes to their USD prices
    ///
    /// # Returns
    ///
    /// The basket value in USD as a Decimal
    ///
    /// # Errors
    ///
    /// Returns error if any required price is missing
    ///
    /// # Example
    ///
    /// ```rust
    /// use meridian_basket::CurrencyBasket;
    /// use rust_decimal::Decimal;
    /// use std::collections::HashMap;
    ///
    /// let basket = CurrencyBasket::new_single_currency(
    ///     "EUR Basket".to_string(),
    ///     "EUR".to_string(),
    ///     "0xb49f677943BC038e9857d61E7d053CaA2C1734C1".to_string(),
    /// ).unwrap();
    ///
    /// let mut prices = HashMap::new();
    /// prices.insert("EUR".to_string(), Decimal::new(108, 2)); // 1.08
    ///
    /// let value = basket.calculate_value(&prices).unwrap();
    /// assert_eq!(value, Decimal::new(108, 2));
    /// ```
    pub fn calculate_value(&self, prices: &HashMap<String, Decimal>) -> Result<Decimal, BasketError> {
        let mut total_value = Decimal::ZERO;
        let hundred = Decimal::new(100, 0);

        for component in &self.components {
            let price = prices
                .get(&component.currency_code)
                .ok_or_else(|| BasketError::PriceNotAvailable(component.currency_code.clone()))?;

            // Value = (weight / 100) * price
            let component_value = (component.target_weight / hundred)
                .checked_mul(*price)
                .ok_or_else(|| {
                    BasketError::CalculationError("Overflow in value calculation".to_string())
                })?;

            total_value = total_value.checked_add(component_value).ok_or_else(|| {
                BasketError::CalculationError("Overflow in total value".to_string())
            })?;
        }

        Ok(total_value)
    }

    /// Determines if the basket needs rebalancing
    ///
    /// Checks current weights against target weights based on the
    /// configured rebalancing strategy.
    ///
    /// # Arguments
    ///
    /// * `prices` - Current market prices in USD
    ///
    /// # Returns
    ///
    /// `true` if rebalancing is needed, `false` otherwise
    ///
    /// # Example
    ///
    /// ```rust
    /// use meridian_basket::{CurrencyBasket, CurrencyComponent, RebalanceStrategy};
    /// use rust_decimal::Decimal;
    /// use std::collections::HashMap;
    ///
    /// let eur = CurrencyComponent::new(
    ///     "EUR".to_string(),
    ///     Decimal::new(50, 0),
    ///     Decimal::new(45, 0),
    ///     Decimal::new(55, 0),
    ///     "0xb49f677943BC038e9857d61E7d053CaA2C1734C1".to_string(),
    /// ).unwrap();
    ///
    /// let usd = CurrencyComponent::new(
    ///     "USD".to_string(),
    ///     Decimal::new(50, 0),
    ///     Decimal::new(45, 0),
    ///     Decimal::new(55, 0),
    ///     "0x0000000000000000000000000000000000000001".to_string(),
    /// ).unwrap();
    ///
    /// let basket = CurrencyBasket::new_custom_basket(
    ///     "EUR-USD".to_string(),
    ///     vec![eur, usd],
    ///     RebalanceStrategy::ThresholdBased {
    ///         max_deviation_percent: Decimal::new(3, 0),
    ///     },
    /// ).unwrap();
    ///
    /// let mut prices = HashMap::new();
    /// prices.insert("EUR".to_string(), Decimal::new(108, 2));
    /// prices.insert("USD".to_string(), Decimal::ONE);
    ///
    /// let needs_rebalance = basket.needs_rebalancing(&prices).unwrap();
    /// ```
    pub fn needs_rebalancing(&self, prices: &HashMap<String, Decimal>) -> Result<bool, BasketError> {
        match &self.rebalance_strategy {
            RebalanceStrategy::None => Ok(false),

            RebalanceStrategy::Fixed { interval_days } => {
                if let Some(last_rebalanced) = self.last_rebalanced {
                    let elapsed = Utc::now()
                        .signed_duration_since(last_rebalanced)
                        .num_days();
                    Ok(elapsed >= *interval_days as i64)
                } else {
                    // Never rebalanced, so rebalance now
                    Ok(true)
                }
            }

            RebalanceStrategy::ThresholdBased {
                max_deviation_percent,
            } => {
                // Calculate current weights based on market prices
                let current_weights = self.calculate_current_weights(prices)?;

                // Check if any component is outside its bounds
                for component in &self.components {
                    let current_weight = current_weights
                        .get(&component.currency_code)
                        .ok_or_else(|| {
                            BasketError::ComponentNotFound(component.currency_code.clone())
                        })?;

                    if !component.is_within_bounds(*current_weight) {
                        tracing::info!(
                            currency = %component.currency_code,
                            target = %component.target_weight,
                            current = %current_weight,
                            "Component outside bounds, rebalancing needed"
                        );
                        return Ok(true);
                    }

                    // Also check absolute deviation from target
                    let deviation = (*current_weight - component.target_weight).abs();
                    if deviation > *max_deviation_percent {
                        tracing::info!(
                            currency = %component.currency_code,
                            deviation = %deviation,
                            threshold = %max_deviation_percent,
                            "Deviation threshold exceeded"
                        );
                        return Ok(true);
                    }
                }

                Ok(false)
            }

            RebalanceStrategy::Scheduled { schedule } => {
                let now = Utc::now();
                Ok(schedule.iter().any(|scheduled_time| {
                    now >= *scheduled_time
                        && self
                            .last_rebalanced
                            .is_none_or(|last| last < *scheduled_time)
                }))
            }
        }
    }

    /// Calculates current weights based on market prices
    ///
    /// This is used internally to determine if rebalancing is needed.
    ///
    /// # Arguments
    ///
    /// * `prices` - Current market prices in USD
    ///
    /// # Returns
    ///
    /// Map of currency codes to their current weights as percentages
    fn calculate_current_weights(
        &self,
        prices: &HashMap<String, Decimal>,
    ) -> Result<HashMap<String, Decimal>, BasketError> {
        let total_value = self.calculate_value(prices)?;
        let mut current_weights = HashMap::new();
        let hundred = Decimal::new(100, 0);

        for component in &self.components {
            let price = prices
                .get(&component.currency_code)
                .ok_or_else(|| BasketError::PriceNotAvailable(component.currency_code.clone()))?;

            let component_value = (component.target_weight / hundred)
                .checked_mul(*price)
                .ok_or_else(|| {
                    BasketError::CalculationError("Overflow in weight calculation".to_string())
                })?;

            let current_weight = (component_value / total_value)
                .checked_mul(hundred)
                .ok_or_else(|| {
                    BasketError::CalculationError("Overflow in weight percentage".to_string())
                })?;

            current_weights.insert(component.currency_code.clone(), current_weight);
        }

        Ok(current_weights)
    }

    /// Marks the basket as rebalanced at current timestamp
    ///
    /// This should be called after a rebalancing operation completes.
    pub fn mark_rebalanced(&mut self) {
        self.last_rebalanced = Some(Utc::now());
    }

    /// Gets a component by currency code
    pub fn get_component(&self, currency_code: &str) -> Option<&CurrencyComponent> {
        self.components
            .iter()
            .find(|c| c.currency_code == currency_code)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper function to create a standard price map for testing
    fn create_test_prices() -> HashMap<String, Decimal> {
        let mut prices = HashMap::new();
        prices.insert("USD".to_string(), Decimal::ONE);
        prices.insert("EUR".to_string(), Decimal::new(108, 2)); // 1.08
        prices.insert("GBP".to_string(), Decimal::new(127, 2)); // 1.27
        prices.insert("JPY".to_string(), Decimal::new(67, 4)); // 0.0067
        prices.insert("CNY".to_string(), Decimal::new(14, 2)); // 0.14
        prices.insert("BRL".to_string(), Decimal::new(20, 2)); // 0.20
        prices
    }

    #[test]
    fn test_single_currency_basket_creation() {
        let basket = CurrencyBasket::new_single_currency(
            "EUR Basket".to_string(),
            "EUR".to_string(),
            "0xb49f677943BC038e9857d61E7d053CaA2C1734C1".to_string(),
        )
        .unwrap();

        assert_eq!(basket.basket_type, BasketType::SingleCurrency);
        assert_eq!(basket.components.len(), 1);
        assert_eq!(basket.components[0].currency_code, "EUR");
        assert_eq!(basket.components[0].target_weight, Decimal::new(100, 0));
        assert_eq!(basket.rebalance_strategy, RebalanceStrategy::None);
    }

    #[test]
    fn test_single_currency_basket_valuation() {
        let basket = CurrencyBasket::new_single_currency(
            "EUR Basket".to_string(),
            "EUR".to_string(),
            "0xb49f677943BC038e9857d61E7d053CaA2C1734C1".to_string(),
        )
        .unwrap();

        let prices = create_test_prices();
        let value = basket.calculate_value(&prices).unwrap();

        // EUR is 100% weight at 1.08 USD = 1.08 USD total
        assert_eq!(value, Decimal::new(108, 2));
    }

    #[test]
    fn test_imf_sdr_basket_creation() {
        let mut feeds = HashMap::new();
        feeds.insert(
            "USD".to_string(),
            "0x0000000000000000000000000000000000000001".to_string(),
        );
        feeds.insert(
            "EUR".to_string(),
            "0xb49f677943BC038e9857d61E7d053CaA2C1734C1".to_string(),
        );
        feeds.insert(
            "CNY".to_string(),
            "0xeF8A4aF35cd47424672E3C590aBD37FBB7A7759a".to_string(),
        );
        feeds.insert(
            "JPY".to_string(),
            "0xBcE206caE7f0ec07b545EddE332A47C2F75bbeb3".to_string(),
        );
        feeds.insert(
            "GBP".to_string(),
            "0x5c0Ab2d9b5a7ed9f470386e82BB36A3613cDd4b5".to_string(),
        );

        let basket = CurrencyBasket::new_imf_sdr("IMF SDR Basket".to_string(), feeds).unwrap();

        assert_eq!(basket.basket_type, BasketType::ImfSdr);
        assert_eq!(basket.components.len(), 5);

        // Verify SDR weights
        let usd = basket.get_component("USD").unwrap();
        assert_eq!(usd.target_weight, Decimal::from_str_exact("43.38").unwrap());

        let eur = basket.get_component("EUR").unwrap();
        assert_eq!(eur.target_weight, Decimal::from_str_exact("29.31").unwrap());

        let cny = basket.get_component("CNY").unwrap();
        assert_eq!(cny.target_weight, Decimal::from_str_exact("12.28").unwrap());

        let jpy = basket.get_component("JPY").unwrap();
        assert_eq!(jpy.target_weight, Decimal::from_str_exact("7.59").unwrap());

        let gbp = basket.get_component("GBP").unwrap();
        assert_eq!(gbp.target_weight, Decimal::from_str_exact("7.44").unwrap());

        // Verify total weight is 100%
        let total: Decimal = basket.components.iter().map(|c| c.target_weight).sum();
        assert_eq!(total, Decimal::new(100, 0));
    }

    #[test]
    fn test_imf_sdr_basket_valuation() {
        let mut feeds = HashMap::new();
        feeds.insert(
            "USD".to_string(),
            "0x0000000000000000000000000000000000000001".to_string(),
        );
        feeds.insert(
            "EUR".to_string(),
            "0xb49f677943BC038e9857d61E7d053CaA2C1734C1".to_string(),
        );
        feeds.insert(
            "CNY".to_string(),
            "0xeF8A4aF35cd47424672E3C590aBD37FBB7A7759a".to_string(),
        );
        feeds.insert(
            "JPY".to_string(),
            "0xBcE206caE7f0ec07b545EddE332A47C2F75bbeb3".to_string(),
        );
        feeds.insert(
            "GBP".to_string(),
            "0x5c0Ab2d9b5a7ed9f470386e82BB36A3613cDd4b5".to_string(),
        );

        let basket = CurrencyBasket::new_imf_sdr("IMF SDR".to_string(), feeds).unwrap();
        let prices = create_test_prices();
        let value = basket.calculate_value(&prices).unwrap();

        // Expected calculation:
        // USD: 43.38% * 1.00 = 0.4338
        // EUR: 29.31% * 1.08 = 0.316548
        // CNY: 12.28% * 0.14 = 0.017192
        // JPY: 7.59% * 0.0067 = 0.00050853
        // GBP: 7.44% * 1.27 = 0.094488
        // Total ≈ 0.862536

        let expected = Decimal::from_str_exact("0.862536").unwrap();
        let tolerance = Decimal::new(1, 6); // 0.000001 tolerance

        assert!(
            (value - expected).abs() < tolerance,
            "Expected {}, got {}",
            expected,
            value
        );
    }

    #[test]
    fn test_custom_basket_creation() {
        let eur = CurrencyComponent::new(
            "EUR".to_string(),
            Decimal::new(60, 0),
            Decimal::new(55, 0),
            Decimal::new(65, 0),
            "0xb49f677943BC038e9857d61E7d053CaA2C1734C1".to_string(),
        )
        .unwrap();

        let gbp = CurrencyComponent::new(
            "GBP".to_string(),
            Decimal::new(40, 0),
            Decimal::new(35, 0),
            Decimal::new(45, 0),
            "0x5c0Ab2d9b5a7ed9f470386e82BB36A3613cDd4b5".to_string(),
        )
        .unwrap();

        let basket = CurrencyBasket::new_custom_basket(
            "EUR-GBP Basket".to_string(),
            vec![eur, gbp],
            RebalanceStrategy::ThresholdBased {
                max_deviation_percent: Decimal::new(3, 0),
            },
        )
        .unwrap();

        assert_eq!(basket.basket_type, BasketType::CustomBasket);
        assert_eq!(basket.components.len(), 2);

        let total: Decimal = basket.components.iter().map(|c| c.target_weight).sum();
        assert_eq!(total, Decimal::new(100, 0));
    }

    #[test]
    fn test_custom_basket_invalid_weights() {
        let eur = CurrencyComponent::new(
            "EUR".to_string(),
            Decimal::new(60, 0), // 60%
            Decimal::new(55, 0),
            Decimal::new(65, 0),
            "0xb49f677943BC038e9857d61E7d053CaA2C1734C1".to_string(),
        )
        .unwrap();

        let gbp = CurrencyComponent::new(
            "GBP".to_string(),
            Decimal::new(50, 0), // 50% - total is 110%!
            Decimal::new(45, 0),
            Decimal::new(55, 0),
            "0x5c0Ab2d9b5a7ed9f470386e82BB36A3613cDd4b5".to_string(),
        )
        .unwrap();

        let result = CurrencyBasket::new_custom_basket(
            "Invalid Basket".to_string(),
            vec![eur, gbp],
            RebalanceStrategy::None,
        );

        assert!(result.is_err());
        match result.unwrap_err() {
            BasketError::InvalidWeights { actual } => {
                assert_eq!(actual, Decimal::new(110, 0));
            }
            _ => panic!("Expected InvalidWeights error"),
        }
    }

    #[test]
    fn test_custom_basket_valuation() {
        let eur = CurrencyComponent::new(
            "EUR".to_string(),
            Decimal::new(70, 0),
            Decimal::new(65, 0),
            Decimal::new(75, 0),
            "0xb49f677943BC038e9857d61E7d053CaA2C1734C1".to_string(),
        )
        .unwrap();

        let brl = CurrencyComponent::new(
            "BRL".to_string(),
            Decimal::new(30, 0),
            Decimal::new(25, 0),
            Decimal::new(35, 0),
            "0x971E8F1B779A5F1C36e1cd7ef44Ba1Cc2F5EeE0f".to_string(),
        )
        .unwrap();

        let basket = CurrencyBasket::new_custom_basket(
            "EUR-BRL Basket".to_string(),
            vec![eur, brl],
            RebalanceStrategy::None,
        )
        .unwrap();

        let prices = create_test_prices();
        let value = basket.calculate_value(&prices).unwrap();

        // Expected: 70% * 1.08 + 30% * 0.20 = 0.756 + 0.06 = 0.816
        let expected = Decimal::new(816, 3);
        assert_eq!(value, expected);
    }

    #[test]
    fn test_rebalancing_threshold_within_bounds() {
        let eur = CurrencyComponent::new(
            "EUR".to_string(),
            Decimal::new(50, 0),
            Decimal::new(45, 0),
            Decimal::new(55, 0),
            "0xb49f677943BC038e9857d61E7d053CaA2C1734C1".to_string(),
        )
        .unwrap();

        let usd = CurrencyComponent::new(
            "USD".to_string(),
            Decimal::new(50, 0),
            Decimal::new(45, 0),
            Decimal::new(55, 0),
            "0x0000000000000000000000000000000000000001".to_string(),
        )
        .unwrap();

        let basket = CurrencyBasket::new_custom_basket(
            "EUR-USD".to_string(),
            vec![eur, usd],
            RebalanceStrategy::ThresholdBased {
                max_deviation_percent: Decimal::new(3, 0),
            },
        )
        .unwrap();

        // Prices where weights stay balanced
        let mut prices = HashMap::new();
        prices.insert("EUR".to_string(), Decimal::ONE);
        prices.insert("USD".to_string(), Decimal::ONE);

        let needs_rebalance = basket.needs_rebalancing(&prices).unwrap();
        assert!(!needs_rebalance, "Should not need rebalancing when balanced");
    }

    #[test]
    fn test_rebalancing_threshold_exceeded() {
        let eur = CurrencyComponent::new(
            "EUR".to_string(),
            Decimal::new(50, 0),
            Decimal::new(45, 0),
            Decimal::new(55, 0),
            "0xb49f677943BC038e9857d61E7d053CaA2C1734C1".to_string(),
        )
        .unwrap();

        let usd = CurrencyComponent::new(
            "USD".to_string(),
            Decimal::new(50, 0),
            Decimal::new(45, 0),
            Decimal::new(55, 0),
            "0x0000000000000000000000000000000000000001".to_string(),
        )
        .unwrap();

        let basket = CurrencyBasket::new_custom_basket(
            "EUR-USD".to_string(),
            vec![eur, usd],
            RebalanceStrategy::ThresholdBased {
                max_deviation_percent: Decimal::new(3, 0), // 3% threshold
            },
        )
        .unwrap();

        // Simulate EUR appreciating significantly
        let mut prices = HashMap::new();
        prices.insert("EUR".to_string(), Decimal::new(15, 1)); // 1.5
        prices.insert("USD".to_string(), Decimal::ONE);

        let needs_rebalance = basket.needs_rebalancing(&prices).unwrap();
        assert!(
            needs_rebalance,
            "Should need rebalancing when EUR appreciates significantly"
        );
    }

    #[test]
    fn test_invalid_currency_code() {
        let result = CurrencyComponent::new(
            "EURO".to_string(), // Invalid: too long
            Decimal::new(100, 0),
            Decimal::new(95, 0),
            Decimal::new(105, 0),
            "0xb49f677943BC038e9857d61E7d053CaA2C1734C1".to_string(),
        );

        assert!(result.is_err());
        match result.unwrap_err() {
            BasketError::InvalidCurrencyCode(code) => {
                assert_eq!(code, "EURO");
            }
            _ => panic!("Expected InvalidCurrencyCode error"),
        }
    }

    #[test]
    fn test_invalid_weight_range() {
        let result = CurrencyComponent::new(
            "EUR".to_string(),
            Decimal::new(50, 0),  // target
            Decimal::new(60, 0),  // min > target (invalid!)
            Decimal::new(70, 0),  // max
            "0xb49f677943BC038e9857d61E7d053CaA2C1734C1".to_string(),
        );

        assert!(result.is_err());
        match result.unwrap_err() {
            BasketError::InvalidWeightRange { min, max, target } => {
                assert_eq!(min, Decimal::new(60, 0));
                assert_eq!(max, Decimal::new(70, 0));
                assert_eq!(target, Decimal::new(50, 0));
            }
            _ => panic!("Expected InvalidWeightRange error"),
        }
    }

    #[test]
    fn test_empty_basket() {
        let result = CurrencyBasket::new_custom_basket(
            "Empty".to_string(),
            vec![],
            RebalanceStrategy::None,
        );

        assert!(result.is_err());
        match result.unwrap_err() {
            BasketError::EmptyBasket => {}
            _ => panic!("Expected EmptyBasket error"),
        }
    }

    #[test]
    fn test_missing_price() {
        let basket = CurrencyBasket::new_single_currency(
            "EUR Basket".to_string(),
            "EUR".to_string(),
            "0xb49f677943BC038e9857d61E7d053CaA2C1734C1".to_string(),
        )
        .unwrap();

        let prices = HashMap::new(); // No prices!

        let result = basket.calculate_value(&prices);
        assert!(result.is_err());
        match result.unwrap_err() {
            BasketError::PriceNotAvailable(currency) => {
                assert_eq!(currency, "EUR");
            }
            _ => panic!("Expected PriceNotAvailable error"),
        }
    }

    #[test]
    fn test_fixed_interval_rebalancing() {
        let basket = CurrencyBasket::new_single_currency(
            "EUR Basket".to_string(),
            "EUR".to_string(),
            "0xb49f677943BC038e9857d61E7d053CaA2C1734C1".to_string(),
        )
        .unwrap();

        // Override rebalance strategy
        let mut basket_with_fixed = basket.clone();
        basket_with_fixed.rebalance_strategy = RebalanceStrategy::Fixed { interval_days: 30 };

        let prices = create_test_prices();

        // Should need rebalancing when never rebalanced
        let needs_rebalance = basket_with_fixed.needs_rebalancing(&prices).unwrap();
        assert!(needs_rebalance, "Should need rebalancing when never rebalanced");

        // Mark as rebalanced
        basket_with_fixed.mark_rebalanced();

        // Should NOT need rebalancing immediately after
        let needs_rebalance = basket_with_fixed.needs_rebalancing(&prices).unwrap();
        assert!(
            !needs_rebalance,
            "Should not need rebalancing immediately after rebalancing"
        );
    }

    #[test]
    fn test_no_rebalancing_strategy() {
        let basket = CurrencyBasket::new_single_currency(
            "EUR Basket".to_string(),
            "EUR".to_string(),
            "0xb49f677943BC038e9857d61E7d053CaA2C1734C1".to_string(),
        )
        .unwrap();

        let prices = create_test_prices();
        let needs_rebalance = basket.needs_rebalancing(&prices).unwrap();

        assert!(
            !needs_rebalance,
            "Single currency basket with None strategy should never need rebalancing"
        );
    }

    #[test]
    fn test_decimal_precision_no_floating_point() {
        // This test verifies we're using Decimal throughout, not f64
        let eur = CurrencyComponent::new(
            "EUR".to_string(),
            Decimal::new(333333, 5), // 3.33333% (repeating decimal that would break f64)
            Decimal::new(30, 1),
            Decimal::new(40, 1),
            "0xb49f677943BC038e9857d61E7d053CaA2C1734C1".to_string(),
        )
        .unwrap();

        let usd = CurrencyComponent::new(
            "USD".to_string(),
            Decimal::new(666667, 5), // 6.66667%
            Decimal::new(60, 1),
            Decimal::new(70, 1),
            "0x0000000000000000000000000000000000000001".to_string(),
        )
        .unwrap();

        let gbp = CurrencyComponent::new(
            "GBP".to_string(),
            Decimal::new(90, 0), // 90%
            Decimal::new(85, 0),
            Decimal::new(95, 0),
            "0x5c0Ab2d9b5a7ed9f470386e82BB36A3613cDd4b5".to_string(),
        )
        .unwrap();

        // Total should be exactly 100%
        let basket = CurrencyBasket::new_custom_basket(
            "Precision Test".to_string(),
            vec![eur, usd, gbp],
            RebalanceStrategy::None,
        )
        .unwrap();

        let prices = create_test_prices();
        let value = basket.calculate_value(&prices).unwrap();

        // Value should be deterministic and precise
        assert!(value > Decimal::ZERO);
    }
}

