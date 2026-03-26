#!/bin/bash

# Railway Deployment Script for Meridian Backend
# This script automates the deployment process to Railway

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}╔════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║   MERIDIAN RAILWAY DEPLOYMENT              ║${NC}"
echo -e "${BLUE}╔════════════════════════════════════════════╗${NC}"
echo ""

# Check if Railway CLI is installed
if ! command -v railway &> /dev/null; then
    echo -e "${RED}✗ Railway CLI not found${NC}"
    echo "Install it with: npm i -g @railway/cli"
    exit 1
fi

echo -e "${GREEN}✓ Railway CLI found${NC}"

# Check if logged in
if ! railway whoami &> /dev/null; then
    echo -e "${YELLOW}! Not logged in to Railway${NC}"
    echo "Logging in..."
    railway login
fi

echo -e "${GREEN}✓ Logged in to Railway${NC}"

# Check if project is linked
if [ ! -f "railway.toml" ]; then
    echo -e "${YELLOW}! railway.toml not found${NC}"
    echo "Have you initialized Railway? (y/n)"
    read -r response
    if [ "$response" = "y" ]; then
        echo "Initializing Railway project..."
        railway init
    else
        exit 1
    fi
fi

# Generate JWT secret if not exists
if ! railway variables get JWT_SECRET &> /dev/null; then
    echo -e "${YELLOW}! Generating JWT_SECRET${NC}"
    JWT_SECRET=$(openssl rand -base64 64)
    railway variables set JWT_SECRET="$JWT_SECRET"
    echo -e "${GREEN}✓ JWT_SECRET set${NC}"
fi

# Check required environment variables
echo -e "\n${YELLOW}Checking environment variables...${NC}"

REQUIRED_VARS=("DATABASE_URL" "JWT_SECRET" "PORT")
MISSING_VARS=()

for var in "${REQUIRED_VARS[@]}"; do
    if railway variables get "$var" &> /dev/null; then
        echo -e "${GREEN}✓ $var is set${NC}"
    else
        echo -e "${RED}✗ $var is missing${NC}"
        MISSING_VARS+=("$var")
    fi
done

if [ ${#MISSING_VARS[@]} -gt 0 ]; then
    echo -e "\n${RED}Missing required variables: ${MISSING_VARS[*]}${NC}"
    echo "Set them with: railway variables set VARIABLE_NAME=value"
    exit 1
fi

# Run database migrations
echo -e "\n${YELLOW}Running database migrations...${NC}"
echo "This will connect to Railway database and run migrations."
echo "Continue? (y/n)"
read -r response

if [ "$response" = "y" ]; then
    railway run bash -c "cd crates/db && sqlx migrate run"
    echo -e "${GREEN}✓ Migrations completed${NC}"
else
    echo -e "${YELLOW}⚠ Skipping migrations${NC}"
fi

# Build locally first to catch errors
echo -e "\n${YELLOW}Testing local build...${NC}"
cargo build --release --bin meridian-api
echo -e "${GREEN}✓ Local build successful${NC}"

# Deploy to Railway
echo -e "\n${YELLOW}Deploying to Railway...${NC}"
railway up

echo -e "\n${GREEN}✓ Deployment initiated${NC}"
echo ""
echo "Monitor deployment with: railway logs"
echo "Check status with: railway status"
echo ""

# Wait for deployment
echo "Waiting for deployment to complete..."
sleep 10

# Get deployment URL
DEPLOYMENT_URL=$(railway status | grep "URL" | awk '{print $2}')

if [ -n "$DEPLOYMENT_URL" ]; then
    echo -e "\n${GREEN}╔════════════════════════════════════════════╗${NC}"
    echo -e "${GREEN}║   DEPLOYMENT SUCCESSFUL                    ║${NC}"
    echo -e "${GREEN}╔════════════════════════════════════════════╗${NC}"
    echo ""
    echo -e "Your API is live at: ${BLUE}$DEPLOYMENT_URL${NC}"
    echo ""
    echo "Test with:"
    echo "  curl $DEPLOYMENT_URL/api/v1/health"
    echo ""
    echo "Next steps:"
    echo "  1. Update frontend NEXT_PUBLIC_API_URL to: $DEPLOYMENT_URL"
    echo "  2. Run production test suite: ./scripts/test_production.sh"
    echo "  3. Deploy frontend to Vercel"
else
    echo -e "${YELLOW}⚠ Could not retrieve deployment URL${NC}"
    echo "Check deployment status with: railway status"
fi

# Open logs
echo ""
echo "Open deployment logs? (y/n)"
read -r response
if [ "$response" = "y" ]; then
    railway logs
fi

