-- Convert decimal columns from NUMERIC to TEXT
-- This is a temporary workaround for SQLx-Decimal type compatibility
-- TODO: Migrate back to NUMERIC when SQLx-Decimal compatibility is resolved

-- Convert price_history.price from NUMERIC to TEXT
ALTER TABLE price_history 
    ALTER COLUMN price TYPE TEXT USING price::TEXT;

-- Convert price_history.round_id from NUMERIC to TEXT
ALTER TABLE price_history 
    ALTER COLUMN round_id TYPE TEXT USING round_id::TEXT;

-- Convert stablecoins decimal columns to TEXT
ALTER TABLE stablecoins 
    ALTER COLUMN total_supply TYPE TEXT USING total_supply::TEXT;

ALTER TABLE stablecoins 
    ALTER COLUMN total_reserve_value TYPE TEXT USING total_reserve_value::TEXT;

-- Add check constraints to ensure valid decimal format
ALTER TABLE price_history 
    ADD CONSTRAINT price_valid_decimal 
    CHECK (price ~ '^-?\d+\.?\d*$');

ALTER TABLE stablecoins 
    ADD CONSTRAINT total_supply_valid_decimal 
    CHECK (total_supply ~ '^-?\d+\.?\d*$');

ALTER TABLE stablecoins 
    ADD CONSTRAINT total_reserve_valid_decimal 
    CHECK (total_reserve_value ~ '^-?\d+\.?\d*$');

