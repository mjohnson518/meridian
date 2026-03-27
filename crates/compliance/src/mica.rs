//! # MiCA Reserve Composition Rules
//!
//! EU Markets in Crypto-Assets Regulation (MiCA) reserve requirements
//! for Asset-Referenced Tokens (ARTs) and E-Money Tokens (EMTs).
//!
//! ## Key Articles
//!
//! **Art. 36 — Reserve of assets (ART)**
//! - At least 30% of reserves must be liquid: cash, deposits at credit institutions,
//!   central bank reserves, or money market funds qualifying under UCITS Directive
//! - Maximum 70% may be government bonds or other high-quality debt
//!
//! **Art. 37 — Custody of reserve assets**
//! - Assets must be held with credit institutions or qualified crypto-asset custodians
//! - Reserve assets must be legally segregated from issuer's own assets
//!
//! **Art. 38 — Investment of reserve assets**
//! - Maximum 10% concentration per single issuer (except EEA sovereign/central bank debt)
//! - No exposure to crypto-assets other than the issued ART
//!
//! **Art. 45 — LEI code**
//! - Issuers must obtain and maintain a Legal Entity Identifier (LEI)
//!
//! ## Usage
//!
//! ```rust
//! use meridian_compliance::mica::{ReserveComposition, validate_reserve_composition};
//! use rust_decimal_macros::dec;
//!
//! let composition = ReserveComposition {
//!     liquid_pct: dec!(35), // 35% cash/deposits
//!     sovereign_bond_pct: dec!(55), // 55% EEA sovereign bonds
//!     corporate_bond_pct: dec!(10), // 10% corporate
//!     other_pct: dec!(0),
//!     issuers: vec![],
//! };
//! assert!(validate_reserve_composition(&composition).is_ok());
//! ```

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// A MiCA compliance violation
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum MicaViolation {
    #[error("Art. 36: Liquid asset ratio {actual:.1}% is below the 30% minimum (current: {actual:.1}%)")]
    InsufficientLiquidAssets { actual: Decimal },

    #[error("Art. 36: Bond allocation {actual:.1}% exceeds the 70% maximum")]
    ExcessBondAllocation { actual: Decimal },

    #[error("Art. 38: Issuer '{issuer}' has {pct:.1}% concentration, exceeding the 10% limit")]
    IssuerConcentration { issuer: String, pct: Decimal },

    #[error("Art. 36: Reserve composition percentages sum to {sum:.1}%, expected 100%")]
    InvalidCompositionTotal { sum: Decimal },

    #[error("Art. 38: Exposure to non-EEA crypto-assets is not permitted in reserve")]
    CryptoAssetExposure,

    #[error("Art. 45: LEI code is required for MiCA-regulated issuers")]
    MissingLei,
}

/// Issuer concentration within the reserve portfolio
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssuerConcentration {
    /// Issuer name or ISIN prefix (e.g., "DE" for German federal government)
    pub issuer_id: String,
    /// Percentage of total reserve value held from this issuer
    pub pct: Decimal,
    /// Whether this is an EEA sovereign/central bank issuer (exempt from 10% limit)
    pub is_eca_sovereign: bool,
}

/// Full reserve composition breakdown for MiCA compliance check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReserveComposition {
    /// Cash, central bank reserves, qualifying MMFs, credit institution deposits
    pub liquid_pct: Decimal,
    /// EEA sovereign/central bank bonds (e.g., German Bunds, French OATs)
    pub sovereign_bond_pct: Decimal,
    /// Non-sovereign government bonds and corporate debt
    pub corporate_bond_pct: Decimal,
    /// Other asset classes (should be zero for compliant reserves)
    pub other_pct: Decimal,
    /// Per-issuer concentration data
    pub issuers: Vec<IssuerConcentration>,
}

impl ReserveComposition {
    /// Total bond allocation (sovereign + corporate)
    pub fn total_bond_pct(&self) -> Decimal {
        self.sovereign_bond_pct + self.corporate_bond_pct
    }

    /// Sum of all allocations
    pub fn total_pct(&self) -> Decimal {
        self.liquid_pct + self.sovereign_bond_pct + self.corporate_bond_pct + self.other_pct
    }
}

/// MiCA Art. 36 minimum liquid asset percentage
pub const MIN_LIQUID_PCT: Decimal = Decimal::from_parts(30, 0, 0, false, 0); // 30%
/// MiCA Art. 36 maximum bond allocation percentage
pub const MAX_BOND_PCT: Decimal = Decimal::from_parts(70, 0, 0, false, 0); // 70%
/// MiCA Art. 38 maximum single-issuer concentration (non-sovereign)
pub const MAX_ISSUER_PCT: Decimal = Decimal::from_parts(10, 0, 0, false, 0); // 10%
/// Tolerance for total percentage validation (rounding)
pub const PCT_TOLERANCE: Decimal = Decimal::from_parts(1, 0, 0, false, 1); // 0.1%

/// Validate reserve composition against MiCA Articles 36, 37, and 38.
///
/// Returns `Ok(())` if compliant, or all violations found.
pub fn validate_reserve_composition(
    composition: &ReserveComposition,
) -> Result<(), Vec<MicaViolation>> {
    let mut violations = Vec::new();

    // Validate percentages sum to ~100%
    let total = composition.total_pct();
    let diff = (total - Decimal::ONE_HUNDRED).abs();
    if diff > PCT_TOLERANCE {
        violations.push(MicaViolation::InvalidCompositionTotal { sum: total });
    }

    // Art. 36: Minimum 30% liquid assets
    if composition.liquid_pct < MIN_LIQUID_PCT {
        violations.push(MicaViolation::InsufficientLiquidAssets {
            actual: composition.liquid_pct,
        });
    }

    // Art. 36: Maximum 70% bonds
    let total_bonds = composition.total_bond_pct();
    if total_bonds > MAX_BOND_PCT {
        violations.push(MicaViolation::ExcessBondAllocation { actual: total_bonds });
    }

    // Art. 38: Maximum 10% per non-sovereign issuer
    for issuer in &composition.issuers {
        if !issuer.is_eca_sovereign && issuer.pct > MAX_ISSUER_PCT {
            violations.push(MicaViolation::IssuerConcentration {
                issuer: issuer.issuer_id.clone(),
                pct: issuer.pct,
            });
        }
    }

    // Art. 38: No other/crypto-asset exposure
    if composition.other_pct > Decimal::ZERO {
        violations.push(MicaViolation::CryptoAssetExposure);
    }

    if violations.is_empty() {
        Ok(())
    } else {
        Err(violations)
    }
}

/// Compute a reserve composition from raw custody holdings.
///
/// `bond_issuers`: list of (isin_prefix, market_value, is_sovereign)
/// `liquid_value`: total liquid asset value (cash + deposits + MMF)
/// `total_value`: total reserve value
pub fn compute_composition(
    liquid_value: Decimal,
    sovereign_bond_value: Decimal,
    corporate_bond_value: Decimal,
    bond_issuers: Vec<(String, Decimal, bool)>, // (issuer_id, value, is_sovereign)
    total_value: Decimal,
) -> ReserveComposition {
    if total_value.is_zero() {
        return ReserveComposition {
            liquid_pct: Decimal::ZERO,
            sovereign_bond_pct: Decimal::ZERO,
            corporate_bond_pct: Decimal::ZERO,
            other_pct: Decimal::ZERO,
            issuers: vec![],
        };
    }

    let hundred = Decimal::ONE_HUNDRED;
    let liquid_pct = (liquid_value / total_value) * hundred;
    let sovereign_bond_pct = (sovereign_bond_value / total_value) * hundred;
    let corporate_bond_pct = (corporate_bond_value / total_value) * hundred;
    let other_pct = hundred - liquid_pct - sovereign_bond_pct - corporate_bond_pct;

    let issuers = bond_issuers
        .into_iter()
        .map(|(issuer_id, value, is_eca_sovereign)| IssuerConcentration {
            issuer_id,
            pct: (value / total_value) * hundred,
            is_eca_sovereign,
        })
        .collect();

    ReserveComposition {
        liquid_pct,
        sovereign_bond_pct,
        corporate_bond_pct,
        other_pct: other_pct.max(Decimal::ZERO),
        issuers,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn compliant() -> ReserveComposition {
        ReserveComposition {
            liquid_pct: dec!(35),
            sovereign_bond_pct: dec!(60),
            corporate_bond_pct: dec!(5),
            other_pct: dec!(0),
            issuers: vec![
                IssuerConcentration { issuer_id: "DE".to_string(), pct: dec!(60), is_eca_sovereign: true },
                IssuerConcentration { issuer_id: "CORP-A".to_string(), pct: dec!(5), is_eca_sovereign: false },
            ],
        }
    }

    #[test]
    fn test_compliant_composition() {
        assert!(validate_reserve_composition(&compliant()).is_ok());
    }

    #[test]
    fn test_insufficient_liquid() {
        let mut c = compliant();
        c.liquid_pct = dec!(20);
        c.sovereign_bond_pct = dec!(75);
        let result = validate_reserve_composition(&c);
        assert!(result.is_err());
        let errs = result.unwrap_err();
        assert!(errs.iter().any(|e| matches!(e, MicaViolation::InsufficientLiquidAssets { .. })));
    }

    #[test]
    fn test_excess_bonds() {
        let mut c = compliant();
        c.liquid_pct = dec!(25);
        c.sovereign_bond_pct = dec!(75);
        let result = validate_reserve_composition(&c);
        assert!(result.is_err());
    }

    #[test]
    fn test_issuer_concentration_sovereign_exempt() {
        let mut c = compliant();
        // 100% German Bunds — sovereign exempt from 10% limit
        c.issuers = vec![IssuerConcentration {
            issuer_id: "DE".to_string(),
            pct: dec!(65),
            is_eca_sovereign: true,
        }];
        assert!(validate_reserve_composition(&c).is_ok());
    }

    #[test]
    fn test_issuer_concentration_violation() {
        let mut c = compliant();
        c.issuers.push(IssuerConcentration {
            issuer_id: "CORP-B".to_string(),
            pct: dec!(15),
            is_eca_sovereign: false,
        });
        let result = validate_reserve_composition(&c);
        assert!(result.is_err());
        let errs = result.unwrap_err();
        assert!(errs.iter().any(|e| matches!(e, MicaViolation::IssuerConcentration { .. })));
    }

    #[test]
    fn test_other_asset_violation() {
        let mut c = compliant();
        c.other_pct = dec!(5);
        c.sovereign_bond_pct = dec!(55);
        let result = validate_reserve_composition(&c);
        assert!(result.is_err());
        assert!(result.unwrap_err().iter().any(|e| matches!(e, MicaViolation::CryptoAssetExposure)));
    }
}
