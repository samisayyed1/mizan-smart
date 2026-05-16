CREATE TABLE manual_fee_entries (
  id TEXT PRIMARY KEY,
  fee_date TEXT NOT NULL,
  category TEXT NOT NULL CHECK (category IN (
    'broker_fees',
    'transaction_fees',
    'platform_fees',
    'advisory_fees',
    'fund_expense_ratio_manual',
    'insurance_ulip_charges',
    'fx_fees',
    'private_fund_fees',
    'custody_admin_fees',
    'other'
  )),
  amount TEXT NOT NULL,
  currency TEXT NOT NULL CHECK (length(currency) = 3),
  account_id TEXT REFERENCES accounts(id) ON DELETE SET NULL,
  asset_id TEXT REFERENCES assets(id) ON DELETE SET NULL,
  source_citation_id TEXT REFERENCES source_citations(id) ON DELETE SET NULL,
  notes TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE INDEX idx_manual_fee_entries_date ON manual_fee_entries(fee_date);
CREATE INDEX idx_manual_fee_entries_category ON manual_fee_entries(category);
CREATE INDEX idx_manual_fee_entries_account ON manual_fee_entries(account_id);
CREATE INDEX idx_manual_fee_entries_asset ON manual_fee_entries(asset_id);

DROP INDEX IF EXISTS idx_report_runs_type_created;

PRAGMA foreign_keys=off;

CREATE TABLE report_runs_new (
  id TEXT PRIMARY KEY,
  report_type TEXT NOT NULL CHECK (report_type IN ('net_worth', 'portfolio_summary', 'income', 'data_quality', 'tax_pack', 'monthly_wealth_letter', 'estate_binder', 'fee_report')),
  base_currency TEXT NOT NULL,
  status TEXT NOT NULL CHECK (status IN ('generated', 'exported')),
  created_at TEXT NOT NULL,
  completed_at TEXT
);

INSERT INTO report_runs_new (id, report_type, base_currency, status, created_at, completed_at)
SELECT id, report_type, base_currency, status, created_at, completed_at
FROM report_runs;

DROP TABLE report_runs;
ALTER TABLE report_runs_new RENAME TO report_runs;

PRAGMA foreign_keys=on;

CREATE INDEX idx_report_runs_status ON report_runs(status);
CREATE INDEX idx_report_runs_type_created ON report_runs(report_type, created_at);
