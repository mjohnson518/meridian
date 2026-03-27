-- Webhook endpoints and delivery tracking
-- Phase C.3: Push notifications for operation.completed, reserve.attestation, etc.

CREATE TABLE IF NOT EXISTS webhooks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    -- Target URL for delivery
    url TEXT NOT NULL,
    -- Subscribed event types
    -- e.g., ["operation.completed", "reserve.attestation", "compliance.alert", "kyc.approved"]
    events TEXT[] NOT NULL,
    -- HMAC-SHA256 signing secret (stored as SHA-256 hash; raw returned once on creation)
    secret_hash VARCHAR(64) NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    -- Connection timeout in seconds (default 10s)
    timeout_secs INTEGER NOT NULL DEFAULT 10,
    created_by INTEGER REFERENCES users(id),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Delivery log — immutable record of every attempt
CREATE TABLE IF NOT EXISTS webhook_deliveries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    webhook_id UUID NOT NULL REFERENCES webhooks(id) ON DELETE CASCADE,
    event_type VARCHAR(50) NOT NULL,
    payload JSONB NOT NULL,
    -- Delivery status
    status VARCHAR(20) NOT NULL DEFAULT 'PENDING'
        CHECK (status IN ('PENDING', 'DELIVERED', 'FAILED', 'RETRYING', 'ABANDONED')),
    -- Retry tracking
    attempts INTEGER NOT NULL DEFAULT 0,
    max_attempts INTEGER NOT NULL DEFAULT 3,
    next_attempt_at TIMESTAMP WITH TIME ZONE,
    -- Response from target server
    last_response_code INTEGER,
    last_error TEXT,
    -- Timing
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    delivered_at TIMESTAMP WITH TIME ZONE
);

-- Trigger: prevent UPDATE/DELETE on deliveries (immutable audit trail)
CREATE OR REPLACE FUNCTION prevent_webhook_delivery_mutation()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'DELETE' THEN
        RAISE EXCEPTION 'webhook_deliveries rows are immutable audit records';
    END IF;
    -- Allow status/attempts/next_attempt_at updates (retry logic), but not payload changes
    IF NEW.payload IS DISTINCT FROM OLD.payload OR
       NEW.webhook_id IS DISTINCT FROM OLD.webhook_id OR
       NEW.event_type IS DISTINCT FROM OLD.event_type THEN
        RAISE EXCEPTION 'webhook_deliveries: payload, webhook_id, event_type are immutable';
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER webhook_delivery_immutable
    BEFORE UPDATE OR DELETE ON webhook_deliveries
    FOR EACH ROW EXECUTE FUNCTION prevent_webhook_delivery_mutation();

-- Indexes
CREATE INDEX IF NOT EXISTS idx_webhooks_tenant_id ON webhooks(tenant_id);
CREATE INDEX IF NOT EXISTS idx_webhooks_active ON webhooks(tenant_id) WHERE is_active = TRUE;
CREATE INDEX IF NOT EXISTS idx_webhook_deliveries_webhook_id ON webhook_deliveries(webhook_id);
CREATE INDEX IF NOT EXISTS idx_webhook_deliveries_status ON webhook_deliveries(status)
    WHERE status IN ('PENDING', 'RETRYING');
CREATE INDEX IF NOT EXISTS idx_webhook_deliveries_next_attempt ON webhook_deliveries(next_attempt_at)
    WHERE status = 'RETRYING';
