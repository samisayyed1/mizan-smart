CREATE TABLE purification_entries (
    id TEXT PRIMARY KEY NOT NULL,
    asset_id TEXT NOT NULL,
    period_start TEXT NOT NULL,
    period_end TEXT NOT NULL,
    total_impure_income TEXT,
    outstanding_shares TEXT,
    user_shares TEXT,
    dividend_received TEXT,
    impure_income_ratio TEXT,
    purification_amount TEXT NOT NULL,
    calculation_method TEXT NOT NULL CHECK (
        calculation_method IN ('impure_income_per_share', 'dividend_ratio', 'needs_review')
    ),
    status TEXT NOT NULL CHECK (status IN ('calculated', 'paid', 'waived')),
    source_citation_id TEXT,
    notes TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY(asset_id) REFERENCES assets(id) ON DELETE CASCADE,
    FOREIGN KEY(source_citation_id) REFERENCES source_citations(id) ON DELETE SET NULL
);

CREATE INDEX idx_purification_entries_asset
ON purification_entries(asset_id);

CREATE INDEX idx_purification_entries_period
ON purification_entries(period_start, period_end);

CREATE INDEX idx_purification_entries_status
ON purification_entries(status);

CREATE INDEX idx_purification_entries_source_citation
ON purification_entries(source_citation_id);
