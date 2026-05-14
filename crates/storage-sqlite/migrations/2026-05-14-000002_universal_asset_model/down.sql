-- Reverse the universal asset model foundation migration. Tables are
-- dropped in reverse dependency order; the assets.classification column
-- is dropped last. Cascading FKs handle any orphaned typed-detail rows
-- left over from broken installs.

DROP TABLE IF EXISTS asset_liability_details;
DROP TABLE IF EXISTS asset_collectible_details;
DROP TABLE IF EXISTS asset_business_details;
DROP TABLE IF EXISTS asset_commodity_details;
DROP TABLE IF EXISTS asset_insurance_details;
DROP TABLE IF EXISTS asset_private_investment_details;
DROP TABLE IF EXISTS asset_real_estate_details;
DROP TABLE IF EXISTS asset_fixed_income_details;
DROP TABLE IF EXISTS asset_public_market_details;

DROP INDEX IF EXISTS idx_valuations_source_type;
DROP INDEX IF EXISTS idx_valuations_currency;
DROP INDEX IF EXISTS idx_valuations_asset_date;
DROP TABLE IF EXISTS valuations;

DROP INDEX IF EXISTS idx_assets_classification;
ALTER TABLE assets DROP COLUMN classification;
