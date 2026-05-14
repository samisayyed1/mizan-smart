DROP INDEX IF EXISTS idx_document_links_entity;
DROP INDEX IF EXISTS idx_document_links_document_id;
DROP TABLE IF EXISTS document_links;

DROP INDEX IF EXISTS idx_document_files_document_id;
DROP TABLE IF EXISTS document_files;

DROP INDEX IF EXISTS idx_documents_created_at;
DROP INDEX IF EXISTS idx_documents_status;
DROP TABLE IF EXISTS documents;
