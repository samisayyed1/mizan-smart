use std::sync::Arc;

use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use mizan_core::report_builder::{
    GenerateReportRequest, ReportBuilderRepositoryTrait, ReportExportBundle, ReportRun,
};

use crate::error::{ApiError, ApiResult};
use crate::main_lib::AppState;

async fn generate_report(
    State(state): State<Arc<AppState>>,
    Json(request): Json<GenerateReportRequest>,
) -> ApiResult<Json<ReportRun>> {
    state
        .report_builder_repository
        .generate_report(request)
        .await
        .map(Json)
        .map_err(ApiError::from)
}

async fn get_report_run(
    State(state): State<Arc<AppState>>,
    Path(report_run_id): Path<String>,
) -> ApiResult<Json<Option<ReportRun>>> {
    state
        .report_builder_repository
        .get_report_run(&report_run_id)
        .map(Json)
        .map_err(ApiError::from)
}

async fn export_report(
    State(state): State<Arc<AppState>>,
    Path(report_run_id): Path<String>,
) -> ApiResult<Json<ReportExportBundle>> {
    state
        .report_builder_repository
        .export_report(&report_run_id)
        .map(Json)
        .map_err(ApiError::from)
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/report-runs", post(generate_report))
        .route("/report-runs/{report_run_id}", get(get_report_run))
        .route("/report-runs/{report_run_id}/export", post(export_report))
}
