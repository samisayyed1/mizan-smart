use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use diesel::prelude::*;
use diesel::r2d2::{self, Pool};
use diesel::SqliteConnection;
use uuid::Uuid;

use mizan_core::documents::{
    DocumentBoundingBox, DocumentJobType, DocumentParser, DocumentParserCapabilities,
    ParsedDocument, ParsedDocumentPage, ParsedTable, ParsedTableCell, ParsedTextBlock,
};
use mizan_core::errors::{DatabaseError, Error, ValidationError};
use mizan_core::Result;

use crate::db::{get_connection, WriteHandle};
use crate::documents::jobs::DocumentJobProcessor;
use crate::documents::DocumentVaultRepository;
use crate::errors::StorageError;
use crate::schema::{
    document_pages, document_table_cells, document_tables, document_text_blocks, documents,
};

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = document_pages)]
struct NewDocumentPageRow {
    id: String,
    document_id: String,
    page_number: i32,
    width: Option<f64>,
    height: Option<f64>,
    rotation: Option<i32>,
    created_at: String,
}

#[derive(Debug, Clone, Queryable)]
struct DocumentPageRow {
    page_number: i32,
    width: Option<f64>,
    height: Option<f64>,
    rotation: Option<i32>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = document_text_blocks)]
struct NewDocumentTextBlockRow {
    id: String,
    document_id: String,
    page_number: i32,
    text: String,
    bounding_box_json: Option<String>,
    block_order: i32,
    confidence: Option<f64>,
    created_at: String,
}

#[derive(Debug, Clone, Queryable)]
struct DocumentTextBlockRow {
    page_number: i32,
    text: String,
    bounding_box_json: Option<String>,
    block_order: i32,
    confidence: Option<f64>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = document_tables)]
struct NewDocumentTableRow {
    id: String,
    document_id: String,
    page_number: i32,
    bounding_box_json: Option<String>,
    created_at: String,
}

#[derive(Debug, Clone, Queryable)]
struct DocumentTableRow {
    id: String,
    page_number: i32,
    bounding_box_json: Option<String>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = document_table_cells)]
struct NewDocumentTableCellRow {
    id: String,
    table_id: String,
    row_index: i32,
    column_index: i32,
    text: String,
    bounding_box_json: Option<String>,
    confidence: Option<f64>,
}

#[derive(Debug, Clone, Queryable)]
struct DocumentTableCellRow {
    table_id: String,
    row_index: i32,
    column_index: i32,
    text: String,
    bounding_box_json: Option<String>,
    confidence: Option<f64>,
}

#[derive(Clone)]
pub struct DocumentExtractionRepository {
    pool: Arc<Pool<r2d2::ConnectionManager<SqliteConnection>>>,
    writer: WriteHandle,
}

impl DocumentExtractionRepository {
    pub fn new(
        pool: Arc<Pool<r2d2::ConnectionManager<SqliteConnection>>>,
        writer: WriteHandle,
    ) -> Self {
        Self { pool, writer }
    }

    pub async fn save_parsed_document(&self, parsed: ParsedDocument) -> Result<ParsedDocument> {
        validate_parsed_document(&parsed)?;
        let saved = parsed.clone();
        self.writer
            .exec_tx(move |tx| -> Result<()> {
                let conn = tx.conn();
                ensure_document_exists(conn, &parsed.document_id)?;
                diesel::delete(
                    document_pages::table
                        .filter(document_pages::document_id.eq(&parsed.document_id)),
                )
                .execute(conn)
                .map_err(StorageError::from)?;
                diesel::delete(
                    document_text_blocks::table
                        .filter(document_text_blocks::document_id.eq(&parsed.document_id)),
                )
                .execute(conn)
                .map_err(StorageError::from)?;
                diesel::delete(
                    document_tables::table
                        .filter(document_tables::document_id.eq(&parsed.document_id)),
                )
                .execute(conn)
                .map_err(StorageError::from)?;

                let now = Utc::now();
                let page_rows: Vec<_> = parsed
                    .pages
                    .iter()
                    .map(|page| NewDocumentPageRow {
                        id: Uuid::new_v4().to_string(),
                        document_id: parsed.document_id.clone(),
                        page_number: page.page_number,
                        width: page.width,
                        height: page.height,
                        rotation: page.rotation,
                        created_at: now.to_rfc3339(),
                    })
                    .collect();
                if !page_rows.is_empty() {
                    diesel::insert_into(document_pages::table)
                        .values(&page_rows)
                        .execute(conn)
                        .map_err(StorageError::from)?;
                }

                let text_rows: Vec<_> = parsed
                    .text_blocks
                    .iter()
                    .map(|block| {
                        Ok(NewDocumentTextBlockRow {
                            id: Uuid::new_v4().to_string(),
                            document_id: parsed.document_id.clone(),
                            page_number: block.page_number,
                            text: block.text.clone(),
                            bounding_box_json: serialize_box(block.bounding_box.as_ref())?,
                            block_order: block.block_order,
                            confidence: block.confidence,
                            created_at: now.to_rfc3339(),
                        })
                    })
                    .collect::<Result<Vec<_>>>()?;
                if !text_rows.is_empty() {
                    diesel::insert_into(document_text_blocks::table)
                        .values(&text_rows)
                        .execute(conn)
                        .map_err(StorageError::from)?;
                }

                for table in &parsed.tables {
                    let table_id = Uuid::new_v4().to_string();
                    let table_row = NewDocumentTableRow {
                        id: table_id.clone(),
                        document_id: parsed.document_id.clone(),
                        page_number: table.page_number,
                        bounding_box_json: serialize_box(table.bounding_box.as_ref())?,
                        created_at: now.to_rfc3339(),
                    };
                    diesel::insert_into(document_tables::table)
                        .values(&table_row)
                        .execute(conn)
                        .map_err(StorageError::from)?;
                    let cell_rows: Vec<_> = table
                        .cells
                        .iter()
                        .map(|cell| {
                            Ok(NewDocumentTableCellRow {
                                id: Uuid::new_v4().to_string(),
                                table_id: table_id.clone(),
                                row_index: cell.row_index,
                                column_index: cell.column_index,
                                text: cell.text.clone(),
                                bounding_box_json: serialize_box(cell.bounding_box.as_ref())?,
                                confidence: cell.confidence,
                            })
                        })
                        .collect::<Result<Vec<_>>>()?;
                    if !cell_rows.is_empty() {
                        diesel::insert_into(document_table_cells::table)
                            .values(&cell_rows)
                            .execute(conn)
                            .map_err(StorageError::from)?;
                    }
                }
                Ok(())
            })
            .await?;
        Ok(saved)
    }

    pub fn get_parsed_document(&self, document_id: &str) -> Result<ParsedDocument> {
        let mut conn = get_connection(&self.pool)?;
        ensure_document_exists(&mut conn, document_id)?;
        let page_rows = document_pages::table
            .filter(document_pages::document_id.eq(document_id))
            .order(document_pages::page_number.asc())
            .select((
                document_pages::page_number,
                document_pages::width,
                document_pages::height,
                document_pages::rotation,
            ))
            .load::<DocumentPageRow>(&mut conn)
            .map_err(StorageError::from)?;
        let text_rows = document_text_blocks::table
            .filter(document_text_blocks::document_id.eq(document_id))
            .order((
                document_text_blocks::page_number.asc(),
                document_text_blocks::block_order.asc(),
            ))
            .select((
                document_text_blocks::page_number,
                document_text_blocks::text,
                document_text_blocks::bounding_box_json,
                document_text_blocks::block_order,
                document_text_blocks::confidence,
            ))
            .load::<DocumentTextBlockRow>(&mut conn)
            .map_err(StorageError::from)?;
        let table_rows = document_tables::table
            .filter(document_tables::document_id.eq(document_id))
            .order(document_tables::page_number.asc())
            .select((
                document_tables::id,
                document_tables::page_number,
                document_tables::bounding_box_json,
            ))
            .load::<DocumentTableRow>(&mut conn)
            .map_err(StorageError::from)?;
        let table_ids: Vec<String> = table_rows.iter().map(|row| row.id.clone()).collect();
        let cell_rows = if table_ids.is_empty() {
            Vec::new()
        } else {
            document_table_cells::table
                .filter(document_table_cells::table_id.eq_any(&table_ids))
                .order((
                    document_table_cells::table_id.asc(),
                    document_table_cells::row_index.asc(),
                    document_table_cells::column_index.asc(),
                ))
                .select((
                    document_table_cells::table_id,
                    document_table_cells::row_index,
                    document_table_cells::column_index,
                    document_table_cells::text,
                    document_table_cells::bounding_box_json,
                    document_table_cells::confidence,
                ))
                .load::<DocumentTableCellRow>(&mut conn)
                .map_err(StorageError::from)?
        };

        let pages = page_rows
            .into_iter()
            .map(|row| ParsedDocumentPage {
                page_number: row.page_number,
                width: row.width,
                height: row.height,
                rotation: row.rotation,
            })
            .collect();
        let text_blocks = text_rows
            .into_iter()
            .map(|row| {
                Ok(ParsedTextBlock {
                    page_number: row.page_number,
                    text: row.text,
                    bounding_box: deserialize_box(row.bounding_box_json.as_deref())?,
                    block_order: row.block_order,
                    confidence: row.confidence,
                })
            })
            .collect::<Result<Vec<_>>>()?;
        let tables = table_rows
            .into_iter()
            .map(|row| {
                let cells = cell_rows
                    .iter()
                    .filter(|cell| cell.table_id == row.id)
                    .map(|cell| {
                        Ok(ParsedTableCell {
                            row_index: cell.row_index,
                            column_index: cell.column_index,
                            text: cell.text.clone(),
                            bounding_box: deserialize_box(cell.bounding_box_json.as_deref())?,
                            confidence: cell.confidence,
                        })
                    })
                    .collect::<Result<Vec<_>>>()?;
                Ok(ParsedTable {
                    page_number: row.page_number,
                    bounding_box: deserialize_box(row.bounding_box_json.as_deref())?,
                    cells,
                })
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(ParsedDocument {
            document_id: document_id.to_string(),
            pages,
            text_blocks,
            tables,
        })
    }
}

pub struct LocalDocumentParser {
    vault: Arc<DocumentVaultRepository>,
}

impl LocalDocumentParser {
    pub fn new(vault: Arc<DocumentVaultRepository>) -> Self {
        Self { vault }
    }
}

#[async_trait]
impl DocumentParser for LocalDocumentParser {
    fn capabilities(&self) -> DocumentParserCapabilities {
        DocumentParserCapabilities {
            text: true,
            layout: false,
            tables: false,
            ocr: false,
        }
    }

    async fn parse_document(&self, document_id: &str) -> Result<ParsedDocument> {
        self.parse_text(document_id).await
    }

    async fn parse_text(&self, document_id: &str) -> Result<ParsedDocument> {
        let record = self.vault.get_metadata(document_id)?;
        let content = self.vault.read_decrypted(document_id)?;
        if is_plain_text(&record.document.mime_type, &record.document.original_name) {
            let text = String::from_utf8(content).map_err(|err| {
                Error::Validation(ValidationError::InvalidInput(format!(
                    "Document text is not valid UTF-8: {err}"
                )))
            })?;
            return Ok(parsed_from_pages(document_id, vec![text]));
        }
        if is_pdf(&record.document.mime_type, &record.document.original_name) {
            let pages = pdf_extract::extract_text_from_mem_by_pages(&content).map_err(|err| {
                Error::Validation(ValidationError::InvalidInput(format!(
                    "Failed to parse PDF document locally: {err}"
                )))
            })?;
            return Ok(parsed_from_pages(document_id, pages));
        }
        Err(Error::Validation(ValidationError::InvalidInput(format!(
            "Document text parser does not support MIME type {}",
            record.document.mime_type
        ))))
    }

    async fn parse_layout(&self, _document_id: &str) -> Result<ParsedDocument> {
        Err(Error::Validation(ValidationError::InvalidInput(
            "Document layout extraction is not available on this machine".into(),
        )))
    }

    async fn parse_tables(&self, _document_id: &str) -> Result<ParsedDocument> {
        Err(Error::Validation(ValidationError::InvalidInput(
            "Document table extraction is not available on this machine".into(),
        )))
    }
}

pub struct DocumentExtractionJobProcessor {
    parser: Arc<dyn DocumentParser>,
    extraction_repository: Arc<DocumentExtractionRepository>,
}

impl DocumentExtractionJobProcessor {
    pub fn new(
        parser: Arc<dyn DocumentParser>,
        extraction_repository: Arc<DocumentExtractionRepository>,
    ) -> Self {
        Self {
            parser,
            extraction_repository,
        }
    }
}

#[async_trait]
impl DocumentJobProcessor for DocumentExtractionJobProcessor {
    fn capabilities(&self) -> DocumentParserCapabilities {
        self.parser.capabilities()
    }

    async fn process(&self, job: &mizan_core::documents::DocumentProcessingJob) -> Result<()> {
        let parsed = match job.job_type {
            DocumentJobType::ParseText => self.parser.parse_text(&job.document_id).await?,
            DocumentJobType::ExtractLayout => self.parser.parse_layout(&job.document_id).await?,
            DocumentJobType::ExtractTables => self.parser.parse_tables(&job.document_id).await?,
            DocumentJobType::Ocr => {
                return Err(Error::Validation(ValidationError::InvalidInput(
                    "Document OCR runtime is not available on this machine".into(),
                )));
            }
            DocumentJobType::VlmExtract => {
                return Err(Error::Validation(ValidationError::InvalidInput(
                    "Document vision-language extraction runtime is not available on this machine"
                        .into(),
                )));
            }
            DocumentJobType::Embed => {
                return Err(Error::Validation(ValidationError::InvalidInput(
                    "Document embedding runtime is not available on this machine".into(),
                )));
            }
        };
        self.extraction_repository
            .save_parsed_document(parsed)
            .await?;
        Ok(())
    }
}

fn parsed_from_pages(document_id: &str, page_texts: Vec<String>) -> ParsedDocument {
    let pages: Vec<_> = page_texts
        .iter()
        .enumerate()
        .map(|(index, _)| ParsedDocumentPage {
            page_number: (index + 1) as i32,
            width: None,
            height: None,
            rotation: None,
        })
        .collect();
    let text_blocks: Vec<_> = page_texts
        .into_iter()
        .enumerate()
        .filter_map(|(index, text)| {
            let trimmed = text.trim().to_string();
            (!trimmed.is_empty()).then_some(ParsedTextBlock {
                page_number: (index + 1) as i32,
                text: trimmed,
                bounding_box: None,
                block_order: 0,
                confidence: None,
            })
        })
        .collect();
    ParsedDocument {
        document_id: document_id.to_string(),
        pages,
        text_blocks,
        tables: Vec::new(),
    }
}

fn is_pdf(mime_type: &str, name: &str) -> bool {
    mime_type.eq_ignore_ascii_case("application/pdf") || name.to_ascii_lowercase().ends_with(".pdf")
}

fn is_plain_text(mime_type: &str, name: &str) -> bool {
    mime_type.starts_with("text/") || name.to_ascii_lowercase().ends_with(".txt")
}

fn validate_parsed_document(parsed: &ParsedDocument) -> Result<()> {
    if parsed.document_id.trim().is_empty() {
        return Err(Error::Validation(ValidationError::MissingField(
            "document_id".into(),
        )));
    }
    for page in &parsed.pages {
        if page.page_number <= 0 {
            return Err(Error::Validation(ValidationError::InvalidInput(
                "document page_number must be greater than zero".into(),
            )));
        }
    }
    for block in &parsed.text_blocks {
        validate_confidence(block.confidence)?;
        if block.page_number <= 0 || block.block_order < 0 {
            return Err(Error::Validation(ValidationError::InvalidInput(
                "document text block page_number and block_order must be non-negative".into(),
            )));
        }
    }
    for table in &parsed.tables {
        if table.page_number <= 0 {
            return Err(Error::Validation(ValidationError::InvalidInput(
                "document table page_number must be greater than zero".into(),
            )));
        }
        for cell in &table.cells {
            validate_confidence(cell.confidence)?;
            if cell.row_index < 0 || cell.column_index < 0 {
                return Err(Error::Validation(ValidationError::InvalidInput(
                    "document table cell indexes must be non-negative".into(),
                )));
            }
        }
    }
    Ok(())
}

fn validate_confidence(confidence: Option<f64>) -> Result<()> {
    if let Some(value) = confidence {
        if !(0.0..=1.0).contains(&value) || !value.is_finite() {
            return Err(Error::Validation(ValidationError::InvalidInput(
                "document extraction confidence must be between 0 and 1".into(),
            )));
        }
    }
    Ok(())
}

fn serialize_box(value: Option<&DocumentBoundingBox>) -> Result<Option<String>> {
    value
        .map(|bbox| serde_json::to_string(bbox).map_err(Error::from))
        .transpose()
}

fn deserialize_box(value: Option<&str>) -> Result<Option<DocumentBoundingBox>> {
    value
        .map(|raw| serde_json::from_str(raw).map_err(Error::from))
        .transpose()
}

fn ensure_document_exists(conn: &mut SqliteConnection, document_id: &str) -> Result<()> {
    let exists = documents::table
        .find(document_id)
        .select(documents::id)
        .first::<String>(conn)
        .optional()
        .map_err(StorageError::from)?;
    if exists.is_none() {
        return Err(Error::Database(DatabaseError::NotFound(
            document_id.to_string(),
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::write_actor;
    use crate::db::{create_pool, init, run_migrations};
    use crate::documents::{generate_document_vault_key_hex, parse_document_vault_key_hex};
    use mizan_core::documents::{DocumentJobStatus, UploadDocumentRequest};
    use tempfile::tempdir;

    struct TestContext {
        vault: Arc<DocumentVaultRepository>,
        extraction: Arc<DocumentExtractionRepository>,
        jobs: crate::documents::jobs::DocumentJobRepository,
        _app_data: tempfile::TempDir,
    }

    fn setup() -> TestContext {
        let app_data = tempdir().expect("tempdir");
        let db_path = init(app_data.path().to_str().expect("path")).expect("init");
        run_migrations(&db_path).expect("migrate");
        let pool = create_pool(&db_path).expect("pool");
        let writer = write_actor::spawn_writer(pool.as_ref().clone()).expect("writer");
        let key = parse_document_vault_key_hex(&generate_document_vault_key_hex()).expect("key");
        let vault = Arc::new(
            DocumentVaultRepository::new(
                pool.clone(),
                writer.clone(),
                app_data.path().join("document-vault"),
                key,
            )
            .expect("vault"),
        );
        let extraction = Arc::new(DocumentExtractionRepository::new(
            pool.clone(),
            writer.clone(),
        ));
        let parser = Arc::new(LocalDocumentParser::new(vault.clone()));
        let processor = Arc::new(DocumentExtractionJobProcessor::new(
            parser,
            extraction.clone(),
        ));
        let jobs = crate::documents::jobs::DocumentJobRepository::new_with_processor(
            pool,
            writer,
            processor,
            std::time::Duration::from_secs(2),
        );
        TestContext {
            vault,
            extraction,
            jobs,
            _app_data: app_data,
        }
    }

    fn upload_request(name: &str, mime_type: &str, content: Vec<u8>) -> UploadDocumentRequest {
        UploadDocumentRequest {
            original_name: name.into(),
            mime_type: mime_type.into(),
            content,
            source_type: None,
        }
    }

    #[tokio::test]
    async fn text_document_job_persists_extracted_text() {
        let ctx = setup();
        let record = ctx
            .vault
            .upload(upload_request(
                "notes.txt",
                "text/plain",
                b"Account value: 123".to_vec(),
            ))
            .await
            .expect("upload");
        let job = ctx.jobs.run_next().await.expect("run").job.expect("job");
        assert_eq!(job.status, DocumentJobStatus::Succeeded);
        let parsed = ctx
            .extraction
            .get_parsed_document(&record.document.id)
            .expect("parsed");
        assert_eq!(parsed.pages.len(), 1);
        assert_eq!(parsed.text_blocks[0].text, "Account value: 123");
    }

    #[tokio::test]
    async fn fixture_pdf_parses_text_when_local_parser_is_available() {
        let ctx = setup();
        let record = ctx
            .vault
            .upload(upload_request(
                "statement.pdf",
                "application/pdf",
                fixture_pdf_bytes("Hello Mizan"),
            ))
            .await
            .expect("upload");
        let job = ctx.jobs.run_next().await.expect("run").job.expect("job");
        assert_eq!(job.status, DocumentJobStatus::Succeeded);
        let parsed = ctx
            .extraction
            .get_parsed_document(&record.document.id)
            .expect("parsed");
        assert!(parsed
            .text_blocks
            .iter()
            .any(|block| block.text.contains("Hello Mizan")));
    }

    #[tokio::test]
    async fn corrupted_pdf_fails_cleanly() {
        let ctx = setup();
        ctx.vault
            .upload(upload_request(
                "bad.pdf",
                "application/pdf",
                b"%PDF-1.4 not a real pdf".to_vec(),
            ))
            .await
            .expect("upload");
        let job = ctx.jobs.run_next().await.expect("run").job.expect("job");
        assert_eq!(job.status, DocumentJobStatus::Failed);
        assert!(job
            .error_message
            .expect("error")
            .contains("Failed to parse PDF document locally"));
    }

    #[tokio::test]
    async fn unsupported_parser_returns_clear_error() {
        let ctx = setup();
        ctx.vault
            .upload(upload_request(
                "image.png",
                "image/png",
                b"not text".to_vec(),
            ))
            .await
            .expect("upload");
        let job = ctx.jobs.run_next().await.expect("run").job.expect("job");
        assert_eq!(job.status, DocumentJobStatus::Failed);
        assert!(job
            .error_message
            .expect("error")
            .contains("does not support MIME type image/png"));
    }

    fn fixture_pdf_bytes(text: &str) -> Vec<u8> {
        let escaped = text
            .replace('\\', "\\\\")
            .replace('(', "\\(")
            .replace(')', "\\)");
        let stream = format!("BT /F1 24 Tf 100 700 Td ({escaped}) Tj ET");
        let objects = [
            "<< /Type /Catalog /Pages 2 0 R >>".to_string(),
            "<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_string(),
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Resources << /Font << /F1 4 0 R >> >> /Contents 5 0 R >>".to_string(),
            "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>".to_string(),
            format!("<< /Length {} >>\nstream\n{}\nendstream", stream.len(), stream),
        ];
        let mut pdf = b"%PDF-1.4\n".to_vec();
        let mut offsets = Vec::new();
        for (index, object) in objects.iter().enumerate() {
            offsets.push(pdf.len());
            pdf.extend_from_slice(format!("{} 0 obj\n{}\nendobj\n", index + 1, object).as_bytes());
        }
        let xref_offset = pdf.len();
        pdf.extend_from_slice(
            format!("xref\n0 {}\n0000000000 65535 f \n", objects.len() + 1).as_bytes(),
        );
        for offset in offsets {
            pdf.extend_from_slice(format!("{offset:010} 00000 n \n").as_bytes());
        }
        pdf.extend_from_slice(
            format!(
                "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref_offset}\n%%EOF\n",
                objects.len() + 1
            )
            .as_bytes(),
        );
        pdf
    }
}
