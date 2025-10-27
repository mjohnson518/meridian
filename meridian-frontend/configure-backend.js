#!/usr/bin/env node

// Script to configure backend API URLs for Meridian frontend

const fs = require('fs');
const path = require('path');
const readline = require('readline');

const rl = readline.createInterface({
  input: process.stdin,
  output: process.stdout
});

const envPath = path.join(__dirname, '.env.local');

function question(query) {
  return new Promise((resolve) => {
    rl.question(query, resolve);
  });
}

async function main() {
  console.log('\n🔧 Meridian Frontend - Backend Configuration\n');
  console.log('This script will help you configure the backend API connection.\n');
  
  // Get current env vars or defaults
  let currentEnv = {};
  if (fs.existsSync(envPath)) {
    const envContent = fs.readFileSync(envPath, 'utf8');
    envContent.split('\n').forEach(line => {
      const [key, value] = line.split('=');
      if (key && value) {
        currentEnv[key.trim()] = value.trim();
      }
    });
  }
  
  // Ask for backend URLs
  const apiUrl = await question(
    `Backend API URL (current: ${currentEnv.NEXT_PUBLIC_API_URL || 'http://localhost:8080/api/v1'}): `
  ) || currentEnv.NEXT_PUBLIC_API_URL || 'http://localhost:8080/api/v1';
  
  const wsUrl = await question(
    `WebSocket URL (current: ${currentEnv.NEXT_PUBLIC_WS_URL || 'ws://localhost:8080/ws'}): `
  ) || currentEnv.NEXT_PUBLIC_WS_URL || 'ws://localhost:8080/ws';
  
  // Update env vars
  currentEnv.NEXT_PUBLIC_API_URL = apiUrl;
  currentEnv.NEXT_PUBLIC_WS_URL = wsUrl;
  
  // Keep other env vars with defaults if not set
  currentEnv.NEXT_PUBLIC_SEPOLIA_RPC_URL = currentEnv.NEXT_PUBLIC_SEPOLIA_RPC_URL || 'https://ethereum-sepolia.publicnode.com';
  currentEnv.NEXT_PUBLIC_WALLETCONNECT_PROJECT_ID = currentEnv.NEXT_PUBLIC_WALLETCONNECT_PROJECT_ID || 'your_project_id_here';
  currentEnv.NEXT_PUBLIC_FACTORY_ADDRESS = currentEnv.NEXT_PUBLIC_FACTORY_ADDRESS || '0x51bfa9b86ED48bA42a5B8a3A63c0729fD19f27a8';
  currentEnv.NEXT_PUBLIC_EUR_STABLECOIN_ADDRESS = currentEnv.NEXT_PUBLIC_EUR_STABLECOIN_ADDRESS || '0xdc1E0C3e3FE9A3A1f05CbaFC1c1Fb8D82EAc4FF8';
  
  // Write .env.local file
  const envContent = Object.entries(currentEnv)
    .map(([key, value]) => `${key}=${value}`)
    .join('\n');
  
  fs.writeFileSync(envPath, envContent + '\n');
  
  console.log('\n✅ Configuration saved to .env.local\n');
  console.log('Backend API URL:', apiUrl);
  console.log('WebSocket URL:', wsUrl);
  console.log('\nRestart the development server for changes to take effect:\n');
  console.log('  npm run dev\n');
  
  rl.close();
}

main().catch(err => {
  console.error('Error:', err);
  process.exit(1);
});
