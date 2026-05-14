//! REST endpoints for the p6 manual valuation bulk-update grid.

use std::sync::Arc;

use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};

use mizan_core::universal_assets::{
    BulkUpdateValuationsRequest, BulkUpdateValuationsResult, ManualValuationAsset, Valuation,
};

use crate::error::{ApiError, ApiResult};
use crate::main_lib::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManualValuationHistoryRow {
    pub id: String,
    pub asset_id: String,
    pub valuation_date: String,
    pub value_native: String,
    pub currency: String,
    pub notes: Option<String>,
    pub created_at: String,
}

impl From<Valuation> for ManualValuationHistoryRow {
    fn from(value: Valuation) -> Self {
        Self {
            id: value.id,
            asset_id: value.asset_id,
            valuation_date: value.valuation_date.to_string(),
            value_native: value.value_native.normalize().to_string(),
            currency: value.currency,
            notes: value.notes,
            created_at: value.created_at.to_rfc3339(),
        }
    }
}

async fn list_manual_valuation_assets(
    State(state): State<Arc<AppState>>,
) -> ApiResult<Json<Vec<ManualValuationAsset>>> {
    let rows = state
        .manual_valuation_repository
        .list_assets(Utc::now().date_naive())
        .map_err(ApiError::from)?;
    Ok(Json(rows))
}

async fn bulk_update_valuations(
    State(state): State<Arc<AppState>>,
    Json(request): Json<BulkUpdateValuationsRequest>,
) -> ApiResult<Json<BulkUpdateValuationsResult>> {
    let result = state
        .manual_valuation_repository
        .bulk_update(request)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(result))
}

async fn get_manual_valuation_history(
    State(state): State<Arc<AppState>>,
    Path(asset_id): Path<String>,
) -> ApiResult<Json<Vec<ManualValuationHistoryRow>>> {
    let rows = state
        .manual_valuation_repository
        .history(&asset_id)
        .map_err(ApiError::from)?
        .into_iter()
        .map(ManualValuationHistoryRow::from)
        .collect();
    Ok(Json(rows))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/manual-valuations", get(list_manual_valuation_assets))
        .route(
            "/manual-valuations/bulk-update",
            post(bulk_update_valuations),
        )
        .route(
            "/manual-valuations/{asset_id}/history",
            get(get_manual_valuation_history),
        )
}
