use std::sync::Arc;

use chrono::{NaiveDate, Utc};
use log::error;
use mizan_core::liquidity_ladder::{LiquidityLadderReport, LiquidityLadderRepositoryTrait};
use tauri::State;

use crate::context::ServiceContext;

#[tauri::command]
pub async fn get_liquidity_ladder(
    as_of: Option<NaiveDate>,
    state: State<'_, Arc<ServiceContext>>,
) -> Result<LiquidityLadderReport, String> {
    state
        .liquidity_ladder_repository()
        .get_ladder(as_of.unwrap_or_else(|| Utc::now().date_naive()))
        .await
        .map_err(command_error("get_liquidity_ladder"))
}

fn command_error(command: &'static str) -> impl FnOnce(mizan_core::Error) -> String {
    move |err| {
        error!("{command} failed: {err}");
        format!("{command} failed: {err}")
    }
}
