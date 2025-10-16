# Meridian Multi-Currency Stablecoin Platform

Meridian provides turnkey infrastructure for launching compliant stablecoins backed by sovereign bonds and fiat reserves. Unlike competitors who focus exclusively on USD, Meridian enables the 99% of the market that's currently underserved: non-USD currencies.

## Why Meridian Exists

**The Problem:** Today's stablecoin infrastructure is USD-centric. Circle, Tether, and Stripe/Bridge focus almost exclusively on USD-backed stablecoins, leaving 195+ countries and their currencies underserved. Banks and fintechs in Europe, Latin America, Asia, and Africa face an 18-month DIY journey to launch a compliant EUR, GBP, or JPY stablecoin—requiring blockchain expertise, regulatory navigation, and oracle integration they don't have.

**The Opportunity:** The EUR, CNY, JPY, GBP, and INR represent $100+ trillion in annual transaction volume. Yet these markets lack the stablecoin infrastructure that USD enjoys. Meridian democratizes stablecoin issuance for the global majority.

**The Solution:** Meridian is "Stripe for Multi-Currency Stablecoins" — turnkey infra that enables any institution to launch a compliant, multi-currency stablecoin in 30 days instead of 18 months. We provide the smart contracts, oracle integration, compliance modules, and regulatory templates so banks can focus on their customers, not blockchain plumbing.

**The Impact:** By making multi-currency stablecoins accessible, Meridian unlocks:
- **Cross-border payments** with local currency stablecoins (no USD intermediary)
- **Emerging market access** to blockchain finance (BRL, INR, MXN stablecoins)
- **Corporate treasury tools** with custom multi-currency baskets for FX hedging
- **Financial inclusion** for the ~7 billion people in non-USD economies

## Product Vision

Launch compliant stablecoins in **30 days** vs. 18 months DIY:
- Support **50+ currency pairs** and custom baskets
- **Automated compliance** (GENIUS Act, MiCA, multi-jurisdiction)
- **Chainlink oracle integration** for real-time FX data
- **Multi-chain deployment** (Ethereum, Tempo, Arc, Base)

## Architecture

```
meridian/
├── crates/
│   ├── basket/           # Currency basket engine
│   ├── oracle/           # Chainlink integration
│   ├── api/              # REST API server
│   ├── db/               # PostgreSQL database layer
│   └── compliance/       # Compliance module (planned)
├── contracts/            # Solidity smart contracts
└── dashboard/            # Next.js frontend (planned)
```

## Components

### 1. Currency Basket Engine - COMPLETE

Core business logic for managing multi-currency stablecoins and baskets.

**Features:**
- Single-currency stablecoin configuration (EUR, GBP, JPY, etc.)
- IMF SDR basket implementation with official weights
- Custom basket creation with arbitrary weights
- Automated rebalancing based on deviation thresholds
- Real-time basket valuation with precise decimal arithmetic

**Example Usage:**
```rust
use meridian_basket::{CurrencyBasket, CurrencyComponent, RebalanceStrategy};
use rust_decimal::Decimal;
use std::collections::HashMap;

// Create a single-currency EUR basket
let eur_basket = CurrencyBasket::new_single_currency(
    "EUR Basket".to_string(),
    "EUR".to_string(),
    "0xb49f677943BC038e9857d61E7d053CaA2C1734C1".to_string(),
)?;

// Create IMF SDR basket
let mut feeds = HashMap::new();
feeds.insert("USD".to_string(), "0x...".to_string());
feeds.insert("EUR".to_string(), "0x...".to_string());
// ... more currencies

let sdr_basket = CurrencyBasket::new_imf_sdr("IMF SDR".to_string(), feeds)?;

// Calculate basket value
let mut prices = HashMap::new();
prices.insert("EUR".to_string(), Decimal::new(108, 2)); // 1.08 EUR/USD

let value = eur_basket.calculate_value(&prices)?;
```

**Test Coverage:**
- Single-currency basket creation and valuation
- IMF SDR basket with official weights (43.38% USD, 29.31% EUR, etc.)
- Custom basket weight validation (must sum to 100%)
- Rebalancing triggers based on deviation thresholds
- Invalid currency code detection
- Missing price error handling
- Decimal precision verification (no floating-point errors)

### 2. Oracle Integration (Chainlink) - COMPLETE

Real-time FX price feeds for all supported currency pairs using Chainlink's decentralized oracle network.

**Implemented Features:**
- Connect to Ethereum mainnet via HTTP RPC (Alchemy/Infura)
- Query Chainlink price feeds for 8+ currency pairs (EUR, GBP, JPY, CNY, CHF, BRL, MXN, INR)
- Automatic staleness detection (>1 hour)
- Deviation threshold monitoring (configurable, default 10%)
- Precise decimal conversion (no floating-point errors)
- Production-grade error handling
- Comprehensive integration tests

**Example Usage:**
```rust
use meridian_oracle::{ChainlinkOracle, mainnet_feeds};
use rust_decimal::Decimal;

// Connect to Ethereum mainnet
let oracle = ChainlinkOracle::new(rpc_url, Decimal::new(10, 0)).await?;

// Register EUR/USD feed
oracle.register_price_feed("EUR/USD", mainnet_feeds::eur_usd()).await?;

// Fetch latest price
let price = oracle.update_price("EUR/USD").await?;
println!("EUR/USD: ${}", price);
```

**Test Coverage:**
- Decimal conversion accuracy
- Price feed registration
- Staleness detection
- Multiple currency pairs
- Error handling

**Future Enhancements:**
- WebSocket subscriptions for real-time updates
- Multi-source aggregation (Band Protocol, Pyth)
- Historical price caching in Redis

### 3. Smart Contracts - COMPLETE (OpenZeppelin v5)

ERC-20 compatible stablecoins with multi-currency support using UUPS upgradeable proxy pattern. Migrated to OpenZeppelin v5 with all tests passing.

**Implemented Contracts:**
- `MeridianStablecoin.sol` - Main ERC-20 token with basket support, role-based access control, compliance features
- `MeridianFactory.sol` - Deployment factory for new stablecoin instances with registry management

**Features:**
- UUPS upgradeable proxy pattern
- Role-based access control (MINTER, BURNER, PAUSER, UPGRADER)
- Mint with 1:1 reserve verification
- Burn with pro-rata reserve release
- Blacklist/whitelist for compliance
- Reserve attestation tracking
- Emergency pause functionality
- Support for SingleCurrency, IMF SDR, and CustomBasket types

**Test Coverage:**
- 49 comprehensive tests (31 stablecoin + 18 factory) - 100% passing
- Minting and burning logic
- Access control enforcement
- Compliance features
- Emergency mechanisms
- Reserve calculations
- OpenZeppelin v5 compatibility validated

**Status:** Ready for Sepolia testnet deployment

**Deployment:**
```bash
# Dependencies already installed (OpenZeppelin v5.4.0)
cd contracts

# Build and test
forge build
forge test  # All 49 tests should pass

# Deploy to Sepolia
forge script script/Deploy.s.sol --rpc-url $SEPOLIA_RPC_URL --broadcast --verify
```

### 4. REST API - COMPLETE

Backend API for web dashboard and customer integrations using Actix-web.

**Implemented Endpoints:**

**Basket Management:**
- `POST /api/v1/baskets/single-currency` - Create single-currency basket
- `POST /api/v1/baskets/imf-sdr` - Create IMF SDR basket
- `POST /api/v1/baskets/custom` - Create custom basket
- `GET /api/v1/baskets` - List all baskets
- `GET /api/v1/baskets/{id}` - Get basket details
- `GET /api/v1/baskets/{id}/value` - Calculate basket value

**Oracle Operations:**
- `GET /api/v1/oracle/prices` - Get all current prices
- `GET /api/v1/oracle/prices/{pair}` - Get specific price
- `POST /api/v1/oracle/prices/{pair}/update` - Update price from blockchain
- `POST /api/v1/oracle/feeds` - Register new price feed

**Features:**
- CORS support for web clients
- JSON request/response serialization
- Comprehensive error handling with proper HTTP status codes
- Health check endpoint
- Structured logging
- Thread-safe shared state

**Test Coverage:**
- 9 integration tests covering all endpoints
- Error handling (404s, validation errors)
- CORS headers

**Usage:**
```bash
# Start server
export MERIDIAN_API_PORT=8080
export ETHEREUM_RPC_URL=https://eth-mainnet.g.alchemy.com/v2/YOUR_KEY
cargo run --bin meridian-api

# Test endpoint
curl http://localhost:8080/health
```

### 5. Database Layer - COMPLETE

PostgreSQL database layer with SQLx for persistent storage.

**Features:**
- Repository pattern for clean data access
- Connection pooling with PgPool
- Type-safe queries with SQLx compile-time verification
- Database migrations (5 tables)
- Transaction support
- JSONB storage for complex structures
- Immutable audit trail

**Tables:**
- `baskets` - Currency basket configurations
- `price_history` - Historical FX price data
- `stablecoins` - Deployed stablecoin instances
- `audit_logs` - Immutable audit trail (cannot be modified/deleted)

**Repositories:**
- `BasketRepository` - CRUD operations for baskets
- `PriceRepository` - Price history and statistics
- `StablecoinRepository` - Stablecoin management
- `AuditRepository` - Audit trail queries

**Usage:**
```bash
# Setup database
createdb meridian_dev
export DATABASE_URL="postgresql://postgres:password@localhost/meridian_dev"

# Run migrations
cd crates/db
sqlx migrate run

# Or programmatically
use meridian_db::{create_pool, run_migrations};
let pool = create_pool(&database_url).await?;
run_migrations(&pool).await?;
```

**Test Coverage:**
- Repository tests with PostgreSQL
- Migration tests
- Audit log immutability
- Transaction rollback

### 6. Web Dashboard

Customer-facing interface for managing stablecoins (Next.js).

### 7. Compliance Module

Automated regulatory compliance for multiple jurisdictions.

## Getting Started

### Prerequisites

- Rust 1.70+ (latest stable)
- PostgreSQL 14+
- Redis 7+
- Node.js 18+ (for dashboard)

### Installation

```bash
# Clone the repository
git clone https://github.com/your-org/meridian.git
cd meridian

# Build all crates
cargo build --all

# Run tests
cargo test --all

# Run basket engine tests specifically
cargo test --package meridian-basket
```

### Running Tests

```bash
# All tests
cargo test --all

# Basket engine only
cargo test --package meridian-basket

# With output
cargo test --package meridian-basket -- --nocapture

# Specific test
cargo test --package meridian-basket test_imf_sdr_basket_valuation
```

## Security Principles

1. **Never trust user input** - Validate everything
2. **Defense in depth** - Multiple layers of security
3. **Principle of least privilege** - Minimal permissions
4. **Fail securely** - Default to deny, not allow
5. **Immutable audit trail** - Log all sensitive operations
6. **Crypto-agility** - Design for algorithm upgrades
7. **No floating-point for money** - Always use `rust_decimal::Decimal`

## Development Status

### Phase 1 Progress

| Component | Status |
|-----------|--------|
| Basket Engine | Complete |
| Chainlink Integration | Complete |
| Smart Contracts | Complete |
| REST API | Complete |
| Database Layer | Complete |
| Web Dashboard | In Progress |
| Compliance Module | Planned |

## Code Quality Standards

### Rust Code
- Follow Rust API guidelines
- Use `cargo fmt` and `cargo clippy` before every commit
- Write comprehensive unit tests (>80% coverage)
- Document all public APIs with rustdoc comments
- Use `thiserror` for error handling, never `unwrap()` in production
- Implement proper logging with `tracing` crate
- Use `rust_decimal` for all financial calculations

### Testing Strategy
- **Unit Tests**: Test individual functions and modules
- **Integration Tests**: Test component interactions
- **Security Tests**: Fuzzing with Echidna, static analysis with Slither
- **Manual Tests**: Penetration testing before production

## License

MIT License - See LICENSE file for details

## 🤝 Contributing

This is production financial infrastructure. Every line of code matters.

1. Follow all code quality standards
2. Write comprehensive tests
3. Document all public APIs
4. Never use floating-point for financial calculations
5. All PRs require security review

## Contact

For questions or support, contact the Meridian team.

---

**Built with Rust 🦀 for security, precision, and performance.**

