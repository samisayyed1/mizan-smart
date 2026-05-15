INSERT OR IGNORE INTO app_settings (setting_key, setting_value)
VALUES ('shariah_mode_enabled', 'false');

CREATE TABLE shariah_screening_profiles (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    debt_threshold TEXT NOT NULL,
    liquid_assets_threshold TEXT NOT NULL,
    impure_income_threshold TEXT NOT NULL,
    is_default INTEGER NOT NULL DEFAULT 0 CHECK (is_default IN (0, 1)),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE UNIQUE INDEX idx_shariah_screening_profiles_default
ON shariah_screening_profiles(is_default)
WHERE is_default = 1;

CREATE TABLE asset_shariah_screening (
    id TEXT PRIMARY KEY NOT NULL,
    asset_id TEXT NOT NULL,
    profile_id TEXT NOT NULL,
    status TEXT NOT NULL CHECK (
        status IN ('compliant', 'non_compliant', 'questionable', 'unknown', 'needs_review')
    ),
    debt_ratio TEXT,
    liquid_assets_ratio TEXT,
    impure_income_ratio TEXT,
    source_citation_id TEXT,
    manual_override_reason TEXT,
    reviewed_at TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY(asset_id) REFERENCES assets(id) ON DELETE CASCADE,
    FOREIGN KEY(profile_id) REFERENCES shariah_screening_profiles(id) ON DELETE RESTRICT,
    FOREIGN KEY(source_citation_id) REFERENCES source_citations(id) ON DELETE SET NULL,
    UNIQUE(asset_id, profile_id)
);

CREATE INDEX idx_asset_shariah_screening_asset
ON asset_shariah_screening(asset_id);

CREATE INDEX idx_asset_shariah_screening_profile
ON asset_shariah_screening(profile_id);

CREATE INDEX idx_asset_shariah_screening_status
ON asset_shariah_screening(status);

INSERT OR IGNORE INTO shariah_screening_profiles (
    id,
    name,
    debt_threshold,
    liquid_assets_threshold,
    impure_income_threshold,
    is_default,
    created_at,
    updated_at
) VALUES (
    'system_default_shariah_screening_profile',
    'Default screening profile',
    '0.30',
    '0.30',
    '0.05',
    1,
    '2026-05-15T00:00:00Z',
    '2026-05-15T00:00:00Z'
);
