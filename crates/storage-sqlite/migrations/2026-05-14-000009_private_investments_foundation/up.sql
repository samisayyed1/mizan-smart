CREATE TABLE private_investments (
    asset_id TEXT PRIMARY KEY NOT NULL REFERENCES assets(id) ON DELETE CASCADE,
    manager TEXT NOT NULL,
    strategy TEXT NOT NULL,
    vintage_year INTEGER,
    commitment_amount TEXT NOT NULL,
    commitment_currency TEXT NOT NULL,
    inception_date TEXT,
    notes TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE private_investment_valuations (
    id TEXT PRIMARY KEY NOT NULL,
    asset_id TEXT NOT NULL REFERENCES assets(id) ON DELETE CASCADE,
    valuation_date TEXT NOT NULL,
    nav TEXT NOT NULL,
    currency TEXT NOT NULL,
    source_citation_id TEXT REFERENCES source_citations(id) ON DELETE SET NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE capital_calls (
    id TEXT PRIMARY KEY NOT NULL,
    asset_id TEXT NOT NULL REFERENCES assets(id) ON DELETE CASCADE,
    notice_date TEXT NOT NULL,
    due_date TEXT NOT NULL,
    amount TEXT NOT NULL,
    currency TEXT NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('expected', 'due', 'paid', 'cancelled')),
    source_citation_id TEXT REFERENCES source_citations(id) ON DELETE SET NULL,
    notes TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE private_distributions (
    id TEXT PRIMARY KEY NOT NULL,
    asset_id TEXT NOT NULL REFERENCES assets(id) ON DELETE CASCADE,
    distribution_date TEXT NOT NULL,
    amount TEXT NOT NULL,
    currency TEXT NOT NULL,
    recallable INTEGER NOT NULL DEFAULT 0 CHECK (recallable IN (0, 1)),
    source_citation_id TEXT REFERENCES source_citations(id) ON DELETE SET NULL,
    notes TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX idx_private_investments_manager ON private_investments(manager);
CREATE INDEX idx_private_investment_valuations_asset_date
    ON private_investment_valuations(asset_id, valuation_date);
CREATE INDEX idx_capital_calls_asset_status ON capital_calls(asset_id, status);
CREATE INDEX idx_capital_calls_due_date ON capital_calls(due_date);
CREATE INDEX idx_private_distributions_asset_date
    ON private_distributions(asset_id, distribution_date);
