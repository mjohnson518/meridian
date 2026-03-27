-- API key management for machine-to-machine authentication
-- Phase C.2: Enables institutional clients to authenticate without user sessions

CREATE TABLE IF NOT EXISTS api_keys (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    -- Human-readable label for the key
    name VARCHAR(255) NOT NULL,
    -- SHA-256(raw_key || API_KEY_SALT) — the raw key is returned once on creation
    key_hash VARCHAR(64) NOT NULL UNIQUE,
    -- First 12 chars of raw key for display (e.g., "mk_a1b2c3d4e5")
    key_prefix VARCHAR(12) NOT NULL,
    -- Permissions granted to this key (array of strings)
    -- e.g., ["mint", "burn", "reserves:read", "webhooks:manage"]
    permissions JSONB NOT NULL DEFAULT '[]',
    -- Rate limit (requests per minute), 0 = use tenant default
    rate_limit_per_minute INTEGER NOT NULL DEFAULT 60,
    -- Optional expiry (NULL = never expires)
    expires_at TIMESTAMP WITH TIME ZONE,
    -- Audit fields
    last_used_at TIMESTAMP WITH TIME ZONE,
    revoked_at TIMESTAMP WITH TIME ZONE,
    created_by INTEGER REFERENCES users(id),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_api_keys_tenant_id ON api_keys(tenant_id);
CREATE INDEX IF NOT EXISTS idx_api_keys_key_hash ON api_keys(key_hash);
-- Partial index: only active (non-revoked) keys
CREATE INDEX IF NOT EXISTS idx_api_keys_active ON api_keys(key_hash)
    WHERE revoked_at IS NULL;
