-- Create stablecoins table
CREATE TABLE IF NOT EXISTS stablecoins (
    id UUID PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    symbol VARCHAR(20) NOT NULL,
    contract_address VARCHAR(42) UNIQUE,
    basket_id UUID REFERENCES baskets(id) ON DELETE SET NULL,
    chain_id INTEGER NOT NULL,
    total_supply NUMERIC(28, 18) DEFAULT 0,
    total_reserve_value NUMERIC(28, 2) DEFAULT 0,
    status VARCHAR(50) NOT NULL DEFAULT 'deploying' CHECK (status IN ('deploying', 'active', 'paused', 'deprecated')),
    deployed_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Indexes for efficient lookups
CREATE INDEX IF NOT EXISTS idx_stablecoins_contract_address 
    ON stablecoins(contract_address);
CREATE INDEX IF NOT EXISTS idx_stablecoins_basket_id 
    ON stablecoins(basket_id);
CREATE INDEX IF NOT EXISTS idx_stablecoins_chain_id 
    ON stablecoins(chain_id);
CREATE INDEX IF NOT EXISTS idx_stablecoins_status 
    ON stablecoins(status);

-- Updated timestamp trigger
CREATE TRIGGER update_stablecoins_updated_at
    BEFORE UPDATE ON stablecoins
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

