use std::sync::Arc;

use log::error;
use mizan_core::report_builder::{
    GenerateReportRequest, ReportBuilderRepositoryTrait, ReportExportBundle, ReportRun,
};
use tauri::State;

use crate::context::ServiceContext;

#[tauri::command]
pub async fn generate_report(
    request: GenerateReportRequest,
    state: State<'_, Arc<ServiceContext>>,
) -> Result<ReportRun, String> {
    state
        .report_builder_repository()
        .generate_report(request)
        .await
        .map_err(command_error("generate_report"))
}

#[tauri::command]
pub async fn get_report_run(
    report_run_id: String,
    state: State<'_, Arc<ServiceContext>>,
) -> Result<Option<ReportRun>, String> {
    state
        .report_builder_repository()
        .get_report_run(&report_run_id)
        .map_err(command_error("get_report_run"))
}

#[tauri::command]
pub async fn export_report(
    report_run_id: String,
    state: State<'_, Arc<ServiceContext>>,
) -> Result<ReportExportBundle, String> {
    state
        .report_builder_repository()
        .export_report(&report_run_id)
        .map_err(command_error("export_report"))
}

fn command_error(command: &'static str) -> impl FnOnce(mizan_core::Error) -> String {
    move |err| {
        error!("{command} failed: {err}");
        format!("{command} failed: {err}")
    }
}
