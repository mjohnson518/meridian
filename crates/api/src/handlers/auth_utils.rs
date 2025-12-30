//! Shared authentication utilities
//!
//! HIGH-003: Centralized token hashing to eliminate code duplication
//! across handlers (previously duplicated in 6 files)

use sha2::{Sha256, Digest};
use std::sync::OnceLock;

/// Hash token using SHA-256 with salt for database lookup
///
/// This function is used for session token verification across all handlers.
/// Raw token is returned to client, salted hash is stored in database.
///
/// # Security
/// - BE-CRIT-004: Salt prevents rainbow table attacks
/// - Uses OnceLock for thread-safe, one-time initialization
/// - Panics in production if SESSION_TOKEN_SALT is not set
pub fn hash_token_for_lookup(token: &str) -> String {
    // Use static OnceLock to cache the salt for performance
    static TOKEN_SALT: OnceLock<String> = OnceLock::new();

    let salt = TOKEN_SALT.get_or_init(|| {
        std::env::var("SESSION_TOKEN_SALT").unwrap_or_else(|_| {
            // In development, use a default salt with warning
            if std::env::var("ENVIRONMENT")
                .map(|e| e.to_lowercase() == "production")
                .unwrap_or(false)
            {
                // Production MUST have salt configured - panic to prevent insecure operation
                panic!("SESSION_TOKEN_SALT must be set in production environment");
            }
            tracing::warn!("Using default session token salt - set SESSION_TOKEN_SALT in production");
            "dev-session-salt-not-for-production".to_string()
        })
    });

    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    hasher.update(salt.as_bytes());
    hex::encode(hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_token_produces_consistent_output() {
        let token = "test-token-123";
        let hash1 = hash_token_for_lookup(token);
        let hash2 = hash_token_for_lookup(token);

        assert_eq!(hash1, hash2, "Same token should produce same hash");
        assert_eq!(hash1.len(), 64, "SHA-256 hex output should be 64 chars");
    }

    #[test]
    fn test_different_tokens_produce_different_hashes() {
        let hash1 = hash_token_for_lookup("token-a");
        let hash2 = hash_token_for_lookup("token-b");

        assert_ne!(hash1, hash2, "Different tokens should produce different hashes");
    }

    #[test]
    fn test_hash_is_hexadecimal() {
        let hash = hash_token_for_lookup("any-token");

        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()),
                "Hash should only contain hex characters");
    }
}
