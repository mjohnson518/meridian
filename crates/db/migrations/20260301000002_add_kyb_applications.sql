-- KYB (Know Your Business) applications for institutional onboarding
-- Phase E.3: Corporate entity verification required for MiCA compliance

CREATE TABLE IF NOT EXISTS kyb_applications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,

    -- Entity identity
    legal_name VARCHAR(255) NOT NULL,
    trading_name VARCHAR(255),
    jurisdiction VARCHAR(2) NOT NULL,  -- ISO 3166-1 alpha-2
    registration_number VARCHAR(100) NOT NULL,
    -- LEI Code (ISO 17442, 20 chars) — mandatory for MiCA Art. 45
    lei_code VARCHAR(20),
    registered_address TEXT NOT NULL,
    business_description TEXT NOT NULL,

    -- Verification status
    status VARCHAR(30) NOT NULL DEFAULT 'PENDING_DOCUMENTS'
        CHECK (status IN (
            'PENDING_DOCUMENTS',
            'UNDER_REVIEW',
            'ADDITIONAL_INFO_REQUIRED',
            'APPROVED',
            'REJECTED',
            'SUSPENDED'
        )),
    rejection_reason TEXT,

    -- UBO declarations (stored as JSONB array)
    ubos JSONB NOT NULL DEFAULT '[]',

    -- Submitted documents metadata (actual files stored in object storage)
    documents JSONB NOT NULL DEFAULT '[]',

    -- External KYB provider reference (e.g., Comply Advantage entity ID)
    external_kyb_id VARCHAR(255),

    -- Review audit
    reviewed_by INTEGER REFERENCES users(id),
    reviewed_at TIMESTAMP WITH TIME ZONE,

    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Auto-update timestamp
DROP TRIGGER IF EXISTS update_kyb_applications_updated_at ON kyb_applications;
CREATE TRIGGER update_kyb_applications_updated_at
    BEFORE UPDATE ON kyb_applications
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Indexes
CREATE INDEX IF NOT EXISTS idx_kyb_applications_tenant_id ON kyb_applications(tenant_id);
CREATE INDEX IF NOT EXISTS idx_kyb_applications_status ON kyb_applications(status);
CREATE INDEX IF NOT EXISTS idx_kyb_applications_lei ON kyb_applications(lei_code)
    WHERE lei_code IS NOT NULL;
