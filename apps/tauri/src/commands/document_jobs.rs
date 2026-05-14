//! Tauri commands for Document Vault processing jobs.

use std::sync::Arc;

use log::error;
use tauri::State;

use mizan_core::documents::{
    DocumentParserCapabilities, DocumentProcessingJob, EnqueueDocumentJobRequest, ParsedDocument,
    RunDocumentJobResult,
};

use crate::context::ServiceContext;

#[tauri::command]
pub async fn enqueue_document_job(
    request: EnqueueDocumentJobRequest,
    state: State<'_, Arc<ServiceContext>>,
) -> Result<DocumentProcessingJob, String> {
    state
        .document_job_repository()
        .enqueue(request)
        .await
        .map_err(|err| {
            error!("enqueue_document_job failed: {}", err);
            err.to_string()
        })
}

#[tauri::command]
pub fn list_document_jobs(
    document_id: Option<String>,
    state: State<'_, Arc<ServiceContext>>,
) -> Result<Vec<DocumentProcessingJob>, String> {
    state
        .document_job_repository()
        .list(document_id.as_deref())
        .map_err(|err| {
            error!("list_document_jobs failed: {}", err);
            err.to_string()
        })
}

#[tauri::command]
pub fn get_document_parser_capabilities(
    state: State<'_, Arc<ServiceContext>>,
) -> Result<DocumentParserCapabilities, String> {
    Ok(state.document_job_repository().processor_capabilities())
}

#[tauri::command]
pub fn get_parsed_document(
    document_id: String,
    state: State<'_, Arc<ServiceContext>>,
) -> Result<ParsedDocument, String> {
    state
        .document_extraction_repository()
        .get_parsed_document(&document_id)
        .map_err(|err| {
            error!("get_parsed_document failed: {}", err);
            err.to_string()
        })
}

#[tauri::command]
pub async fn run_next_document_job(
    state: State<'_, Arc<ServiceContext>>,
) -> Result<RunDocumentJobResult, String> {
    state
        .document_job_repository()
        .run_next()
        .await
        .map_err(|err| {
            error!("run_next_document_job failed: {}", err);
            err.to_string()
        })
}

#[tauri::command]
pub async fn cancel_document_job(
    job_id: String,
    state: State<'_, Arc<ServiceContext>>,
) -> Result<DocumentProcessingJob, String> {
    state
        .document_job_repository()
        .cancel(&job_id)
        .await
        .map_err(|err| {
            error!("cancel_document_job failed: {}", err);
            err.to_string()
        })
}

#[tauri::command]
pub async fn retry_document_job(
    job_id: String,
    state: State<'_, Arc<ServiceContext>>,
) -> Result<DocumentProcessingJob, String> {
    state
        .document_job_repository()
        .retry_failed(&job_id)
        .await
        .map_err(|err| {
            error!("retry_document_job failed: {}", err);
            err.to_string()
        })
}
