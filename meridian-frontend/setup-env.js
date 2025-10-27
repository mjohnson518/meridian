#!/usr/bin/env node

const fs = require('fs');
const path = require('path');

const envExample = `# API Configuration
NEXT_PUBLIC_API_URL=http://localhost:8080/api/v1

# Sepolia RPC Configuration
NEXT_PUBLIC_SEPOLIA_RPC_URL=https://rpc.sepolia.org

# WalletConnect Project ID (get one from https://cloud.walletconnect.com)
NEXT_PUBLIC_WALLETCONNECT_PROJECT_ID=your_project_id_here

# Contract Addresses (Sepolia)
NEXT_PUBLIC_FACTORY_ADDRESS=0xbe35619896F963dD0Eac90A93A135c53547499C9
NEXT_PUBLIC_EUR_STABLECOIN_ADDRESS=0xDcd19e3b07AB23F6771aDda3ab76d7e6823B5D2f
`;

const envPath = path.join(__dirname, '.env.local');
const envExamplePath = path.join(__dirname, '.env.example');

// Create .env.example if it doesn't exist
if (!fs.existsSync(envExamplePath)) {
  fs.writeFileSync(envExamplePath, envExample);
  console.log('✅ Created .env.example');
}

// Create .env.local if it doesn't exist
if (!fs.existsSync(envPath)) {
  fs.writeFileSync(envPath, envExample);
  console.log('✅ Created .env.local');
  console.log('');
  console.log('⚠️  Please update .env.local with your configuration:');
  console.log('   - Get a WalletConnect Project ID from https://cloud.walletconnect.com');
  console.log('   - Update API_URL if your backend is running on a different port');
  console.log('   - Update RPC_URL if you have a custom Sepolia RPC endpoint');
} else {
  console.log('✅ .env.local already exists');
}

console.log('');
console.log('📝 Environment setup complete!');
console.log('   Run "npm run dev" to start the development server.');
