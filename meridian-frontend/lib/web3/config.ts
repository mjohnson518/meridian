import { createConfig, http } from 'wagmi'
import { sepolia } from 'wagmi/chains'
import { injected, walletConnect } from 'wagmi/connectors'

// Contract addresses on Sepolia
export const CONTRACT_ADDRESSES = {
  FACTORY: '0xbe35619896F963dD0Eac90A93A135c53547499C9' as const,
  EUR_STABLECOIN: '0xDcd19e3b07AB23F6771aDda3ab76d7e6823B5D2f' as const,
} as const

// Contract ABIs (simplified for now, will expand later)
export const FACTORY_ABI = [
  {
    inputs: [],
    name: 'getDeployedStablecoins',
    outputs: [{ internalType: 'address[]', name: '', type: 'address[]' }],
    stateMutability: 'view',
    type: 'function',
  },
  {
    inputs: [
      { internalType: 'string', name: 'name', type: 'string' },
      { internalType: 'string', name: 'symbol', type: 'string' },
      { internalType: 'address', name: 'admin', type: 'address' },
      { internalType: 'address', name: 'complianceModule', type: 'address' },
    ],
    name: 'deployStablecoin',
    outputs: [{ internalType: 'address', name: '', type: 'address' }],
    stateMutability: 'nonpayable',
    type: 'function',
  },
] as const

export const STABLECOIN_ABI = [
  {
    inputs: [],
    name: 'totalSupply',
    outputs: [{ internalType: 'uint256', name: '', type: 'uint256' }],
    stateMutability: 'view',
    type: 'function',
  },
  {
    inputs: [],
    name: 'getReserveRatio',
    outputs: [{ internalType: 'uint256', name: '', type: 'uint256' }],
    stateMutability: 'view',
    type: 'function',
  },
  {
    inputs: [],
    name: 'lastAttestationTimestamp',
    outputs: [{ internalType: 'uint256', name: '', type: 'uint256' }],
    stateMutability: 'view',
    type: 'function',
  },
  {
    inputs: [{ internalType: 'address', name: 'account', type: 'address' }],
    name: 'balanceOf',
    outputs: [{ internalType: 'uint256', name: '', type: 'uint256' }],
    stateMutability: 'view',
    type: 'function',
  },
  {
    inputs: [],
    name: 'name',
    outputs: [{ internalType: 'string', name: '', type: 'string' }],
    stateMutability: 'view',
    type: 'function',
  },
  {
    inputs: [],
    name: 'symbol',
    outputs: [{ internalType: 'string', name: '', type: 'string' }],
    stateMutability: 'view',
    type: 'function',
  },
  {
    inputs: [],
    name: 'decimals',
    outputs: [{ internalType: 'uint8', name: '', type: 'uint8' }],
    stateMutability: 'view',
    type: 'function',
  },
  {
    inputs: [
      { internalType: 'address', name: 'to', type: 'address' },
      { internalType: 'uint256', name: 'amount', type: 'uint256' },
    ],
    name: 'mint',
    outputs: [],
    stateMutability: 'nonpayable',
    type: 'function',
  },
  {
    inputs: [{ internalType: 'uint256', name: 'amount', type: 'uint256' }],
    name: 'burn',
    outputs: [],
    stateMutability: 'nonpayable',
    type: 'function',
  },
  {
    inputs: [],
    name: 'paused',
    outputs: [{ internalType: 'bool', name: '', type: 'bool' }],
    stateMutability: 'view',
    type: 'function',
  },
] as const

// WalletConnect project ID - get from https://cloud.walletconnect.com
// SECURITY: Validate project ID in production to prevent misconfiguration
const getWalletConnectProjectId = (): string => {
  const projectId = process.env.NEXT_PUBLIC_WALLETCONNECT_PROJECT_ID;
  const isProduction = process.env.NODE_ENV === 'production';

  if (!projectId || projectId === 'YOUR_PROJECT_ID') {
    if (isProduction) {
      console.error('[Web3] SECURITY ERROR: NEXT_PUBLIC_WALLETCONNECT_PROJECT_ID not configured for production');
      // Return empty string to disable WalletConnect in production without crashing
      return '';
    }
    // Development fallback - WalletConnect may not work but app runs
    console.warn('[Web3] WalletConnect project ID not configured - wallet connection may fail');
    return 'YOUR_PROJECT_ID';
  }
  return projectId;
};

const walletConnectProjectId = getWalletConnectProjectId();

// Wagmi config
export const config = createConfig({
  chains: [sepolia],
  connectors: [
    injected(),
    walletConnect({ projectId: walletConnectProjectId }),
  ],
  transports: {
    [sepolia.id]: http(process.env.NEXT_PUBLIC_SEPOLIA_RPC_URL || 'https://rpc.sepolia.org'),
  },
})

// Helper function to get block explorer URL
export function getBlockExplorerUrl(txHash: string): string {
  return `https://sepolia.etherscan.io/tx/${txHash}`
}

export function getAddressExplorerUrl(address: string): string {
  return `https://sepolia.etherscan.io/address/${address}`
}
