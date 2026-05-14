//! Tauri command for the p9 Wealth Inbox.

use std::sync::Arc;

use chrono::Utc;
use log::error;
use mizan_core::{
    alerts::{AlertStatus, AlertStore},
    inbox::{build_wealth_inbox, InboxItem},
};
use tauri::State;

use crate::context::ServiceContext;

#[tauri::command]
pub async fn list_wealth_inbox_items(
    state: State<'_, Arc<ServiceContext>>,
) -> Result<Vec<InboxItem>, String> {
    let now = Utc::now();
    let alerts = state
        .smart_alert_repository()
        .list(Some(AlertStatus::Active))
        .await
        .map_err(|err| {
            error!("list_wealth_inbox_items alert load failed: {}", err);
            format!("Failed to load inbox alerts: {}", err)
        })?;
    let manual_valuations = state
        .manual_valuation_repository()
        .list_assets(now.date_naive())
        .map_err(|err| {
            error!("list_wealth_inbox_items valuation load failed: {}", err);
            format!("Failed to load valuation tasks: {}", err)
        })?;

    Ok(build_wealth_inbox(alerts, manual_valuations, now))
}
