-- Create price_history table
CREATE TABLE IF NOT EXISTS price_history (
    id BIGSERIAL PRIMARY KEY,
    currency_pair VARCHAR(20) NOT NULL,
    price NUMERIC(28, 18) NOT NULL,
    source VARCHAR(50) NOT NULL DEFAULT 'chainlink',
    is_stale BOOLEAN NOT NULL DEFAULT FALSE,
    round_id NUMERIC(20, 0),
    timestamp TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Indexes for efficient queries
CREATE INDEX IF NOT EXISTS idx_price_history_pair_timestamp 
    ON price_history(currency_pair, timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_price_history_timestamp 
    ON price_history(timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_price_history_source 
    ON price_history(source);

-- Partition by month for better performance (optional, for production)
-- CREATE INDEX idx_price_history_pair_month 
--     ON price_history(currency_pair, date_trunc('month', timestamp));

