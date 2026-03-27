-- Multi-tenancy: tenants table and tenant_id columns
-- Phase C.1: Each institution gets an isolated tenant context

CREATE TABLE IF NOT EXISTS tenants (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    legal_entity VARCHAR(255) NOT NULL,
    jurisdiction VARCHAR(10) NOT NULL,  -- ISO 3166-1 alpha-2, e.g. "DE", "GB"
    status VARCHAR(20) NOT NULL DEFAULT 'ACTIVE'
        CHECK (status IN ('ACTIVE', 'SUSPENDED', 'PENDING_KYB', 'TERMINATED')),
    -- Per-tenant custody and chain configuration overrides (JSONB for flexibility)
    custody_config JSONB NOT NULL DEFAULT '{}',
    chain_config JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Add tenant_id to users (nullable for migration compatibility)
ALTER TABLE users
    ADD COLUMN IF NOT EXISTS tenant_id UUID REFERENCES tenants(id) ON DELETE SET NULL;

-- Add tenant_id to operations
ALTER TABLE operations
    ADD COLUMN IF NOT EXISTS tenant_id UUID REFERENCES tenants(id) ON DELETE SET NULL;

-- Add tenant_id to stablecoins
ALTER TABLE stablecoins
    ADD COLUMN IF NOT EXISTS tenant_id UUID REFERENCES tenants(id) ON DELETE SET NULL;

-- Indexes
CREATE INDEX IF NOT EXISTS idx_tenants_status ON tenants(status);
CREATE INDEX IF NOT EXISTS idx_users_tenant_id ON users(tenant_id);
CREATE INDEX IF NOT EXISTS idx_operations_tenant_id ON operations(tenant_id);
