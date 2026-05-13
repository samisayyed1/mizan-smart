-- Smart Alerts engine — Phase 1 / Prompt 8 of docs/mizan-smart-plan/PLAN.md.
--
-- A deterministic alert is something Mizan flags about the user's data, with
-- a stable fingerprint so re-running the rules does not produce duplicates.
-- Alerts can be snoozed, dismissed, or resolved by the user. AI never writes
-- to this table directly — only the rule engine does.

CREATE TABLE smart_alerts (
    id TEXT PRIMARY KEY NOT NULL,
    fingerprint TEXT NOT NULL UNIQUE,
    rule_name TEXT NOT NULL,
    category TEXT NOT NULL,
    severity TEXT NOT NULL CHECK (severity IN ('info', 'warning', 'critical')),
    title TEXT NOT NULL,
    message TEXT NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('active', 'snoozed', 'dismissed', 'resolved')),
    source_entity_type TEXT,
    source_entity_id TEXT,
    action_route TEXT,
    first_seen_at TEXT NOT NULL,
    last_seen_at TEXT NOT NULL,
    snoozed_until TEXT,
    dismissed_at TEXT,
    resolved_at TEXT,
    metadata_json TEXT
);

CREATE INDEX idx_smart_alerts_status ON smart_alerts(status);
CREATE INDEX idx_smart_alerts_severity ON smart_alerts(severity);
CREATE INDEX idx_smart_alerts_category ON smart_alerts(category);
CREATE INDEX idx_smart_alerts_rule_name ON smart_alerts(rule_name);
CREATE INDEX idx_smart_alerts_source_entity ON smart_alerts(source_entity_type, source_entity_id);
