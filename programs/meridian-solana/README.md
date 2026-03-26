# Meridian Solana Stablecoin Program

Multi-currency stablecoin implementation for Solana using Anchor framework.

## Features

- ✅ PDA-based mint authority (no admin keys)
- ✅ On-chain reserve tracking
- ✅ 1:1 reserve verification on mint
- ✅ Pro-rata reserve release on burn
- ✅ Reserve attestation for compliance
- ✅ Pausable for emergency response
- ✅ All operations emit events for audit trail

## Architecture

```
┌─────────────────────────────────────┐
│   Stablecoin Account (PDA)          │
│  - name, symbol, basket_id          │
│  - mint authority (PDA)             │
│  - total_supply, reserves           │
│  - pause state                      │
└─────────────────────────────────────┘
         │
         ├─ SPL Token Mint (owned by PDA)
         ├─ User Token Accounts
         └─ Reserve Tracking (on-chain)
```

## Instructions

### 1. Initialize

Creates a new stablecoin with PDA mint authority.

```rust
initialize(
    name: String,        // "EUR Meridian"
    symbol: String,      // "EURM"
    basket_id: String,   // "EUR_BASKET"
    basket_type: BasketType, // SingleCurrency
    decimals: u8,        // 9
)
```

### 2. Mint Tokens

Mints new tokens with reserve verification.

```rust
mint_tokens(
    amount: u64,         // Tokens to mint
    reserve_value: u64,  // USD cents (must be >= amount)
)
```

**Checks:**
- ✅ Not paused
- ✅ Reserve >= amount (1:1 backing)
- ✅ Overflow protection

### 3. Burn Tokens

Burns tokens and releases pro-rata reserves.

```rust
burn_tokens(
    amount: u64,         // Tokens to burn
)
```

**Calculation:**
```
reserve_to_release = (amount * total_reserve_value) / total_supply
```

### 4. Attest Reserves

Monthly reserve attestation for compliance.

```rust
attest_reserves(
    attested_reserve_value: u64,  // Total reserves (must meet min ratio)
)
```

### 5. Pause / Unpause

Emergency controls (authority only).

```rust
pause()
unpause()
```

## Security

### No Admin Rug Risk
- Mint authority is a PDA (program-derived address)
- No private key controls the mint
- Only the program can mint tokens

### Reserve Enforcement
- All mints check 1:1 reserve backing
- Burns release reserves pro-rata
- Overflow/underflow protection on all arithmetic

### Access Control
- Authority can: pause, unpause, attest
- Anyone can: burn their own tokens
- Only program PDA can: mint

### Audit Trail
- All operations emit events
- On-chain reserve tracking
- Attestation timestamps

## Building

### Prerequisites

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Solana
sh -c "$(curl -sSfL https://release.solana.com/stable/install)"

# Install Anchor
cargo install --git https://github.com/coral-xyz/anchor avm --locked --force
avm install latest
avm use latest
```

### Build

```bash
cd programs/meridian-solana
anchor build
```

### Test

```bash
anchor test
```

## Deployment

### Devnet

```bash
# Configure for devnet
solana config set --url https://api.devnet.solana.com

# Airdrop SOL for deployment
solana airdrop 2

# Deploy
anchor deploy --provider.cluster devnet
```

### Mainnet

```bash
# Configure for mainnet
solana config set --url https://api.mainnet-beta.solana.com

# Deploy (requires SOL for fees)
anchor deploy --provider.cluster mainnet
```

## Integration with Rust Backend

```rust
use anchor_client::solana_sdk::pubkey::Pubkey;
use anchor_client::Client;

// Initialize client
let client = Client::new(...);

// Find stablecoin PDA
let (stablecoin_pda, bump) = Pubkey::find_program_address(
    &[b"stablecoin", mint.as_ref()],
    &program_id,
);

// Mint tokens
let tx = program
    .request()
    .accounts(meridian_solana::accounts::MintTokens {
        stablecoin: stablecoin_pda,
        mint,
        recipient_token_account,
        authority,
        token_program: token::ID,
    })
    .args(meridian_solana::instruction::MintTokens {
        amount: 1000,
        reserve_value: 1000,
    })
    .send()?;
```

## Account Structure

```rust
pub struct Stablecoin {
    name: String,                  // "EUR Meridian"
    symbol: String,                // "EURM"
    basket_id: String,             // "EUR_BASKET"
    basket_type: BasketType,       // SingleCurrency | ImfSdr | CustomBasket
    mint: Pubkey,                  // SPL Token mint
    decimals: u8,                  // Usually 9 for Solana
    total_supply: u64,             // Total tokens minted
    total_reserve_value: u64,      // USD cents (2 decimals)
    min_reserve_ratio: u16,        // Basis points (10000 = 100%)
    is_paused: bool,               // Emergency pause flag
    last_attestation: i64,         // Unix timestamp
    authority: Pubkey,             // Admin authority
    bump: u8,                      // PDA bump seed
}
```

**Account Size:** ~200 bytes

## Events

All operations emit events for audit trail:

- `StablecoinInitialized` - New stablecoin created
- `TokensMinted` - Tokens minted with reserve
- `TokensBurned` - Tokens burned, reserves released
- `ReservesAttested` - Reserve attestation submitted
- `Paused` - Stablecoin paused
- `Unpaused` - Stablecoin unpaused

## Error Codes

- `NameTooLong` - Name > 32 chars
- `SymbolTooLong` - Symbol > 10 chars
- `BasketIdTooLong` - Basket ID > 64 chars
- `InsufficientReserveBacking` - Reserve < amount on mint
- `Overflow` - Arithmetic overflow
- `Underflow` - Arithmetic underflow
- `Paused` - Operation attempted while paused
- `Unauthorized` - Caller not authority
- `AttestationBelowMinimum` - Reserve attestation insufficient

## Testing

```bash
# Run all tests
anchor test

# Run specific test
cargo test --package meridian-solana test_initialize

# With logs
anchor test -- --nocapture
```

## License

MIT

## TODO

- [ ] Add freeze authority for compliance
- [ ] Add metadata (name, symbol) via Metaplex
- [ ] Add blacklist/whitelist accounts
- [ ] Integration with Pyth oracle for Solana-native prices
- [ ] Cross-chain bridge support (Wormhole)

