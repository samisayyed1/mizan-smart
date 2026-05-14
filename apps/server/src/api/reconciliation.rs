//! REST endpoints for deterministic reconciliation runs.

use std::sync::Arc;

use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};

use mizan_core::reconciliation::{
    AcceptReconciliationAdjustmentRequest, AcceptReconciliationAdjustmentResult,
    IgnoreReconciliationMatchRequest, ManualReconciliationMatchRequest, ReconcileAccountRequest,
    ReconcileDocumentFactsRequest, ReconcileImportPreviewRequest, ReconciliationRepositoryTrait,
    ReconciliationRunDetail,
};

use crate::error::{ApiError, ApiResult};
use crate::main_lib::AppState;

async fn reconcile_import_preview(
    State(state): State<Arc<AppState>>,
    Json(request): Json<ReconcileImportPreviewRequest>,
) -> ApiResult<Json<ReconciliationRunDetail>> {
    let detail = state
        .reconciliation_repository
        .reconcile_import_preview(request)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(detail))
}

async fn reconcile_account(
    State(state): State<Arc<AppState>>,
    Json(request): Json<ReconcileAccountRequest>,
) -> ApiResult<Json<ReconciliationRunDetail>> {
    let detail = state
        .reconciliation_repository
        .reconcile_account(request)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(detail))
}

async fn reconcile_document_facts(
    State(state): State<Arc<AppState>>,
    Json(request): Json<ReconcileDocumentFactsRequest>,
) -> ApiResult<Json<ReconciliationRunDetail>> {
    let detail = state
        .reconciliation_repository
        .reconcile_document_facts(request)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(detail))
}

async fn get_reconciliation_run(
    State(state): State<Arc<AppState>>,
    Path(run_id): Path<String>,
) -> ApiResult<Json<ReconciliationRunDetail>> {
    let detail = state
        .reconciliation_repository
        .get_reconciliation_run(&run_id)
        .map_err(ApiError::from)?;
    Ok(Json(detail))
}

async fn accept_adjustment(
    State(state): State<Arc<AppState>>,
    Json(request): Json<AcceptReconciliationAdjustmentRequest>,
) -> ApiResult<Json<AcceptReconciliationAdjustmentResult>> {
    let result = state
        .reconciliation_repository
        .accept_adjustment(request)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(result))
}

async fn ignore_match(
    State(state): State<Arc<AppState>>,
    Json(request): Json<IgnoreReconciliationMatchRequest>,
) -> ApiResult<()> {
    state
        .reconciliation_repository
        .ignore_match(request)
        .await
        .map_err(ApiError::from)
}

async fn manual_match(
    State(state): State<Arc<AppState>>,
    Json(request): Json<ManualReconciliationMatchRequest>,
) -> ApiResult<Json<mizan_core::reconciliation::ReconciliationMatch>> {
    let result = state
        .reconciliation_repository
        .manual_match(request)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(result))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route(
            "/reconciliation/import-preview",
            post(reconcile_import_preview),
        )
        .route("/reconciliation/account", post(reconcile_account))
        .route(
            "/reconciliation/document-facts",
            post(reconcile_document_facts),
        )
        .route("/reconciliation/runs/{run_id}", get(get_reconciliation_run))
        .route("/reconciliation/accept-adjustment", post(accept_adjustment))
        .route("/reconciliation/ignore", post(ignore_match))
        .route("/reconciliation/manual-match", post(manual_match))
}
