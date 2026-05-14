CREATE TABLE fixed_income_cashflows (
    id TEXT PRIMARY KEY NOT NULL,
    asset_id TEXT NOT NULL REFERENCES assets(id) ON DELETE CASCADE,
    expected_date TEXT NOT NULL,
    cashflow_type TEXT NOT NULL CHECK (
        cashflow_type IN ('coupon', 'profit', 'principal', 'maturity', 'interest')
    ),
    expected_amount TEXT NOT NULL,
    actual_amount TEXT,
    currency TEXT NOT NULL,
    status TEXT NOT NULL CHECK (
        status IN ('expected', 'received', 'missed', 'cancelled')
    ),
    source_citation_id TEXT REFERENCES source_citations(id) ON DELETE SET NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX idx_fixed_income_cashflows_asset_date
    ON fixed_income_cashflows(asset_id, expected_date);
CREATE INDEX idx_fixed_income_cashflows_status
    ON fixed_income_cashflows(status);
CREATE INDEX idx_fixed_income_cashflows_source_citation
    ON fixed_income_cashflows(source_citation_id);
