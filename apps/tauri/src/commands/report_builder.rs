use std::sync::Arc;

use log::error;
use mizan_core::report_builder::{
    FeeIntelligenceSummary, GenerateReportRequest, ManualFeeEntry, ManualFeeEntryInput,
    ReportBuilderRepositoryTrait, ReportExportBundle, ReportRun,
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

#[tauri::command]
pub async fn add_manual_fee_entry(
    input: ManualFeeEntryInput,
    state: State<'_, Arc<ServiceContext>>,
) -> Result<ManualFeeEntry, String> {
    state
        .report_builder_repository()
        .add_manual_fee_entry(input)
        .await
        .map_err(command_error("add_manual_fee_entry"))
}

#[tauri::command]
pub async fn get_fee_intelligence_summary(
    period_month: Option<String>,
    state: State<'_, Arc<ServiceContext>>,
) -> Result<FeeIntelligenceSummary, String> {
    state
        .report_builder_repository()
        .get_fee_intelligence_summary(period_month)
        .map_err(command_error("get_fee_intelligence_summary"))
}

fn command_error(command: &'static str) -> impl FnOnce(mizan_core::Error) -> String {
    move |err| {
        error!("{command} failed: {err}");
        format!("{command} failed: {err}")
    }
}
