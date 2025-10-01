-- Create audit_logs table for immutable audit trail
CREATE TABLE IF NOT EXISTS audit_logs (
    id BIGSERIAL PRIMARY KEY,
    operation VARCHAR(100) NOT NULL,
    actor VARCHAR(255),
    stablecoin_id UUID REFERENCES stablecoins(id),
    basket_id UUID REFERENCES baskets(id),
    details JSONB NOT NULL,
    timestamp TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Indexes for audit queries
CREATE INDEX IF NOT EXISTS idx_audit_logs_operation 
    ON audit_logs(operation);
CREATE INDEX IF NOT EXISTS idx_audit_logs_timestamp 
    ON audit_logs(timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_audit_logs_actor 
    ON audit_logs(actor);
CREATE INDEX IF NOT EXISTS idx_audit_logs_stablecoin_id 
    ON audit_logs(stablecoin_id);

-- Prevent deletion or modification of audit logs (immutable)
CREATE OR REPLACE FUNCTION prevent_audit_log_modification()
RETURNS TRIGGER AS $$
BEGIN
    RAISE EXCEPTION 'Audit logs are immutable and cannot be modified or deleted';
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER prevent_audit_log_update
    BEFORE UPDATE ON audit_logs
    FOR EACH ROW
    EXECUTE FUNCTION prevent_audit_log_modification();

CREATE TRIGGER prevent_audit_log_delete
    BEFORE DELETE ON audit_logs
    FOR EACH ROW
    EXECUTE FUNCTION prevent_audit_log_modification();

