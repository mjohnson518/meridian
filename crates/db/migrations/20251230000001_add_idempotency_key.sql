-- CRIT-003: Add idempotency key support to operations table
-- Prevents duplicate mint/burn operations from double-submission

-- Add idempotency_key column to operations table
ALTER TABLE operations
ADD COLUMN IF NOT EXISTS idempotency_key VARCHAR(128);

-- Add partial unique index for idempotency key (unique per user/operation_type for 24 hours)
-- Partial index only covers non-null keys
CREATE UNIQUE INDEX IF NOT EXISTS idx_operations_idempotency
ON operations(user_id, operation_type, idempotency_key)
WHERE idempotency_key IS NOT NULL;

-- Add index for efficient idempotency lookups (with created_at for TTL filtering)
CREATE INDEX IF NOT EXISTS idx_operations_idempotency_lookup
ON operations(user_id, idempotency_key, operation_type, created_at)
WHERE idempotency_key IS NOT NULL;

COMMENT ON COLUMN operations.idempotency_key IS
'Client-provided unique key to prevent duplicate operations. Valid for 24 hours.';
