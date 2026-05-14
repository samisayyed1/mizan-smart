DROP INDEX IF EXISTS idx_private_distributions_asset_date;
DROP INDEX IF EXISTS idx_capital_calls_due_date;
DROP INDEX IF EXISTS idx_capital_calls_asset_status;
DROP INDEX IF EXISTS idx_private_investment_valuations_asset_date;
DROP INDEX IF EXISTS idx_private_investments_manager;

DROP TABLE IF EXISTS private_distributions;
DROP TABLE IF EXISTS capital_calls;
DROP TABLE IF EXISTS private_investment_valuations;
DROP TABLE IF EXISTS private_investments;
