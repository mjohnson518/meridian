-- Create baskets table
CREATE TABLE IF NOT EXISTS baskets (
    id UUID PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    basket_type VARCHAR(50) NOT NULL CHECK (basket_type IN ('single_currency', 'imf_sdr', 'custom_basket')),
    components JSONB NOT NULL,
    rebalance_strategy JSONB NOT NULL,
    last_rebalanced TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Index for efficient lookups
CREATE INDEX IF NOT EXISTS idx_baskets_type ON baskets(basket_type);
CREATE INDEX IF NOT EXISTS idx_baskets_created_at ON baskets(created_at DESC);

-- Updated timestamp trigger
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER update_baskets_updated_at
    BEFORE UPDATE ON baskets
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

