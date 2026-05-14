CREATE TABLE reconciliation_runs (
  id TEXT PRIMARY KEY NOT NULL,
  scope_type TEXT NOT NULL CHECK (scope_type IN ('account', 'asset', 'document', 'import')),
  scope_id TEXT NOT NULL,
  status TEXT NOT NULL CHECK (status IN ('open', 'completed', 'failed')),
  date_tolerance_days INTEGER NOT NULL DEFAULT 0 CHECK (date_tolerance_days BETWEEN 0 AND 31),
  created_at TEXT NOT NULL,
  completed_at TEXT NULL
);

CREATE TABLE reconciliation_items (
  id TEXT PRIMARY KEY NOT NULL,
  run_id TEXT NOT NULL REFERENCES reconciliation_runs(id) ON DELETE CASCADE,
  item_type TEXT NOT NULL,
  source_side TEXT NOT NULL CHECK (source_side IN ('mizan', 'external')),
  raw_json TEXT NOT NULL,
  normalized_hash TEXT NOT NULL,
  amount TEXT NULL,
  currency TEXT NULL CHECK (currency IS NULL OR length(currency) = 3),
  effective_date TEXT NULL,
  status TEXT NOT NULL CHECK (status IN ('open', 'ignored', 'accepted_adjustment'))
);

CREATE TABLE reconciliation_matches (
  id TEXT PRIMARY KEY NOT NULL,
  run_id TEXT NOT NULL REFERENCES reconciliation_runs(id) ON DELETE CASCADE,
  mizan_item_id TEXT NULL REFERENCES reconciliation_items(id) ON DELETE CASCADE,
  external_item_id TEXT NULL REFERENCES reconciliation_items(id) ON DELETE CASCADE,
  match_status TEXT NOT NULL CHECK (match_status IN ('matched', 'possible_match', 'missing_in_mizan', 'missing_in_external', 'duplicate', 'mismatch')),
  confidence TEXT NOT NULL,
  reason TEXT NOT NULL,
  created_at TEXT NOT NULL
);

CREATE INDEX idx_reconciliation_runs_scope ON reconciliation_runs(scope_type, scope_id, created_at);
CREATE INDEX idx_reconciliation_items_run_side ON reconciliation_items(run_id, source_side);
CREATE INDEX idx_reconciliation_items_hash ON reconciliation_items(run_id, normalized_hash);
CREATE INDEX idx_reconciliation_matches_run_status ON reconciliation_matches(run_id, match_status);
