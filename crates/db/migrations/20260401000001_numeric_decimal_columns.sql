-- Migrate decimal columns from TEXT back to NUMERIC(38,18)
-- Resolves the temporary workaround introduced in 20251001000005_convert_decimals_to_text.sql
-- SQLx rust_decimal feature now enabled; direct NUMERIC binding is supported.

-- Drop TEXT check constraints
ALTER TABLE price_history DROP CONSTRAINT IF EXISTS price_valid_decimal;
ALTER TABLE stablecoins DROP CONSTRAINT IF EXISTS total_supply_valid_decimal;
ALTER TABLE stablecoins DROP CONSTRAINT IF EXISTS total_reserve_valid_decimal;

-- Convert price_history columns
ALTER TABLE price_history
    ALTER COLUMN price TYPE NUMERIC(38, 18) USING price::NUMERIC;

ALTER TABLE price_history
    ALTER COLUMN round_id TYPE NUMERIC(38, 18) USING round_id::NUMERIC;

-- Convert stablecoins columns
ALTER TABLE stablecoins
    ALTER COLUMN total_supply TYPE NUMERIC(38, 18) USING total_supply::NUMERIC;

ALTER TABLE stablecoins
    ALTER COLUMN total_reserve_value TYPE NUMERIC(38, 18) USING total_reserve_value::NUMERIC;
