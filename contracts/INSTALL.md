# Meridian Contracts - Installation & Setup

## Prerequisites

### 1. Install Foundry

```bash
curl -L https://foundry.paradigm.xyz | bash
foundryup
```

Verify installation:
```bash
forge --version
# Should output: forge 0.2.0 or higher
```

### 2. Install Dependencies

```bash
cd contracts

# Install OpenZeppelin contracts
forge install OpenZeppelin/openzeppelin-contracts@v5.0.0
forge install OpenZeppelin/openzeppelin-contracts-upgradeable@v5.0.0

# Update dependencies
forge update
```

## Building

```bash
# Build all contracts
forge build

# Build with optimizations
forge build --optimize --optimizer-runs 200
```

## Testing

### Run All Tests

```bash
forge test
```

### Run with Detailed Output

```bash
# Show test traces
forge test -vv

# Show very detailed traces (includes call traces)
forge test -vvv

# Show stack traces and setup
forge test -vvvv
```

### Run Specific Tests

```bash
# Test specific contract
forge test --match-contract MeridianStablecoinTest

# Test specific function
forge test --match-test test_MintWithSufficientReserve

# Run only failed tests
forge test --failed
```

### Gas Reporting

```bash
forge test --gas-report
```

### Coverage

```bash
# Generate coverage report
forge coverage

# Generate detailed lcov report
forge coverage --report lcov
```

## Deployment

### Environment Setup

Create a `.env` file:

```bash
# Copy example
cp .env.example .env

# Edit with your values
nano .env
```

Required variables:
```bash
SEPOLIA_RPC_URL=https://sepolia.infura.io/v3/YOUR_KEY
MAINNET_RPC_URL=https://mainnet.infura.io/v3/YOUR_KEY
PRIVATE_KEY=your_private_key_without_0x
ETHERSCAN_API_KEY=your_etherscan_key
```

### Deploy to Local Network

```bash
# Terminal 1: Start local node
anvil

# Terminal 2: Deploy contracts
forge script script/Deploy.s.sol \
  --rpc-url http://localhost:8545 \
  --broadcast
```

### Deploy to Sepolia Testnet

```bash
# Load environment variables
source .env

# Deploy
forge script script/Deploy.s.sol \
  --rpc-url $SEPOLIA_RPC_URL \
  --private-key $PRIVATE_KEY \
  --broadcast \
  --verify \
  --etherscan-api-key $ETHERSCAN_API_KEY
```

### Deploy EUR Stablecoin

After deploying the factory:

```bash
# Set factory address
export FACTORY_ADDRESS=0x...
export ADMIN_ADDRESS=0x...
export COMPLIANCE_ORACLE_ADDRESS=0x...

# Deploy EUR stablecoin
forge script script/Deploy.s.sol:DeployEURScript \
  --rpc-url $SEPOLIA_RPC_URL \
  --private-key $PRIVATE_KEY \
  --broadcast
```

## Verification

### Verify on Etherscan

```bash
forge verify-contract \
  --chain-id 11155111 \
  --etherscan-api-key $ETHERSCAN_API_KEY \
  --constructor-args $(cast abi-encode "constructor(address)" $IMPLEMENTATION_ADDRESS) \
  $FACTORY_ADDRESS \
  src/MeridianFactory.sol:MeridianFactory
```

## Interacting with Contracts

### Using Cast (Foundry CLI)

```bash
# Get token name
cast call $STABLECOIN_ADDRESS "name()" --rpc-url $SEPOLIA_RPC_URL

# Get total supply
cast call $STABLECOIN_ADDRESS "totalSupply()" --rpc-url $SEPOLIA_RPC_URL

# Get reserve ratio
cast call $STABLECOIN_ADDRESS "getReserveRatio()" --rpc-url $SEPOLIA_RPC_URL

# Grant minter role
cast send $STABLECOIN_ADDRESS \
  "grantRole(bytes32,address)" \
  $(cast keccak "MINTER_ROLE") \
  $MINTER_ADDRESS \
  --private-key $PRIVATE_KEY \
  --rpc-url $SEPOLIA_RPC_URL
```

## Security Analysis

### Run Slither

```bash
# Install Slither
pip3 install slither-analyzer

# Run analysis
slither contracts/src/MeridianStablecoin.sol
```

### Run Mythril

```bash
# Install Mythril
pip3 install mythril

# Analyze
myth analyze contracts/src/MeridianStablecoin.sol
```

## Troubleshooting

### "Compiler version not found"

```bash
# Install specific solc version
forge install --solc-version 0.8.20
```

### "Out of gas" in tests

Increase gas limit in foundry.toml:
```toml
gas_limit = "18446744073709551615"
```

### Dependencies not found

```bash
# Clean and reinstall
rm -rf lib/
forge install OpenZeppelin/openzeppelin-contracts@v5.0.0
forge install OpenZeppelin/openzeppelin-contracts-upgradeable@v5.0.0
```

## Useful Commands

```bash
# Format code
forge fmt

# Clean build artifacts
forge clean

# Update dependencies
forge update

# Create new test
forge create --template test MyTest

# Flatten contract for verification
forge flatten src/MeridianStablecoin.sol

# Check contract size
forge build --sizes
```

## Next Steps

After deployment:

1. **Grant Roles**: Grant MINTER_ROLE to authorized minters
2. **Configure Compliance**: Set up compliance oracle
3. **Attest Reserves**: Submit initial reserve attestation
4. **Test Minting**: Mint test tokens
5. **Verify Contracts**: Verify on Etherscan

## Documentation

- [Foundry Book](https://book.getfoundry.sh/)
- [OpenZeppelin Upgradeable Contracts](https://docs.openzeppelin.com/contracts/5.x/upgradeable)
- [UUPS Proxy Pattern](https://docs.openzeppelin.com/contracts/5.x/api/proxy#UUPSUpgradeable)

