//! Known Chainlink price feed addresses on Ethereum Mainnet

use ethers::types::Address;
use std::str::FromStr;

/// Known Chainlink price feed addresses on Ethereum Mainnet
pub mod mainnet_feeds {
    use super::*;

    /// Returns the Chainlink price feed address for EUR/USD on Ethereum Mainnet
    pub fn eur_usd() -> Address {
        Address::from_str("0xb49f677943BC038e9857d61E7d053CaA2C1734C1")
            .expect("Valid EUR/USD address")
    }

    /// Returns the Chainlink price feed address for GBP/USD on Ethereum Mainnet
    pub fn gbp_usd() -> Address {
        Address::from_str("0x5c0Ab2d9b5a7ed9f470386e82BB36A3613cDd4b5")
            .expect("Valid GBP/USD address")
    }

    /// Returns the Chainlink price feed address for JPY/USD on Ethereum Mainnet
    pub fn jpy_usd() -> Address {
        Address::from_str("0xBcE206caE7f0ec07b545EddE332A47C2F75bbeb3")
            .expect("Valid JPY/USD address")
    }

    /// Returns the Chainlink price feed address for CNY/USD on Ethereum Mainnet
    pub fn cny_usd() -> Address {
        Address::from_str("0xeF8A4aF35cd47424672E3C590aBD37FBB7A7759a")
            .expect("Valid CNY/USD address")
    }

    /// Returns the Chainlink price feed address for CHF/USD on Ethereum Mainnet
    pub fn chf_usd() -> Address {
        Address::from_str("0x449d117117838fFA61263B61dA6301AA2a88B13A")
            .expect("Valid CHF/USD address")
    }

    /// Returns the Chainlink price feed address for BRL/USD on Ethereum Mainnet
    pub fn brl_usd() -> Address {
        Address::from_str("0x971E8F1B779A5F1C36e1cd7ef44Ba1Cc2F5EeE0f")
            .expect("Valid BRL/USD address")
    }

    /// Returns the Chainlink price feed address for MXN/USD on Ethereum Mainnet
    pub fn mxn_usd() -> Address {
        Address::from_str("0xe6F5377DE93A361cd5531bDCad24E88C4867bc10")
            .expect("Valid MXN/USD address")
    }

    /// Returns the Chainlink price feed address for INR/USD on Ethereum Mainnet
    pub fn inr_usd() -> Address {
        Address::from_str("0x605D5c2fBCeDb217D7987FC0951B5753069bC360")
            .expect("Valid INR/USD address")
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
}
