use std::sync::Arc;

use log::error;
use mizan_core::private_investments::{
    CapitalCall, CreateCapitalCallRequest, CreatePrivateDistributionRequest,
    CreatePrivateInvestmentValuationRequest, PrivateDistribution, PrivateInvestment,
    PrivateInvestmentRepositoryTrait, PrivateInvestmentSummary, PrivateInvestmentValuation,
    UpdateCapitalCallStatusRequest, UpsertPrivateInvestmentRequest,
};
use tauri::State;

use crate::context::ServiceContext;

#[tauri::command]
pub async fn upsert_private_investment(
    request: UpsertPrivateInvestmentRequest,
    state: State<'_, Arc<ServiceContext>>,
) -> Result<PrivateInvestment, String> {
    state
        .private_investment_repository()
        .upsert_investment(request)
        .await
        .map_err(command_error("upsert_private_investment"))
}

#[tauri::command]
pub async fn get_private_investment(
    asset_id: String,
    state: State<'_, Arc<ServiceContext>>,
) -> Result<Option<PrivateInvestment>, String> {
    state
        .private_investment_repository()
        .get_investment(&asset_id)
        .await
        .map_err(command_error("get_private_investment"))
}

#[tauri::command]
pub async fn delete_private_investment(
    asset_id: String,
    state: State<'_, Arc<ServiceContext>>,
) -> Result<(), String> {
    state
        .private_investment_repository()
        .delete_investment(&asset_id)
        .await
        .map_err(command_error("delete_private_investment"))
}

#[tauri::command]
pub async fn add_private_investment_valuation(
    request: CreatePrivateInvestmentValuationRequest,
    state: State<'_, Arc<ServiceContext>>,
) -> Result<PrivateInvestmentValuation, String> {
    state
        .private_investment_repository()
        .add_valuation(request)
        .await
        .map_err(command_error("add_private_investment_valuation"))
}

#[tauri::command]
pub async fn add_capital_call(
    request: CreateCapitalCallRequest,
    state: State<'_, Arc<ServiceContext>>,
) -> Result<CapitalCall, String> {
    state
        .private_investment_repository()
        .add_capital_call(request)
        .await
        .map_err(command_error("add_capital_call"))
}

#[tauri::command]
pub async fn update_capital_call_status(
    request: UpdateCapitalCallStatusRequest,
    state: State<'_, Arc<ServiceContext>>,
) -> Result<CapitalCall, String> {
    state
        .private_investment_repository()
        .update_capital_call_status(request)
        .await
        .map_err(command_error("update_capital_call_status"))
}

#[tauri::command]
pub async fn add_private_distribution(
    request: CreatePrivateDistributionRequest,
    state: State<'_, Arc<ServiceContext>>,
) -> Result<PrivateDistribution, String> {
    state
        .private_investment_repository()
        .add_distribution(request)
        .await
        .map_err(command_error("add_private_distribution"))
}

#[tauri::command]
pub async fn get_private_investment_summary(
    asset_id: String,
    state: State<'_, Arc<ServiceContext>>,
) -> Result<Option<PrivateInvestmentSummary>, String> {
    state
        .private_investment_repository()
        .get_summary(&asset_id)
        .await
        .map_err(command_error("get_private_investment_summary"))
}

fn command_error(command: &'static str) -> impl FnOnce(mizan_core::Error) -> String {
    move |err| {
        error!("{command} failed: {err}");
        format!("{command} failed: {err}")
    }
}
