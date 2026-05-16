DELETE FROM report_lines
WHERE section_id IN (
  SELECT report_sections.id
  FROM report_sections
  JOIN report_runs ON report_runs.id = report_sections.report_run_id
  WHERE report_runs.report_type = 'fee_report'
);

DELETE FROM report_sections
WHERE report_run_id IN (
  SELECT id FROM report_runs WHERE report_type = 'fee_report'
);

DELETE FROM report_runs WHERE report_type = 'fee_report';

DROP INDEX IF EXISTS idx_report_runs_type_created;
DROP INDEX IF EXISTS idx_report_runs_status;

PRAGMA foreign_keys=off;

CREATE TABLE report_runs_new (
  id TEXT PRIMARY KEY,
  report_type TEXT NOT NULL CHECK (report_type IN ('net_worth', 'portfolio_summary', 'income', 'data_quality', 'tax_pack', 'monthly_wealth_letter', 'estate_binder')),
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

DROP INDEX IF EXISTS idx_manual_fee_entries_asset;
DROP INDEX IF EXISTS idx_manual_fee_entries_account;
DROP INDEX IF EXISTS idx_manual_fee_entries_category;
DROP INDEX IF EXISTS idx_manual_fee_entries_date;
DROP TABLE IF EXISTS manual_fee_entries;
