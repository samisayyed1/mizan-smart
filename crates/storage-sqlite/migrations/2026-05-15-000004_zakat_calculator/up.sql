CREATE TABLE zakat_snapshots (
    id TEXT PRIMARY KEY NOT NULL,
    snapshot_date TEXT NOT NULL,
    base_currency TEXT NOT NULL CHECK (length(base_currency) = 3),
    total_zakatable_assets TEXT NOT NULL,
    deductible_liabilities TEXT NOT NULL,
    net_zakatable_wealth TEXT NOT NULL,
    nisab_value TEXT NOT NULL,
    zakat_due TEXT NOT NULL,
    notes TEXT,
    created_at TEXT NOT NULL
);

CREATE TABLE zakat_lines (
    id TEXT PRIMARY KEY NOT NULL,
    snapshot_id TEXT NOT NULL,
    asset_id TEXT,
    category TEXT NOT NULL CHECK (length(trim(category)) > 0),
    amount TEXT NOT NULL,
    included INTEGER NOT NULL DEFAULT 1 CHECK (included IN (0, 1)),
    explanation TEXT NOT NULL CHECK (length(trim(explanation)) > 0),
    source_citation_id TEXT,
    FOREIGN KEY(snapshot_id) REFERENCES zakat_snapshots(id) ON DELETE CASCADE,
    FOREIGN KEY(asset_id) REFERENCES assets(id) ON DELETE SET NULL,
    FOREIGN KEY(source_citation_id) REFERENCES source_citations(id) ON DELETE SET NULL
);

CREATE INDEX idx_zakat_snapshots_date
ON zakat_snapshots(snapshot_date);

CREATE INDEX idx_zakat_lines_snapshot
ON zakat_lines(snapshot_id);

CREATE INDEX idx_zakat_lines_asset
ON zakat_lines(asset_id);

CREATE INDEX idx_zakat_lines_source_citation
ON zakat_lines(source_citation_id);
