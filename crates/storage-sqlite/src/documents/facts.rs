//! Citation-backed extracted facts for Document Vault outputs.

use std::sync::Arc;

use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel::r2d2::{self, Pool};
use diesel::SqliteConnection;
use uuid::Uuid;

use mizan_core::documents::{
    CreateExtractedFactRequest, CreateExtractedFactResult, DocumentBoundingBox, ExtractedFact,
    ExtractedFactStatus, ExtractionMethod, ReviewExtractedFactRequest, SourceCitation,
    SourceCitationType,
};
use mizan_core::errors::{DatabaseError, Error, ValidationError};
use mizan_core::Result;

use crate::db::{get_connection, WriteHandle};
use crate::errors::StorageError;
use crate::schema::{documents, extracted_facts, source_citations};

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = extracted_facts)]
struct NewExtractedFactRow {
    id: String,
    document_id: String,
    page_number: Option<i32>,
    fact_type: String,
    raw_value: String,
    normalized_value: Option<String>,
    currency: Option<String>,
    date_value: Option<String>,
    confidence_score: Option<f64>,
    bounding_box_json: Option<String>,
    extraction_method: String,
    extraction_version: String,
    status: String,
    created_at: String,
    reviewed_at: Option<String>,
    review_notes: Option<String>,
}

#[derive(Debug, Clone, Queryable)]
struct ExtractedFactRow {
    id: String,
    document_id: String,
    page_number: Option<i32>,
    fact_type: String,
    raw_value: String,
    normalized_value: Option<String>,
    currency: Option<String>,
    date_value: Option<String>,
    confidence_score: Option<f64>,
    bounding_box_json: Option<String>,
    extraction_method: String,
    extraction_version: String,
    status: String,
    created_at: String,
    reviewed_at: Option<String>,
    review_notes: Option<String>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = source_citations)]
struct NewSourceCitationRow {
    id: String,
    source_type: String,
    source_id: Option<String>,
    document_id: Option<String>,
    extracted_fact_id: Option<String>,
    page_number: Option<i32>,
    bounding_box_json: Option<String>,
    citation_label: String,
    created_at: String,
}

#[derive(Debug, Clone, Queryable)]
struct SourceCitationRow {
    id: String,
    source_type: String,
    source_id: Option<String>,
    document_id: Option<String>,
    extracted_fact_id: Option<String>,
    page_number: Option<i32>,
    bounding_box_json: Option<String>,
    citation_label: String,
    created_at: String,
}

#[derive(Clone)]
pub struct ExtractedFactRepository {
    pool: Arc<Pool<r2d2::ConnectionManager<SqliteConnection>>>,
    writer: WriteHandle,
}

impl ExtractedFactRepository {
    pub fn new(
        pool: Arc<Pool<r2d2::ConnectionManager<SqliteConnection>>>,
        writer: WriteHandle,
    ) -> Self {
        Self { pool, writer }
    }

    pub async fn create_extracted_fact(
        &self,
        request: CreateExtractedFactRequest,
    ) -> Result<CreateExtractedFactResult> {
        validate_create_request(&request)?;
        let fact_id = Uuid::new_v4().to_string();
        let citation_id = Uuid::new_v4().to_string();
        let now = Utc::now();
        let created_at = now.to_rfc3339();
        let bounding_box_json = serialize_box(request.bounding_box.as_ref())?;

        let fact_row = NewExtractedFactRow {
            id: fact_id.clone(),
            document_id: request.document_id.trim().to_string(),
            page_number: request.page_number,
            fact_type: request.fact_type.trim().to_string(),
            raw_value: request.raw_value.trim().to_string(),
            normalized_value: trimmed_optional(request.normalized_value),
            currency: trimmed_optional(request.currency).map(|value| value.to_uppercase()),
            date_value: trimmed_optional(request.date_value),
            confidence_score: request.confidence_score,
            bounding_box_json: bounding_box_json.clone(),
            extraction_method: request.extraction_method.as_str().to_string(),
            extraction_version: request.extraction_version.trim().to_string(),
            status: ExtractedFactStatus::Pending.as_str().to_string(),
            created_at: created_at.clone(),
            reviewed_at: None,
            review_notes: None,
        };
        let citation_row = NewSourceCitationRow {
            id: citation_id.clone(),
            source_type: SourceCitationType::Document.as_str().to_string(),
            source_id: Some(fact_row.document_id.clone()),
            document_id: Some(fact_row.document_id.clone()),
            extracted_fact_id: Some(fact_id.clone()),
            page_number: fact_row.page_number,
            bounding_box_json,
            citation_label: request.citation_label.trim().to_string(),
            created_at,
        };
        let fact_for_return = fact_row.clone();
        let citation_for_return = citation_row.clone();

        self.writer
            .exec_tx(move |tx| -> Result<()> {
                let conn = tx.conn();
                ensure_document_exists(conn, &fact_row.document_id)?;
                diesel::insert_into(extracted_facts::table)
                    .values(&fact_row)
                    .execute(conn)
                    .map_err(StorageError::from)?;
                diesel::insert_into(source_citations::table)
                    .values(&citation_row)
                    .execute(conn)
                    .map_err(StorageError::from)?;
                Ok(())
            })
            .await?;

        Ok(CreateExtractedFactResult {
            fact: ExtractedFactRow::from(fact_for_return).try_into()?,
            citation: SourceCitationRow::from(citation_for_return).try_into()?,
        })
    }

    pub fn list_pending_extracted_facts(&self) -> Result<Vec<ExtractedFact>> {
        let mut conn = get_connection(&self.pool)?;
        let rows = extracted_facts::table
            .filter(extracted_facts::status.eq(ExtractedFactStatus::Pending.as_str()))
            .order(extracted_facts::created_at.asc())
            .load::<ExtractedFactRow>(&mut conn)
            .map_err(StorageError::from)?;
        rows.into_iter().map(ExtractedFact::try_from).collect()
    }

    pub fn get_source_citation(&self, citation_id: &str) -> Result<SourceCitation> {
        let mut conn = get_connection(&self.pool)?;
        source_citations::table
            .find(citation_id)
            .first::<SourceCitationRow>(&mut conn)
            .optional()
            .map_err(StorageError::from)?
            .ok_or_else(|| Error::Database(DatabaseError::NotFound(citation_id.to_string())))?
            .try_into()
    }

    pub async fn approve_extracted_fact(
        &self,
        fact_id: &str,
        request: ReviewExtractedFactRequest,
    ) -> Result<ExtractedFact> {
        self.review_fact(fact_id, ExtractedFactStatus::Approved, request)
            .await
    }

    pub async fn reject_extracted_fact(
        &self,
        fact_id: &str,
        request: ReviewExtractedFactRequest,
    ) -> Result<ExtractedFact> {
        self.review_fact(fact_id, ExtractedFactStatus::Rejected, request)
            .await
    }

    async fn review_fact(
        &self,
        fact_id: &str,
        next_status: ExtractedFactStatus,
        request: ReviewExtractedFactRequest,
    ) -> Result<ExtractedFact> {
        let fact_id = fact_id.to_string();
        let reviewed_at = Utc::now().to_rfc3339();
        let review_notes = trimmed_optional(request.review_notes);
        self.writer
            .exec_tx({
                let fact_id = fact_id.clone();
                move |tx| -> Result<()> {
                    let conn = tx.conn();
                    let row = extracted_facts::table
                        .find(&fact_id)
                        .first::<ExtractedFactRow>(conn)
                        .optional()
                        .map_err(StorageError::from)?
                        .ok_or_else(|| {
                            Error::Database(DatabaseError::NotFound(fact_id.to_string()))
                        })?;
                    if row.status != ExtractedFactStatus::Pending.as_str() {
                        return Err(Error::Validation(ValidationError::InvalidInput(
                            "Only pending extracted facts can be reviewed".into(),
                        )));
                    }
                    ensure_document_exists(conn, &row.document_id)?;
                    diesel::update(extracted_facts::table.find(&fact_id))
                        .set((
                            extracted_facts::status.eq(next_status.as_str()),
                            extracted_facts::reviewed_at.eq(Some(reviewed_at.clone())),
                            extracted_facts::review_notes.eq(review_notes.clone()),
                        ))
                        .execute(conn)
                        .map_err(StorageError::from)?;
                    Ok(())
                }
            })
            .await?;
        self.get_extracted_fact(&fact_id)
    }

    fn get_extracted_fact(&self, fact_id: &str) -> Result<ExtractedFact> {
        let mut conn = get_connection(&self.pool)?;
        extracted_facts::table
            .find(fact_id)
            .first::<ExtractedFactRow>(&mut conn)
            .optional()
            .map_err(StorageError::from)?
            .ok_or_else(|| Error::Database(DatabaseError::NotFound(fact_id.to_string())))?
            .try_into()
    }
}

impl From<NewExtractedFactRow> for ExtractedFactRow {
    fn from(row: NewExtractedFactRow) -> Self {
        Self {
            id: row.id,
            document_id: row.document_id,
            page_number: row.page_number,
            fact_type: row.fact_type,
            raw_value: row.raw_value,
            normalized_value: row.normalized_value,
            currency: row.currency,
            date_value: row.date_value,
            confidence_score: row.confidence_score,
            bounding_box_json: row.bounding_box_json,
            extraction_method: row.extraction_method,
            extraction_version: row.extraction_version,
            status: row.status,
            created_at: row.created_at,
            reviewed_at: row.reviewed_at,
            review_notes: row.review_notes,
        }
    }
}

impl From<NewSourceCitationRow> for SourceCitationRow {
    fn from(row: NewSourceCitationRow) -> Self {
        Self {
            id: row.id,
            source_type: row.source_type,
            source_id: row.source_id,
            document_id: row.document_id,
            extracted_fact_id: row.extracted_fact_id,
            page_number: row.page_number,
            bounding_box_json: row.bounding_box_json,
            citation_label: row.citation_label,
            created_at: row.created_at,
        }
    }
}

impl TryFrom<ExtractedFactRow> for ExtractedFact {
    type Error = Error;

    fn try_from(row: ExtractedFactRow) -> std::result::Result<Self, Self::Error> {
        Ok(Self {
            id: row.id,
            document_id: row.document_id,
            page_number: row.page_number,
            fact_type: row.fact_type,
            raw_value: row.raw_value,
            normalized_value: row.normalized_value,
            currency: row.currency,
            date_value: row.date_value,
            confidence_score: row.confidence_score,
            bounding_box: deserialize_box(row.bounding_box_json.as_deref())?,
            extraction_method: ExtractionMethod::try_from(row.extraction_method.as_str())
                .map_err(|err| Error::Database(DatabaseError::Internal(err)))?,
            extraction_version: row.extraction_version,
            status: ExtractedFactStatus::try_from(row.status.as_str())
                .map_err(|err| Error::Database(DatabaseError::Internal(err)))?,
            created_at: parse_rfc3339(&row.created_at)?,
            reviewed_at: row.reviewed_at.as_deref().map(parse_rfc3339).transpose()?,
            review_notes: row.review_notes,
        })
    }
}

impl TryFrom<SourceCitationRow> for SourceCitation {
    type Error = Error;

    fn try_from(row: SourceCitationRow) -> std::result::Result<Self, Self::Error> {
        Ok(Self {
            id: row.id,
            source_type: SourceCitationType::try_from(row.source_type.as_str())
                .map_err(|err| Error::Database(DatabaseError::Internal(err)))?,
            source_id: row.source_id,
            document_id: row.document_id,
            extracted_fact_id: row.extracted_fact_id,
            page_number: row.page_number,
            bounding_box: deserialize_box(row.bounding_box_json.as_deref())?,
            citation_label: row.citation_label,
            created_at: parse_rfc3339(&row.created_at)?,
        })
    }
}

fn validate_create_request(request: &CreateExtractedFactRequest) -> Result<()> {
    if request.document_id.trim().is_empty() {
        return Err(Error::Validation(ValidationError::MissingField(
            "document_id".into(),
        )));
    }
    if request.fact_type.trim().is_empty() {
        return Err(Error::Validation(ValidationError::MissingField(
            "fact_type".into(),
        )));
    }
    if request.raw_value.trim().is_empty() {
        return Err(Error::Validation(ValidationError::MissingField(
            "raw_value".into(),
        )));
    }
    if request.extraction_version.trim().is_empty() {
        return Err(Error::Validation(ValidationError::MissingField(
            "extraction_version".into(),
        )));
    }
    if request.citation_label.trim().is_empty() {
        return Err(Error::Validation(ValidationError::MissingField(
            "citation_label".into(),
        )));
    }
    if matches!(request.page_number, Some(page) if page <= 0) {
        return Err(Error::Validation(ValidationError::InvalidInput(
            "page_number must be greater than zero".into(),
        )));
    }
    if matches!(request.confidence_score, Some(score) if !(0.0..=1.0).contains(&score)) {
        return Err(Error::Validation(ValidationError::InvalidInput(
            "confidence_score must be between 0 and 1".into(),
        )));
    }
    Ok(())
}

fn ensure_document_exists(conn: &mut SqliteConnection, document_id: &str) -> Result<()> {
    let exists = documents::table
        .find(document_id)
        .select(documents::id)
        .first::<String>(conn)
        .optional()
        .map_err(StorageError::from)?
        .is_some();
    if exists {
        Ok(())
    } else {
        Err(Error::Database(DatabaseError::NotFound(
            document_id.to_string(),
        )))
    }
}

fn serialize_box(value: Option<&DocumentBoundingBox>) -> Result<Option<String>> {
    value
        .map(serde_json::to_string)
        .transpose()
        .map_err(Into::into)
}

fn deserialize_box(value: Option<&str>) -> Result<Option<DocumentBoundingBox>> {
    value
        .map(serde_json::from_str)
        .transpose()
        .map_err(Into::into)
}

fn parse_rfc3339(value: &str) -> Result<DateTime<Utc>> {
    Ok(DateTime::parse_from_rfc3339(value)?.with_timezone(&Utc))
}

fn trimmed_optional(value: Option<String>) -> Option<String> {
    value.and_then(|raw| {
        let trimmed = raw.trim().to_string();
        (!trimmed.is_empty()).then_some(trimmed)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::write_actor::spawn_writer;
    use crate::db::{create_pool, init, run_migrations};
    use tempfile::tempdir;

    struct TestDb {
        pool: Arc<Pool<r2d2::ConnectionManager<SqliteConnection>>>,
        repo: ExtractedFactRepository,
        _app_data: tempfile::TempDir,
    }

    fn setup() -> TestDb {
        let app_data = tempdir().expect("tempdir");
        let db_path = init(app_data.path().to_str().expect("path")).expect("init");
        run_migrations(&db_path).expect("migrate");
        let pool = create_pool(&db_path).expect("pool");
        let writer = spawn_writer(pool.as_ref().clone()).expect("writer");
        let repo = ExtractedFactRepository::new(pool.clone(), writer);
        TestDb {
            pool,
            repo,
            _app_data: app_data,
        }
    }

    fn seed_document(pool: &Arc<Pool<r2d2::ConnectionManager<SqliteConnection>>>, id: &str) {
        let mut conn = get_connection(pool).expect("conn");
        diesel::insert_into(documents::table)
            .values((
                documents::id.eq(id),
                documents::file_hash.eq(format!("hash-{id}")),
                documents::original_name.eq("statement.pdf"),
                documents::mime_type.eq("application/pdf"),
                documents::file_size_bytes.eq(100_i64),
                documents::encrypted_storage_path.eq(format!("{id}.mizdoc")),
                documents::status.eq("processed"),
                documents::source_type.eq::<Option<String>>(None),
                documents::error_message.eq::<Option<String>>(None),
                documents::created_at.eq("2026-05-14T00:00:00Z"),
                documents::updated_at.eq("2026-05-14T00:00:00Z"),
            ))
            .execute(&mut conn)
            .expect("seed document");
    }

    fn request(document_id: &str) -> CreateExtractedFactRequest {
        CreateExtractedFactRequest {
            document_id: document_id.to_string(),
            page_number: Some(1),
            fact_type: "statement_balance".into(),
            raw_value: "$1,250.00".into(),
            normalized_value: Some("1250.00".into()),
            currency: Some("usd".into()),
            date_value: None,
            confidence_score: Some(0.91),
            bounding_box: Some(DocumentBoundingBox {
                x: 1.0,
                y: 2.0,
                width: 3.0,
                height: 4.0,
            }),
            extraction_method: ExtractionMethod::Parser,
            extraction_version: "local-parser-v1".into(),
            citation_label: "statement.pdf p.1".into(),
        }
    }

    #[tokio::test]
    async fn foreign_key_rejects_missing_document() {
        let db = setup();
        let err = db
            .repo
            .create_extracted_fact(request("missing-doc"))
            .await
            .expect_err("missing document rejected");
        assert!(err.to_string().contains("Record not found"));
    }

    #[tokio::test]
    async fn source_citation_foreign_key_is_enforced() {
        let db = setup();
        seed_document(&db.pool, "doc-1");
        let mut conn = get_connection(&db.pool).expect("conn");
        let err = diesel::sql_query(
            "
            INSERT INTO source_citations (
                id,
                source_type,
                document_id,
                extracted_fact_id,
                citation_label,
                created_at
            )
            VALUES (
                'citation-1',
                'document',
                'doc-1',
                'missing-fact',
                'statement.pdf p.1',
                '2026-05-14T00:00:00Z'
            )
            ",
        )
        .execute(&mut conn)
        .expect_err("missing fact rejected");
        assert!(err.to_string().contains("FOREIGN KEY"));
    }

    #[tokio::test]
    async fn pending_fact_can_be_approved() {
        let db = setup();
        seed_document(&db.pool, "doc-1");
        let created = db
            .repo
            .create_extracted_fact(request("doc-1"))
            .await
            .expect("create fact");

        let approved = db
            .repo
            .approve_extracted_fact(
                &created.fact.id,
                ReviewExtractedFactRequest {
                    review_notes: Some("Matches statement".into()),
                },
            )
            .await
            .expect("approve fact");

        assert_eq!(approved.status, ExtractedFactStatus::Approved);
        assert!(approved.reviewed_at.is_some());
        assert_eq!(approved.review_notes.as_deref(), Some("Matches statement"));
        assert!(db
            .repo
            .list_pending_extracted_facts()
            .expect("list")
            .is_empty());
    }

    #[tokio::test]
    async fn pending_fact_can_be_rejected() {
        let db = setup();
        seed_document(&db.pool, "doc-1");
        let created = db
            .repo
            .create_extracted_fact(request("doc-1"))
            .await
            .expect("create fact");

        let rejected = db
            .repo
            .reject_extracted_fact(
                &created.fact.id,
                ReviewExtractedFactRequest {
                    review_notes: Some("Wrong account".into()),
                },
            )
            .await
            .expect("reject fact");

        assert_eq!(rejected.status, ExtractedFactStatus::Rejected);
        assert!(rejected.reviewed_at.is_some());
        assert_eq!(rejected.review_notes.as_deref(), Some("Wrong account"));
    }

    #[tokio::test]
    async fn approved_fact_cannot_be_approved_again() {
        let db = setup();
        seed_document(&db.pool, "doc-1");
        let created = db
            .repo
            .create_extracted_fact(request("doc-1"))
            .await
            .expect("create fact");
        db.repo
            .approve_extracted_fact(
                &created.fact.id,
                ReviewExtractedFactRequest { review_notes: None },
            )
            .await
            .expect("approve fact");

        let err = db
            .repo
            .approve_extracted_fact(
                &created.fact.id,
                ReviewExtractedFactRequest { review_notes: None },
            )
            .await
            .expect_err("second approval rejected");
        assert!(err.to_string().contains("Only pending extracted facts"));
    }

    #[tokio::test]
    async fn cannot_approve_deleted_document_fact() {
        let db = setup();
        seed_document(&db.pool, "doc-1");
        let created = db
            .repo
            .create_extracted_fact(request("doc-1"))
            .await
            .expect("create fact");
        let mut conn = get_connection(&db.pool).expect("conn");
        diesel::delete(documents::table.find("doc-1"))
            .execute(&mut conn)
            .expect("delete document");

        let err = db
            .repo
            .approve_extracted_fact(
                &created.fact.id,
                ReviewExtractedFactRequest { review_notes: None },
            )
            .await
            .expect_err("deleted document cannot be approved");
        assert!(err.to_string().contains("Record not found"));
    }

    #[tokio::test]
    async fn citation_lookup_round_trips_document_source() {
        let db = setup();
        seed_document(&db.pool, "doc-1");
        let created = db
            .repo
            .create_extracted_fact(request("doc-1"))
            .await
            .expect("create fact");

        let citation = db
            .repo
            .get_source_citation(&created.citation.id)
            .expect("get citation");

        assert_eq!(citation.source_type, SourceCitationType::Document);
        assert_eq!(citation.document_id.as_deref(), Some("doc-1"));
        assert_eq!(
            citation.extracted_fact_id.as_deref(),
            Some(created.fact.id.as_str())
        );
        assert_eq!(citation.page_number, Some(1));
        assert_eq!(citation.citation_label, "statement.pdf p.1");
    }
}
