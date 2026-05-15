use std::sync::Arc;

use log::error;
use mizan_core::islamic_mode::{
    evaluate_shariah_screening, ShariahScreeningEvaluation, ShariahScreeningProfile,
    ShariahScreeningRatios, ShariahScreeningRepositoryTrait,
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

fn command_error(command: &'static str) -> impl FnOnce(mizan_core::Error) -> String {
    move |err| {
        error!("{command} failed: {err}");
        format!("{command} failed: {err}")
    }
}
