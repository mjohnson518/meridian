-- HIGH-014 + HIGH-026: Add missing performance indexes
-- These indexes optimize common query patterns identified in the production readiness audit

-- HIGH-014: Index for operations.created_at
-- Optimizes: Queries filtering operations by time (daily summaries, recent operations)
CREATE INDEX IF NOT EXISTS idx_operations_created_at
ON operations(created_at);

-- HIGH-026: Composite index for session auth lookups
-- Optimizes: Session validation queries that check token AND expiry together
-- Query pattern: WHERE access_token = $1 AND expires_at > NOW()
CREATE INDEX IF NOT EXISTS idx_sessions_auth_lookup
ON sessions(access_token, expires_at);

-- Index for agent_transactions by agent_id and created_at
-- Optimizes: get_daily_spent queries (WHERE agent_id = $1 AND created_at > NOW() - '24 hours')
CREATE INDEX IF NOT EXISTS idx_agent_transactions_daily_spent
ON agent_transactions(agent_id, created_at)
WHERE status IN ('PENDING', 'COMPLETED');

-- Index for operations by user_id and created_at
-- Optimizes: User history queries ordered by time
CREATE INDEX IF NOT EXISTS idx_operations_user_history
ON operations(user_id, created_at DESC);

-- Index for KYC applications by user and status
-- Optimizes: Status lookups for specific users
CREATE INDEX IF NOT EXISTS idx_kyc_applications_user_status
ON kyc_applications(user_id, status);
