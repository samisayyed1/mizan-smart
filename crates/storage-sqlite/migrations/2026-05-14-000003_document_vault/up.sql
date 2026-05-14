CREATE TABLE documents (
    id TEXT PRIMARY KEY NOT NULL,
    file_hash TEXT NOT NULL UNIQUE,
    original_name TEXT NOT NULL CHECK (length(trim(original_name)) > 0),
    mime_type TEXT NOT NULL,
    file_size_bytes INTEGER NOT NULL CHECK (file_size_bytes >= 0),
    encrypted_storage_path TEXT NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('ingested', 'queued', 'processing', 'processed', 'reviewed', 'error')),
    source_type TEXT,
    error_message TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX idx_documents_status ON documents(status);
CREATE INDEX idx_documents_created_at ON documents(created_at);

CREATE TABLE document_files (
    id TEXT PRIMARY KEY NOT NULL,
    document_id TEXT NOT NULL,
    encryption_version INTEGER NOT NULL CHECK (encryption_version > 0),
    nonce TEXT NOT NULL,
    checksum_sha256 TEXT NOT NULL,
    storage_path TEXT NOT NULL,
    created_at TEXT NOT NULL,
    FOREIGN KEY (document_id) REFERENCES documents(id) ON DELETE CASCADE
);

CREATE INDEX idx_document_files_document_id ON document_files(document_id);

CREATE TABLE document_links (
    id TEXT PRIMARY KEY NOT NULL,
    document_id TEXT NOT NULL,
    linked_entity_type TEXT NOT NULL CHECK (length(trim(linked_entity_type)) > 0),
    linked_entity_id TEXT NOT NULL CHECK (length(trim(linked_entity_id)) > 0),
    created_at TEXT NOT NULL,
    FOREIGN KEY (document_id) REFERENCES documents(id) ON DELETE CASCADE
);

CREATE INDEX idx_document_links_document_id ON document_links(document_id);
CREATE INDEX idx_document_links_entity ON document_links(linked_entity_type, linked_entity_id);
