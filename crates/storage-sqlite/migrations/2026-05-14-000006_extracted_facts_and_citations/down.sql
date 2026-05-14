DROP INDEX IF EXISTS idx_valuations_source_citation_id;
DROP INDEX IF EXISTS idx_valuations_asset_date;
DROP INDEX IF EXISTS idx_valuations_currency;
DROP INDEX IF EXISTS idx_valuations_source_type;

ALTER TABLE valuations RENAME TO valuations_with_source_citations;

CREATE TABLE valuations (
    id TEXT PRIMARY KEY NOT NULL,
    asset_id TEXT NOT NULL REFERENCES assets(id) ON DELETE CASCADE,
    valuation_date TEXT NOT NULL,
    value_native TEXT NOT NULL,
    currency TEXT NOT NULL,
    source_type TEXT NOT NULL CHECK (
        source_type IN ('manual', 'market', 'document', 'import', 'web_evidence', 'calculated')
    ),
    source_id TEXT,
    confidence TEXT,
    notes TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

INSERT INTO valuations (
    id,
    asset_id,
    valuation_date,
    value_native,
    currency,
    source_type,
    source_id,
    confidence,
    notes,
    created_at,
    updated_at
)
SELECT
    id,
    asset_id,
    valuation_date,
    value_native,
    currency,
    source_type,
    source_id,
    confidence,
    notes,
    created_at,
    updated_at
FROM valuations_with_source_citations;

DROP TABLE valuations_with_source_citations;

CREATE INDEX idx_valuations_asset_date ON valuations(asset_id, valuation_date);
CREATE INDEX idx_valuations_currency ON valuations(currency);
CREATE INDEX idx_valuations_source_type ON valuations(source_type);

DROP TABLE IF EXISTS source_citations;
DROP TABLE IF EXISTS extracted_facts;
