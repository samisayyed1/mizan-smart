CREATE TABLE document_processing_jobs (
    id TEXT PRIMARY KEY NOT NULL,
    document_id TEXT NOT NULL,
    job_type TEXT NOT NULL CHECK (job_type IN ('parse_text', 'extract_layout', 'extract_tables', 'ocr', 'vlm_extract', 'embed')),
    status TEXT NOT NULL CHECK (status IN ('queued', 'running', 'succeeded', 'failed', 'cancelled')),
    priority INTEGER NOT NULL DEFAULT 0,
    attempts INTEGER NOT NULL DEFAULT 0 CHECK (attempts >= 0),
    max_attempts INTEGER NOT NULL DEFAULT 3 CHECK (max_attempts > 0),
    error_message TEXT,
    started_at TEXT,
    completed_at TEXT,
    created_at TEXT NOT NULL,
    FOREIGN KEY (document_id) REFERENCES documents(id) ON DELETE CASCADE
);

CREATE INDEX idx_document_processing_jobs_document_id ON document_processing_jobs(document_id);
CREATE INDEX idx_document_processing_jobs_status ON document_processing_jobs(status);
CREATE INDEX idx_document_processing_jobs_queue ON document_processing_jobs(status, priority DESC, created_at ASC);
