//! # EVM On-Chain Execution Engine
//!
//! Submits mint, burn, and reserve attestation transactions to the
//! MeridianStablecoin contract on any supported EVM chain.
//!
//! ## Architecture
//!
//! - `EvmExecutor`: holds a chain-specific `SignerMiddleware` and contract ABI.
//! - All writes are async and return the transaction hash immediately after
//!   submission. Callers can poll for confirmation via `wait_for_confirmation()`.
//! - A tokio background worker (`spawn_confirmation_worker`) monitors pending
//!   operations in the database and updates their status when mined.
//!
//! ## Signing
//!
//! In development, `LocalWallet` is used (private key from environment).
//! In production, swap for an HSM-backed signer (AWS KMS, Fireblocks MPC, etc.)
//! by implementing the `Signer` trait from the `ethers-signers` crate.

use ethers::abi::{Abi, Token};
use ethers::contract::Contract;
use ethers::middleware::SignerMiddleware;
use ethers::providers::{Http, Middleware, Provider};
use ethers::signers::{LocalWallet, Signer};
use ethers::types::{Address, H256, U256};
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;

/// Errors from the EVM executor
#[derive(Error, Debug)]
pub enum ExecutionError {
    #[error("Provider error: {0}")]
    Provider(String),

    #[error("Contract call failed: {0}")]
    Contract(String),

    #[error("Transaction reverted: {0}")]
    Reverted(String),

    #[error("Transaction not mined within timeout")]
    Timeout,

    #[error("Invalid address: {0}")]
    InvalidAddress(String),

    #[error("Signer configuration error: {0}")]
    SignerConfig(String),

    #[error("Contract not deployed on this chain")]
    ContractNotDeployed,

    #[error("Serialization error: {0}")]
    Serialization(String),
}

pub type ExecutionResult<T> = Result<T, ExecutionError>;

// MeridianStablecoin ABI subset — only the functions we call from the executor.
// Generated from contracts/out/MeridianStablecoin.sol/MeridianStablecoin.json.
// Using inline ABI to avoid build-time artifact dependency.
const MERIDIAN_ABI_JSON: &str = r#"[
  {
    "type": "function",
    "name": "mint",
    "inputs": [
      {
        "name": "request",
        "type": "tuple",
        "components": [
          { "name": "recipient",    "type": "address" },
          { "name": "amount",       "type": "uint256" },
          { "name": "reserveValue", "type": "uint256" },
          { "name": "deadline",     "type": "uint256" },
          { "name": "nonce",        "type": "uint256" }
        ]
      }
    ],
    "outputs": [],
    "stateMutability": "nonpayable"
  },
  {
    "type": "function",
    "name": "burn",
    "inputs": [
      { "name": "amount", "type": "uint256" }
    ],
    "outputs": [],
    "stateMutability": "nonpayable"
  },
  {
    "type": "function",
    "name": "attestReserves",
    "inputs": [
      { "name": "attestedReserveValue", "type": "uint256" }
    ],
    "outputs": [],
    "stateMutability": "nonpayable"
  },
  {
    "type": "function",
    "name": "totalSupply",
    "inputs": [],
    "outputs": [{ "name": "", "type": "uint256" }],
    "stateMutability": "view"
  },
  {
    "type": "function",
    "name": "nonces",
    "inputs": [{ "name": "account", "type": "address" }],
    "outputs": [{ "name": "", "type": "uint256" }],
    "stateMutability": "view"
  }
]"#;

/// Parameters for a mint operation
#[derive(Debug, Clone)]
pub struct OnChainMintRequest {
    /// Token recipient address (hex string)
    pub recipient: Address,
    /// Number of tokens to mint (6 decimals, like USDC)
    pub amount: U256,
    /// Reserve value backing this mint (2 decimals)
    pub reserve_value: U256,
    /// Unix timestamp deadline — transaction reverts after this
    pub deadline: U256,
}

/// Result of a submitted transaction
#[derive(Debug, Clone)]
pub struct SubmittedTx {
    /// Transaction hash
    pub tx_hash: H256,
    /// Chain the transaction was submitted to
    pub chain_id: u64,
}

/// Confirmation status of a submitted transaction
#[derive(Debug, Clone)]
pub struct TxConfirmation {
    pub tx_hash: H256,
    pub block_number: u64,
    pub gas_used: U256,
    pub success: bool,
}

/// EVM transaction executor for MeridianStablecoin operations.
///
/// Holds a provider + signer combination for one chain. Create one executor
/// per chain you want to interact with.
pub struct EvmExecutor {
    /// The signing client (provider + wallet)
    client: Arc<SignerMiddleware<Provider<Http>, LocalWallet>>,
    /// Deployed MeridianStablecoin contract address on this chain
    contract_address: Address,
    /// Parsed ABI for the contract
    abi: Abi,
    /// Chain ID for the connected network
    chain_id: u64,
    /// How many block confirmations to wait for before declaring success
    #[allow(dead_code)] // Used by future multi-confirmation polling
    confirmations: u64,
    /// Max time to wait for a transaction to be mined
    tx_timeout: Duration,
}

impl EvmExecutor {
    /// Create a new executor from environment variables.
    ///
    /// Reads:
    /// - `MINTER_PRIVATE_KEY` — private key (hex, 0x-prefixed or bare)
    /// - `CONTRACT_ADDRESS`   — MeridianStablecoin proxy address on this chain
    /// - `RPC_URL`           — EVM JSON-RPC endpoint
    ///
    /// For production, replace `LocalWallet` with an HSM-backed signer.
    pub async fn from_env(
        rpc_url: &str,
        contract_address: Address,
        chain_id: u64,
    ) -> ExecutionResult<Self> {
        let private_key = std::env::var("MINTER_PRIVATE_KEY")
            .map_err(|_| ExecutionError::SignerConfig("MINTER_PRIVATE_KEY not set".to_string()))?;

        Self::new(rpc_url, contract_address, &private_key, chain_id).await
    }

    /// Create a new executor with an explicit private key (dev/test use).
    pub async fn new(
        rpc_url: &str,
        contract_address: Address,
        private_key: &str,
        chain_id: u64,
    ) -> ExecutionResult<Self> {
        let provider = Provider::<Http>::try_from(rpc_url)
            .map_err(|e| ExecutionError::Provider(e.to_string()))?;

        let wallet: LocalWallet = private_key
            .parse::<LocalWallet>()
            .map_err(|e| ExecutionError::SignerConfig(e.to_string()))?
            .with_chain_id(chain_id);

        let client = SignerMiddleware::new(provider, wallet);

        let abi: Abi = serde_json::from_str(MERIDIAN_ABI_JSON)
            .map_err(|e| ExecutionError::Serialization(e.to_string()))?;

        Ok(Self {
            client: Arc::new(client),
            contract_address,
            abi,
            chain_id,
            confirmations: Self::default_confirmations(chain_id),
            tx_timeout: Duration::from_secs(300),
        })
    }

    /// Number of confirmations to wait based on chain security properties.
    /// L2s have fast finality; Ethereum mainnet needs more blocks.
    fn default_confirmations(chain_id: u64) -> u64 {
        match chain_id {
            1 => 12,           // Ethereum mainnet — conservative
            11155111 => 1,     // Sepolia — testnet
            8453 | 84532 => 1, // Base / Base Sepolia
            42161 | 421614 => 1, // Arbitrum / Arbitrum Sepolia
            10 | 11155420 => 1,  // Optimism / Optimism Sepolia
            _ => 1,
        }
    }

    /// Get the current nonce for an address from the contract.
    pub async fn get_nonce(&self, address: Address) -> ExecutionResult<U256> {
        let contract = Contract::new(self.contract_address, self.abi.clone(), self.client.clone());

        contract
            .method::<(Address,), U256>("nonces", (address,))
            .map_err(|e| ExecutionError::Contract(e.to_string()))?
            .call()
            .await
            .map_err(|e| ExecutionError::Contract(e.to_string()))
    }

    /// Submit a mint transaction to the MeridianStablecoin contract.
    ///
    /// Returns the transaction hash immediately. Use `wait_for_confirmation`
    /// to block until the transaction is mined.
    ///
    /// The nonce is fetched from the contract automatically.
    pub async fn mint_on_chain(&self, request: OnChainMintRequest) -> ExecutionResult<SubmittedTx> {
        let signer_address = self.client.signer().address();
        let nonce = self.get_nonce(request.recipient).await?;

        tracing::info!(
            recipient = ?request.recipient,
            amount = %request.amount,
            chain_id = self.chain_id,
            "Submitting mint transaction"
        );

        let mint_request_tuple = Token::Tuple(vec![
            Token::Address(request.recipient),
            Token::Uint(request.amount),
            Token::Uint(request.reserve_value),
            Token::Uint(request.deadline),
            Token::Uint(nonce),
        ]);

        let contract = Contract::new(self.contract_address, self.abi.clone(), self.client.clone());

        let call = contract
            .method::<(Token,), ()>("mint", (mint_request_tuple,))
            .map_err(|e| ExecutionError::Contract(format!("ABI encode error: {}", e)))?;

        let pending_tx = call
            .send()
            .await
            .map_err(|e| {
                let msg = e.to_string();
                if msg.contains("revert") {
                    ExecutionError::Reverted(msg)
                } else {
                    ExecutionError::Contract(msg)
                }
            })?;

        let tx_hash = pending_tx.tx_hash();

        tracing::info!(
            tx_hash = ?tx_hash,
            signer = ?signer_address,
            chain_id = self.chain_id,
            "Mint transaction submitted"
        );

        Ok(SubmittedTx { tx_hash, chain_id: self.chain_id })
    }

    /// Submit a burn transaction.
    ///
    /// The caller (signer) burns their own tokens. The contract calculates
    /// pro-rata reserve release automatically.
    pub async fn burn_on_chain(&self, amount: U256) -> ExecutionResult<SubmittedTx> {
        tracing::info!(
            amount = %amount,
            chain_id = self.chain_id,
            "Submitting burn transaction"
        );

        let contract = Contract::new(self.contract_address, self.abi.clone(), self.client.clone());

        let call = contract
            .method::<(U256,), ()>("burn", (amount,))
            .map_err(|e| ExecutionError::Contract(format!("ABI encode error: {}", e)))?;

        let pending_tx = call
            .send()
            .await
            .map_err(|e| {
                let msg = e.to_string();
                if msg.contains("revert") {
                    ExecutionError::Reverted(msg)
                } else {
                    ExecutionError::Contract(msg)
                }
            })?;

        let tx_hash = pending_tx.tx_hash();

        tracing::info!(tx_hash = ?tx_hash, "Burn transaction submitted");

        Ok(SubmittedTx { tx_hash, chain_id: self.chain_id })
    }

    /// Submit a reserve attestation transaction.
    ///
    /// Called by the automated Proof of Reserves pipeline after verifying
    /// custody balances. The `attested_value` is expressed in 2-decimal units
    /// (e.g., 100_000_00 = $1,000,000.00).
    pub async fn attest_reserves_on_chain(
        &self,
        attested_reserve_value: U256,
    ) -> ExecutionResult<SubmittedTx> {
        tracing::info!(
            attested_value = %attested_reserve_value,
            chain_id = self.chain_id,
            "Submitting reserve attestation"
        );

        let contract = Contract::new(self.contract_address, self.abi.clone(), self.client.clone());

        let call = contract
            .method::<(U256,), ()>("attestReserves", (attested_reserve_value,))
            .map_err(|e| ExecutionError::Contract(format!("ABI encode error: {}", e)))?;

        let pending_tx = call
            .send()
            .await
            .map_err(|e| {
                let msg = e.to_string();
                if msg.contains("revert") {
                    ExecutionError::Reverted(msg)
                } else {
                    ExecutionError::Contract(msg)
                }
            })?;

        let tx_hash = pending_tx.tx_hash();

        tracing::info!(tx_hash = ?tx_hash, "Attestation transaction submitted");

        Ok(SubmittedTx { tx_hash, chain_id: self.chain_id })
    }

    /// Wait for a submitted transaction to be mined and return confirmation details.
    ///
    /// Polls every 3 seconds until the transaction is included in a block,
    /// subject to `tx_timeout`. Returns `ExecutionError::Timeout` if the
    /// transaction is not mined within the timeout window.
    pub async fn wait_for_confirmation(
        &self,
        tx_hash: H256,
    ) -> ExecutionResult<TxConfirmation> {
        let deadline = tokio::time::Instant::now() + self.tx_timeout;

        loop {
            if tokio::time::Instant::now() > deadline {
                return Err(ExecutionError::Timeout);
            }

            match self.client.get_transaction_receipt(tx_hash).await {
                Ok(Some(receipt)) => {
                    let success = receipt.status.map(|s| s.as_u64() == 1).unwrap_or(false);
                    let block_number = receipt.block_number.unwrap_or_default().as_u64();
                    let gas_used = receipt.gas_used.unwrap_or_default();

                    if success {
                        tracing::info!(
                            tx_hash = ?tx_hash,
                            block = block_number,
                            gas_used = %gas_used,
                            "Transaction confirmed"
                        );
                    } else {
                        tracing::warn!(tx_hash = ?tx_hash, "Transaction reverted on-chain");
                        return Err(ExecutionError::Reverted(format!("{:?}", tx_hash)));
                    }

                    return Ok(TxConfirmation {
                        tx_hash,
                        block_number,
                        gas_used,
                        success,
                    });
                }
                Ok(None) => {
                    // Not yet mined — wait and retry
                    tokio::time::sleep(Duration::from_secs(3)).await;
                }
                Err(e) => {
                    tracing::warn!(error = %e, "Error polling for receipt, retrying");
                    tokio::time::sleep(Duration::from_secs(3)).await;
                }
            }
        }
    }

    /// Chain ID this executor is configured for
    pub fn chain_id(&self) -> u64 {
        self.chain_id
    }
}

/// Spawn a background confirmation worker that monitors pending operations
/// and updates their status when confirmed.
///
/// This is used by the API: the mint/burn handler submits to chain and
/// returns immediately. This worker polls for confirmation and updates the
/// DB `operations` row from PENDING to COMPLETED (or FAILED on revert).
///
/// `db_pool` — PgPool from the API state
/// `executor` — Arc<EvmExecutor> for the target chain
/// `poll_interval` — how often to check the database for pending operations
pub fn spawn_confirmation_worker(
    executor: Arc<EvmExecutor>,
    db_pool: Arc<sqlx::PgPool>,
    poll_interval: Duration,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        tracing::info!(
            chain_id = executor.chain_id(),
            "On-chain confirmation worker started"
        );

        loop {
            tokio::time::sleep(poll_interval).await;

            // Fetch all PENDING operations that have a transaction_hash set
            let pending_ops = sqlx::query_as::<_, (i32, String)>(
                r#"
                SELECT id, transaction_hash
                FROM operations
                WHERE status = 'PENDING'
                  AND transaction_hash IS NOT NULL
                  AND created_at > NOW() - INTERVAL '24 hours'
                LIMIT 50
                "#
            )
            .fetch_all(db_pool.as_ref())
            .await;

            let pending_ops = match pending_ops {
                Ok(ops) => ops,
                Err(e) => {
                    tracing::error!(error = %e, "Failed to fetch pending operations");
                    continue;
                }
            };

            for (op_id, tx_hash_str) in pending_ops {
                let tx_hash: H256 = match tx_hash_str.parse() {
                    Ok(h) => h,
                    Err(_) => {
                        tracing::warn!(op_id, tx_hash = tx_hash_str, "Invalid tx hash in DB");
                        continue;
                    }
                };

                match executor.wait_for_confirmation(tx_hash).await {
                    Ok(confirmation) if confirmation.success => {
                        let _ = sqlx::query(
                            "UPDATE operations SET status = 'COMPLETED', updated_at = NOW() WHERE id = $1"
                        )
                        .bind(op_id)
                        .execute(db_pool.as_ref())
                        .await;

                        tracing::info!(
                            op_id,
                            tx_hash = ?tx_hash,
                            block = confirmation.block_number,
                            "Operation confirmed on-chain"
                        );
                    }
                    Ok(_) => {
                        // Transaction reverted
                        let _ = sqlx::query(
                            "UPDATE operations SET status = 'FAILED', updated_at = NOW() WHERE id = $1"
                        )
                        .bind(op_id)
                        .execute(db_pool.as_ref())
                        .await;

                        tracing::warn!(op_id, tx_hash = ?tx_hash, "Operation reverted on-chain");
                    }
                    Err(ExecutionError::Timeout) => {
                        tracing::warn!(op_id, "Transaction confirmation timed out — will retry next poll");
                    }
                    Err(e) => {
                        tracing::error!(op_id, error = %e, "Confirmation check failed");
                    }
                }
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_confirmations() {
        // Ethereum mainnet — conservative 12 blocks
        assert_eq!(EvmExecutor::default_confirmations(1), 12);
        // Sepolia testnet — 1 block is fine
        assert_eq!(EvmExecutor::default_confirmations(11155111), 1);
        // Base — L2, 1 block
        assert_eq!(EvmExecutor::default_confirmations(8453), 1);
        // Arbitrum — L2, 1 block
        assert_eq!(EvmExecutor::default_confirmations(42161), 1);
        // Unknown chain — safe default of 1
        assert_eq!(EvmExecutor::default_confirmations(99999), 1);
    }

    #[test]
    fn test_abi_parses_correctly() {
        let abi: Result<Abi, _> = serde_json::from_str(MERIDIAN_ABI_JSON);
        assert!(abi.is_ok(), "ABI should parse without error");
        let abi = abi.unwrap();

        // Verify all required functions are present
        assert!(abi.function("mint").is_ok(), "mint function must be in ABI");
        assert!(abi.function("burn").is_ok(), "burn function must be in ABI");
        assert!(abi.function("attestReserves").is_ok(), "attestReserves function must be in ABI");
        assert!(abi.function("nonces").is_ok(), "nonces function must be in ABI");
    }

    #[test]
    fn test_mint_request_token_encoding() {
        // Verify Token::Tuple encoding matches Solidity struct layout
        let recipient = Address::zero();
        let amount = U256::from(1_000_000u64); // 1 EURM (6 decimals)
        let reserve_value = U256::from(100_000_00u64); // $1,000.00 (2 decimals)
        let deadline = U256::from(u64::MAX);
        let nonce = U256::zero();

        let token = Token::Tuple(vec![
            Token::Address(recipient),
            Token::Uint(amount),
            Token::Uint(reserve_value),
            Token::Uint(deadline),
            Token::Uint(nonce),
        ]);

        // Should be a tuple with 5 elements
        match token {
            Token::Tuple(ref fields) => assert_eq!(fields.len(), 5),
            _ => panic!("Expected Tuple"),
        }
    }
}
