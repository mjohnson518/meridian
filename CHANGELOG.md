# Meridian Platform - Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Complete Rust workspace structure with modular crate architecture
- Currency Basket Engine (`meridian-basket`) with 100% test coverage
  - Single-currency stablecoin support (EUR, GBP, JPY, etc.)
  - IMF SDR basket implementation with official 2024 weights
  - Custom multi-currency basket creation
  - Automated rebalancing logic
  - Real-time basket valuation
- Comprehensive documentation
  - `README.md` - Project overview and getting started guide
  - `PHASE1_PROGRESS.md` - Detailed development progress
  - `NAMING_CONVENTIONS.md` - Standard naming conventions
  - `meridian_issuer_tech_spec.md` - Technical specification
- Production-grade error handling with `thiserror`
- Zero floating-point arithmetic (all `rust_decimal::Decimal`)
- 16 unit tests + 6 documentation tests (all passing)

### Changed
- **[BREAKING]** Renamed platform from "Nova" to "Meridian" throughout codebase
  - Updated `meridian_issuer_tech_spec.md` architecture diagram
  - Updated smart contract name from `NovaMultiCurrencyStablecoin` to `MeridianMultiCurrencyStablecoin`
  - All imports and references now use "Meridian" consistently

### Security
- Implemented input validation for all currency basket operations
- Zero `unwrap()` calls in production code
- Type-safe error handling prevents panics
- ISO 4217 currency code validation
- Weight range validation prevents invalid basket configurations

## [0.1.0] - 2025-10-01

### Added - Phase 1, Week 1-4: Currency Basket Engine

#### Core Features
- **Single-Currency Baskets**
  - Create stablecoins pegged to any ISO 4217 currency
  - 100% weight allocation
  - No rebalancing required

- **IMF SDR Baskets**
  - Official 2024 IMF Special Drawing Rights composition:
    - USD: 43.38%
    - EUR: 29.31%
    - CNY: 12.28%
    - JPY: 7.59%
    - GBP: 7.44%
  - Configurable min/max weight bounds
  - Threshold-based rebalancing (5% deviation default)

- **Custom Multi-Currency Baskets**
  - Arbitrary currency combinations
  - User-defined weights (must sum to 100%)
  - Flexible rebalancing strategies:
    - None (single-currency)
    - Fixed interval (every N days)
    - Threshold-based (deviation exceeds X%)
    - Scheduled (specific timestamps)

#### Technical Implementation
- Rust workspace with 5 crates:
  - `meridian-basket` (complete)
  - `meridian-oracle` (stub)
  - `meridian-api` (stub)
  - `meridian-db` (stub)
  - `meridian-compliance` (stub)

- Dependencies:
  - `rust_decimal` v1.38 - Financial precision
  - `serde` v1.0 - Serialization
  - `chrono` v0.4 - Date/time handling
  - `uuid` v1.18 - Unique identifiers
  - `thiserror` v1.0 - Error handling

#### Test Coverage
- 16 comprehensive unit tests covering:
  - Basket creation (single, SDR, custom)
  - Value calculation
  - Rebalancing logic
  - Error handling
  - Edge cases
- 6 documentation tests verifying all examples
- 100% coverage of critical paths

#### Documentation
- Full rustdoc documentation for all public APIs
- Example code in every public function
- Comprehensive README with getting started guide
- Detailed progress tracking in PHASE1_PROGRESS.md
- Naming conventions documented in NAMING_CONVENTIONS.md

---

## Version History

### Current Status: v0.1.0 (Phase 1, Week 4)

✅ **Complete:**
- Currency Basket Engine
- Project structure and workspace setup
- Documentation and naming conventions
- Test suite with 100% critical path coverage

🚧 **In Progress:**
- None (Week 4 deliverables complete)

📅 **Next Up (Phase 1, Month 2):**
- Chainlink Oracle Integration
- Smart Contract Development (Solidity)

---

## Contributing

When contributing changes:

1. Update this CHANGELOG.md with your changes
2. Follow the naming conventions in NAMING_CONVENTIONS.md
3. Ensure all tests pass: `cargo test --all`
4. Run clippy: `cargo clippy --all-targets --all-features`
5. Format code: `cargo fmt --all`
6. Update documentation as needed

---

## Links

- [Technical Specification](./meridian_issuer_tech_spec.md)
- [Progress Tracking](./PHASE1_PROGRESS.md)
- [Naming Conventions](./NAMING_CONVENTIONS.md)
- [README](./README.md)

---

**Last Updated:** October 1, 2025  
**Current Version:** 0.1.0  
**Phase:** 1 - MVP Development (Week 4)

