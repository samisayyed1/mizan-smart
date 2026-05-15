use std::sync::Arc;

use log::error;
use mizan_core::islamic_mode::{
    evaluate_shariah_screening, validate_shariah_mode_enabled, AssetShariahScreening,
    CalculateZakatSnapshotRequest, PurificationEntry, PurificationPeriodSummary,
    ShariahScreeningAuditEntry, ShariahScreeningEvaluation, ShariahScreeningProfile,
    ShariahScreeningRatios, ShariahScreeningRepositoryTrait, UpsertAssetShariahScreeningRequest,
    UpsertPurificationEntryRequest, ZakatSnapshot,
};
use tauri::State;

use crate::context::ServiceContext;

#[tauri::command]
pub async fn list_shariah_screening_profiles(
    state: State<'_, Arc<ServiceContext>>,
) -> Result<Vec<ShariahScreeningProfile>, String> {
    state
        .shariah_screening_repository()
        .list_profiles()
        .map_err(command_error("list_shariah_screening_profiles"))
}

#[tauri::command]
pub async fn evaluate_shariah_screening_ratios(
    ratios: ShariahScreeningRatios,
    state: State<'_, Arc<ServiceContext>>,
) -> Result<ShariahScreeningEvaluation, String> {
    let profile = state
        .shariah_screening_repository()
        .get_default_profile()
        .map_err(command_error("evaluate_shariah_screening_ratios"))?;
    Ok(evaluate_shariah_screening(&profile, &ratios))
}

#[tauri::command]
pub async fn evaluate_shariah_compliance(
    asset_id: String,
    profile_id: String,
    state: State<'_, Arc<ServiceContext>>,
) -> Result<ShariahScreeningEvaluation, String> {
    ensure_enabled(&state)?;
    let repo = state.shariah_screening_repository();
    let profile = repo
        .get_profile(&profile_id)
        .map_err(command_error("evaluate_shariah_compliance"))?;
    let screening = repo
        .get_asset_screening_for_profile(&asset_id, &profile_id)
        .map_err(command_error("evaluate_shariah_compliance"))?
        .ok_or_else(|| {
            "evaluate_shariah_compliance failed: screening result not found".to_string()
        })?;
    Ok(evaluate_shariah_screening(
        &profile,
        &ShariahScreeningRatios {
            debt_ratio: screening.debt_ratio,
            liquid_assets_ratio: screening.liquid_assets_ratio,
            impure_income_ratio: screening.impure_income_ratio,
        },
    ))
}

#[tauri::command]
pub async fn upsert_asset_shariah_screening(
    request: UpsertAssetShariahScreeningRequest,
    state: State<'_, Arc<ServiceContext>>,
) -> Result<AssetShariahScreening, String> {
    ensure_enabled(&state)?;
    state
        .shariah_screening_repository()
        .upsert_asset_screening(request)
        .await
        .map_err(command_error("upsert_asset_shariah_screening"))
}

#[tauri::command]
pub async fn get_asset_shariah_screening(
    asset_id: String,
    profile_id: String,
    state: State<'_, Arc<ServiceContext>>,
) -> Result<Option<AssetShariahScreening>, String> {
    ensure_enabled(&state)?;
    state
        .shariah_screening_repository()
        .get_asset_screening_for_profile(&asset_id, &profile_id)
        .map_err(command_error("get_asset_shariah_screening"))
}

#[tauri::command]
pub async fn list_shariah_screening_audit(
    asset_id: String,
    profile_id: String,
    state: State<'_, Arc<ServiceContext>>,
) -> Result<Vec<ShariahScreeningAuditEntry>, String> {
    ensure_enabled(&state)?;
    state
        .shariah_screening_repository()
        .list_screening_audit(&asset_id, &profile_id)
        .map_err(command_error("list_shariah_screening_audit"))
}

#[tauri::command]
pub async fn calculate_zakat_snapshot(
    request: CalculateZakatSnapshotRequest,
    state: State<'_, Arc<ServiceContext>>,
) -> Result<ZakatSnapshot, String> {
    ensure_enabled(&state)?;
    state
        .shariah_screening_repository()
        .calculate_zakat_snapshot(request)
        .await
        .map_err(command_error("calculate_zakat_snapshot"))
}

#[tauri::command]
pub async fn upsert_purification_entry(
    request: UpsertPurificationEntryRequest,
    state: State<'_, Arc<ServiceContext>>,
) -> Result<PurificationEntry, String> {
    ensure_enabled(&state)?;
    state
        .shariah_screening_repository()
        .upsert_purification_entry(request)
        .await
        .map_err(command_error("upsert_purification_entry"))
}

#[tauri::command]
pub async fn mark_purification_paid(
    entry_id: String,
    state: State<'_, Arc<ServiceContext>>,
) -> Result<PurificationEntry, String> {
    ensure_enabled(&state)?;
    state
        .shariah_screening_repository()
        .mark_purification_paid(&entry_id)
        .await
        .map_err(command_error("mark_purification_paid"))
}

#[tauri::command]
pub async fn get_purification_period_summary(
    period_start: chrono::NaiveDate,
    period_end: chrono::NaiveDate,
    state: State<'_, Arc<ServiceContext>>,
) -> Result<PurificationPeriodSummary, String> {
    ensure_enabled(&state)?;
    state
        .shariah_screening_repository()
        .purification_period_summary(period_start, period_end)
        .map_err(command_error("get_purification_period_summary"))
}

fn ensure_enabled(state: &State<'_, Arc<ServiceContext>>) -> Result<(), String> {
    let settings = state
        .settings_service()
        .get_settings()
        .map_err(command_error("load_settings_for_shariah_screening"))?;
    validate_shariah_mode_enabled(settings.shariah_mode_enabled)
        .map_err(command_error("shariah_screening_disabled"))
}

fn command_error(command: &'static str) -> impl FnOnce(mizan_core::Error) -> String {
    move |err| {
        error!("{command} failed: {err}");
        format!("{command} failed: {err}")
    }
}
