//! REST endpoints for Document Vault processing jobs.

use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;

use mizan_core::documents::{
    DocumentParserCapabilities, DocumentProcessingJob, EnqueueDocumentJobRequest, ParsedDocument,
    RunDocumentJobResult,
};

use crate::error::{ApiError, ApiResult};
use crate::main_lib::AppState;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ListDocumentJobsQuery {
    document_id: Option<String>,
}

async fn enqueue_document_job(
    State(state): State<Arc<AppState>>,
    Json(request): Json<EnqueueDocumentJobRequest>,
) -> ApiResult<Json<DocumentProcessingJob>> {
    let job = state
        .document_job_repository
        .enqueue(request)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(job))
}

async fn list_document_jobs(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListDocumentJobsQuery>,
) -> ApiResult<Json<Vec<DocumentProcessingJob>>> {
    let jobs = state
        .document_job_repository
        .list(query.document_id.as_deref())
        .map_err(ApiError::from)?;
    Ok(Json(jobs))
}

async fn get_document_parser_capabilities(
    State(state): State<Arc<AppState>>,
) -> Json<DocumentParserCapabilities> {
    Json(state.document_job_repository.processor_capabilities())
}

async fn get_parsed_document(
    State(state): State<Arc<AppState>>,
    Path(document_id): Path<String>,
) -> ApiResult<Json<ParsedDocument>> {
    let parsed = state
        .document_extraction_repository
        .get_parsed_document(&document_id)
        .map_err(ApiError::from)?;
    Ok(Json(parsed))
}

async fn run_next_document_job(
    State(state): State<Arc<AppState>>,
) -> ApiResult<Json<RunDocumentJobResult>> {
    let result = state
        .document_job_repository
        .run_next()
        .await
        .map_err(ApiError::from)?;
    Ok(Json(result))
}

async fn cancel_document_job(
    State(state): State<Arc<AppState>>,
    Path(job_id): Path<String>,
) -> ApiResult<Json<DocumentProcessingJob>> {
    let job = state
        .document_job_repository
        .cancel(&job_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(job))
}

async fn retry_document_job(
    State(state): State<Arc<AppState>>,
    Path(job_id): Path<String>,
) -> ApiResult<Json<DocumentProcessingJob>> {
    let job = state
        .document_job_repository
        .retry_failed(&job_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(job))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route(
            "/document-jobs",
            get(list_document_jobs).post(enqueue_document_job),
        )
        .route(
            "/document-jobs/capabilities",
            get(get_document_parser_capabilities),
        )
        .route("/documents/{document_id}/parsed", get(get_parsed_document))
        .route("/document-jobs/run-next", post(run_next_document_job))
        .route("/document-jobs/{job_id}/cancel", post(cancel_document_job))
        .route("/document-jobs/{job_id}/retry", post(retry_document_job))
}
