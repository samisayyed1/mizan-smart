DELETE FROM report_lines
WHERE section_id IN (
  SELECT report_sections.id
  FROM report_sections
  JOIN report_runs ON report_runs.id = report_sections.report_run_id
  WHERE report_runs.report_type = 'monthly_wealth_letter'
);

DELETE FROM report_sections
WHERE report_run_id IN (
  SELECT id FROM report_runs WHERE report_type = 'monthly_wealth_letter'
);

DELETE FROM report_runs WHERE report_type = 'monthly_wealth_letter';

DROP INDEX IF EXISTS idx_report_runs_type_created;

PRAGMA foreign_keys = OFF;

CREATE TABLE report_runs_new (
  id TEXT PRIMARY KEY NOT NULL,
  report_type TEXT NOT NULL CHECK (report_type IN ('net_worth', 'portfolio_summary', 'income', 'data_quality', 'tax_pack')),
  base_currency TEXT NOT NULL CHECK (length(base_currency) = 3 AND base_currency = upper(base_currency)),
  status TEXT NOT NULL CHECK (status IN ('generated', 'exported')),
  created_at TEXT NOT NULL,
  completed_at TEXT NULL
);

INSERT INTO report_runs_new (id, report_type, base_currency, status, created_at, completed_at)
SELECT id, report_type, base_currency, status, created_at, completed_at
FROM report_runs;

DROP TABLE report_runs;
ALTER TABLE report_runs_new RENAME TO report_runs;

PRAGMA foreign_keys = ON;

CREATE INDEX idx_report_runs_type_created ON report_runs(report_type, created_at);
