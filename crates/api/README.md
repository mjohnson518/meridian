# Meridian REST API

HTTP API service for managing multi-currency stablecoins and accessing oracle price feeds.

## Features

- ✅ Currency basket management (create, read, value calculation)
- ✅ Oracle price feed integration (Chainlink)
- ✅ Real-time FX price queries
- ✅ CORS support for web clients
- ✅ JSON request/response serialization
- ✅ Comprehensive error handling
- ✅ Health check endpoint

## Endpoints

### Health Check

```
GET /health
```

**Response:**
```json
{
  "status": "ok",
  "version": "0.1.0",
  "oracle_enabled": true,
  "baskets_count": 5
}
```

### Basket Endpoints

#### Create Single-Currency Basket

```
POST /api/v1/baskets/single-currency
```

**Request:**
```json
{
  "name": "EUR Basket",
  "currency_code": "EUR",
  "chainlink_feed": "0xb49f677943BC038e9857d61E7d053CaA2C1734C1"
}
```

**Response:**
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "name": "EUR Basket",
  "basket_type": "single_currency",
  "components": [...],
  "rebalance_strategy": "none",
  "created_at": "2025-10-01T12:00:00Z"
}
```

#### Create IMF SDR Basket

```
POST /api/v1/baskets/imf-sdr
```

**Request:**
```json
{
  "name": "IMF SDR Basket",
  "chainlink_feeds": {
    "USD": "0x...",
    "EUR": "0xb49f677943BC038e9857d61E7d053CaA2C1734C1",
    "CNY": "0xeF8A4aF35cd47424672E3C590aBD37FBB7A7759a",
    "JPY": "0xBcE206caE7f0ec07b545EddE332A47C2F75bbeb3",
    "GBP": "0x5c0Ab2d9b5a7ed9f470386e82BB36A3613cDd4b5"
  }
}
```

#### Create Custom Basket

```
POST /api/v1/baskets/custom
```

**Request:**
```json
{
  "name": "EUR-GBP Basket",
  "components": [
    {
      "currency_code": "EUR",
      "target_weight": "60.0",
      "min_weight": "55.0",
      "max_weight": "65.0",
      "chainlink_feed": "0xb49f677943BC038e9857d61E7d053CaA2C1734C1"
    },
    {
      "currency_code": "GBP",
      "target_weight": "40.0",
      "min_weight": "35.0",
      "max_weight": "45.0",
      "chainlink_feed": "0x5c0Ab2d9b5a7ed9f470386e82BB36A3613cDd4b5"
    }
  ],
  "rebalance_strategy": {
    "type": "threshold_based",
    "max_deviation_percent": "3.0"
  }
}
```

#### List All Baskets

```
GET /api/v1/baskets
```

**Response:**
```json
[
  {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "name": "EUR Basket",
    "basket_type": "single_currency",
    ...
  }
]
```

#### Get Basket by ID

```
GET /api/v1/baskets/{id}
```

**Response:**
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "name": "EUR Basket",
  "basket_type": "single_currency",
  "components": [...],
  "rebalance_strategy": "none",
  "created_at": "2025-10-01T12:00:00Z"
}
```

#### Calculate Basket Value

```
GET /api/v1/baskets/{id}/value
```

**Response:**
```json
{
  "basket_id": "550e8400-e29b-41d4-a716-446655440000",
  "value_usd": "1.08",
  "prices_used": {
    "EUR": "1.08"
  },
  "needs_rebalancing": false,
  "calculated_at": "2025-10-01T12:00:00Z"
}
```

### Oracle Endpoints

#### Get All Prices

```
GET /api/v1/oracle/prices
```

**Response:**
```json
{
  "prices": {
    "EUR/USD": {
      "price_usd": "1.08",
      "is_stale": false,
      "updated_at": "2025-10-01T12:00:00Z"
    },
    "GBP/USD": {
      "price_usd": "1.27",
      "is_stale": false,
      "updated_at": "2025-10-01T12:00:00Z"
    }
  }
}
```

#### Get Specific Price

```
GET /api/v1/oracle/prices/EUR%2FUSD
```

**Response:**
```json
{
  "pair": "EUR/USD",
  "price_usd": "1.08",
  "is_stale": false,
  "updated_at": "2025-10-01T12:00:00Z"
}
```

#### Update Price

```
POST /api/v1/oracle/prices/EUR%2FUSD/update
```

**Response:**
```json
{
  "pair": "EUR/USD",
  "price_usd": "1.08",
  "is_stale": false,
  "updated_at": "2025-10-01T12:00:00Z"
}
```

#### Register Price Feed

```
POST /api/v1/oracle/feeds
```

**Request:**
```json
{
  "pair": "EUR/USD",
  "chainlink_address": "0xb49f677943BC038e9857d61E7d053CaA2C1734C1"
}
```

**Response:**
```json
{
  "success": true,
  "pair": "EUR/USD",
  "address": "0xb49f677943BC038e9857d61E7d053CaA2C1734C1"
}
```

## Running the API

### Prerequisites

- Rust 1.70+
- (Optional) Ethereum RPC URL for oracle features

### Configuration

Set environment variables:

```bash
export MERIDIAN_API_HOST=127.0.0.1
export MERIDIAN_API_PORT=8080
export ETHEREUM_RPC_URL=https://eth-mainnet.g.alchemy.com/v2/YOUR_KEY  # Optional
```

### Start the Server

```bash
cargo run --bin meridian-api
```

The server will start at `http://127.0.0.1:8080`

### Testing

```bash
# Run integration tests
cargo test --package meridian-api

# Test health endpoint
curl http://localhost:8080/health

# Create a basket
curl -X POST http://localhost:8080/api/v1/baskets/single-currency \
  -H "Content-Type: application/json" \
  -d '{
    "name": "EUR Basket",
    "currency_code": "EUR",
    "chainlink_feed": "0xb49f677943BC038e9857d61E7d053CaA2C1734C1"
  }'

# List baskets
curl http://localhost:8080/api/v1/baskets
```

## Error Responses

All errors return JSON with the following structure:

```json
{
  "error": "error_type",
  "message": "Human-readable error message",
  "details": "Optional additional details"
}
```

**Error Types:**
- `basket_error` - Basket validation or operation failed
- `oracle_error` - Oracle operation failed
- `not_found` - Resource not found (404)
- `bad_request` - Invalid request (400)
- `oracle_not_configured` - Oracle features disabled (503)
- `internal_error` - Server error (500)

## Architecture

```
┌─────────────────────────────────────┐
│        Actix-web HTTP Server        │
├─────────────────────────────────────┤
│  Routes                             │
│  ├── /health                        │
│  ├── /api/v1/baskets/*              │
│  └── /api/v1/oracle/*               │
├─────────────────────────────────────┤
│  Handlers                           │
│  ├── baskets.rs                     │
│  ├── oracle.rs                      │
│  └── health.rs                      │
├─────────────────────────────────────┤
│  Application State                  │
│  ├── Baskets Registry               │
│  └── Chainlink Oracle (optional)    │
├─────────────────────────────────────┤
│  Core Libraries                     │
│  ├── meridian-basket                │
│  └── meridian-oracle                │
└─────────────────────────────────────┘
```

## Security

### CORS
Configured to allow any origin in development. **Configure restrictively in production:**

```rust
let cors = Cors::default()
    .allowed_origin("https://yourdomain.com")
    .allowed_methods(vec!["GET", "POST"])
    .max_age(3600);
```

### Input Validation
- All basket creation validates currency codes (ISO 4217)
- Weights must sum to 100%
- Chainlink addresses validated via ethers-rs

### Error Handling
- Never exposes internal errors to clients
- All errors logged with tracing
- Graceful degradation when oracle unavailable

## Performance

### Latency
- **Basket operations**: <10ms (in-memory)
- **Oracle price (cached)**: <1ms
- **Oracle price (update)**: ~200-500ms (blockchain call)

### Concurrency
- Fully async with tokio
- Thread-safe shared state with Arc<RwLock<>>
- Can handle 1000+ concurrent requests

## Development

### Run with hot reload

```bash
cargo watch -x 'run --bin meridian-api'
```

### Run tests with output

```bash
cargo test --package meridian-api -- --nocapture
```

### Check for errors

```bash
cargo clippy --package meridian-api
```

## License

MIT

