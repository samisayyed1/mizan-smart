//! Tauri commands for the encrypted Document Vault.

use std::sync::Arc;

use log::error;
use tauri::State;

use mizan_core::documents::{DocumentMetadata, DocumentRecord, UploadDocumentRequest};

use crate::context::ServiceContext;

#[tauri::command]
pub async fn upload_document(
    request: UploadDocumentRequest,
    state: State<'_, Arc<ServiceContext>>,
) -> Result<DocumentRecord, String> {
    state
        .document_vault_repository()
        .upload(request)
        .await
        .map_err(|err| {
            error!("upload_document failed: {}", err);
            err.to_string()
        })
}

#[tauri::command]
pub fn list_documents(
    state: State<'_, Arc<ServiceContext>>,
) -> Result<Vec<DocumentMetadata>, String> {
    state.document_vault_repository().list().map_err(|err| {
        error!("list_documents failed: {}", err);
        err.to_string()
    })
}

#[tauri::command]
pub fn get_document_metadata(
    document_id: String,
    state: State<'_, Arc<ServiceContext>>,
) -> Result<DocumentRecord, String> {
    state
        .document_vault_repository()
        .get_metadata(&document_id)
        .map_err(|err| {
            error!("get_document_metadata failed: {}", err);
            err.to_string()
        })
}

#[tauri::command]
pub async fn delete_document(
    document_id: String,
    state: State<'_, Arc<ServiceContext>>,
) -> Result<(), String> {
    state
        .document_vault_repository()
        .delete(&document_id)
        .await
        .map_err(|err| {
            error!("delete_document failed: {}", err);
            err.to_string()
        })
}

#[tauri::command]
pub fn read_document_bytes(
    document_id: String,
    state: State<'_, Arc<ServiceContext>>,
) -> Result<Vec<u8>, String> {
    state
        .document_vault_repository()
        .read_decrypted(&document_id)
        .map_err(|err| {
            error!("read_document_bytes failed: {}", err);
            err.to_string()
        })
}
