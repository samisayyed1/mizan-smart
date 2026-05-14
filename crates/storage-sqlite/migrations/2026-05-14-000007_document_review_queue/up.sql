CREATE TABLE extracted_fact_audit_log (
    id TEXT PRIMARY KEY NOT NULL,
    extracted_fact_id TEXT NOT NULL,
    action TEXT NOT NULL CHECK (action IN ('created', 'edited', 'approved', 'rejected', 'linked', 'deferred')),
    previous_status TEXT CHECK (
        previous_status IS NULL OR previous_status IN ('pending', 'approved', 'rejected', 'superseded')
    ),
    next_status TEXT CHECK (
        next_status IS NULL OR next_status IN ('pending', 'approved', 'rejected', 'superseded')
    ),
    before_json TEXT,
    after_json TEXT,
    notes TEXT,
    created_at TEXT NOT NULL,
    FOREIGN KEY (extracted_fact_id) REFERENCES extracted_facts(id) ON DELETE CASCADE
);

CREATE TABLE extracted_fact_entity_links (
    id TEXT PRIMARY KEY NOT NULL,
    extracted_fact_id TEXT NOT NULL,
    entity_type TEXT NOT NULL CHECK (entity_type IN ('asset', 'account')),
    entity_id TEXT NOT NULL,
    created_at TEXT NOT NULL,
    FOREIGN KEY (extracted_fact_id) REFERENCES extracted_facts(id) ON DELETE CASCADE,
    UNIQUE (extracted_fact_id, entity_type, entity_id)
);

CREATE INDEX idx_extracted_fact_audit_fact_created ON extracted_fact_audit_log(extracted_fact_id, created_at);
CREATE INDEX idx_extracted_fact_entity_links_fact ON extracted_fact_entity_links(extracted_fact_id);
CREATE INDEX idx_extracted_fact_entity_links_entity ON extracted_fact_entity_links(entity_type, entity_id);
