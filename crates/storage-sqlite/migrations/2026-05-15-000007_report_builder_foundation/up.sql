CREATE TABLE report_runs (
  id TEXT PRIMARY KEY NOT NULL,
  report_type TEXT NOT NULL CHECK (report_type IN ('net_worth', 'portfolio_summary', 'income', 'data_quality', 'tax_pack')),
  base_currency TEXT NOT NULL CHECK (length(base_currency) = 3 AND base_currency = upper(base_currency)),
  status TEXT NOT NULL CHECK (status IN ('generated', 'exported')),
  created_at TEXT NOT NULL,
  completed_at TEXT NULL
);

CREATE TABLE report_sections (
  id TEXT PRIMARY KEY NOT NULL,
  report_run_id TEXT NOT NULL REFERENCES report_runs(id) ON DELETE CASCADE,
  title TEXT NOT NULL CHECK (length(trim(title)) > 0),
  section_order INTEGER NOT NULL CHECK (section_order >= 0),
  metadata_json TEXT NULL
);

CREATE TABLE report_lines (
  id TEXT PRIMARY KEY NOT NULL,
  section_id TEXT NOT NULL REFERENCES report_sections(id) ON DELETE CASCADE,
  label TEXT NOT NULL CHECK (length(trim(label)) > 0),
  amount TEXT NULL,
  currency TEXT NULL CHECK (currency IS NULL OR (length(currency) = 3 AND currency = upper(currency))),
  value_text TEXT NULL,
  source_citation_id TEXT NULL REFERENCES source_citations(id) ON DELETE SET NULL,
  metadata_json TEXT NULL,
  CHECK (amount IS NOT NULL OR value_text IS NOT NULL)
);

CREATE INDEX idx_report_runs_type_created ON report_runs(report_type, created_at);
CREATE INDEX idx_report_sections_run_order ON report_sections(report_run_id, section_order);
CREATE INDEX idx_report_lines_section ON report_lines(section_id);
CREATE INDEX idx_report_lines_source_citation ON report_lines(source_citation_id);
