//! x402 Agent payment handlers

use crate::error::ApiError;
use crate::state::AppState;
use actix_web::{web, HttpResponse};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
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
    req: web::Json<CreateAgentRequest>,
) -> Result<HttpResponse, ApiError> {
    tracing::info!(
        user_id = req.user_id,
        agent_name = %req.agent_name,
        "Creating agent wallet"
    );

    // Verify user exists and is KYC approved
    let user = sqlx::query!("SELECT kyc_status FROM users WHERE id = $1", req.user_id)
        .fetch_optional(state.db_pool.as_ref())
        .await
        .map_err(|e| ApiError::InternalError(format!("Database error: {}", e)))?;

    let user = match user {
        Some(u) => u,
        None => return Err(ApiError::NotFound("User not found".to_string())),
    };

    if user.kyc_status != "APPROVED" {
        return Err(ApiError::Forbidden(
            "KYC approval required to create agent wallets".to_string(),
        ));
    }

    // Generate agent ID and API key
    let agent_id = format!("agent_{}", Uuid::new_v4().to_string().replace("-", ""));
    let api_key = generate_api_key();
    let api_key_hash = hash_api_key(&api_key);

    // Generate deterministic wallet address (simplified - in production use proper key derivation)
    let wallet_address = generate_wallet_address(&agent_id);

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
pub async fn agent_pay(
    state: web::Data<Arc<AppState>>,
    req: web::Json<AgentPaymentRequest>,
) -> Result<HttpResponse, ApiError> {
    tracing::info!(
        agent_id = %req.agent_id,
        recipient = %req.recipient,
        amount = %req.amount,
        currency = %req.currency,
        "Agent payment request"
    );

    // Verify API key
    let agent = verify_agent_api_key(state.db_pool.as_ref(), &req.agent_id, &req.api_key).await?;

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

    // TODO: Execute on-chain transaction via smart contract
    // For now, mark as completed (mock execution)
    let tx_hash = format!("0x{}", Uuid::new_v4().to_string().replace("-", ""));

    sqlx::query!(
        "UPDATE agent_transactions SET status = 'COMPLETED', transaction_hash = $1 WHERE id = $2",
        tx_hash,
        transaction.id
    )
    .execute(state.db_pool.as_ref())
    .await
    .ok();

    tracing::info!(
        transaction_id = transaction.id,
        tx_hash = %tx_hash,
        "Agent payment executed"
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
    user_id: web::Path<i32>,
) -> Result<HttpResponse, ApiError> {
    let user_id = user_id.into_inner();

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
    .map_err(|e| ApiError::InternalError(format!("Database error: {}", e)))?;

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
    agent_id: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let agent_id = agent_id.into_inner();

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
    .map_err(|e| ApiError::InternalError(format!("Database error: {}", e)))?;

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
        SELECT agent_id, wallet_address, spending_limit_daily, 
               spending_limit_transaction, is_active
        FROM agent_wallets
        WHERE agent_id = $1 AND api_key_hash = $2
        "#,
        agent_id,
        api_key_hash
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| ApiError::InternalError(format!("Database error: {}", e)))?;

    match agent {
        Some(a) => Ok(AgentWallet {
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
    .map_err(|e| ApiError::InternalError(format!("Database error: {}", e)))?;

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
    let mut hasher = Sha256::new();
    hasher.update(api_key.as_bytes());
    hasher.update(b"meridian_agent_salt");
    format!("{:x}", hasher.finalize())
}

fn generate_wallet_address(agent_id: &str) -> String {
    // In production, derive proper Ethereum address
    // For now, generate deterministic mock address
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(agent_id.as_bytes());
    let hash = hasher.finalize();
    format!("0x{}", hex::encode(&hash[0..20]))
}

fn is_valid_ethereum_address(address: &str) -> bool {
    address.starts_with("0x") && address.len() == 42
}

struct AgentWallet {
    agent_id: String,
    wallet_address: String,
    spending_limit_daily: String,
    spending_limit_transaction: String,
    is_active: bool,
}

