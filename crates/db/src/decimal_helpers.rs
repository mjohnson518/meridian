//! Decimal conversion helpers for TEXT-based storage
//!
//! TODO: Remove this module when SQLx-Decimal compatibility is resolved
//! and migrate back to native NUMERIC storage

use crate::error::DbError;
use rust_decimal::Decimal;
use std::str::FromStr;

/// Converts a Decimal to TEXT for database storage
///
/// # Arguments
///
/// * `value` - Decimal value to convert
///
/// # Returns
///
/// String representation suitable for PostgreSQL TEXT column
///
/// # Example
///
/// ```
/// use rust_decimal::Decimal;
/// use meridian_db::decimal_to_text;
///
/// let value = Decimal::new(108, 2); // 1.08
/// let text = decimal_to_text(value);
/// assert_eq!(text, "1.08");
/// ```
pub fn decimal_to_text(value: Decimal) -> String {
    value.to_string()
}

/// Converts TEXT from database to Decimal
///
/// # Arguments
///
/// * `text` - String representation of decimal from database
///
/// # Returns
///
/// Decimal value or error if invalid format
///
/// # Errors
///
/// Returns `DbError::SerializationError` if text cannot be parsed as decimal
///
/// # Example
///
/// ```
/// use meridian_db::text_to_decimal;
///
/// let decimal = text_to_decimal("1.08").unwrap();
/// assert_eq!(decimal.to_string(), "1.08");
/// ```
pub fn text_to_decimal(text: &str) -> Result<Decimal, DbError> {
    Decimal::from_str(text)
        .map_err(|e| DbError::SerializationError(format!("Invalid decimal: {}", e)))
}

/// Converts Option<TEXT> to Option<Decimal>
pub fn opt_text_to_decimal(text: Option<&str>) -> Result<Option<Decimal>, DbError> {
    match text {
        Some(t) => Ok(Some(text_to_decimal(t)?)),
        None => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decimal_roundtrip() {
        let original = Decimal::new(108, 2); // 1.08
        let text = decimal_to_text(original);
        let recovered = text_to_decimal(&text).unwrap();
        assert_eq!(original, recovered);
    }

    #[test]
    fn test_high_precision_decimal() {
        // Test with maximum precision: 28 digits
        let original = Decimal::from_str("1.123456789012345678901234567").unwrap();
        let text = decimal_to_text(original);
        let recovered = text_to_decimal(&text).unwrap();
        assert_eq!(original, recovered);
    }

    #[test]
    fn test_zero() {
        let original = Decimal::ZERO;
        let text = decimal_to_text(original);
        assert_eq!(text, "0");
        let recovered = text_to_decimal(&text).unwrap();
        assert_eq!(original, recovered);
    }

    #[test]
    fn test_negative_decimal() {
        let original = Decimal::new(-12345, 2); // -123.45
        let text = decimal_to_text(original);
        let recovered = text_to_decimal(&text).unwrap();
        assert_eq!(original, recovered);
    }

    #[test]
    fn test_invalid_text() {
        let result = text_to_decimal("not_a_number");
        assert!(result.is_err());
    }

    #[test]
    fn test_optional_conversion() {
        let result = opt_text_to_decimal(Some("1.08")).unwrap();
        assert_eq!(result, Some(Decimal::new(108, 2)));

        let result = opt_text_to_decimal(None).unwrap();
        assert_eq!(result, None);
    }
}
