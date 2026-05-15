use std::sync::Arc;

use log::error;
use mizan_core::corporate_actions::{
    AppliedCorporateAction, ApplyCorporateActionRequest, CorporateAction, CorporateActionPreview,
    CorporateActionsRepositoryTrait,
};
use tauri::State;

use crate::context::ServiceContext;

#[tauri::command]
pub async fn preview_corporate_action(
    request: ApplyCorporateActionRequest,
    state: State<'_, Arc<ServiceContext>>,
) -> Result<CorporateActionPreview, String> {
    state
        .corporate_actions_repository()
        .preview_action(request)
        .await
        .map_err(command_error("preview_corporate_action"))
}

#[tauri::command]
pub async fn apply_corporate_action(
    request: ApplyCorporateActionRequest,
    state: State<'_, Arc<ServiceContext>>,
) -> Result<AppliedCorporateAction, String> {
    state
        .corporate_actions_repository()
        .apply_action(request)
        .await
        .map_err(command_error("apply_corporate_action"))
}

#[tauri::command]
pub async fn list_corporate_actions(
    asset_id: String,
    state: State<'_, Arc<ServiceContext>>,
) -> Result<Vec<CorporateAction>, String> {
    state
        .corporate_actions_repository()
        .list_actions(&asset_id)
        .await
        .map_err(command_error("list_corporate_actions"))
}

fn command_error(command: &'static str) -> impl FnOnce(mizan_core::Error) -> String {
    move |err| {
        error!("{command} failed: {err}");
        format!("{command} failed: {err}")
    }
}
