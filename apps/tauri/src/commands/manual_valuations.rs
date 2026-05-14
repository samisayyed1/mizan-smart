//! Tauri commands for the p6 manual valuation bulk-update grid.

use std::sync::Arc;

use chrono::Utc;
use log::error;
use serde::{Deserialize, Serialize};
use tauri::State;

use mizan_core::universal_assets::{
    BulkUpdateValuationsRequest, BulkUpdateValuationsResult, ManualValuationAsset, Valuation,
};

use crate::context::ServiceContext;

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

#[tauri::command]
pub async fn list_manual_valuation_assets(
    state: State<'_, Arc<ServiceContext>>,
) -> Result<Vec<ManualValuationAsset>, String> {
    state
        .manual_valuation_repository()
        .list_assets(Utc::now().date_naive())
        .map_err(|err| {
            error!("list_manual_valuation_assets failed: {}", err);
            format!("Failed to load manual valuation assets: {}", err)
        })
}

#[tauri::command]
pub async fn bulk_update_valuations(
    request: BulkUpdateValuationsRequest,
    state: State<'_, Arc<ServiceContext>>,
) -> Result<BulkUpdateValuationsResult, String> {
    state
        .manual_valuation_repository()
        .bulk_update(request)
        .await
        .map_err(|err| {
            error!("bulk_update_valuations failed: {}", err);
            format!("Failed to update valuations: {}", err)
        })
}

#[tauri::command]
pub async fn get_manual_valuation_history(
    asset_id: String,
    state: State<'_, Arc<ServiceContext>>,
) -> Result<Vec<ManualValuationHistoryRow>, String> {
    state
        .manual_valuation_repository()
        .history(&asset_id)
        .map(|rows| {
            rows.into_iter()
                .map(ManualValuationHistoryRow::from)
                .collect()
        })
        .map_err(|err| {
            error!("get_manual_valuation_history failed: {}", err);
            format!("Failed to load valuation history: {}", err)
        })
}
