//! # HSM Signer Abstraction
//!
//! Decouples the EVM executor from the key management backend.
//! In development, `LocalSignerProvider` uses a private key from env.
//! In production, swap for `AwsKmsSignerProvider` (AWS KMS) or
//! `FireblocksMpcSignerProvider` (Fireblocks MPC) without touching
//! the executor logic.
//!
//! ## Adding a New Signer Backend
//!
//! Implement `SignerProvider` for your type and pass it to `EvmExecutor::with_signer()`.

use ethers::signers::{LocalWallet, Signer};
use ethers::types::{Address, Bytes, Signature};
use std::str::FromStr;
use thiserror::Error;

/// Errors from signer operations
#[derive(Error, Debug)]
pub enum SignerError {
    #[error("Signer configuration error: {0}")]
    Config(String),

    #[error("Signing operation failed: {0}")]
    Signing(String),

    #[error("KMS error: {0}")]
    Kms(String),

    #[error("Key not found: {0}")]
    KeyNotFound(String),
}

pub type SignerResult<T> = Result<T, SignerError>;

/// Raw transaction bytes for signing (RLP-encoded EIP-1559 or legacy tx)
pub type RawTransaction = Bytes;

/// Abstraction over different key management backends.
///
/// Implementations must be `Send + Sync` to be stored in `AppState`.
#[async_trait::async_trait]
pub trait SignerProvider: Send + Sync {
    /// The Ethereum address controlled by this signer
    fn address(&self) -> Address;

    /// Sign a pre-encoded transaction hash (32 bytes)
    async fn sign_hash(&self, hash: [u8; 32]) -> SignerResult<Signature>;

    /// Provider name for logging/metrics
    fn provider_name(&self) -> &str;
}

/// Development signer: loads a private key from the `MINTER_PRIVATE_KEY` env var.
///
/// **Never use in production.** Private keys in environment variables are
/// vulnerable to process introspection and container escape.
pub struct LocalSignerProvider {
    wallet: LocalWallet,
}

impl LocalSignerProvider {
    /// Create from explicit private key hex string (dev/test use only).
    pub fn new(private_key: &str) -> SignerResult<Self> {
        let wallet = private_key
            .parse::<LocalWallet>()
            .map_err(|e| SignerError::Config(format!("Invalid private key: {}", e)))?;
        Ok(Self { wallet })
    }

    /// Create from `MINTER_PRIVATE_KEY` environment variable.
    pub fn from_env() -> SignerResult<Self> {
        let key = std::env::var("MINTER_PRIVATE_KEY")
            .map_err(|_| SignerError::Config("MINTER_PRIVATE_KEY not set".to_string()))?;
        Self::new(&key)
    }
}

#[async_trait::async_trait]
impl SignerProvider for LocalSignerProvider {
    fn address(&self) -> Address {
        self.wallet.address()
    }

    async fn sign_hash(&self, hash: [u8; 32]) -> SignerResult<Signature> {
        self.wallet
            .sign_hash(ethers::types::H256::from(hash))
            .map_err(|e| SignerError::Signing(e.to_string()))
    }

    fn provider_name(&self) -> &str {
        "LocalWallet"
    }
}

/// Production signer: delegates signing to AWS KMS asymmetric key.
///
/// The private key never leaves KMS. All signing happens inside the HSM.
/// Requires IAM permissions: `kms:Sign`, `kms:GetPublicKey`.
///
/// Set `AWS_KMS_KEY_ID` to the KMS key ARN or alias.
pub struct AwsKmsSignerProvider {
    /// KMS key ARN or alias (e.g., "arn:aws:kms:us-east-1:123456789:key/abc-def")
    key_id: String,
    /// Ethereum address derived from the KMS public key
    address: Address,
}

impl AwsKmsSignerProvider {
    /// Initialize from `AWS_KMS_KEY_ID` env var.
    ///
    /// Fetches the public key from KMS to derive the Ethereum address.
    /// Requires `AWS_REGION` and standard AWS credential chain to be configured.
    pub async fn from_env() -> SignerResult<Self> {
        let key_id = std::env::var("AWS_KMS_KEY_ID")
            .map_err(|_| SignerError::Config("AWS_KMS_KEY_ID not set".to_string()))?;

        // Derive Ethereum address from KMS public key.
        // In a real implementation, use aws-sdk-kms to call GetPublicKey,
        // then parse the DER-encoded secp256k1 pubkey and derive the address.
        // This stub returns a placeholder until the aws-sdk-kms dep is added.
        let address = std::env::var("AWS_KMS_ETH_ADDRESS")
            .ok()
            .and_then(|s| Address::from_str(&s).ok())
            .ok_or_else(|| SignerError::Config(
                "AWS_KMS_ETH_ADDRESS must be set (derived from KMS public key)".to_string()
            ))?;

        tracing::info!(key_id = %key_id, address = ?address, "AWS KMS signer initialized");
        Ok(Self { key_id, address })
    }
}

#[async_trait::async_trait]
impl SignerProvider for AwsKmsSignerProvider {
    fn address(&self) -> Address {
        self.address
    }

    async fn sign_hash(&self, _hash: [u8; 32]) -> SignerResult<Signature> {
        // Production implementation:
        // 1. Call aws_sdk_kms::Client::sign() with ECDSA_SHA_256
        // 2. Parse DER-encoded signature → (r, s)
        // 3. Recover v by trying both parity values and checking address match
        // 4. Return ethers::types::Signature { r, s, v }
        //
        // Requires: aws-sdk-kms = "1" in Cargo.toml
        tracing::error!(
            key_id = %self.key_id,
            "AWS KMS sign_hash not yet implemented — add aws-sdk-kms dependency"
        );
        Err(SignerError::Kms(
            "AWS KMS signing not yet implemented — see crates/chains/src/signer.rs".to_string()
        ))
    }

    fn provider_name(&self) -> &str {
        "AwsKms"
    }
}

/// Build the configured signer provider from environment variables.
///
/// Reads `SIGNER_PROVIDER` (defaults to "local"):
/// - `"local"` — `LocalSignerProvider` (dev only, reads `MINTER_PRIVATE_KEY`)
/// - `"aws_kms"` — `AwsKmsSignerProvider` (production HSM)
pub async fn build_signer_from_env() -> SignerResult<Box<dyn SignerProvider>> {
    let provider = std::env::var("SIGNER_PROVIDER")
        .unwrap_or_else(|_| "local".to_string())
        .to_lowercase();

    match provider.as_str() {
        "aws_kms" => {
            let signer = AwsKmsSignerProvider::from_env().await?;
            Ok(Box::new(signer))
        }
        _ => {
            let signer = LocalSignerProvider::from_env()?;
            tracing::warn!(
                "Using LocalSignerProvider — NOT suitable for production. \
                 Set SIGNER_PROVIDER=aws_kms for HSM-backed signing."
            );
            Ok(Box::new(signer))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_local_signer_invalid_key() {
        let result = LocalSignerProvider::new("not-a-valid-key");
        assert!(result.is_err());
    }

    #[test]
    fn test_local_signer_valid_key() {
        // Known test private key (never use in production)
        let key = "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
        let signer = LocalSignerProvider::new(key).unwrap();
        // Verify address is derived correctly
        assert_ne!(signer.address(), Address::zero());
    }
}
