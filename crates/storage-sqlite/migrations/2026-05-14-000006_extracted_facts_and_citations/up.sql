CREATE TABLE extracted_facts (
    id TEXT PRIMARY KEY NOT NULL,
    document_id TEXT NOT NULL,
    page_number INTEGER CHECK (page_number IS NULL OR page_number > 0),
    fact_type TEXT NOT NULL CHECK (length(trim(fact_type)) > 0),
    raw_value TEXT NOT NULL CHECK (length(trim(raw_value)) > 0),
    normalized_value TEXT,
    currency TEXT,
    date_value TEXT,
    confidence_score DOUBLE CHECK (
        confidence_score IS NULL OR (confidence_score >= 0.0 AND confidence_score <= 1.0)
    ),
    bounding_box_json TEXT,
    extraction_method TEXT NOT NULL CHECK (extraction_method IN ('parser', 'ocr', 'vlm', 'manual')),
    extraction_version TEXT NOT NULL CHECK (length(trim(extraction_version)) > 0),
    status TEXT NOT NULL CHECK (status IN ('pending', 'approved', 'rejected', 'superseded')),
    created_at TEXT NOT NULL,
    reviewed_at TEXT,
    review_notes TEXT,
    FOREIGN KEY (document_id) REFERENCES documents(id) ON DELETE CASCADE
);

CREATE TABLE source_citations (
    id TEXT PRIMARY KEY NOT NULL,
    source_type TEXT NOT NULL CHECK (
        source_type IN ('document', 'manual', 'import', 'web_evidence', 'calculated')
    ),
    source_id TEXT,
    document_id TEXT,
    extracted_fact_id TEXT,
    page_number INTEGER CHECK (page_number IS NULL OR page_number > 0),
    bounding_box_json TEXT,
    citation_label TEXT NOT NULL CHECK (length(trim(citation_label)) > 0),
    created_at TEXT NOT NULL,
    FOREIGN KEY (document_id) REFERENCES documents(id) ON DELETE SET NULL,
    FOREIGN KEY (extracted_fact_id) REFERENCES extracted_facts(id) ON DELETE SET NULL
);

ALTER TABLE valuations ADD COLUMN source_citation_id TEXT REFERENCES source_citations(id);

CREATE INDEX idx_extracted_facts_document_status ON extracted_facts(document_id, status);
CREATE INDEX idx_extracted_facts_status_created ON extracted_facts(status, created_at);
CREATE INDEX idx_source_citations_document_id ON source_citations(document_id);
CREATE INDEX idx_source_citations_extracted_fact_id ON source_citations(extracted_fact_id);
CREATE INDEX idx_valuations_source_citation_id ON valuations(source_citation_id);
