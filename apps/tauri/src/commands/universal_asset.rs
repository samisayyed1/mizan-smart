//! Tauri command for the universal Add Asset flow (Phase 1 / Prompt 5).
//!
//! Thin wrapper that takes a [`UniversalAssetCreateRequest`] from the
//! frontend, delegates to the storage-layer transactional repository,
//! and returns the new asset id + initial valuation id so the UI can
//! navigate straight to the asset detail page.

use std::sync::Arc;

use log::error;
use serde::{Deserialize, Serialize};
use tauri::State;

use mizan_core::universal_assets::create_request::UniversalAssetCreateRequest;
use mizan_core::universal_assets::AssetClassification;

use crate::context::ServiceContext;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateUniversalAssetResponse {
    pub asset_id: String,
    /// Echoed back so the frontend doesn't need a second parse step.
    pub classification: AssetClassification,
    pub valuation_id: String,
}

/// Creates a base asset row + matching typed-detail row + initial
/// `valuations` row in a single SQLite transaction.
///
/// Returns the new asset id so the UI can route to the asset detail
/// page after a successful save.
#[tauri::command]
pub async fn create_universal_asset(
    request: UniversalAssetCreateRequest,
    state: State<'_, Arc<ServiceContext>>,
) -> Result<CreateUniversalAssetResponse, String> {
    let repo = state.universal_asset_create_repository();
    let result = repo.create(request).await.map_err(|err| {
        error!("create_universal_asset failed: {}", err);
        format!("Failed to create asset: {}", err)
    })?;
    Ok(CreateUniversalAssetResponse {
        asset_id: result.asset_id,
        classification: result.classification,
        valuation_id: result.valuation_id,
    })
}
