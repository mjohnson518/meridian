# Meridian Frontend

Professional multi-currency stablecoin platform frontend built with Next.js 14.

## Design Philosophy

The Meridian frontend implements a minimalist, terminal-aesthetic design system inspired by [Sacred (SRCL)](https://github.com/internet-development/www-sacred) - an open-source React component and style repository for building applications with terminal aesthetics. We've adapted Sacred's principles for financial data display with high-contrast interfaces, monospace typography, and dense information layouts.

## Features

- **Minimalist Design System**: Terminal-aesthetic interface optimized for financial data
- **Public Reserve Dashboard**: Real-time view of reserves, bond holdings, and attestation status  
- **Web3 Integration**: Connected to Sepolia testnet with deployed contracts
- **Responsive Layout**: Mobile-first design with 8px grid system
- **Financial Components**: Custom components for currency display, reserve ratios, and metrics

## Tech Stack

- **Framework**: Next.js 14 with App Router
- **Styling**: Tailwind CSS with Sacred design tokens
- **Typography**: Inter (sans) and IBM Plex Mono (monospace)
- **Web3**: wagmi v2 + viem
- **State**: Zustand
- **Data Fetching**: TanStack Query
- **Validation**: Zod

## Getting Started

### Prerequisites

- Node.js 20+
- npm or yarn
- Ethereum wallet (MetaMask recommended)

### Installation

```bash
# Install dependencies
npm install

# Copy environment variables
cp .env.example .env.local

# Update .env.local with your values:
# - NEXT_PUBLIC_API_URL (backend API endpoint)
# - NEXT_PUBLIC_SEPOLIA_RPC_URL (Sepolia RPC)
# - NEXT_PUBLIC_WALLETCONNECT_PROJECT_ID (from cloud.walletconnect.com)
```

### Development

```bash
# Start development server
npm run dev

# Open http://localhost:3000
```

### Production Build

```bash
# Create production build
npm run build

# Start production server
npm start
```

## Project Structure

```
meridian-frontend/
├── app/                      # Next.js app router pages
│   ├── layout.tsx           # Root layout with header/footer
│   ├── page.tsx            # Landing page
│   └── reserves/           # Public reserve dashboard
│       └── page.tsx
├── components/
│   ├── sacred/             # Base Sacred design components
│   │   ├── Button.tsx
│   │   ├── Card.tsx
│   │   ├── Grid.tsx
│   │   ├── Table.tsx
│   │   └── Typography.tsx
│   └── meridian/          # Financial-specific components
│       ├── MetricDisplay.tsx
│       └── ReserveRatioBar.tsx
├── lib/
│   ├── api/              # Backend API client
│   │   └── client.ts
│   ├── web3/             # Web3 configuration
│   │   └── config.ts
│   └── utils.ts          # Utility functions
└── styles/
    ├── globals.css       # Global styles
    └── sacred-tokens.css # Design system tokens
```

## Design Principles

The frontend follows the minimalist philosophy inspired by [Sacred](https://github.com/internet-development/www-sacred):

1. **Monospace for Financial Data**: All numbers use IBM Plex Mono with tabular-nums
2. **High Contrast**: Black/white/gray palette for maximum readability
3. **8px Grid System**: Consistent spacing using multiples of 8px
4. **Dense Information**: Tables and data displays optimized for information density
5. **Minimal Animation**: Only opacity and transform transitions
6. **Terminal Aesthetic**: Technical sections use console-style interfaces

## Key Components

### MonoDisplay
Displays financial values with proper decimal alignment and currency formatting:
```tsx
<MonoDisplay value={1234.56} currency="EUR" precision={2} />
```

### ReserveRatioBar
Visual indicator for reserve health with threshold markers:
```tsx
<ReserveRatioBar ratio={100.42} threshold={100} />
```

### MetricDisplay
Comprehensive metric display with trends and status indicators:
```tsx
<MetricDisplay 
  label="Total Reserves"
  value={10042250}
  format="currency"
  trend={0.42}
  status="healthy"
/>
```

## Contract Addresses (Sepolia)

- **Factory**: `0xbe35619896F963dD0Eac90A93A135c53547499C9`
- **EUR Stablecoin**: `0xDcd19e3b07AB23F6771aDda3ab76d7e6823B5D2f`

## API Integration

The frontend connects to the Meridian backend API for:
- Reserve data and attestation status
- Basket management
- Oracle price feeds
- Transaction history

Currently using mock data for development. Update `NEXT_PUBLIC_API_URL` in `.env.local` to connect to the backend.

## Next Steps

- [ ] Institutional portal with KYC/AML flow
- [ ] x402 agent payment integration
- [ ] Developer portal with API documentation
- [ ] Real-time WebSocket updates
- [ ] Hardware wallet integration
- [ ] Advanced charting for reserve history

## License

MIT