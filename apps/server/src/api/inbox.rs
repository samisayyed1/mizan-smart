//! REST endpoint for the p9 Wealth Inbox.

use std::sync::Arc;

use axum::{extract::State, routing::get, Json, Router};
use chrono::Utc;
use mizan_core::{
    alerts::{AlertStatus, AlertStore},
    inbox::{build_wealth_inbox, InboxItem},
};

use crate::error::{ApiError, ApiResult};
use crate::main_lib::AppState;

async fn list_wealth_inbox_items(
    State(state): State<Arc<AppState>>,
) -> ApiResult<Json<Vec<InboxItem>>> {
    let now = Utc::now();
    let alerts = state
        .smart_alert_repository
        .list(Some(AlertStatus::Active))
        .await
        .map_err(ApiError::from)?;
    let manual_valuations = state
        .manual_valuation_repository
        .list_assets(now.date_naive())
        .map_err(ApiError::from)?;

    Ok(Json(build_wealth_inbox(alerts, manual_valuations, now)))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/inbox/items", get(list_wealth_inbox_items))
}
