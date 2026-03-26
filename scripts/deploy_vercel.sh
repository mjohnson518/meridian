#!/bin/bash

# Vercel Deployment Script for Meridian Frontend
# This script automates the deployment process to Vercel

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}╔════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║   MERIDIAN VERCEL DEPLOYMENT               ║${NC}"
echo -e "${BLUE}╔════════════════════════════════════════════╗${NC}"
echo ""

# Check if Vercel CLI is installed
if ! command -v vercel &> /dev/null; then
    echo -e "${RED}✗ Vercel CLI not found${NC}"
    echo "Install it with: npm i -g vercel"
    exit 1
fi

echo -e "${GREEN}✓ Vercel CLI found${NC}"

# Check if logged in
if ! vercel whoami &> /dev/null; then
    echo -e "${YELLOW}! Not logged in to Vercel${NC}"
    echo "Logging in..."
    vercel login
fi

echo -e "${GREEN}✓ Logged in to Vercel${NC}"

# Navigate to frontend directory
cd meridian-frontend

# Check if project is linked
if [ ! -f ".vercel/project.json" ]; then
    echo -e "${YELLOW}! Project not linked to Vercel${NC}"
    echo "Linking project..."
    vercel link
fi

# Get Railway API URL
echo ""
echo -e "${YELLOW}Enter your Railway API URL:${NC}"
echo "(e.g., https://meridian-api-production.up.railway.app)"
read -r API_URL

if [ -z "$API_URL" ]; then
    echo -e "${RED}✗ API URL is required${NC}"
    exit 1
fi

# Get Alchemy API Key
echo ""
echo -e "${YELLOW}Enter your Alchemy API Key:${NC}"
echo "(Get one from: https://www.alchemy.com/)"
read -r ALCHEMY_KEY

if [ -z "$ALCHEMY_KEY" ]; then
    echo -e "${RED}✗ Alchemy API Key is required${NC}"
    exit 1
fi

# Set environment variables
echo ""
echo -e "${YELLOW}Setting environment variables...${NC}"

vercel env add NEXT_PUBLIC_API_URL production <<< "$API_URL"
vercel env add NEXT_PUBLIC_SEPOLIA_RPC_URL production <<< "https://eth-sepolia.g.alchemy.com/v2/$ALCHEMY_KEY"
vercel env add NEXT_PUBLIC_ETHEREUM_RPC_URL production <<< "https://eth-mainnet.g.alchemy.com/v2/$ALCHEMY_KEY"
vercel env add NEXT_PUBLIC_CHAIN_ID production <<< "1"
vercel env add NEXT_PUBLIC_TESTNET_CHAIN_ID production <<< "11155111"

echo -e "${GREEN}✓ Environment variables set${NC}"

# Install dependencies
echo ""
echo -e "${YELLOW}Installing dependencies...${NC}"
npm install
echo -e "${GREEN}✓ Dependencies installed${NC}"

# Build locally first to catch errors
echo ""
echo -e "${YELLOW}Testing local build...${NC}"
npm run build
echo -e "${GREEN}✓ Local build successful${NC}"

# Deploy to Vercel
echo ""
echo -e "${YELLOW}Deploy to production? (y/n)${NC}"
read -r response

if [ "$response" != "y" ]; then
    echo -e "${YELLOW}Deployment cancelled${NC}"
    exit 0
fi

echo -e "${YELLOW}Deploying to Vercel...${NC}"
vercel --prod

echo -e "\n${GREEN}╔════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║   DEPLOYMENT SUCCESSFUL                    ║${NC}"
echo -e "${GREEN}╔════════════════════════════════════════════╗${NC}"
echo ""

# Get deployment URL
DEPLOYMENT_URL=$(vercel ls --json | grep -o '"url":"[^"]*"' | head -1 | cut -d'"' -f4)

if [ -n "$DEPLOYMENT_URL" ]; then
    echo -e "Your frontend is live at: ${BLUE}https://$DEPLOYMENT_URL${NC}"
    echo ""
    echo "Next steps:"
    echo "  1. Update backend CORS_ALLOWED_ORIGINS to include: https://$DEPLOYMENT_URL"
    echo "  2. Run production test suite: ../scripts/test_production.sh"
    echo "  3. Configure custom domain in Vercel dashboard"
fi

echo ""
echo -e "${GREEN}✓ Deployment complete!${NC}"

