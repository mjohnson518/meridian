//! Known Chainlink price feed addresses on Ethereum Mainnet
//!
//! These addresses are compile-time constants that are validated during initialization.
//! The addresses are cached in static storage after first parse.

use ethers::types::Address;
use std::str::FromStr;
use std::sync::OnceLock;

/// Known Chainlink price feed addresses on Ethereum Mainnet
///
/// All addresses in this module are official Chainlink price feed contracts.
/// They are parsed from hex strings at first access and cached for subsequent calls.
///
/// # Safety
/// These are hardcoded Ethereum addresses that are known to be valid.
/// The OnceLock pattern ensures they are only parsed once.
pub mod mainnet_feeds {
    use super::*;

    // Static storage for parsed addresses - avoids expect() on every call
    static EUR_USD: OnceLock<Address> = OnceLock::new();
    static GBP_USD: OnceLock<Address> = OnceLock::new();
    static JPY_USD: OnceLock<Address> = OnceLock::new();
    static CNY_USD: OnceLock<Address> = OnceLock::new();
    static CHF_USD: OnceLock<Address> = OnceLock::new();
    static BRL_USD: OnceLock<Address> = OnceLock::new();
    static MXN_USD: OnceLock<Address> = OnceLock::new();
    static INR_USD: OnceLock<Address> = OnceLock::new();

    /// Helper to parse address, falling back to zero address if somehow invalid
    /// (should never happen with hardcoded valid addresses)
    fn parse_address(hex: &str) -> Address {
        Address::from_str(hex).unwrap_or_else(|e| {
            tracing::error!(
                address = %hex,
                error = %e,
                "CRITICAL: Hardcoded Chainlink address failed to parse"
            );
            Address::zero() // Fallback - calls to this address will fail gracefully
        })
    }

    /// Returns the Chainlink price feed address for EUR/USD on Ethereum Mainnet
    pub fn eur_usd() -> Address {
        *EUR_USD.get_or_init(|| parse_address("0xb49f677943BC038e9857d61E7d053CaA2C1734C1"))
    }

    /// Returns the Chainlink price feed address for GBP/USD on Ethereum Mainnet
    pub fn gbp_usd() -> Address {
        *GBP_USD.get_or_init(|| parse_address("0x5c0Ab2d9b5a7ed9f470386e82BB36A3613cDd4b5"))
    }

    /// Returns the Chainlink price feed address for JPY/USD on Ethereum Mainnet
    pub fn jpy_usd() -> Address {
        *JPY_USD.get_or_init(|| parse_address("0xBcE206caE7f0ec07b545EddE332A47C2F75bbeb3"))
    }

    /// Returns the Chainlink price feed address for CNY/USD on Ethereum Mainnet
    pub fn cny_usd() -> Address {
        *CNY_USD.get_or_init(|| parse_address("0xeF8A4aF35cd47424672E3C590aBD37FBB7A7759a"))
    }

    /// Returns the Chainlink price feed address for CHF/USD on Ethereum Mainnet
    pub fn chf_usd() -> Address {
        *CHF_USD.get_or_init(|| parse_address("0x449d117117838fFA61263B61dA6301AA2a88B13A"))
    }

    /// Returns the Chainlink price feed address for BRL/USD on Ethereum Mainnet
    pub fn brl_usd() -> Address {
        *BRL_USD.get_or_init(|| parse_address("0x971E8F1B779A5F1C36e1cd7ef44Ba1Cc2F5EeE0f"))
    }

    /// Returns the Chainlink price feed address for MXN/USD on Ethereum Mainnet
    pub fn mxn_usd() -> Address {
        *MXN_USD.get_or_init(|| parse_address("0xe6F5377DE93A361cd5531bDCad24E88C4867bc10"))
    }

    /// Returns the Chainlink price feed address for INR/USD on Ethereum Mainnet
    pub fn inr_usd() -> Address {
        *INR_USD.get_or_init(|| parse_address("0x605D5c2fBCeDb217D7987FC0951B5753069bC360"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mainnet_feed_addresses() {
        // Verify addresses are valid
        let eur_usd = mainnet_feeds::eur_usd();
        assert_eq!(
            format!("{:?}", eur_usd),
            "0xb49f677943bc038e9857d61e7d053caa2c1734c1"
        );

        let gbp_usd = mainnet_feeds::gbp_usd();
        assert_eq!(
            format!("{:?}", gbp_usd),
            "0x5c0ab2d9b5a7ed9f470386e82bb36a3613cdd4b5"
        );

        let jpy_usd = mainnet_feeds::jpy_usd();
        assert_eq!(
            format!("{:?}", jpy_usd),
            "0xbce206cae7f0ec07b545edde332a47c2f75bbeb3"
        );
    }

    #[test]
    fn test_all_feeds_are_non_zero() {
        // Ensure none of our hardcoded addresses resolve to zero
        assert_ne!(mainnet_feeds::eur_usd(), Address::zero());
        assert_ne!(mainnet_feeds::gbp_usd(), Address::zero());
        assert_ne!(mainnet_feeds::jpy_usd(), Address::zero());
        assert_ne!(mainnet_feeds::cny_usd(), Address::zero());
        assert_ne!(mainnet_feeds::chf_usd(), Address::zero());
        assert_ne!(mainnet_feeds::brl_usd(), Address::zero());
        assert_ne!(mainnet_feeds::mxn_usd(), Address::zero());
        assert_ne!(mainnet_feeds::inr_usd(), Address::zero());
    }
}
