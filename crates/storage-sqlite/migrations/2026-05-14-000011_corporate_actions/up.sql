CREATE TABLE corporate_actions (
  id TEXT PRIMARY KEY NOT NULL,
  asset_id TEXT NOT NULL,
  action_type TEXT NOT NULL CHECK (
    action_type IN (
      'split',
      'reverse_split',
      'merger',
      'spinoff',
      'symbol_change',
      'return_of_capital',
      'stock_dividend'
    )
  ),
  effective_date DATE NOT NULL,
  ratio_numerator TEXT,
  ratio_denominator TEXT,
  new_symbol TEXT,
  metadata_json TEXT,
  source_citation_id TEXT,
  created_at TEXT NOT NULL,
  FOREIGN KEY (asset_id) REFERENCES assets(id) ON DELETE CASCADE,
  FOREIGN KEY (source_citation_id) REFERENCES source_citations(id) ON DELETE SET NULL,
  CHECK (
    action_type NOT IN ('split', 'reverse_split')
    OR (ratio_numerator IS NOT NULL AND ratio_denominator IS NOT NULL)
  ),
  CHECK (action_type != 'symbol_change' OR new_symbol IS NOT NULL)
);

CREATE INDEX idx_corporate_actions_asset_date
  ON corporate_actions(asset_id, effective_date);

CREATE INDEX idx_corporate_actions_source_citation
  ON corporate_actions(source_citation_id);
