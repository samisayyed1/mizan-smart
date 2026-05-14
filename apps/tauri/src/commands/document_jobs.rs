//! Tauri commands for Document Vault processing jobs.

use std::sync::Arc;

use log::error;
use tauri::State;

use mizan_core::documents::{
    DocumentProcessingJob, EnqueueDocumentJobRequest, RunDocumentJobResult,
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
