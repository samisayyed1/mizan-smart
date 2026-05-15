DROP INDEX IF EXISTS idx_asset_shariah_screening_status;
DROP INDEX IF EXISTS idx_asset_shariah_screening_profile;
DROP INDEX IF EXISTS idx_asset_shariah_screening_asset;
DROP TABLE IF EXISTS asset_shariah_screening;
DROP INDEX IF EXISTS idx_shariah_screening_profiles_default;
DROP TABLE IF EXISTS shariah_screening_profiles;
DELETE FROM app_settings WHERE setting_key = 'shariah_mode_enabled';
