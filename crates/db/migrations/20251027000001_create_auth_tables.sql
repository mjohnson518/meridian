-- Authentication and authorization tables

-- Users table
CREATE TABLE IF NOT EXISTS users (
    id SERIAL PRIMARY KEY,
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    role VARCHAR(20) NOT NULL CHECK (role IN ('ADMIN', 'TREASURY', 'COMPLIANCE', 'VIEWER')),
    organization VARCHAR(255) NOT NULL,
    wallet_address VARCHAR(42),
    kyc_status VARCHAR(20) NOT NULL DEFAULT 'NOT_STARTED' 
        CHECK (kyc_status IN ('NOT_STARTED', 'IN_PROGRESS', 'PENDING_REVIEW', 'APPROVED', 'REJECTED')),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    last_login_at TIMESTAMP WITH TIME ZONE
);

-- Sessions table
CREATE TABLE IF NOT EXISTS sessions (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    access_token VARCHAR(512) NOT NULL UNIQUE,
    refresh_token VARCHAR(512) NOT NULL UNIQUE,
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    ip_address VARCHAR(45),
    user_agent TEXT
);

-- KYC applications table
CREATE TABLE IF NOT EXISTS kyc_applications (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    application_data JSONB NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'PENDING_REVIEW'
        CHECK (status IN ('PENDING_REVIEW', 'APPROVED', 'REJECTED', 'NEEDS_INFO')),
    reviewed_by INTEGER REFERENCES users(id),
    reviewed_at TIMESTAMP WITH TIME ZONE,
    rejection_reason TEXT,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Operations (mint/burn) table
CREATE TABLE IF NOT EXISTS operations (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id),
    operation_type VARCHAR(10) NOT NULL CHECK (operation_type IN ('MINT', 'BURN')),
    currency VARCHAR(3) NOT NULL,
    amount TEXT NOT NULL, -- TEXT for precise decimal storage
    usd_value TEXT NOT NULL,
    bond_requirement TEXT,
    fees_charged TEXT,
    status VARCHAR(20) NOT NULL DEFAULT 'PENDING'
        CHECK (status IN ('PENDING', 'BOND_PURCHASE', 'SETTLEMENT', 'COMPLETED', 'FAILED', 'CANCELLED')),
    transaction_hash VARCHAR(66),
    settlement_date TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Agent wallets table (x402)
CREATE TABLE IF NOT EXISTS agent_wallets (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    agent_id VARCHAR(64) UNIQUE NOT NULL,
    agent_name VARCHAR(255),
    wallet_address VARCHAR(42) NOT NULL,
    api_key_hash VARCHAR(255) NOT NULL,
    spending_limit_daily TEXT NOT NULL, -- TEXT for precise decimal
    spending_limit_transaction TEXT NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Agent transactions table (x402)
CREATE TABLE IF NOT EXISTS agent_transactions (
    id SERIAL PRIMARY KEY,
    agent_id VARCHAR(64) NOT NULL REFERENCES agent_wallets(agent_id),
    currency VARCHAR(3) NOT NULL,
    amount TEXT NOT NULL,
    recipient VARCHAR(42) NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'PENDING'
        CHECK (status IN ('PENDING', 'COMPLETED', 'FAILED', 'REJECTED')),
    transaction_hash VARCHAR(66),
    failure_reason TEXT,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Compliance flags table
CREATE TABLE IF NOT EXISTS compliance_flags (
    id SERIAL PRIMARY KEY,
    transaction_id INTEGER,
    transaction_type VARCHAR(20) NOT NULL,
    risk_score INTEGER NOT NULL CHECK (risk_score >= 0 AND risk_score <= 100),
    flag_reason TEXT NOT NULL,
    flagged_by VARCHAR(50) NOT NULL DEFAULT 'AUTOMATED',
    reviewed_by INTEGER REFERENCES users(id),
    status VARCHAR(20) NOT NULL DEFAULT 'PENDING'
        CHECK (status IN ('PENDING', 'CLEARED', 'SAR_FILED', 'ESCALATED')),
    sar_filed_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    reviewed_at TIMESTAMP WITH TIME ZONE
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);
CREATE INDEX IF NOT EXISTS idx_sessions_user_id ON sessions(user_id);
CREATE INDEX IF NOT EXISTS idx_sessions_access_token ON sessions(access_token);
CREATE INDEX IF NOT EXISTS idx_kyc_user_id ON kyc_applications(user_id);
CREATE INDEX IF NOT EXISTS idx_operations_user_id ON operations(user_id);
CREATE INDEX IF NOT EXISTS idx_operations_status ON operations(status);
CREATE INDEX IF NOT EXISTS idx_agent_wallets_user_id ON agent_wallets(user_id);
CREATE INDEX IF NOT EXISTS idx_agent_wallets_agent_id ON agent_wallets(agent_id);
CREATE INDEX IF NOT EXISTS idx_agent_transactions_agent_id ON agent_transactions(agent_id);
CREATE INDEX IF NOT EXISTS idx_compliance_flags_status ON compliance_flags(status);

