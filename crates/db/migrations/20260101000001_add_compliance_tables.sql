-- Compliance tables: records, alerts, sanctions screenings
-- Wires the meridian-compliance crate data model into PostgreSQL

-- Add country_code to users for compliance lookups (from KYC application_data)
ALTER TABLE users ADD COLUMN IF NOT EXISTS country_code VARCHAR(2);

-- Compliance records: per-customer compliance state
CREATE TABLE IF NOT EXISTS compliance_records (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    status VARCHAR(20) NOT NULL DEFAULT 'NOT_STARTED'
        CHECK (status IN ('NOT_STARTED', 'PENDING', 'APPROVED', 'REJECTED', 'SUSPENDED', 'REVIEW_REQUIRED')),
    risk_score SMALLINT NOT NULL DEFAULT 0 CHECK (risk_score >= 0 AND risk_score <= 100),
    risk_level VARCHAR(12) NOT NULL DEFAULT 'MEDIUM'
        CHECK (risk_level IN ('LOW', 'MEDIUM', 'HIGH', 'PROHIBITED')),
    frameworks JSONB NOT NULL DEFAULT '[]',
    edd_required BOOLEAN NOT NULL DEFAULT FALSE,
    kyc_verified_at TIMESTAMP WITH TIME ZONE,
    kyc_expires_at TIMESTAMP WITH TIME ZONE,
    last_review_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    next_review_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT (NOW() + INTERVAL '1 year'),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    UNIQUE (user_id)
);

-- Compliance alerts: raised by monitoring or manual review
CREATE TABLE IF NOT EXISTS compliance_alerts (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    operation_id INTEGER REFERENCES operations(id),
    alert_type VARCHAR(50) NOT NULL,
    risk_score SMALLINT NOT NULL CHECK (risk_score >= 0 AND risk_score <= 100),
    flags JSONB NOT NULL DEFAULT '[]',
    required_actions JSONB NOT NULL DEFAULT '[]',
    status VARCHAR(20) NOT NULL DEFAULT 'OPEN'
        CHECK (status IN ('OPEN', 'REVIEWING', 'CLEARED', 'ESCALATED', 'SAR_FILED')),
    reviewer_id INTEGER REFERENCES users(id),
    reviewed_at TIMESTAMP WITH TIME ZONE,
    notes TEXT,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Sanctions screenings: immutable audit trail of every screening call
CREATE TABLE IF NOT EXISTS sanctions_screenings (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    operation_id INTEGER REFERENCES operations(id),
    screened_name TEXT NOT NULL,
    screened_address VARCHAR(42),
    has_match BOOLEAN NOT NULL,
    confidence SMALLINT NOT NULL DEFAULT 0 CHECK (confidence >= 0 AND confidence <= 100),
    matched_lists JSONB NOT NULL DEFAULT '[]',
    match_details JSONB NOT NULL DEFAULT '[]',
    screened_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
    -- Immutable: no UPDATE or DELETE allowed (enforced by trigger below)
);

-- Prevent mutations to sanctions_screenings (immutable audit trail)
CREATE OR REPLACE FUNCTION prevent_sanctions_screening_mutation()
RETURNS TRIGGER AS $$
BEGIN
    RAISE EXCEPTION 'sanctions_screenings records are immutable';
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS enforce_sanctions_screening_immutability ON sanctions_screenings;
CREATE TRIGGER enforce_sanctions_screening_immutability
    BEFORE UPDATE OR DELETE ON sanctions_screenings
    FOR EACH ROW EXECUTE FUNCTION prevent_sanctions_screening_mutation();

-- Auto-updated timestamps
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS update_compliance_records_updated_at ON compliance_records;
CREATE TRIGGER update_compliance_records_updated_at
    BEFORE UPDATE ON compliance_records
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

DROP TRIGGER IF EXISTS update_compliance_alerts_updated_at ON compliance_alerts;
CREATE TRIGGER update_compliance_alerts_updated_at
    BEFORE UPDATE ON compliance_alerts
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Performance indexes
CREATE INDEX IF NOT EXISTS idx_compliance_records_user_id ON compliance_records(user_id);
CREATE INDEX IF NOT EXISTS idx_compliance_records_status ON compliance_records(status);
CREATE INDEX IF NOT EXISTS idx_compliance_records_risk_level ON compliance_records(risk_level);
CREATE INDEX IF NOT EXISTS idx_compliance_alerts_user_id ON compliance_alerts(user_id);
CREATE INDEX IF NOT EXISTS idx_compliance_alerts_status ON compliance_alerts(status);
CREATE INDEX IF NOT EXISTS idx_sanctions_screenings_user_id ON sanctions_screenings(user_id);
CREATE INDEX IF NOT EXISTS idx_sanctions_screenings_has_match ON sanctions_screenings(has_match) WHERE has_match = TRUE;
