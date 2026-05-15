CREATE TABLE tax_packs (
  id TEXT PRIMARY KEY NOT NULL,
  tax_year INTEGER NOT NULL CHECK (tax_year >= 1900 AND tax_year <= 9999),
  jurisdiction TEXT NOT NULL CHECK (jurisdiction IN ('US', 'UK', 'Singapore', 'GCC', 'General')),
  base_currency TEXT NOT NULL CHECK (length(base_currency) = 3 AND base_currency = upper(base_currency)),
  status TEXT NOT NULL CHECK (status IN ('draft', 'finalized', 'exported')),
  created_at TEXT NOT NULL,
  finalized_at TEXT NULL
);

CREATE TABLE tax_pack_lines (
  id TEXT PRIMARY KEY NOT NULL,
  tax_pack_id TEXT NOT NULL REFERENCES tax_packs(id) ON DELETE CASCADE,
  category TEXT NOT NULL CHECK (category IN ('realized_gain', 'dividend', 'interest', 'coupon', 'fx', 'private_distribution', 'fee', 'other')),
  asset_id TEXT NULL REFERENCES assets(id) ON DELETE SET NULL,
  activity_id TEXT NULL REFERENCES activities(id) ON DELETE SET NULL,
  amount TEXT NOT NULL,
  currency TEXT NOT NULL CHECK (length(currency) = 3 AND currency = upper(currency)),
  taxable_date TEXT NOT NULL,
  source_citation_id TEXT NULL REFERENCES source_citations(id) ON DELETE SET NULL,
  notes TEXT NULL
);

CREATE TABLE tax_pack_missing_items (
  id TEXT PRIMARY KEY NOT NULL,
  tax_pack_id TEXT NOT NULL REFERENCES tax_packs(id) ON DELETE CASCADE,
  severity TEXT NOT NULL CHECK (severity IN ('info', 'warning')),
  message TEXT NOT NULL,
  related_activity_id TEXT NULL REFERENCES activities(id) ON DELETE SET NULL,
  related_asset_id TEXT NULL REFERENCES assets(id) ON DELETE SET NULL
);

CREATE INDEX idx_tax_packs_year_jurisdiction ON tax_packs(tax_year, jurisdiction);
CREATE INDEX idx_tax_pack_lines_pack ON tax_pack_lines(tax_pack_id, taxable_date);
CREATE INDEX idx_tax_pack_lines_activity ON tax_pack_lines(activity_id);
CREATE INDEX idx_tax_pack_missing_items_pack ON tax_pack_missing_items(tax_pack_id);
