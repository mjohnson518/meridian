# Meridian Database Layer

PostgreSQL database access layer with SQLx for persisting baskets, prices, and stablecoins.

## Features

- ✅ Repository pattern for data access
- ✅ Connection pooling with PgPool
- ✅ Type-safe queries with SQLx compile-time verification
- ✅ Database migrations
- ✅ Transaction support
- ✅ JSONB storage for complex structures
- ✅ Immutable audit trail

## Database Schema

### Tables

**baskets**
- Stores currency basket configurations
- JSONB for flexible component storage
- Supports single-currency, IMF SDR, and custom baskets

**price_history**
- Historical price data from oracles
- Indexed by currency pair and timestamp
- Tracks source (Chainlink, Band, Pyth)

**stablecoins**
- Deployed stablecoin instances
- Links to baskets
- Tracks contract addresses and chain IDs

**audit_logs**
- Immutable audit trail
- Cannot be modified or deleted
- Tracks all operations

## Setup

### 1. Install PostgreSQL

```bash
# macOS
brew install postgresql@14
brew services start postgresql@14

# Linux
sudo apt-get install postgresql-14
```

### 2. Create Database

```bash
createdb meridian_dev
createdb meridian_test  # For testing
```

### 3. Set Environment Variable

```bash
export DATABASE_URL="postgresql://postgres:password@localhost/meridian_dev"
```

### 4. Run Migrations

```bash
cd crates/db
sqlx migrate run
```

Or programmatically:

```rust
use meridian_db::{create_pool, run_migrations};

let pool = create_pool(&database_url).await?;
run_migrations(&pool).await?;
```

## Usage

### Create Connection Pool

```rust
use meridian_db::create_pool;

let pool = create_pool("postgresql://postgres:password@localhost/meridian_dev").await?;
```

### Basket Operations

```rust
use meridian_db::BasketRepository;
use meridian_basket::CurrencyBasket;

let repo = BasketRepository::new(pool.clone());

// Create basket
let basket = CurrencyBasket::new_single_currency(
    "EUR Basket".to_string(),
    "EUR".to_string(),
    "0xb49f677943BC038e9857d61E7d053CaA2C1734C1".to_string(),
)?;

let basket_id = repo.create(&basket).await?;

// Find basket
let found = repo.find_by_id(basket_id).await?;

// List baskets with pagination
let baskets = repo.list(10, 0).await?; // 10 items, offset 0

// Count baskets
let total = repo.count().await?;

// Delete basket
repo.delete(basket_id).await?;
```

### Price History Operations

```rust
use meridian_db::{PriceRepository, InsertPriceRequest};
use rust_decimal::Decimal;

let repo = PriceRepository::new(pool.clone());

// Insert price
let request = InsertPriceRequest {
    currency_pair: "EUR/USD".to_string(),
    price: Decimal::new(108, 2),
    source: "chainlink".to_string(),
    is_stale: false,
    round_id: Some(Decimal::from(12345)),
};

repo.insert(request).await?;

// Get latest price
let latest = repo.get_latest("EUR/USD").await?;

// Get price history
let start = chrono::Utc::now() - chrono::Duration::days(7);
let end = chrono::Utc::now();
let history = repo.get_history("EUR/USD", start, end, 100).await?;

// Get statistics
let stats = repo.get_stats("EUR/USD", start).await?;
println!("Min: {}, Max: {}, Avg: {}", stats.min_price, stats.max_price, stats.avg_price);
```

### Stablecoin Operations

```rust
use meridian_db::{StablecoinRepository, CreateStablecoinRequest};

let repo = StablecoinRepository::new(pool.clone());

// Create stablecoin record
let request = CreateStablecoinRequest {
    name: "EUR Meridian".to_string(),
    symbol: "EURM".to_string(),
    basket_id: Some(basket_id),
    chain_id: 11155111, // Sepolia
};

let stablecoin_id = repo.create(request).await?;

// Set contract address after deployment
repo.set_contract_address(stablecoin_id, "0x...").await?;

// Update balances
repo.update_balances(
    stablecoin_id,
    Decimal::from(1000000), // total supply
    Decimal::from(1000000), // total reserves
).await?;

// Find by contract address
let stablecoin = repo.find_by_contract_address("0x...").await?;
```

### Audit Logging

```rust
use meridian_db::{AuditRepository, CreateAuditLogRequest};

let repo = AuditRepository::new(pool.clone());

// Log operation
let request = CreateAuditLogRequest {
    operation: "basket_created".to_string(),
    actor: Some("admin@meridian.com".to_string()),
    stablecoin_id: None,
    basket_id: Some(basket_id),
    details: serde_json::json!({
        "basket_name": "EUR Basket",
        "basket_type": "single_currency"
    }),
};

repo.log(request).await?;

// Get basket audit trail
let logs = repo.get_basket_logs(basket_id, 50).await?;

// Get recent operations
let recent = repo.get_recent(100).await?;
```

## Migrations

Migrations are located in `migrations/` and are run automatically with `run_migrations()`.

**Migration Files:**
1. `20251001000001_create_baskets_table.sql`
2. `20251001000002_create_price_history_table.sql`
3. `20251001000003_create_stablecoins_table.sql`
4. `20251001000004_create_audit_logs_table.sql`

### Create New Migration

```bash
# Using SQLx CLI
sqlx migrate add create_new_table

# Manually
touch migrations/$(date +%Y%m%d%H%M%S)_description.sql
```

### Run Migrations

```bash
sqlx migrate run
```

### Revert Last Migration

```bash
sqlx migrate revert
```

## Testing

### Run Tests

Tests require a PostgreSQL database:

```bash
# Set up test database
createdb meridian_test

# Run tests
DATABASE_URL="postgresql://postgres:password@localhost/meridian_test" \
  cargo test --package meridian-db
```

### Test Strategy

All tests use test transactions that rollback, ensuring:
- No test data pollution
- Isolated test execution
- Fast cleanup

## Architecture

```
┌─────────────────────────────────────┐
│       Application Layer             │
│  (API handlers, business logic)     │
└──────────────┬──────────────────────┘
               │
               ▼
┌─────────────────────────────────────┐
│       Repository Layer              │
│  - BasketRepository                 │
│  - PriceRepository                  │
│  - StablecoinRepository             │
│  - AuditRepository                  │
└──────────────┬──────────────────────┘
               │
               ▼
┌─────────────────────────────────────┐
│           SQLx                      │
│  (Compile-time verified queries)    │
└──────────────┬──────────────────────┘
               │
               ▼
┌─────────────────────────────────────┐
│        PostgreSQL 14+               │
│  (Persistent storage)               │
└─────────────────────────────────────┘
```

## Performance

### Connection Pooling
- Max connections: 5
- Min connections: 1
- Acquire timeout: 30s
- Idle timeout: 10 minutes

### Indexes
All tables have appropriate indexes for common queries:
- Baskets: `basket_type`, `created_at`
- Prices: `(currency_pair, timestamp)`, `timestamp`
- Stablecoins: `contract_address`, `basket_id`, `chain_id`, `status`
- Audit logs: `operation`, `timestamp`, `actor`, `stablecoin_id`

### Query Performance
- Find by ID: <1ms (primary key lookup)
- List with pagination: <5ms (indexed scan)
- Latest price: <1ms (indexed DESC order)
- Audit trail: <10ms (indexed scan)

## Security

### Audit Trail
- **Immutable**: Cannot be modified or deleted
- **Comprehensive**: All operations logged
- **Timestamped**: Precise timestamp on each entry
- **Traceable**: Links to actors, baskets, and stablecoins

### SQL Injection Protection
- All queries use parameterized statements
- SQLx compile-time verification
- No string concatenation

### Data Integrity
- Foreign key constraints
- Check constraints on enums
- Unique constraints on critical fields
- NOT NULL constraints where appropriate

## Best Practices

### Use Transactions for Multi-Step Operations

```rust
let mut tx = pool.begin().await?;

// Multiple operations
repo.create_basket_tx(&mut tx, &basket).await?;
repo.log_audit_tx(&mut tx, audit_request).await?;

// Commit or rollback
tx.commit().await?;
```

### Handle Errors Gracefully

```rust
match repo.find_by_id(id).await {
    Ok(basket) => println!("Found: {}", basket.name),
    Err(DbError::NotFound(_)) => println!("Basket not found"),
    Err(e) => eprintln!("Database error: {}", e),
}
```

### Use Pagination for Large Results

```rust
let page_size = 50;
let page = 0;
let offset = page * page_size;

let baskets = repo.list(page_size, offset).await?;
```

## Environment Variables

```bash
# Development
DATABASE_URL=postgresql://postgres:password@localhost/meridian_dev

# Test
DATABASE_URL=postgresql://postgres:password@localhost/meridian_test

# Production (use connection pooler like PgBouncer)
DATABASE_URL=postgresql://user:pass@prod-db.example.com/meridian?sslmode=require
```

## Troubleshooting

### "relation does not exist"

Run migrations:
```bash
sqlx migrate run
```

### "prepared statement already exists"

This can happen with connection pooling. SQLx handles it automatically.

### "too many connections"

Adjust max_connections in pool configuration:
```rust
PgPoolOptions::new().max_connections(10)
```

## License

MIT

