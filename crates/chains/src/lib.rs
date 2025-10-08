//! # Meridian Multi-Chain Configuration
//!
//! Chain registry and configuration for deploying stablecoins across
//! Ethereum, Solana, Base, Arbitrum, Optimism, and other supported chains.

use ethers::types::Address;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use thiserror::Error;

/// Supported blockchain networks
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Chain {
    // Ethereum
    Ethereum,
    EthereumSepolia,

    // Base (Coinbase L2)
    Base,
    BaseSepolia,

    // Arbitrum
    Arbitrum,
    ArbitrumSepolia,

    // Optimism
    Optimism,
    OptimismSepolia,

    // Arc
    Arc,
    ArcTestnet,

    // Tempo
    Tempo,
    TempoTestnet,

    // Solana
    Solana,
    SolanaDevnet,
}

/// Solana program ID type (placeholder until solana-sdk is added)
pub type SolanaPubkey = String;

/// Chain configuration with RPC, explorer, and deployment info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainConfig {
    /// Unique chain identifier (EVM chain ID or Solana cluster)
    pub chain_id: u64,
    /// RPC endpoint URL
    pub rpc_url: String,
    /// Block explorer base URL
    pub explorer_url: String,
    /// Native token symbol (ETH, SOL, etc.)
    pub native_token: String,
    /// Deployed MeridianFactory contract address (EVM only)
    pub contract_address: Option<Address>,
    /// Deployed Solana program ID (Solana only)
    pub program_id: Option<SolanaPubkey>,
}

/// Errors for chain operations
#[derive(Error, Debug)]
pub enum ChainError {
    #[error("Chain not supported: {0}")]
    UnsupportedChain(String),

    #[error("Contract not deployed on {0:?}")]
    ContractNotDeployed(Chain),

    #[error("Invalid configuration for {0:?}")]
    InvalidConfiguration(Chain),

    #[error("RPC URL not configured for {0:?}")]
    RpcUrlNotConfigured(Chain),
}

impl Chain {
    /// Gets the chain configuration
    ///
    /// Returns RPC URL, explorer URL, and deployment addresses.
    /// Contract addresses may be None if not yet deployed.
    pub fn config(&self) -> ChainConfig {
        match self {
            // ============ Ethereum ============
            Chain::Ethereum => ChainConfig {
                chain_id: 1,
                rpc_url: std::env::var("ETHEREUM_RPC_URL").unwrap_or_else(|_| {
                    "https://eth-mainnet.g.alchemy.com/v2/YOUR_KEY".to_string()
                }),
                explorer_url: "https://etherscan.io".to_string(),
                native_token: "ETH".to_string(),
                contract_address: std::env::var("ETHEREUM_FACTORY_ADDRESS")
                    .ok()
                    .and_then(|addr| Address::from_str(&addr).ok()),
                program_id: None,
            },

            Chain::EthereumSepolia => ChainConfig {
                chain_id: 11155111,
                rpc_url: std::env::var("SEPOLIA_RPC_URL").unwrap_or_else(|_| {
                    "https://eth-sepolia.g.alchemy.com/v2/YOUR_KEY".to_string()
                }),
                explorer_url: "https://sepolia.etherscan.io".to_string(),
                native_token: "ETH".to_string(),
                contract_address: std::env::var("SEPOLIA_FACTORY_ADDRESS")
                    .ok()
                    .and_then(|addr| Address::from_str(&addr).ok()),
                program_id: None,
            },

            // ============ Base ============
            Chain::Base => ChainConfig {
                chain_id: 8453,
                rpc_url: std::env::var("BASE_RPC_URL")
                    .unwrap_or_else(|_| "https://mainnet.base.org".to_string()),
                explorer_url: "https://basescan.org".to_string(),
                native_token: "ETH".to_string(),
                contract_address: std::env::var("BASE_FACTORY_ADDRESS")
                    .ok()
                    .and_then(|addr| Address::from_str(&addr).ok()),
                program_id: None,
            },

            Chain::BaseSepolia => ChainConfig {
                chain_id: 84532,
                rpc_url: std::env::var("BASE_SEPOLIA_RPC_URL")
                    .unwrap_or_else(|_| "https://sepolia.base.org".to_string()),
                explorer_url: "https://sepolia.basescan.org".to_string(),
                native_token: "ETH".to_string(),
                contract_address: std::env::var("BASE_SEPOLIA_FACTORY_ADDRESS")
                    .ok()
                    .and_then(|addr| Address::from_str(&addr).ok()),
                program_id: None,
            },

            // ============ Arbitrum ============
            Chain::Arbitrum => ChainConfig {
                chain_id: 42161,
                rpc_url: std::env::var("ARBITRUM_RPC_URL")
                    .unwrap_or_else(|_| "https://arb1.arbitrum.io/rpc".to_string()),
                explorer_url: "https://arbiscan.io".to_string(),
                native_token: "ETH".to_string(),
                contract_address: std::env::var("ARBITRUM_FACTORY_ADDRESS")
                    .ok()
                    .and_then(|addr| Address::from_str(&addr).ok()),
                program_id: None,
            },

            Chain::ArbitrumSepolia => ChainConfig {
                chain_id: 421614,
                rpc_url: std::env::var("ARBITRUM_SEPOLIA_RPC_URL")
                    .unwrap_or_else(|_| "https://sepolia-rollup.arbitrum.io/rpc".to_string()),
                explorer_url: "https://sepolia.arbiscan.io".to_string(),
                native_token: "ETH".to_string(),
                contract_address: std::env::var("ARBITRUM_SEPOLIA_FACTORY_ADDRESS")
                    .ok()
                    .and_then(|addr| Address::from_str(&addr).ok()),
                program_id: None,
            },

            // ============ Optimism ============
            Chain::Optimism => ChainConfig {
                chain_id: 10,
                rpc_url: std::env::var("OPTIMISM_RPC_URL")
                    .unwrap_or_else(|_| "https://mainnet.optimism.io".to_string()),
                explorer_url: "https://optimistic.etherscan.io".to_string(),
                native_token: "ETH".to_string(),
                contract_address: std::env::var("OPTIMISM_FACTORY_ADDRESS")
                    .ok()
                    .and_then(|addr| Address::from_str(&addr).ok()),
                program_id: None,
            },

            Chain::OptimismSepolia => ChainConfig {
                chain_id: 11155420,
                rpc_url: std::env::var("OPTIMISM_SEPOLIA_RPC_URL")
                    .unwrap_or_else(|_| "https://sepolia.optimism.io".to_string()),
                explorer_url: "https://sepolia-optimism.etherscan.io".to_string(),
                native_token: "ETH".to_string(),
                contract_address: std::env::var("OPTIMISM_SEPOLIA_FACTORY_ADDRESS")
                    .ok()
                    .and_then(|addr| Address::from_str(&addr).ok()),
                program_id: None,
            },

            // ============ Arc ============
            Chain::Arc => ChainConfig {
                chain_id: 0, // TODO: Update with actual Arc chain ID when available
                rpc_url: std::env::var("ARC_RPC_URL")
                    .unwrap_or_else(|_| "https://arc-mainnet-rpc.example.com".to_string()),
                explorer_url: "https://arc-explorer.example.com".to_string(),
                native_token: "ARC".to_string(),
                contract_address: std::env::var("ARC_FACTORY_ADDRESS")
                    .ok()
                    .and_then(|addr| Address::from_str(&addr).ok()),
                program_id: None,
            },

            Chain::ArcTestnet => ChainConfig {
                chain_id: 0, // TODO: Update with actual Arc testnet chain ID
                rpc_url: std::env::var("ARC_TESTNET_RPC_URL")
                    .unwrap_or_else(|_| "https://arc-testnet-rpc.example.com".to_string()),
                explorer_url: "https://arc-testnet-explorer.example.com".to_string(),
                native_token: "ARC".to_string(),
                contract_address: None,
                program_id: None,
            },

            // ============ Tempo ============
            Chain::Tempo => ChainConfig {
                chain_id: 0, // TODO: Update with actual Tempo chain ID when available
                rpc_url: std::env::var("TEMPO_RPC_URL")
                    .unwrap_or_else(|_| "https://tempo-mainnet-rpc.example.com".to_string()),
                explorer_url: "https://tempo-explorer.example.com".to_string(),
                native_token: "TEMPO".to_string(),
                contract_address: std::env::var("TEMPO_FACTORY_ADDRESS")
                    .ok()
                    .and_then(|addr| Address::from_str(&addr).ok()),
                program_id: None,
            },

            Chain::TempoTestnet => ChainConfig {
                chain_id: 0, // TODO: Update with actual Tempo testnet chain ID
                rpc_url: std::env::var("TEMPO_TESTNET_RPC_URL")
                    .unwrap_or_else(|_| "https://tempo-testnet-rpc.example.com".to_string()),
                explorer_url: "https://tempo-testnet-explorer.example.com".to_string(),
                native_token: "TEMPO".to_string(),
                contract_address: None,
                program_id: None,
            },

            // ============ Solana ============
            Chain::Solana => ChainConfig {
                chain_id: 0, // Solana doesn't use numeric chain IDs
                rpc_url: std::env::var("SOLANA_RPC_URL")
                    .unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string()),
                explorer_url: "https://explorer.solana.com".to_string(),
                native_token: "SOL".to_string(),
                contract_address: None,
                program_id: Some(std::env::var("SOLANA_PROGRAM_ID").unwrap_or_else(|_| {
                    "Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS".to_string()
                })),
            },

            Chain::SolanaDevnet => {
                ChainConfig {
                    chain_id: 0,
                    rpc_url: std::env::var("SOLANA_DEVNET_RPC_URL")
                        .unwrap_or_else(|_| "https://api.devnet.solana.com".to_string()),
                    explorer_url: "https://explorer.solana.com?cluster=devnet".to_string(),
                    native_token: "SOL".to_string(),
                    contract_address: None,
                    program_id: Some(std::env::var("SOLANA_DEVNET_PROGRAM_ID").unwrap_or_else(
                        |_| "Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS".to_string(),
                    )),
                }
            }
        }
    }

    /// Returns true if this is an EVM-compatible chain
    pub fn is_evm_chain(&self) -> bool {
        !matches!(self, Chain::Solana | Chain::SolanaDevnet)
    }

    /// Returns true if this is a Solana chain
    pub fn is_solana_chain(&self) -> bool {
        matches!(self, Chain::Solana | Chain::SolanaDevnet)
    }

    /// Returns true if this is a testnet chain
    pub fn is_testnet(&self) -> bool {
        matches!(
            self,
            Chain::EthereumSepolia
                | Chain::BaseSepolia
                | Chain::ArbitrumSepolia
                | Chain::OptimismSepolia
                | Chain::ArcTestnet
                | Chain::TempoTestnet
                | Chain::SolanaDevnet
        )
    }

    /// Returns true if this is a mainnet chain
    pub fn is_mainnet(&self) -> bool {
        !self.is_testnet()
    }

    /// Gets the chain name as a string
    pub fn name(&self) -> &'static str {
        match self {
            Chain::Ethereum => "Ethereum",
            Chain::EthereumSepolia => "Ethereum Sepolia",
            Chain::Base => "Base",
            Chain::BaseSepolia => "Base Sepolia",
            Chain::Arbitrum => "Arbitrum",
            Chain::ArbitrumSepolia => "Arbitrum Sepolia",
            Chain::Optimism => "Optimism",
            Chain::OptimismSepolia => "Optimism Sepolia",
            Chain::Arc => "Arc",
            Chain::ArcTestnet => "Arc Testnet",
            Chain::Tempo => "Tempo",
            Chain::TempoTestnet => "Tempo Testnet",
            Chain::Solana => "Solana",
            Chain::SolanaDevnet => "Solana Devnet",
        }
    }
}

impl FromStr for Chain {
    type Err = ChainError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "ethereum" | "eth" => Ok(Chain::Ethereum),
            "ethereum-sepolia" | "sepolia" => Ok(Chain::EthereumSepolia),
            "base" => Ok(Chain::Base),
            "base-sepolia" => Ok(Chain::BaseSepolia),
            "arbitrum" | "arb" => Ok(Chain::Arbitrum),
            "arbitrum-sepolia" => Ok(Chain::ArbitrumSepolia),
            "optimism" | "op" => Ok(Chain::Optimism),
            "optimism-sepolia" => Ok(Chain::OptimismSepolia),
            "arc" => Ok(Chain::Arc),
            "arc-testnet" => Ok(Chain::ArcTestnet),
            "tempo" => Ok(Chain::Tempo),
            "tempo-testnet" => Ok(Chain::TempoTestnet),
            "solana" | "sol" => Ok(Chain::Solana),
            "solana-devnet" | "devnet" => Ok(Chain::SolanaDevnet),
            _ => Err(ChainError::UnsupportedChain(format!(
                "Unknown chain: {}",
                s
            ))),
        }
    }
}

/// Gets the chain configuration
///
/// # Example
///
/// ```
/// use meridian_chains::{Chain, get_chain_config};
///
/// let config = get_chain_config(Chain::Base);
/// assert_eq!(config.chain_id, 8453);
/// assert_eq!(config.native_token, "ETH");
/// ```
pub fn get_chain_config(chain: Chain) -> ChainConfig {
    chain.config()
}

/// Checks if a chain is EVM-compatible
///
/// # Example
///
/// ```
/// use meridian_chains::{Chain, is_evm_chain};
///
/// assert_eq!(is_evm_chain(Chain::Ethereum), true);
/// assert_eq!(is_evm_chain(Chain::Base), true);
/// assert_eq!(is_evm_chain(Chain::Solana), false);
/// ```
pub fn is_evm_chain(chain: Chain) -> bool {
    chain.is_evm_chain()
}

/// Checks if a chain is Solana
///
/// # Example
///
/// ```
/// use meridian_chains::{Chain, is_solana_chain};
///
/// assert_eq!(is_solana_chain(Chain::Solana), true);
/// assert_eq!(is_solana_chain(Chain::Ethereum), false);
/// ```
pub fn is_solana_chain(chain: Chain) -> bool {
    chain.is_solana_chain()
}

/// Lists all supported EVM chains
pub fn list_evm_chains() -> Vec<Chain> {
    vec![
        Chain::Ethereum,
        Chain::EthereumSepolia,
        Chain::Base,
        Chain::BaseSepolia,
        Chain::Arbitrum,
        Chain::ArbitrumSepolia,
        Chain::Optimism,
        Chain::OptimismSepolia,
        Chain::Arc,
        Chain::ArcTestnet,
        Chain::Tempo,
        Chain::TempoTestnet,
    ]
}

/// Lists all supported Solana chains
pub fn list_solana_chains() -> Vec<Chain> {
    vec![Chain::Solana, Chain::SolanaDevnet]
}

/// Lists all mainnet chains
pub fn list_mainnet_chains() -> Vec<Chain> {
    vec![
        Chain::Ethereum,
        Chain::Base,
        Chain::Arbitrum,
        Chain::Optimism,
        Chain::Arc,
        Chain::Tempo,
        Chain::Solana,
    ]
}

/// Lists all testnet chains
pub fn list_testnet_chains() -> Vec<Chain> {
    vec![
        Chain::EthereumSepolia,
        Chain::BaseSepolia,
        Chain::ArbitrumSepolia,
        Chain::OptimismSepolia,
        Chain::ArcTestnet,
        Chain::TempoTestnet,
        Chain::SolanaDevnet,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evm_chain_detection() {
        assert!(Chain::Ethereum.is_evm_chain());
        assert!(Chain::Base.is_evm_chain());
        assert!(Chain::Arbitrum.is_evm_chain());
        assert!(Chain::Optimism.is_evm_chain());
        assert!(!Chain::Solana.is_evm_chain());
    }

    #[test]
    fn test_solana_chain_detection() {
        assert!(Chain::Solana.is_solana_chain());
        assert!(Chain::SolanaDevnet.is_solana_chain());
        assert!(!Chain::Ethereum.is_solana_chain());
    }

    #[test]
    fn test_testnet_detection() {
        assert!(Chain::EthereumSepolia.is_testnet());
        assert!(Chain::BaseSepolia.is_testnet());
        assert!(Chain::SolanaDevnet.is_testnet());
        assert!(!Chain::Ethereum.is_testnet());
        assert!(!Chain::Solana.is_testnet());
    }

    #[test]
    fn test_chain_ids() {
        assert_eq!(Chain::Ethereum.config().chain_id, 1);
        assert_eq!(Chain::EthereumSepolia.config().chain_id, 11155111);
        assert_eq!(Chain::Base.config().chain_id, 8453);
        assert_eq!(Chain::Arbitrum.config().chain_id, 42161);
        assert_eq!(Chain::Optimism.config().chain_id, 10);
    }

    #[test]
    fn test_chain_from_string() {
        assert_eq!(Chain::from_str("ethereum").unwrap(), Chain::Ethereum);
        assert_eq!(Chain::from_str("base").unwrap(), Chain::Base);
        assert_eq!(Chain::from_str("solana").unwrap(), Chain::Solana);
        assert_eq!(Chain::from_str("sepolia").unwrap(), Chain::EthereumSepolia);

        assert!(Chain::from_str("invalid").is_err());
    }

    #[test]
    fn test_chain_names() {
        assert_eq!(Chain::Ethereum.name(), "Ethereum");
        assert_eq!(Chain::Base.name(), "Base");
        assert_eq!(Chain::Solana.name(), "Solana");
    }

    #[test]
    fn test_list_chains() {
        let evm_chains = list_evm_chains();
        assert_eq!(evm_chains.len(), 12);
        assert!(evm_chains.contains(&Chain::Ethereum));
        assert!(evm_chains.contains(&Chain::Base));

        let solana_chains = list_solana_chains();
        assert_eq!(solana_chains.len(), 2);
        assert!(solana_chains.contains(&Chain::Solana));

        let mainnet_chains = list_mainnet_chains();
        assert_eq!(mainnet_chains.len(), 7);
        assert!(mainnet_chains.iter().all(|c| c.is_mainnet()));

        let testnet_chains = list_testnet_chains();
        assert_eq!(testnet_chains.len(), 7);
        assert!(testnet_chains.iter().all(|c| c.is_testnet()));
    }
}
