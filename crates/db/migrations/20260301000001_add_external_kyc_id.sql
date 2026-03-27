-- Add external_kyc_id to kyc_applications for Sumsub/Onfido tracking
-- Phase E.1: Links DB records to external KYC provider applicants

ALTER TABLE kyc_applications
    ADD COLUMN IF NOT EXISTS external_kyc_id VARCHAR(255);

-- Index for webhook lookups (provider sends applicant_id in webhook payload)
CREATE INDEX IF NOT EXISTS idx_kyc_external_id ON kyc_applications(external_kyc_id)
    WHERE external_kyc_id IS NOT NULL;
