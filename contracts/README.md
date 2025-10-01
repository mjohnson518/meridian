# Meridian Smart Contracts

Solidity smart contracts for the Meridian multi-currency stablecoin platform.

## Contracts

### MeridianStablecoin.sol

ERC-20 stablecoin with multi-currency basket support.

**Features:**
- UUPS upgradeable proxy pattern
- Role-based access control (MINTER_ROLE, BURNER_ROLE, PAUSER_ROLE, UPGRADER_ROLE)
- Mint function with 1:1 reserve verification
- Burn function with pro-rata reserve release
- Blacklist/whitelist for compliance
- Reserve attestation tracking
- Emergency pause functionality
- Support for single-currency, IMF SDR, and custom baskets

### MeridianFactory.sol

Factory contract for deploying new stablecoin instances.

**Features:**
- Deploys stablecoins using UUPS proxy pattern
- Maintains registry of all deployed stablecoins
- Lookup by basket ID or address
- Implementation upgrade management

## Setup

### Prerequisites

- [Foundry](https://book.getfoundry.sh/getting-started/installation)
- Solidity 0.8.20+

### Install Dependencies

```bash
# Install OpenZeppelin contracts
forge install OpenZeppelin/openzeppelin-contracts@v5.0.0
forge install OpenZeppelin/openzeppelin-contracts-upgradeable@v5.0.0
```

### Build

```bash
forge build
```

## Testing

### Run All Tests

```bash
forge test
```

### Run with Verbosity

```bash
forge test -vvv
```

### Run Specific Test

```bash
forge test --match-test test_MintWithSufficientReserve
```

### Gas Report

```bash
forge test --gas-report
```

### Coverage

```bash
forge coverage
```

## Deployment

### Local Testnet

```bash
# Start local node
anvil

# Deploy (in another terminal)
forge script script/Deploy.s.sol --rpc-url http://localhost:8545 --broadcast
```

### Sepolia Testnet

```bash
# Set environment variables
export SEPOLIA_RPC_URL="https://sepolia.infura.io/v3/YOUR_KEY"
export PRIVATE_KEY="your_private_key"
export ETHERSCAN_API_KEY="your_etherscan_key"

# Deploy
forge script script/Deploy.s.sol \
  --rpc-url $SEPOLIA_RPC_URL \
  --private-key $PRIVATE_KEY \
  --broadcast \
  --verify
```

## Usage Examples

### Deploy a Single-Currency EUR Stablecoin

```solidity
// Deploy implementation
MeridianStablecoin implementation = new MeridianStablecoin();

// Deploy factory
MeridianFactory factory = new MeridianFactory(address(implementation));

// Deploy EUR stablecoin
address eurStablecoin = factory.deployStablecoin(
    "EUR Meridian",
    "EURM",
    "EUR_BASKET",
    MeridianStablecoin.BasketType.SingleCurrency,
    adminAddress,
    complianceOracleAddress
);
```

### Mint Tokens

```solidity
MeridianStablecoin stablecoin = MeridianStablecoin(eurStablecoin);

// Grant minter role
stablecoin.grantRole(stablecoin.MINTER_ROLE(), minterAddress);

// Create mint request
MeridianStablecoin.MintRequest memory request = MeridianStablecoin.MintRequest({
    recipient: userAddress,
    amount: 1000 ether,
    reserveValue: 1000 ether, // 1:1 backing
    deadline: block.timestamp + 1 hours,
    nonce: 0
});

// Mint tokens
stablecoin.mint(request);
```

### Burn Tokens

```solidity
// Burn 500 tokens
stablecoin.burn(500 ether);
```

## Security

### Access Control

The contract uses role-based access control with the following roles:

- `DEFAULT_ADMIN_ROLE`: Full admin access (blacklist, attestation, pause, upgrade)
- `MINTER_ROLE`: Can mint new tokens
- `BURNER_ROLE`: Can burn tokens (currently unused, users can burn their own)
- `PAUSER_ROLE`: Can pause/unpause contract
- `UPGRADER_ROLE`: Can upgrade contract implementation

### Reserve Backing

- All mints require at least 1:1 reserve backing (configurable)
- Burns release pro-rata share of reserves
- Monthly reserve attestation required
- Reserves tracked on-chain for transparency

### Compliance

- Blacklist functionality for sanctioned addresses
- Whitelist removal for reinstating addresses
- All transfers blocked for blacklisted addresses

### Emergency Functions

- Pause all transfers, mints, and burns
- Unpause to resume operations
- Only PAUSER_ROLE can trigger

### Upgradeability

- UUPS proxy pattern for upgradeability
- Only UPGRADER_ROLE can authorize upgrades
- Each stablecoin upgrades independently

## Architecture

```
┌─────────────────────────────────────┐
│      MeridianFactory                │
│  (Deploys stablecoin proxies)       │
└──────────────┬──────────────────────┘
               │ deploys
               ▼
┌─────────────────────────────────────┐
│    ERC1967Proxy                     │
│  (Upgradeable proxy)                │
└──────────────┬──────────────────────┘
               │ delegates to
               ▼
┌─────────────────────────────────────┐
│  MeridianStablecoin                 │
│  (Implementation contract)          │
│                                     │
│  - ERC20Upgradeable                 │
│  - AccessControlUpgradeable         │
│  - PausableUpgradeable              │
│  - UUPSUpgradeable                  │
└─────────────────────────────────────┘
```

## Test Coverage

### MeridianStablecoin Tests (30 tests)

- ✅ Initialization and configuration
- ✅ Role-based access control
- ✅ Minting with reserve verification
- ✅ Burning with pro-rata calculation
- ✅ Blacklist/whitelist functionality
- ✅ Pause/unpause mechanics
- ✅ Reserve attestation
- ✅ Reserve ratio calculations
- ✅ Nonce and replay protection
- ✅ Transfer restrictions

### MeridianFactory Tests (15 tests)

- ✅ Factory initialization
- ✅ Stablecoin deployment
- ✅ Multiple deployments
- ✅ Duplicate basket ID prevention
- ✅ Registry management
- ✅ Basket ID lookups
- ✅ Implementation updates
- ✅ Integration tests

## Gas Usage

Typical gas costs (approximate):

- Deploy Factory: ~2,000,000 gas
- Deploy Stablecoin: ~1,500,000 gas
- Mint: ~150,000 gas
- Burn: ~100,000 gas
- Transfer: ~80,000 gas
- Blacklist: ~50,000 gas

## License

MIT License

