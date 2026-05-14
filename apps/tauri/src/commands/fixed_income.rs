use std::sync::Arc;

use log::error;
use mizan_core::fixed_income::{
    FixedIncomeProjection, FixedIncomeRepositoryTrait, UpsertFixedIncomeDetailsRequest,
};
use tauri::State;

use crate::context::ServiceContext;

#[tauri::command]
pub async fn upsert_fixed_income_details(
    request: UpsertFixedIncomeDetailsRequest,
    state: State<'_, Arc<ServiceContext>>,
) -> Result<FixedIncomeProjection, String> {
    state
        .fixed_income_repository()
        .upsert_details(request)
        .await
        .map_err(command_error("upsert_fixed_income_details"))
}

#[tauri::command]
pub async fn get_fixed_income_projection(
    asset_id: String,
    state: State<'_, Arc<ServiceContext>>,
) -> Result<Option<FixedIncomeProjection>, String> {
    state
        .fixed_income_repository()
        .get_projection(&asset_id)
        .await
        .map_err(command_error("get_fixed_income_projection"))
}

fn command_error(command: &'static str) -> impl FnOnce(mizan_core::Error) -> String {
    move |err| {
        error!("{command} failed: {err}");
        format!("{command} failed: {err}")
    }
}
