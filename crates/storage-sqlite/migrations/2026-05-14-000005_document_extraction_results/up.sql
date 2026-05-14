CREATE TABLE document_pages (
    id TEXT PRIMARY KEY NOT NULL,
    document_id TEXT NOT NULL,
    page_number INTEGER NOT NULL CHECK (page_number > 0),
    width DOUBLE,
    height DOUBLE,
    rotation INTEGER,
    created_at TEXT NOT NULL,
    FOREIGN KEY (document_id) REFERENCES documents(id) ON DELETE CASCADE,
    UNIQUE (document_id, page_number)
);

CREATE TABLE document_text_blocks (
    id TEXT PRIMARY KEY NOT NULL,
    document_id TEXT NOT NULL,
    page_number INTEGER NOT NULL CHECK (page_number > 0),
    text TEXT NOT NULL,
    bounding_box_json TEXT,
    block_order INTEGER NOT NULL CHECK (block_order >= 0),
    confidence DOUBLE CHECK (confidence IS NULL OR (confidence >= 0.0 AND confidence <= 1.0)),
    created_at TEXT NOT NULL,
    FOREIGN KEY (document_id) REFERENCES documents(id) ON DELETE CASCADE
);

CREATE TABLE document_tables (
    id TEXT PRIMARY KEY NOT NULL,
    document_id TEXT NOT NULL,
    page_number INTEGER NOT NULL CHECK (page_number > 0),
    bounding_box_json TEXT,
    created_at TEXT NOT NULL,
    FOREIGN KEY (document_id) REFERENCES documents(id) ON DELETE CASCADE
);

CREATE TABLE document_table_cells (
    id TEXT PRIMARY KEY NOT NULL,
    table_id TEXT NOT NULL,
    row_index INTEGER NOT NULL CHECK (row_index >= 0),
    column_index INTEGER NOT NULL CHECK (column_index >= 0),
    text TEXT NOT NULL,
    bounding_box_json TEXT,
    confidence DOUBLE CHECK (confidence IS NULL OR (confidence >= 0.0 AND confidence <= 1.0)),
    FOREIGN KEY (table_id) REFERENCES document_tables(id) ON DELETE CASCADE,
    UNIQUE (table_id, row_index, column_index)
);

CREATE INDEX idx_document_pages_document_id ON document_pages(document_id);
CREATE INDEX idx_document_text_blocks_document_page ON document_text_blocks(document_id, page_number, block_order);
CREATE INDEX idx_document_tables_document_page ON document_tables(document_id, page_number);
CREATE INDEX idx_document_table_cells_table_position ON document_table_cells(table_id, row_index, column_index);
