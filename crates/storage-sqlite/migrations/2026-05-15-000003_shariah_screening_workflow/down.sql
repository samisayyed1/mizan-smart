DROP INDEX IF EXISTS idx_shariah_screening_audit_asset_profile;
DROP INDEX IF EXISTS idx_shariah_screening_audit_screening;
DROP TABLE IF EXISTS shariah_screening_audit_log;

ALTER TABLE asset_shariah_screening DROP COLUMN notes;
