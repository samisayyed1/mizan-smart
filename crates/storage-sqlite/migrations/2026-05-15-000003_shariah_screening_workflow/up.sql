ALTER TABLE asset_shariah_screening ADD COLUMN notes TEXT;

CREATE TABLE shariah_screening_audit_log (
  id TEXT PRIMARY KEY NOT NULL,
  screening_id TEXT NOT NULL,
  asset_id TEXT NOT NULL,
  profile_id TEXT NOT NULL,
  previous_status TEXT,
  new_status TEXT NOT NULL,
  notes TEXT,
  created_at TEXT NOT NULL,
  CHECK (
    previous_status IS NULL OR previous_status IN (
      'compliant',
      'non_compliant',
      'questionable',
      'unknown',
      'needs_review'
    )
  ),
  CHECK (
    new_status IN (
      'compliant',
      'non_compliant',
      'questionable',
      'unknown',
      'needs_review'
    )
  ),
  FOREIGN KEY (screening_id) REFERENCES asset_shariah_screening(id) ON DELETE CASCADE,
  FOREIGN KEY (asset_id) REFERENCES assets(id) ON DELETE CASCADE,
  FOREIGN KEY (profile_id) REFERENCES shariah_screening_profiles(id) ON DELETE CASCADE
);

CREATE INDEX idx_shariah_screening_audit_screening
  ON shariah_screening_audit_log(screening_id, created_at DESC);

CREATE INDEX idx_shariah_screening_audit_asset_profile
  ON shariah_screening_audit_log(asset_id, profile_id, created_at DESC);
