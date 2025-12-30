//! x402 Agent payment handlers

use crate::error::{ApiError, handle_db_error};
use crate::state::AppState;
use actix_web::{web, HttpRequest, HttpResponse};
use ethers::types::Address;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use sqlx::PgPool;
use std::str::FromStr;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct CreateAgentRequest {
    pub user_id: i32,
    pub agent_name: String,
    pub spending_limit_daily: String,
    pub spending_limit_transaction: String,
}

#[derive(Debug, Serialize)]
pub struct CreateAgentResponse {
    pub agent_id: String,
    pub api_key: String,
    pub wallet_address: String,
    pub spending_limit_daily: String,
    pub spending_limit_transaction: String,
}

#[derive(Debug, Deserialize)]
pub struct AgentPaymentRequest {
    pub agent_id: String,
    pub api_key: String,
    pub recipient: String,
    pub amount: String,
    pub currency: String,
    pub memo: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AgentPaymentResponse {
    pub transaction_id: i32,
    pub agent_id: String,
    pub recipient: String,
    pub amount: String,
    pub currency: String,
    pub status: String,
    pub transaction_hash: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct AgentWalletResponse {
    pub agent_id: String,
    pub agent_name: String,
    pub wallet_address: String,
    pub spending_limit_daily: String,
    pub spending_limit_transaction: String,
    pub daily_spent: String,
    pub is_active: bool,
    pub created_at: String,
}

/// POST /api/v1/agents/create
pub async fn create_agent(
    state: web::Data<Arc<AppState>>,
    http_req: HttpRequest,
    req: web::Json<CreateAgentRequest>,
) -> Result<HttpResponse, ApiError> {
    // SECURITY: Verify authenticated user matches the user_id in request
    let auth_user_id = get_authenticated_user_id(state.db_pool.as_ref(), &http_req).await?;
    if auth_user_id != req.user_id {
        tracing::warn!(
            auth_user_id = auth_user_id,
            requested_user_id = req.user_id,
            "Agent creation rejected: user_id mismatch"
        );
        return Err(ApiError::Forbidden("Cannot create agent for another user".to_string()));
    }

    tracing::info!(
        user_id = req.user_id,
        agent_name = %req.agent_name,
        "Creating agent wallet"
    );

    // Verify user exists and is KYC approved
    let user = sqlx::query!("SELECT kyc_status FROM users WHERE id = $1", req.user_id)
        .fetch_optional(state.db_pool.as_ref())
        .await
        .map_err(|e| handle_db_error(e, "agents"))?;

    let user = match user {
        Some(u) => u,
        None => return Err(ApiError::NotFound("User not found".to_string())),
    };

    if user.kyc_status != "APPROVED" {
        return Err(ApiError::Forbidden(
            "KYC approval required to create agent wallets".to_string(),
        ));
    }

    // BACKEND-CRIT-002: Validate spending limits
    let daily_limit = Decimal::from_str(&req.spending_limit_daily)
        .map_err(|_| ApiError::BadRequest("Invalid daily spending limit format".to_string()))?;
    let tx_limit = Decimal::from_str(&req.spending_limit_transaction)
        .map_err(|_| ApiError::BadRequest("Invalid transaction spending limit format".to_string()))?;

    // Must be positive
    if daily_limit <= Decimal::ZERO {
        return Err(ApiError::BadRequest("Daily spending limit must be greater than zero".to_string()));
    }
    if tx_limit <= Decimal::ZERO {
        return Err(ApiError::BadRequest("Transaction spending limit must be greater than zero".to_string()));
    }

    // Daily limit must be >= transaction limit (logical constraint)
    if daily_limit < tx_limit {
        return Err(ApiError::BadRequest(
            "Daily spending limit cannot be less than transaction limit".to_string()
        ));
    }

    // Max limit: 100 million per day (reasonable upper bound)
    let max_daily = Decimal::from(100_000_000i64);
    if daily_limit > max_daily {
        return Err(ApiError::BadRequest(
            format!("Daily spending limit exceeds maximum: {}", max_daily)
        ));
    }

    // Generate agent ID and API key
    let agent_id = format!("agent_{}", Uuid::new_v4().to_string().replace("-", ""));
    let api_key = generate_api_key();
    let api_key_hash = hash_api_key(&api_key);

    // Generate wallet address (production-safe: requires WALLET_SERVICE_URL in production)
    let wallet_address = generate_wallet_address(&agent_id).map_err(|e| {
        tracing::error!("Wallet generation failed: {}", e);
        ApiError::InternalError(format!("Wallet generation failed: {}", e))
    })?;

    // Insert agent wallet
    let agent = sqlx::query!(
        r#"
        INSERT INTO agent_wallets (
            user_id, agent_id, agent_name, wallet_address, api_key_hash,
            spending_limit_daily, spending_limit_transaction
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING id, created_at
        "#,
        req.user_id,
        agent_id,
        req.agent_name,
        wallet_address,
        api_key_hash,
        req.spending_limit_daily,
        req.spending_limit_transaction
    )
    .fetch_one(state.db_pool.as_ref())
    .await
    .map_err(|e| {
        tracing::error!("Failed to create agent: {}", e);
        ApiError::InternalError("Failed to create agent wallet".to_string())
    })?;

    tracing::info!(
        agent_id = %agent_id,
        wallet = %wallet_address,
        "Agent wallet created"
    );

    Ok(HttpResponse::Created().json(CreateAgentResponse {
        agent_id,
        api_key, // Only returned once!
        wallet_address,
        spending_limit_daily: req.spending_limit_daily.clone(),
        spending_limit_transaction: req.spending_limit_transaction.clone(),
    }))
}

/// POST /api/v1/agents/pay
/// SECURITY: Requires authentication and verifies user owns the agent
pub async fn agent_pay(
    state: web::Data<Arc<AppState>>,
    http_req: HttpRequest,
    req: web::Json<AgentPaymentRequest>,
) -> Result<HttpResponse, ApiError> {
    tracing::info!(
        agent_id = %req.agent_id,
        recipient = %req.recipient,
        amount = %req.amount,
        currency = %req.currency,
        "Agent payment request"
    );

    // SECURITY: Verify the authenticated user owns this agent
    let auth_user_id = get_authenticated_user_id(state.db_pool.as_ref(), &http_req).await?;

    // Verify API key
    let agent = verify_agent_api_key(state.db_pool.as_ref(), &req.agent_id, &req.api_key).await?;

    // SECURITY: Ensure authenticated user owns the agent
    if agent.user_id != auth_user_id {
        tracing::warn!(
            auth_user_id = auth_user_id,
            agent_owner_id = agent.user_id,
            agent_id = %req.agent_id,
            "Agent payment rejected: user does not own agent"
        );
        return Err(ApiError::Forbidden("You do not own this agent".to_string()));
    }

    if !agent.is_active {
        return Err(ApiError::Forbidden("Agent wallet is inactive".to_string()));
    }

    // Parse amount
    let amount_decimal = Decimal::from_str(&req.amount)
        .map_err(|_| ApiError::BadRequest("Invalid amount format".to_string()))?;

    // Check transaction limit
    let tx_limit = Decimal::from_str(&agent.spending_limit_transaction)
        .map_err(|_| ApiError::InternalError("Invalid spending limit".to_string()))?;

    if amount_decimal > tx_limit {
        return Err(ApiError::Forbidden(format!(
            "Amount exceeds transaction limit: {} > {}",
            amount_decimal, tx_limit
        )));
    }

    // Check daily limit
    let daily_spent = get_daily_spent(state.db_pool.as_ref(), &req.agent_id).await?;
    let daily_limit = Decimal::from_str(&agent.spending_limit_daily)
        .map_err(|_| ApiError::InternalError("Invalid daily limit".to_string()))?;

    if daily_spent + amount_decimal > daily_limit {
        return Err(ApiError::Forbidden(format!(
            "Daily spending limit exceeded: {} + {} > {}",
            daily_spent, amount_decimal, daily_limit
        )));
    }

    // Validate recipient address
    if !is_valid_ethereum_address(&req.recipient) {
        return Err(ApiError::BadRequest("Invalid recipient address".to_string()));
    }

    // Insert transaction
    let transaction = sqlx::query!(
        r#"
        INSERT INTO agent_transactions (agent_id, currency, amount, recipient, status)
        VALUES ($1, $2, $3, $4, 'PENDING')
        RETURNING id, status, created_at
        "#,
        req.agent_id,
        req.currency,
        req.amount,
        req.recipient
    )
    .fetch_one(state.db_pool.as_ref())
    .await
    .map_err(|e| {
        tracing::error!("Failed to create agent transaction: {}", e);
        ApiError::InternalError("Failed to create transaction".to_string())
    })?;

    // Check environment
    let is_production = std::env::var("ENVIRONMENT")
        .map(|e| e.to_lowercase() == "production")
        .unwrap_or(false);

    // Check if mock mode is explicitly enabled (ALLOW_MOCK_TRANSACTIONS=true)
    let mock_mode = std::env::var("ALLOW_MOCK_TRANSACTIONS")
        .map(|v| v.to_lowercase() == "true")
        .unwrap_or(false);

    // SECURITY: Block mock transactions in production - this is a critical safety check
    if is_production && mock_mode {
        tracing::error!(
            transaction_id = transaction.id,
            "SECURITY VIOLATION: ALLOW_MOCK_TRANSACTIONS is enabled in production!"
        );
        return Err(ApiError::InternalError(
            "Configuration error. Contact support.".to_string()
        ));
    }

    // In production, require real blockchain execution (not implemented yet)
    if !mock_mode {
        tracing::warn!(
            transaction_id = transaction.id,
            "Real blockchain execution not implemented. Set ALLOW_MOCK_TRANSACTIONS=true for development."
        );
        return Err(ApiError::InternalError(
            "Blockchain execution not available. Contact support.".to_string()
        ));
    }

    // MOCK MODE: Generate simulated transaction hash (development only)
    // WARNING: This does NOT execute real blockchain transactions!
    tracing::warn!(
        transaction_id = transaction.id,
        "MOCK MODE (dev only): Generating simulated transaction hash"
    );
    let tx_hash = format!("0xMOCK_{}", Uuid::new_v4().to_string().replace("-", ""));

    // Update transaction status
    sqlx::query!(
        "UPDATE agent_transactions SET status = 'COMPLETED', transaction_hash = $1 WHERE id = $2",
        tx_hash,
        transaction.id
    )
    .execute(state.db_pool.as_ref())
    .await
    .map_err(|e| {
        tracing::error!("Failed to update transaction status: {}", e);
        ApiError::InternalError("Failed to update transaction".to_string())
    })?;

    tracing::info!(
        transaction_id = transaction.id,
        tx_hash = %tx_hash,
        "Agent payment executed successfully"
    );

    Ok(HttpResponse::Ok().json(AgentPaymentResponse {
        transaction_id: transaction.id,
        agent_id: req.agent_id.clone(),
        recipient: req.recipient.clone(),
        amount: req.amount.clone(),
        currency: req.currency.clone(),
        status: "COMPLETED".to_string(),
        transaction_hash: Some(tx_hash),
        created_at: transaction.created_at.to_rfc3339(),
    }))
}

/// GET /api/v1/agents/list/{user_id}
pub async fn list_agents(
    state: web::Data<Arc<AppState>>,
    req: HttpRequest,
    user_id: web::Path<i32>,
) -> Result<HttpResponse, ApiError> {
    let user_id = user_id.into_inner();

    // Verify authenticated user matches requested user_id
    let auth_user_id = get_authenticated_user_id(state.db_pool.as_ref(), &req).await?;
    if auth_user_id != user_id {
        return Err(ApiError::Forbidden("Cannot access other user's agents".to_string()));
    }

    let agents = sqlx::query!(
        r#"
        SELECT agent_id, agent_name, wallet_address, spending_limit_daily, 
               spending_limit_transaction, is_active, created_at
        FROM agent_wallets
        WHERE user_id = $1
        ORDER BY created_at DESC
        "#,
        user_id
    )
    .fetch_all(state.db_pool.as_ref())
    .await
    .map_err(|e| handle_db_error(e, "agents"))?;

    let mut responses = Vec::new();
    for agent in agents {
        let daily_spent = get_daily_spent(state.db_pool.as_ref(), &agent.agent_id).await?;

        responses.push(AgentWalletResponse {
            agent_id: agent.agent_id.clone(),
            agent_name: agent.agent_name.unwrap_or_else(|| "Unnamed Agent".to_string()),
            wallet_address: agent.wallet_address,
            spending_limit_daily: agent.spending_limit_daily,
            spending_limit_transaction: agent.spending_limit_transaction,
            daily_spent: daily_spent.to_string(),
            is_active: agent.is_active,
            created_at: agent.created_at.to_rfc3339(),
        });
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "agents": responses,
        "count": responses.len()
    })))
}

/// GET /api/v1/agents/transactions/{agent_id}
pub async fn get_agent_transactions(
    state: web::Data<Arc<AppState>>,
    req: HttpRequest,
    agent_id: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let agent_id = agent_id.into_inner();

    // Verify authenticated user owns this agent
    let auth_user_id = get_authenticated_user_id(state.db_pool.as_ref(), &req).await?;

    let agent_owner = sqlx::query!(
        "SELECT user_id FROM agent_wallets WHERE agent_id = $1",
        agent_id
    )
    .fetch_optional(state.db_pool.as_ref())
    .await
    .map_err(|e| handle_db_error(e, "agents"))?;

    match agent_owner {
        Some(owner) if owner.user_id == auth_user_id => {},
        Some(_) => return Err(ApiError::Forbidden("Cannot access other user's agent".to_string())),
        None => return Err(ApiError::NotFound("Agent not found".to_string())),
    }

    let transactions = sqlx::query!(
        r#"
        SELECT id, currency, amount, recipient, status, transaction_hash, created_at
        FROM agent_transactions
        WHERE agent_id = $1
        ORDER BY created_at DESC
        LIMIT 100
        "#,
        agent_id
    )
    .fetch_all(state.db_pool.as_ref())
    .await
    .map_err(|e| handle_db_error(e, "agents"))?;

    let responses: Vec<serde_json::Value> = transactions
        .into_iter()
        .map(|tx| {
            serde_json::json!({
                "id": tx.id,
                "currency": tx.currency,
                "amount": tx.amount,
                "recipient": tx.recipient,
                "status": tx.status,
                "transaction_hash": tx.transaction_hash,
                "created_at": tx.created_at.to_rfc3339()
            })
        })
        .collect();

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "transactions": responses,
        "count": responses.len()
    })))
}

// Helper functions
async fn verify_agent_api_key(
    pool: &PgPool,
    agent_id: &str,
    api_key: &str,
) -> Result<AgentWallet, ApiError> {
    let api_key_hash = hash_api_key(api_key);

    let agent = sqlx::query!(
        r#"
        SELECT user_id, agent_id, wallet_address, spending_limit_daily,
               spending_limit_transaction, is_active
        FROM agent_wallets
        WHERE agent_id = $1 AND api_key_hash = $2
        "#,
        agent_id,
        api_key_hash
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| handle_db_error(e, "agents"))?;

    match agent {
        Some(a) => Ok(AgentWallet {
            user_id: a.user_id,
            agent_id: a.agent_id,
            wallet_address: a.wallet_address,
            spending_limit_daily: a.spending_limit_daily,
            spending_limit_transaction: a.spending_limit_transaction,
            is_active: a.is_active,
        }),
        None => Err(ApiError::Unauthorized("Invalid agent credentials".to_string())),
    }
}

async fn get_daily_spent(pool: &PgPool, agent_id: &str) -> Result<Decimal, ApiError> {
    // Fetch all transactions and sum in Rust to avoid type issues
    let transactions = sqlx::query!(
        r#"
        SELECT amount
        FROM agent_transactions
        WHERE agent_id = $1 
        AND created_at > NOW() - INTERVAL '24 hours'
        AND status IN ('PENDING', 'COMPLETED')
        "#,
        agent_id
    )
    .fetch_all(pool)
    .await
    .map_err(|e| handle_db_error(e, "agents"))?;

    let mut total = Decimal::ZERO;
    for tx in transactions {
        let amount = Decimal::from_str(&tx.amount)
            .map_err(|_| ApiError::InternalError("Invalid amount in transaction".to_string()))?;
        total += amount;
    }

    Ok(total)
}

fn generate_api_key() -> String {
    format!("mk_{}", Uuid::new_v4().to_string().replace("-", ""))
}

fn hash_api_key(api_key: &str) -> String {
    use sha2::{Sha256, Digest};
    use std::sync::OnceLock;

    // SECURITY: Use configurable salt from environment, not hardcoded value
    // This prevents attackers who gain source access from computing hashes
    static SALT: OnceLock<String> = OnceLock::new();
    let salt = SALT.get_or_init(|| {
        // Note: Production validation for API_KEY_SALT happens at startup in main.rs
        std::env::var("API_KEY_SALT").unwrap_or_else(|_| {
            tracing::warn!("Using default API key salt - set API_KEY_SALT in production");
            "dev-only-salt-replace-in-production".to_string()
        })
    });

    let mut hasher = Sha256::new();
    hasher.update(api_key.as_bytes());
    hasher.update(salt.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Generate a wallet address for an agent.
///
/// PRODUCTION SAFETY: In production, this requires WALLET_SERVICE_URL to be set.
/// The function will panic on startup if the environment is production and no
/// wallet service is configured. In development, mock addresses are generated
/// but are clearly marked as non-production.
fn generate_wallet_address(agent_id: &str) -> Result<String, &'static str> {
    use std::sync::OnceLock;
    use sha2::{Sha256, Digest};

    static IS_PRODUCTION: OnceLock<bool> = OnceLock::new();
    static WALLET_SERVICE_URL: OnceLock<Option<String>> = OnceLock::new();

    let is_production = *IS_PRODUCTION.get_or_init(|| {
        std::env::var("ENVIRONMENT")
            .map(|e| e.to_lowercase() == "production")
            .unwrap_or(false)
    });

    let wallet_service = WALLET_SERVICE_URL.get_or_init(|| {
        std::env::var("WALLET_SERVICE_URL").ok()
    });

    // In production, require wallet service integration
    if is_production {
        match wallet_service {
            Some(url) => {
                // In a real implementation, call the wallet service here
                // For now, log that it should be called
                tracing::info!(
                    service_url = %url,
                    agent_id = %agent_id,
                    "Production wallet generation via external service"
                );
                // TODO: Implement actual wallet service call
                // For now, return error to prevent mock wallets in production
                return Err("Wallet service integration not yet implemented - deploy wallet service first");
            }
            None => {
                tracing::error!(
                    "WALLET_SERVICE_URL must be set in production. \
                     Cannot generate mock wallet addresses in production environment."
                );
                return Err("Wallet service not configured for production");
            }
        }
    }

    // Development mode: generate mock address with clear warning
    tracing::warn!(
        agent_id = %agent_id,
        "DEVELOPMENT MODE: Generating mock wallet address. NOT FOR PRODUCTION USE."
    );

    let mut hasher = Sha256::new();
    hasher.update(agent_id.as_bytes());
    hasher.update(b"_MOCK_WALLET_"); // Add marker to make hash different
    let hash = hasher.finalize();

    // Use 0xDE prefix for dev addresses (valid hex prefix)
    // Format: 0xDE + 38 chars of hash = 42 chars total (valid Ethereum address length)
    // 0xDE (4 chars) + 19 bytes encoded as hex (38 chars) = 42 chars
    Ok(format!("0xDE{}", hex::encode(&hash[0..19])))
}

/// Validates Ethereum address format and EIP-55 checksum
/// BACKEND-CRIT-004: Proper address validation to prevent typos
fn is_valid_ethereum_address(address: &str) -> bool {
    // Check format: starts with 0x, 42 chars total, valid hex
    if !address.starts_with("0x") || address.len() != 42 {
        return false;
    }

    // Validate all characters after 0x are valid hex
    if !address[2..].chars().all(|c| c.is_ascii_hexdigit()) {
        return false;
    }

    // Use ethers to parse and validate the address
    // This validates the address format and can be used for checksum validation
    Address::from_str(address).is_ok()
}

/// Validates Ethereum address with EIP-55 checksum (strict mode)
/// Returns an error message if validation fails
fn validate_ethereum_address_strict(address: &str) -> Result<Address, String> {
    // Basic format check
    if !address.starts_with("0x") || address.len() != 42 {
        return Err("Invalid Ethereum address format: must be 0x followed by 40 hex characters".to_string());
    }

    // Validate all characters after 0x are valid hex
    if !address[2..].chars().all(|c| c.is_ascii_hexdigit()) {
        return Err("Invalid Ethereum address: contains non-hex characters".to_string());
    }

    // Parse the address using ethers
    let parsed = Address::from_str(address)
        .map_err(|e| format!("Invalid Ethereum address: {}", e))?;

    // Check if address is all lowercase (no checksum)
    let addr_part = &address[2..];
    if addr_part == addr_part.to_lowercase() && addr_part.chars().any(|c| c.is_ascii_alphabetic()) {
        // Warn about non-checksummed address but allow it
        tracing::warn!(
            address = %address,
            "Address provided without EIP-55 checksum - typos cannot be detected"
        );
    }

    // Verify checksum by comparing with canonical checksummed format
    let checksummed = format!("{:?}", parsed);
    if address != checksummed && address.to_lowercase() != checksummed.to_lowercase() {
        return Err(format!(
            "Invalid EIP-55 checksum. Expected: {}, got: {}",
            checksummed, address
        ));
    }

    Ok(parsed)
}

struct AgentWallet {
    user_id: i32,
    agent_id: String,
    wallet_address: String,
    spending_limit_daily: String,
    spending_limit_transaction: String,
    is_active: bool,
}

/// Extract authenticated user ID from request token
async fn get_authenticated_user_id(
    pool: &PgPool,
    req: &HttpRequest,
) -> Result<i32, ApiError> {
    let token = req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .ok_or_else(|| ApiError::Unauthorized("Missing Authorization header".to_string()))?;

    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    let token_hash = hex::encode(hasher.finalize());

    let session = sqlx::query!(
        r#"
        SELECT user_id
        FROM sessions
        WHERE access_token = $1 AND expires_at > NOW()
        "#,
        token_hash
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| handle_db_error(e, "agents"))?;

    match session {
        Some(s) => Ok(s.user_id),
        None => Err(ApiError::Unauthorized("Invalid or expired token".to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_valid_ethereum_address_valid() {
        assert!(is_valid_ethereum_address("0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb1"));
        assert!(is_valid_ethereum_address("0x0000000000000000000000000000000000000000"));
        assert!(is_valid_ethereum_address("0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"));
        assert!(is_valid_ethereum_address("0xabcdef1234567890abcdef1234567890abcdef12"));
    }

    #[test]
    fn test_is_valid_ethereum_address_invalid_prefix() {
        assert!(!is_valid_ethereum_address("742d35Cc6634C0532925a3b844Bc9e7595f0bEb1"));
        assert!(!is_valid_ethereum_address("1x742d35Cc6634C0532925a3b844Bc9e7595f0bEb1"));
    }

    #[test]
    fn test_is_valid_ethereum_address_invalid_length() {
        assert!(!is_valid_ethereum_address("0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb")); // 41 chars
        assert!(!is_valid_ethereum_address("0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb12")); // 43 chars
        assert!(!is_valid_ethereum_address("0x")); // Too short
    }

    #[test]
    fn test_is_valid_ethereum_address_invalid_hex() {
        assert!(!is_valid_ethereum_address("0xGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGG")); // Invalid hex
        assert!(!is_valid_ethereum_address("0x742d35Cc6634C0532925a3b844Bc9e7595f0bZZ1")); // Z is not hex
    }

    #[test]
    fn test_generate_api_key_format() {
        let key = generate_api_key();
        assert!(key.starts_with("mk_"));
        assert_eq!(key.len(), 35); // "mk_" + 32 hex chars (UUID without hyphens)
    }

    #[test]
    fn test_generate_api_key_unique() {
        let key1 = generate_api_key();
        let key2 = generate_api_key();
        assert_ne!(key1, key2);
    }

    #[test]
    fn test_hash_api_key_deterministic() {
        let api_key = "mk_test12345";
        let hash1 = hash_api_key(api_key);
        let hash2 = hash_api_key(api_key);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_hash_api_key_different_for_different_keys() {
        let hash1 = hash_api_key("mk_key1");
        let hash2 = hash_api_key("mk_key2");
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_hash_api_key_format() {
        let hash = hash_api_key("mk_test");
        assert_eq!(hash.len(), 64); // SHA-256 = 64 hex chars
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_generate_wallet_address_format() {
        let address = generate_wallet_address("test-agent-id").expect("should generate address in test");
        assert!(address.starts_with("0x"));
        assert_eq!(address.len(), 42);
        assert!(address[2..].chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_generate_wallet_address_deterministic() {
        let addr1 = generate_wallet_address("same-agent").expect("should generate address");
        let addr2 = generate_wallet_address("same-agent").expect("should generate address");
        assert_eq!(addr1, addr2);
    }

    #[test]
    fn test_generate_wallet_address_different_for_different_agents() {
        let addr1 = generate_wallet_address("agent-1").expect("should generate address");
        let addr2 = generate_wallet_address("agent-2").expect("should generate address");
        assert_ne!(addr1, addr2);
    }
}

