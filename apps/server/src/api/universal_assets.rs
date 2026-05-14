//! REST endpoint for the universal Add Asset flow (Phase 1 / Prompt 5).
//!
//! Mirrors the Tauri command in `apps/tauri/src/commands/universal_asset.rs`
//! exactly so the same frontend code path works in both runtimes.

use std::sync::Arc;

use axum::{
    extract::State,
    http::StatusCode,
    routing::{post, Router},
    Json,
};
use serde::{Deserialize, Serialize};

use mizan_core::universal_assets::create_request::UniversalAssetCreateRequest;
use mizan_core::universal_assets::AssetClassification;

use crate::error::{ApiError, ApiResult};
use crate::main_lib::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateUniversalAssetResponse {
    pub asset_id: String,
    pub classification: AssetClassification,
    pub valuation_id: String,
}

async fn create_universal_asset(
    State(state): State<Arc<AppState>>,
    Json(request): Json<UniversalAssetCreateRequest>,
) -> ApiResult<(StatusCode, Json<CreateUniversalAssetResponse>)> {
    let result = state
        .universal_asset_create_repository
        .create(request)
        .await
        .map_err(ApiError::from)?;
    Ok((
        StatusCode::CREATED,
        Json(CreateUniversalAssetResponse {
            asset_id: result.asset_id,
            classification: result.classification,
            valuation_id: result.valuation_id,
        }),
    ))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/universal-assets", post(create_universal_asset))
}
