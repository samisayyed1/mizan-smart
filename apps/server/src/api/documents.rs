//! REST endpoints for the encrypted Document Vault.

use std::sync::Arc;

use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};

use mizan_core::documents::{DocumentMetadata, DocumentRecord, UploadDocumentRequest};

use crate::error::{ApiError, ApiResult};
use crate::main_lib::AppState;

async fn upload_document(
    State(state): State<Arc<AppState>>,
    Json(request): Json<UploadDocumentRequest>,
) -> ApiResult<Json<DocumentRecord>> {
    let record = state
        .document_vault_repository
        .upload(request)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(record))
}

async fn list_documents(
    State(state): State<Arc<AppState>>,
) -> ApiResult<Json<Vec<DocumentMetadata>>> {
    let rows = state
        .document_vault_repository
        .list()
        .map_err(ApiError::from)?;
    Ok(Json(rows))
}

async fn get_document_metadata(
    State(state): State<Arc<AppState>>,
    Path(document_id): Path<String>,
) -> ApiResult<Json<DocumentRecord>> {
    let record = state
        .document_vault_repository
        .get_metadata(&document_id)
        .map_err(ApiError::from)?;
    Ok(Json(record))
}

async fn delete_document(
    State(state): State<Arc<AppState>>,
    Path(document_id): Path<String>,
) -> ApiResult<Json<()>> {
    state
        .document_vault_repository
        .delete(&document_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(()))
}

async fn read_document_bytes(
    State(state): State<Arc<AppState>>,
    Path(document_id): Path<String>,
) -> ApiResult<Json<Vec<u8>>> {
    let bytes = state
        .document_vault_repository
        .read_decrypted(&document_id)
        .map_err(ApiError::from)?;
    Ok(Json(bytes))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/documents", get(list_documents).post(upload_document))
        .route(
            "/documents/{document_id}",
            get(get_document_metadata).delete(delete_document),
        )
        .route("/documents/{document_id}/content", get(read_document_bytes))
}
