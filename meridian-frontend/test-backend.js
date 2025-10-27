#!/usr/bin/env node

// Test script to verify backend connectivity

const http = require('http');

const endpoints = [
  { path: '/api/baskets', name: 'Baskets' },
  { path: '/api/v1/baskets', name: 'Baskets (v1)' },
  { path: '/api/reserves/EUR', name: 'EUR Reserves' },
  { path: '/api/v1/reserves/EUR', name: 'EUR Reserves (v1)' },
  { path: '/api/oracle/EUR-USD', name: 'EUR-USD Price' },
  { path: '/api/v1/oracle/EUR-USD', name: 'EUR-USD Price (v1)' },
  { path: '/health', name: 'Health Check' },
  { path: '/', name: 'Root' },
];

console.log('🔍 Testing Meridian Backend Connection...\n');
console.log('Host: http://localhost:8080');
console.log('----------------------------------------\n');

let foundAny = false;

async function testEndpoint(endpoint) {
  return new Promise((resolve) => {
    const options = {
      hostname: 'localhost',
      port: 8080,
      path: endpoint.path,
      method: 'GET',
      timeout: 2000,
    };

    const req = http.request(options, (res) => {
      if (res.statusCode) {
        console.log(`✅ ${endpoint.name}: ${res.statusCode} ${res.statusMessage}`);
        foundAny = true;
      } else {
        console.log(`❌ ${endpoint.name}: No response`);
      }
      resolve();
    });

    req.on('error', (err) => {
      if (err.code === 'ECONNREFUSED') {
        console.log(`❌ ${endpoint.name}: Connection refused`);
      } else {
        console.log(`❌ ${endpoint.name}: ${err.message}`);
      }
      resolve();
    });

    req.on('timeout', () => {
      console.log(`⏱️  ${endpoint.name}: Timeout`);
      req.destroy();
      resolve();
    });

    req.end();
  });
}

async function testAll() {
  for (const endpoint of endpoints) {
    await testEndpoint(endpoint);
  }

  console.log('\n----------------------------------------');
  
  if (!foundAny) {
    console.log('\n❌ Backend API is not running or not accessible on port 8080.\n');
    console.log('To start the backend:\n');
    console.log('1. Open a new terminal');
    console.log('2. Navigate to the Meridian project root');
    console.log('3. Run the following commands:\n');
    console.log('   export DATABASE_URL="postgresql://marcjohnson@localhost/meridian_dev"');
    console.log('   export ETHEREUM_RPC_URL="https://ethereum-rpc.publicnode.com"');
    console.log('   cargo run --bin meridian-api\n');
    console.log('The frontend will use mock data until the backend is available.');
  } else {
    console.log('\n✅ Backend API is accessible!\n');
    console.log('Configure the frontend to use the backend:');
    console.log('   cd meridian-frontend');
    console.log('   node configure-backend.js');
  }
}

testAll();
