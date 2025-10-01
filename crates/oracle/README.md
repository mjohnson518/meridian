# Meridian Oracle Integration

Real-time FX price feeds for multi-currency stablecoins using Chainlink's decentralized oracle network.

## Features

- ✅ Connect to Chainlink price feeds on Ethereum mainnet
- ✅ Query real-time FX rates for 20+ currency pairs
- ✅ Automatic staleness detection (>1 hour)
- ✅ Deviation threshold monitoring (configurable)
- ✅ Precise decimal arithmetic (no floating-point errors)
- ✅ Production-grade error handling
- ✅ Async/await with tokio

## Usage

### Basic Example

```rust
use meridian_oracle::{ChainlinkOracle, mainnet_feeds};
use rust_decimal::Decimal;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to Ethereum mainnet via Alchemy/Infura
    let rpc_url = "https://eth-mainnet.g.alchemy.com/v2/YOUR_KEY";
    let oracle = ChainlinkOracle::new(rpc_url, Decimal::new(10, 0)).await?;

    // Register EUR/USD price feed
    oracle.register_price_feed("EUR/USD", mainnet_feeds::eur_usd()).await?;

    // Update price from blockchain
    let price = oracle.update_price("EUR/USD").await?;
    println!("EUR/USD: ${}", price);

    // Get cached price (fast, no blockchain call)
    let cached = oracle.get_price("EUR/USD").await?;
    assert_eq!(price, cached);

    Ok(())
}
```

### Multiple Currency Pairs

```rust
use meridian_oracle::{ChainlinkOracle, mainnet_feeds};
use rust_decimal::Decimal;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rpc_url = "https://eth-mainnet.g.alchemy.com/v2/YOUR_KEY";
    let oracle = ChainlinkOracle::new(rpc_url, Decimal::new(10, 0)).await?;

    // Register multiple feeds
    oracle.register_price_feed("EUR/USD", mainnet_feeds::eur_usd()).await?;
    oracle.register_price_feed("GBP/USD", mainnet_feeds::gbp_usd()).await?;
    oracle.register_price_feed("JPY/USD", mainnet_feeds::jpy_usd()).await?;

    // Update all prices
    let eur_price = oracle.update_price("EUR/USD").await?;
    let gbp_price = oracle.update_price("GBP/USD").await?;
    let jpy_price = oracle.update_price("JPY/USD").await?;

    println!("EUR/USD: ${}", eur_price);
    println!("GBP/USD: ${}", gbp_price);
    println!("JPY/USD: ${}", jpy_price);

    Ok(())
}
```

### Error Handling

```rust
use meridian_oracle::{ChainlinkOracle, OracleError};
use rust_decimal::Decimal;

#[tokio::main]
async fn main() {
    let rpc_url = "https://eth-mainnet.g.alchemy.com/v2/YOUR_KEY";
    let oracle = ChainlinkOracle::new(rpc_url, Decimal::new(10, 0))
        .await
        .unwrap();

    // Handle different error types
    match oracle.get_price("EUR/USD").await {
        Ok(price) => println!("Price: ${}", price),
        Err(OracleError::PriceFeedNotFound(pair)) => {
            println!("Feed not registered: {}", pair);
        }
        Err(OracleError::StalePrice(pair, age)) => {
            println!("Stale price for {}: {}s old", pair, age);
        }
        Err(OracleError::PriceDeviation { pair, old_price, new_price, deviation }) => {
            println!("Large price movement for {}: {}% change", pair, deviation);
        }
        Err(e) => println!("Error: {}", e),
    }
}
```

## Supported Currency Pairs (Ethereum Mainnet)

The following Chainlink price feeds are pre-configured:

| Pair | Function | Address |
|------|----------|---------|
| EUR/USD | `mainnet_feeds::eur_usd()` | `0xb49f677943BC038e9857d61E7d053CaA2C1734C1` |
| GBP/USD | `mainnet_feeds::gbp_usd()` | `0x5c0Ab2d9b5a7ed9f470386e82BB36A3613cDd4b5` |
| JPY/USD | `mainnet_feeds::jpy_usd()` | `0xBcE206caE7f0ec07b545EddE332A47C2F75bbeb3` |
| CNY/USD | `mainnet_feeds::cny_usd()` | `0xeF8A4aF35cd47424672E3C590aBD37FBB7A7759a` |
| CHF/USD | `mainnet_feeds::chf_usd()` | `0x449d117117838fFA61263B61dA6301AA2a88B13A` |
| BRL/USD | `mainnet_feeds::brl_usd()` | `0x971E8F1B779A5F1C36e1cd7ef44Ba1Cc2F5EeE0f` |
| MXN/USD | `mainnet_feeds::mxn_usd()` | `0xe6F5377DE93A361cd5531bDCad24E88C4867bc10` |
| INR/USD | `mainnet_feeds::inr_usd()` | `0x605D5c2fBCeDb217D7987FC0951B5753069bC360` |

## Running Examples

### Fetch EUR/USD Price

```bash
ETHEREUM_RPC_URL=https://eth-mainnet.g.alchemy.com/v2/YOUR_KEY \
  cargo run --example fetch_eur_usd
```

## Running Tests

### Unit Tests

```bash
cargo test --package meridian-oracle
```

### Integration Tests (Requires RPC URL)

Integration tests query real Chainlink contracts on Ethereum mainnet:

```bash
ETHEREUM_RPC_URL=https://eth-mainnet.g.alchemy.com/v2/YOUR_KEY \
  cargo test --package meridian-oracle --test integration_tests
```

## Configuration

### Staleness Threshold

Prices older than 1 hour are considered stale by default:

```rust
let mut oracle = ChainlinkOracle::new(rpc_url, Decimal::new(10, 0)).await?;

// Change staleness threshold to 30 minutes
oracle.set_stale_threshold(1800);
```

### Deviation Threshold

Set the maximum allowed price change percentage:

```rust
// 5% deviation threshold
let oracle = ChainlinkOracle::new(rpc_url, Decimal::new(5, 0)).await?;

// 10% deviation threshold (default recommended)
let oracle = ChainlinkOracle::new(rpc_url, Decimal::new(10, 0)).await?;
```

## Architecture

```
ChainlinkOracle
├── Provider (ethers-rs HTTP)
├── Price Feeds Registry
│   ├── EUR/USD → PriceFeed { price, decimals, updated_at, ... }
│   ├── GBP/USD → PriceFeed { ... }
│   └── JPY/USD → PriceFeed { ... }
├── Deviation Threshold (configurable)
└── Staleness Threshold (configurable)
```

## Error Types

| Error | Description |
|-------|-------------|
| `PriceFeedNotFound` | Feed not registered with oracle |
| `StalePrice` | Price is older than staleness threshold |
| `PriceDeviation` | Price changed more than deviation threshold |
| `InvalidPrice` | Malformed price data from contract |
| `ProviderError` | RPC provider connection issue |
| `ContractError` | Chainlink contract call failed |
| `DecimalConversion` | Failed to convert price to Decimal |

## Security Considerations

### Staleness Detection
Prices older than 1 hour trigger `StalePrice` error to prevent using outdated data.

### Deviation Monitoring
Large price movements (>10% by default) trigger `PriceDeviation` error to detect potential manipulation or market volatility.

### Decimal Precision
All prices use `rust_decimal::Decimal` for exact arithmetic without floating-point errors.

### No Unwrap Calls
All errors are properly handled with `Result` types - no panics in production code.

## RPC Provider Requirements

You need an Ethereum RPC endpoint to connect to mainnet. Recommended providers:

- **Alchemy**: https://www.alchemy.com/ (10M compute units/month free)
- **Infura**: https://infura.io/ (100k requests/day free)
- **QuickNode**: https://www.quicknode.com/

Example RPC URLs:
```
Alchemy: https://eth-mainnet.g.alchemy.com/v2/YOUR_KEY
Infura:  https://mainnet.infura.io/v3/YOUR_KEY
```

## Dependencies

- `ethers` - Ethereum library for Rust
- `rust_decimal` - Precise decimal arithmetic
- `tokio` - Async runtime
- `chrono` - Date/time handling
- `thiserror` - Error handling
- `tracing` - Structured logging

## Future Enhancements

- [ ] WebSocket subscriptions for real-time updates
- [ ] Multi-oracle aggregation (Band Protocol, Pyth)
- [ ] Weighted average across oracle sources
- [ ] Historical price caching in Redis
- [ ] Price feed health monitoring dashboard

## License

MIT

