//! Tauri commands for deterministic reconciliation runs.

use std::sync::Arc;

use log::error;
use tauri::State;

use mizan_core::reconciliation::{
    AcceptReconciliationAdjustmentRequest, AcceptReconciliationAdjustmentResult,
    IgnoreReconciliationMatchRequest, ManualReconciliationMatchRequest, ReconcileAccountRequest,
    ReconcileDocumentFactsRequest, ReconcileImportPreviewRequest, ReconciliationMatch,
    ReconciliationRepositoryTrait, ReconciliationRunDetail,
};

use crate::context::ServiceContext;

#[tauri::command]
pub async fn reconcile_import_preview(
    request: ReconcileImportPreviewRequest,
    state: State<'_, Arc<ServiceContext>>,
) -> Result<ReconciliationRunDetail, String> {
    state
        .reconciliation_repository()
        .reconcile_import_preview(request)
        .await
        .map_err(|err| {
            error!("reconcile_import_preview failed: {}", err);
            err.to_string()
        })
}

#[tauri::command]
pub async fn reconcile_account(
    request: ReconcileAccountRequest,
    state: State<'_, Arc<ServiceContext>>,
) -> Result<ReconciliationRunDetail, String> {
    state
        .reconciliation_repository()
        .reconcile_account(request)
        .await
        .map_err(|err| {
            error!("reconcile_account failed: {}", err);
            err.to_string()
        })
}

#[tauri::command]
pub async fn reconcile_document_facts(
    request: ReconcileDocumentFactsRequest,
    state: State<'_, Arc<ServiceContext>>,
) -> Result<ReconciliationRunDetail, String> {
    state
        .reconciliation_repository()
        .reconcile_document_facts(request)
        .await
        .map_err(|err| {
            error!("reconcile_document_facts failed: {}", err);
            err.to_string()
        })
}

#[tauri::command]
pub fn get_reconciliation_run(
    run_id: String,
    state: State<'_, Arc<ServiceContext>>,
) -> Result<ReconciliationRunDetail, String> {
    state
        .reconciliation_repository()
        .get_reconciliation_run(&run_id)
        .map_err(|err| {
            error!("get_reconciliation_run failed: {}", err);
            err.to_string()
        })
}

#[tauri::command]
pub async fn accept_reconciliation_adjustment(
    request: AcceptReconciliationAdjustmentRequest,
    state: State<'_, Arc<ServiceContext>>,
) -> Result<AcceptReconciliationAdjustmentResult, String> {
    state
        .reconciliation_repository()
        .accept_adjustment(request)
        .await
        .map_err(|err| {
            error!("accept_reconciliation_adjustment failed: {}", err);
            err.to_string()
        })
}

#[tauri::command]
pub async fn ignore_reconciliation_match(
    request: IgnoreReconciliationMatchRequest,
    state: State<'_, Arc<ServiceContext>>,
) -> Result<(), String> {
    state
        .reconciliation_repository()
        .ignore_match(request)
        .await
        .map_err(|err| {
            error!("ignore_reconciliation_match failed: {}", err);
            err.to_string()
        })
}

#[tauri::command]
pub async fn manual_reconciliation_match(
    request: ManualReconciliationMatchRequest,
    state: State<'_, Arc<ServiceContext>>,
) -> Result<ReconciliationMatch, String> {
    state
        .reconciliation_repository()
        .manual_match(request)
        .await
        .map_err(|err| {
            error!("manual_reconciliation_match failed: {}", err);
            err.to_string()
        })
}
