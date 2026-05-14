-- Universal asset model foundation — Phase 1 / Prompt 4 of
-- docs/mizan-smart-plan/PLAN.md.
--
-- This migration is PURELY ADDITIVE. It does not touch the existing
-- assets.kind / instrument_type columns or the data in them. Existing
-- portfolio math, holdings snapshots, quotes, and the alternative-asset
-- service all continue to work unchanged.
--
-- What this adds:
--   1. assets.classification — fine-grained "universal" asset class
--      (one of the 22 values in the spec), nullable so legacy rows are
--      unaffected. New assets created through the universal flow set
--      this column to match their typed-detail table.
--   2. valuations — the canonical per-asset point-in-time value table,
--      independent of holdings_snapshots / quotes. Used by Phase 1 P6
--      bulk valuations, Phase 1 P15 Explain-This-Number, Phase 2 P13
--      extracted facts, and Phase 5 web evidence approvals.
--   3. Nine typed extension tables, each 1:1 with assets(id), ON
--      DELETE CASCADE. Detail columns are nullable so partial fills
--      work (incomplete-setup warnings are surfaced by the alert
--      engine, not enforced in SQL).
--
-- No data is migrated from existing assets — `classification` stays
-- NULL until the universal Add Asset flow (Phase 1 P5) writes new rows.

-- ────────────────────────────────────────────────────────────────────
-- 1. assets.classification
-- ────────────────────────────────────────────────────────────────────

ALTER TABLE assets ADD COLUMN classification TEXT;

-- The CHECK constraint is added by recreating the column would require
-- a table rebuild; instead we rely on the domain layer
-- (mizan_core::universal_assets::AssetClassification) to write only
-- valid values. This matches how other free-form text columns in this
-- repo are guarded.

CREATE INDEX idx_assets_classification ON assets(classification);

-- ────────────────────────────────────────────────────────────────────
-- 2. valuations
-- ────────────────────────────────────────────────────────────────────

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

CREATE INDEX idx_valuations_asset_date ON valuations(asset_id, valuation_date);
CREATE INDEX idx_valuations_currency ON valuations(currency);
CREATE INDEX idx_valuations_source_type ON valuations(source_type);

-- ────────────────────────────────────────────────────────────────────
-- 3. Typed extension tables
-- ────────────────────────────────────────────────────────────────────

CREATE TABLE asset_public_market_details (
    asset_id TEXT PRIMARY KEY NOT NULL REFERENCES assets(id) ON DELETE CASCADE,
    sub_class TEXT CHECK (
        sub_class IS NULL OR
        sub_class IN ('public_equity', 'etf', 'mutual_fund')
    ),
    isin TEXT,
    cusip TEXT,
    figi TEXT,
    expense_ratio_bps INTEGER,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE asset_fixed_income_details (
    asset_id TEXT PRIMARY KEY NOT NULL REFERENCES assets(id) ON DELETE CASCADE,
    instrument_subtype TEXT NOT NULL CHECK (
        instrument_subtype IN (
            'bond', 'sukuk', 'treasury_bill', 'fixed_deposit',
            'cd', 'structured_note', 'other'
        )
    ),
    issuer TEXT,
    isin TEXT,
    face_value TEXT,
    currency TEXT,
    purchase_date TEXT,
    maturity_date TEXT,
    coupon_or_profit_rate TEXT,
    payment_frequency TEXT,
    day_count_convention TEXT CHECK (
        day_count_convention IS NULL OR
        day_count_convention IN ('ACT_360', 'ACT_365', 'ACT_ACT', 'THIRTY_360')
    ),
    is_sukuk INTEGER NOT NULL DEFAULT 0,
    source_citation_id TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE asset_real_estate_details (
    asset_id TEXT PRIMARY KEY NOT NULL REFERENCES assets(id) ON DELETE CASCADE,
    property_type TEXT,
    address_approximate TEXT,
    address_exact TEXT,
    size_value TEXT,
    size_unit TEXT CHECK (
        size_unit IS NULL OR size_unit IN ('sqft', 'sqm', 'acre', 'hectare')
    ),
    bedrooms INTEGER,
    purchase_date TEXT,
    purchase_price TEXT,
    purchase_currency TEXT,
    source_citation_id TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE asset_private_investment_details (
    asset_id TEXT PRIMARY KEY NOT NULL REFERENCES assets(id) ON DELETE CASCADE,
    instrument_subtype TEXT NOT NULL CHECK (
        instrument_subtype IN (
            'private_equity', 'private_credit', 'hedge_fund', 'venture_capital'
        )
    ),
    manager TEXT,
    strategy TEXT,
    vintage_year INTEGER,
    commitment_amount TEXT,
    commitment_currency TEXT,
    inception_date TEXT,
    notes TEXT,
    source_citation_id TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE asset_insurance_details (
    asset_id TEXT PRIMARY KEY NOT NULL REFERENCES assets(id) ON DELETE CASCADE,
    policy_type TEXT NOT NULL CHECK (
        policy_type IN ('insurance', 'ulip', 'pension')
    ),
    provider TEXT,
    policy_number_hash TEXT,
    start_date TEXT,
    maturity_date TEXT,
    premium_amount TEXT,
    premium_currency TEXT,
    payment_frequency TEXT,
    source_citation_id TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE asset_commodity_details (
    asset_id TEXT PRIMARY KEY NOT NULL REFERENCES assets(id) ON DELETE CASCADE,
    commodity_type TEXT NOT NULL CHECK (
        commodity_type IN ('gold', 'silver', 'platinum', 'palladium', 'other_commodity')
    ),
    weight_value TEXT,
    weight_unit TEXT CHECK (
        weight_unit IS NULL OR weight_unit IN ('g', 'oz', 'kg', 'ton')
    ),
    purity TEXT,
    storage_location TEXT,
    source_citation_id TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE asset_business_details (
    asset_id TEXT PRIMARY KEY NOT NULL REFERENCES assets(id) ON DELETE CASCADE,
    business_name TEXT,
    ownership_percent TEXT,
    legal_form TEXT,
    country TEXT,
    incorporation_date TEXT,
    notes TEXT,
    source_citation_id TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE asset_collectible_details (
    asset_id TEXT PRIMARY KEY NOT NULL REFERENCES assets(id) ON DELETE CASCADE,
    collectible_type TEXT,
    maker TEXT,
    model_reference TEXT,
    year INTEGER,
    condition TEXT CHECK (
        condition IS NULL OR
        condition IN ('mint', 'excellent', 'good', 'fair', 'poor')
    ),
    has_box INTEGER NOT NULL DEFAULT 0,
    has_papers INTEGER NOT NULL DEFAULT 0,
    source_citation_id TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE asset_liability_details (
    asset_id TEXT PRIMARY KEY NOT NULL REFERENCES assets(id) ON DELETE CASCADE,
    liability_type TEXT NOT NULL CHECK (
        liability_type IN ('mortgage', 'loan', 'credit_card', 'line_of_credit', 'other_liability')
    ),
    lender TEXT,
    principal_original TEXT,
    principal_currency TEXT,
    interest_rate TEXT,
    interest_compounding TEXT,
    start_date TEXT,
    end_date TEXT,
    linked_asset_id TEXT REFERENCES assets(id) ON DELETE SET NULL,
    source_citation_id TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
