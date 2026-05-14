use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel::r2d2::{self, Pool};
use diesel::SqliteConnection;
use tokio::time;
use uuid::Uuid;

use mizan_core::documents::{
    DocumentJobStatus, DocumentJobType, DocumentParserCapabilities, DocumentProcessingJob,
    EnqueueDocumentJobRequest, RunDocumentJobResult,
};
use mizan_core::errors::{DatabaseError, Error, ValidationError};
use mizan_core::Result;

use crate::db::{get_connection, WriteHandle};
use crate::errors::StorageError;
use crate::schema::{document_processing_jobs, documents};

const DEFAULT_JOB_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Debug, Clone, Queryable, Identifiable)]
#[diesel(table_name = document_processing_jobs)]
struct DocumentProcessingJobRow {
    id: String,
    document_id: String,
    job_type: String,
    status: String,
    priority: i32,
    attempts: i32,
    max_attempts: i32,
    error_message: Option<String>,
    started_at: Option<String>,
    completed_at: Option<String>,
    created_at: String,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = document_processing_jobs)]
pub(crate) struct NewDocumentProcessingJobRow {
    id: String,
    document_id: String,
    job_type: String,
    status: String,
    priority: i32,
    attempts: i32,
    max_attempts: i32,
    error_message: Option<String>,
    started_at: Option<String>,
    completed_at: Option<String>,
    created_at: String,
}

impl TryFrom<DocumentProcessingJobRow> for DocumentProcessingJob {
    type Error = Error;

    fn try_from(row: DocumentProcessingJobRow) -> std::result::Result<Self, Self::Error> {
        Ok(Self {
            id: row.id,
            document_id: row.document_id,
            job_type: DocumentJobType::try_from(row.job_type.as_str())
                .map_err(|err| Error::Database(DatabaseError::Internal(err)))?,
            status: DocumentJobStatus::try_from(row.status.as_str())
                .map_err(|err| Error::Database(DatabaseError::Internal(err)))?,
            priority: row.priority,
            attempts: row.attempts,
            max_attempts: row.max_attempts,
            error_message: row.error_message,
            started_at: row.started_at.as_deref().map(parse_rfc3339).transpose()?,
            completed_at: row.completed_at.as_deref().map(parse_rfc3339).transpose()?,
            created_at: parse_rfc3339(&row.created_at)?,
        })
    }
}

#[async_trait]
pub trait DocumentJobProcessor: Send + Sync {
    fn capabilities(&self) -> DocumentParserCapabilities;
    async fn process(&self, job: &DocumentProcessingJob) -> Result<()>;
}

#[derive(Debug, Default)]
pub struct UnavailableDocumentJobProcessor;

#[async_trait]
impl DocumentJobProcessor for UnavailableDocumentJobProcessor {
    fn capabilities(&self) -> DocumentParserCapabilities {
        DocumentParserCapabilities {
            text: false,
            layout: false,
            tables: false,
            ocr: false,
        }
    }

    async fn process(&self, job: &DocumentProcessingJob) -> Result<()> {
        let capability = match job.job_type {
            DocumentJobType::ParseText => "text parser",
            DocumentJobType::ExtractLayout => "layout extractor",
            DocumentJobType::ExtractTables => "table extractor",
            DocumentJobType::Ocr => "OCR runtime",
            DocumentJobType::VlmExtract => "vision-language extraction runtime",
            DocumentJobType::Embed => "embedding runtime",
        };
        Err(Error::Validation(ValidationError::InvalidInput(format!(
            "Document {capability} is not available on this machine"
        ))))
    }
}

pub struct DocumentJobRepository {
    pool: Arc<Pool<r2d2::ConnectionManager<SqliteConnection>>>,
    writer: WriteHandle,
    processor: Arc<dyn DocumentJobProcessor>,
    timeout: Duration,
}

impl DocumentJobRepository {
    pub fn new(
        pool: Arc<Pool<r2d2::ConnectionManager<SqliteConnection>>>,
        writer: WriteHandle,
    ) -> Self {
        Self::new_with_processor(
            pool,
            writer,
            Arc::new(UnavailableDocumentJobProcessor),
            DEFAULT_JOB_TIMEOUT,
        )
    }

    pub fn new_with_processor(
        pool: Arc<Pool<r2d2::ConnectionManager<SqliteConnection>>>,
        writer: WriteHandle,
        processor: Arc<dyn DocumentJobProcessor>,
        timeout: Duration,
    ) -> Self {
        Self {
            pool,
            writer,
            processor,
            timeout,
        }
    }

    pub fn processor_capabilities(&self) -> DocumentParserCapabilities {
        self.processor.capabilities()
    }

    pub async fn enqueue(
        &self,
        request: EnqueueDocumentJobRequest,
    ) -> Result<DocumentProcessingJob> {
        validate_enqueue_request(&request)?;
        let now = Utc::now();
        let row = new_job_row(
            request.document_id,
            request.job_type,
            request.priority,
            request.max_attempts,
            now,
        );
        let job_id = row.id.clone();
        self.writer
            .exec(move |conn| -> Result<()> {
                ensure_document_exists(conn, &row.document_id)?;
                diesel::insert_into(document_processing_jobs::table)
                    .values(&row)
                    .execute(conn)
                    .map_err(StorageError::from)?;
                Ok(())
            })
            .await?;
        self.get(&job_id)
    }

    pub fn list(&self, document_filter: Option<&str>) -> Result<Vec<DocumentProcessingJob>> {
        let mut conn = get_connection(&self.pool)?;
        let rows = match document_filter {
            Some(document_id) => document_processing_jobs::table
                .filter(document_processing_jobs::document_id.eq(document_id))
                .order((
                    document_processing_jobs::created_at.desc(),
                    document_processing_jobs::priority.desc(),
                ))
                .load::<DocumentProcessingJobRow>(&mut conn)
                .map_err(StorageError::from)?,
            None => document_processing_jobs::table
                .order((
                    document_processing_jobs::created_at.desc(),
                    document_processing_jobs::priority.desc(),
                ))
                .load::<DocumentProcessingJobRow>(&mut conn)
                .map_err(StorageError::from)?,
        };
        rows.into_iter()
            .map(DocumentProcessingJob::try_from)
            .collect()
    }

    pub fn get(&self, job_id: &str) -> Result<DocumentProcessingJob> {
        let mut conn = get_connection(&self.pool)?;
        let row = document_processing_jobs::table
            .find(job_id)
            .first::<DocumentProcessingJobRow>(&mut conn)
            .optional()
            .map_err(StorageError::from)?
            .ok_or_else(|| Error::Database(DatabaseError::NotFound(job_id.to_string())))?;
        DocumentProcessingJob::try_from(row)
    }

    pub async fn cancel(&self, job_id: &str) -> Result<DocumentProcessingJob> {
        let job_id = job_id.to_string();
        let write_job_id = job_id.clone();
        self.writer
            .exec(move |conn| -> Result<()> {
                let current = document_processing_jobs::table
                    .find(&write_job_id)
                    .first::<DocumentProcessingJobRow>(conn)
                    .optional()
                    .map_err(StorageError::from)?
                    .ok_or_else(|| {
                        Error::Database(DatabaseError::NotFound(write_job_id.clone()))
                    })?;
                if matches!(
                    DocumentJobStatus::try_from(current.status.as_str())
                        .map_err(|err| { Error::Database(DatabaseError::Internal(err)) })?,
                    DocumentJobStatus::Succeeded | DocumentJobStatus::Cancelled
                ) {
                    return Err(Error::Validation(ValidationError::InvalidInput(
                        "Only queued, running, or failed document jobs can be cancelled".into(),
                    )));
                }
                diesel::update(document_processing_jobs::table.find(&write_job_id))
                    .set((
                        document_processing_jobs::status.eq(DocumentJobStatus::Cancelled.as_str()),
                        document_processing_jobs::completed_at.eq(Some(Utc::now().to_rfc3339())),
                    ))
                    .execute(conn)
                    .map_err(StorageError::from)?;
                Ok(())
            })
            .await?;
        self.get(&job_id)
    }

    pub async fn retry_failed(&self, job_id: &str) -> Result<DocumentProcessingJob> {
        let job_id = job_id.to_string();
        let write_job_id = job_id.clone();
        self.writer
            .exec(move |conn| -> Result<()> {
                let current = document_processing_jobs::table
                    .find(&write_job_id)
                    .first::<DocumentProcessingJobRow>(conn)
                    .optional()
                    .map_err(StorageError::from)?
                    .ok_or_else(|| {
                        Error::Database(DatabaseError::NotFound(write_job_id.clone()))
                    })?;
                let status = DocumentJobStatus::try_from(current.status.as_str())
                    .map_err(|err| Error::Database(DatabaseError::Internal(err)))?;
                if status != DocumentJobStatus::Failed {
                    return Err(Error::Validation(ValidationError::InvalidInput(
                        "Only failed document jobs can be retried".into(),
                    )));
                }
                if current.attempts >= current.max_attempts {
                    return Err(Error::Validation(ValidationError::InvalidInput(
                        "Document job retry limit reached".into(),
                    )));
                }
                diesel::update(document_processing_jobs::table.find(&write_job_id))
                    .set((
                        document_processing_jobs::status.eq(DocumentJobStatus::Queued.as_str()),
                        document_processing_jobs::error_message.eq::<Option<String>>(None),
                        document_processing_jobs::started_at.eq::<Option<String>>(None),
                        document_processing_jobs::completed_at.eq::<Option<String>>(None),
                    ))
                    .execute(conn)
                    .map_err(StorageError::from)?;
                Ok(())
            })
            .await?;
        self.get(&job_id)
    }

    pub async fn run_next(&self) -> Result<RunDocumentJobResult> {
        let job = self.claim_next_job().await?;
        let Some(job) = job else {
            return Ok(RunDocumentJobResult { job: None });
        };

        let result = time::timeout(self.timeout, self.processor.process(&job)).await;
        match result {
            Ok(Ok(())) => self.finish_job(&job.id, true, None).await?,
            Ok(Err(err)) => {
                self.finish_job(&job.id, false, Some(err.to_string()))
                    .await?
            }
            Err(_) => {
                self.finish_job(
                    &job.id,
                    false,
                    Some("Document job timed out before completion".into()),
                )
                .await?
            }
        }
        Ok(RunDocumentJobResult {
            job: Some(self.get(&job.id)?),
        })
    }

    async fn claim_next_job(&self) -> Result<Option<DocumentProcessingJob>> {
        let claimed_id = self
            .writer
            .exec(move |conn| -> Result<Option<String>> {
                let candidate = document_processing_jobs::table
                    .filter(document_processing_jobs::status.eq(DocumentJobStatus::Queued.as_str()))
                    .filter(
                        document_processing_jobs::attempts
                            .lt(document_processing_jobs::max_attempts),
                    )
                    .order((
                        document_processing_jobs::priority.desc(),
                        document_processing_jobs::created_at.asc(),
                    ))
                    .first::<DocumentProcessingJobRow>(conn)
                    .optional()
                    .map_err(StorageError::from)?;
                let Some(candidate) = candidate else {
                    return Ok(None);
                };
                diesel::update(document_processing_jobs::table.find(&candidate.id))
                    .set((
                        document_processing_jobs::status.eq(DocumentJobStatus::Running.as_str()),
                        document_processing_jobs::attempts.eq(candidate.attempts + 1),
                        document_processing_jobs::started_at.eq(Some(Utc::now().to_rfc3339())),
                        document_processing_jobs::completed_at.eq::<Option<String>>(None),
                        document_processing_jobs::error_message.eq::<Option<String>>(None),
                    ))
                    .execute(conn)
                    .map_err(StorageError::from)?;
                Ok(Some(candidate.id))
            })
            .await?;
        claimed_id.map(|id| self.get(&id)).transpose()
    }

    async fn finish_job(&self, job_id: &str, succeeded: bool, error: Option<String>) -> Result<()> {
        let job_id = job_id.to_string();
        self.writer
            .exec(move |conn| -> Result<()> {
                let completed = Utc::now().to_rfc3339();
                let status = if succeeded {
                    DocumentJobStatus::Succeeded
                } else {
                    DocumentJobStatus::Failed
                };
                diesel::update(document_processing_jobs::table.find(&job_id))
                    .set((
                        document_processing_jobs::status.eq(status.as_str()),
                        document_processing_jobs::error_message.eq(error),
                        document_processing_jobs::completed_at.eq(Some(completed)),
                    ))
                    .execute(conn)
                    .map_err(StorageError::from)?;
                Ok(())
            })
            .await
    }
}

pub(crate) fn new_parse_text_job_row(
    document_id: String,
    now: DateTime<Utc>,
) -> NewDocumentProcessingJobRow {
    new_job_row(document_id, DocumentJobType::ParseText, 0, 3, now)
}

pub(crate) fn insert_job_row(
    conn: &mut SqliteConnection,
    row: &NewDocumentProcessingJobRow,
) -> Result<()> {
    diesel::insert_into(document_processing_jobs::table)
        .values(row)
        .execute(conn)
        .map_err(StorageError::from)?;
    Ok(())
}

fn new_job_row(
    document_id: String,
    job_type: DocumentJobType,
    priority: i32,
    max_attempts: i32,
    now: DateTime<Utc>,
) -> NewDocumentProcessingJobRow {
    NewDocumentProcessingJobRow {
        id: Uuid::new_v4().to_string(),
        document_id,
        job_type: job_type.as_str().to_string(),
        status: DocumentJobStatus::Queued.as_str().to_string(),
        priority,
        attempts: 0,
        max_attempts,
        error_message: None,
        started_at: None,
        completed_at: None,
        created_at: now.to_rfc3339(),
    }
}

fn validate_enqueue_request(request: &EnqueueDocumentJobRequest) -> Result<()> {
    if request.document_id.trim().is_empty() {
        return Err(Error::Validation(ValidationError::MissingField(
            "document_id".into(),
        )));
    }
    if request.max_attempts <= 0 {
        return Err(Error::Validation(ValidationError::InvalidInput(
            "max_attempts must be greater than zero".into(),
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
        .map_err(StorageError::from)?;
    if exists.is_none() {
        return Err(Error::Database(DatabaseError::NotFound(
            document_id.to_string(),
        )));
    }
    Ok(())
}

fn parse_rfc3339(value: &str) -> Result<DateTime<Utc>> {
    Ok(DateTime::parse_from_rfc3339(value)?.with_timezone(&Utc))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::write_actor;
    use crate::db::{create_pool, init, run_migrations};
    use crate::documents::{
        generate_document_vault_key_hex, parse_document_vault_key_hex, DocumentVaultRepository,
    };
    use mizan_core::documents::UploadDocumentRequest;
    use tempfile::tempdir;

    struct TestContext {
        vault: DocumentVaultRepository,
        jobs: DocumentJobRepository,
        _app_data: tempfile::TempDir,
    }

    #[derive(Debug, Default)]
    struct SuccessfulProcessor;

    #[async_trait]
    impl DocumentJobProcessor for SuccessfulProcessor {
        fn capabilities(&self) -> DocumentParserCapabilities {
            DocumentParserCapabilities {
                text: true,
                layout: false,
                tables: false,
                ocr: false,
            }
        }

        async fn process(&self, _job: &DocumentProcessingJob) -> Result<()> {
            Ok(())
        }
    }

    fn upload_request() -> UploadDocumentRequest {
        UploadDocumentRequest {
            original_name: format!("statement-{}.pdf", Uuid::new_v4()),
            mime_type: "application/pdf".into(),
            content: Uuid::new_v4().as_bytes().to_vec(),
            source_type: None,
        }
    }

    fn setup(processor: Arc<dyn DocumentJobProcessor>) -> TestContext {
        let app_data = tempdir().expect("tempdir");
        let db_path = init(app_data.path().to_str().expect("path")).expect("init");
        run_migrations(&db_path).expect("migrate");
        let pool = create_pool(&db_path).expect("pool");
        let writer = write_actor::spawn_writer(pool.as_ref().clone()).expect("writer");
        let key = parse_document_vault_key_hex(&generate_document_vault_key_hex()).expect("key");
        let vault = DocumentVaultRepository::new(
            pool.clone(),
            writer.clone(),
            app_data.path().join("document-vault"),
            key,
        )
        .expect("vault");
        let jobs = DocumentJobRepository::new_with_processor(
            pool,
            writer,
            processor,
            Duration::from_secs(1),
        );
        TestContext {
            vault,
            jobs,
            _app_data: app_data,
        }
    }

    #[tokio::test]
    async fn worker_runs_queued_job() {
        let ctx = setup(Arc::new(SuccessfulProcessor));
        ctx.vault.upload(upload_request()).await.expect("upload");
        let result = ctx.jobs.run_next().await.expect("run next");
        let job = result.job.expect("job");
        assert_eq!(job.status, DocumentJobStatus::Succeeded);
        assert_eq!(job.attempts, 1);
        assert!(job.completed_at.is_some());
    }

    #[tokio::test]
    async fn unavailable_parser_failure_is_stored() {
        let ctx = setup(Arc::new(UnavailableDocumentJobProcessor));
        ctx.vault.upload(upload_request()).await.expect("upload");
        let result = ctx.jobs.run_next().await.expect("run next");
        let job = result.job.expect("job");
        assert_eq!(job.status, DocumentJobStatus::Failed);
        assert!(job.error_message.expect("error").contains("not available"));
    }

    #[tokio::test]
    async fn retry_limit_is_enforced() {
        let ctx = setup(Arc::new(UnavailableDocumentJobProcessor));
        let record = ctx.vault.upload(upload_request()).await.expect("upload");
        let queued = ctx
            .jobs
            .list(Some(&record.document.id))
            .expect("list jobs")
            .into_iter()
            .next()
            .expect("job");
        ctx.jobs
            .writer
            .exec({
                let job_id = queued.id.clone();
                move |conn| -> Result<()> {
                    diesel::update(document_processing_jobs::table.find(&job_id))
                        .set(document_processing_jobs::max_attempts.eq(1))
                        .execute(conn)
                        .map_err(StorageError::from)?;
                    Ok(())
                }
            })
            .await
            .expect("set max attempts");
        let failed = ctx.jobs.run_next().await.expect("run").job.expect("job");
        assert_eq!(failed.status, DocumentJobStatus::Failed);
        let err = ctx.jobs.retry_failed(&failed.id).await.expect_err("limit");
        assert!(err.to_string().contains("retry limit"));
    }

    #[tokio::test]
    async fn cancellation_marks_job_cancelled() {
        let ctx = setup(Arc::new(SuccessfulProcessor));
        let record = ctx.vault.upload(upload_request()).await.expect("upload");
        let queued = ctx
            .jobs
            .list(Some(&record.document.id))
            .expect("list jobs")
            .into_iter()
            .next()
            .expect("job");
        let cancelled = ctx.jobs.cancel(&queued.id).await.expect("cancel");
        assert_eq!(cancelled.status, DocumentJobStatus::Cancelled);
        assert!(cancelled.completed_at.is_some());
    }
}
