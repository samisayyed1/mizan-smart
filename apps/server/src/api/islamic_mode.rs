use std::sync::Arc;

use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use mizan_core::islamic_mode::{
    evaluate_shariah_screening, validate_shariah_mode_enabled, AssetShariahScreening,
    ShariahScreeningAuditEntry, ShariahScreeningEvaluation, ShariahScreeningProfile,
    ShariahScreeningRatios, ShariahScreeningRepositoryTrait, UpsertAssetShariahScreeningRequest,
};
use mizan_core::settings::SettingsServiceTrait;

use crate::error::{ApiError, ApiResult};
use crate::main_lib::AppState;

async fn list_shariah_screening_profiles(
    State(state): State<Arc<AppState>>,
) -> ApiResult<Json<Vec<ShariahScreeningProfile>>> {
    state
        .shariah_screening_repository
        .list_profiles()
        .map(Json)
        .map_err(ApiError::from)
}

async fn evaluate_shariah_screening_ratios(
    State(state): State<Arc<AppState>>,
    Json(ratios): Json<ShariahScreeningRatios>,
) -> ApiResult<Json<ShariahScreeningEvaluation>> {
    let profile = state
        .shariah_screening_repository
        .get_default_profile()
        .map_err(ApiError::from)?;
    Ok(Json(evaluate_shariah_screening(&profile, &ratios)))
}

async fn evaluate_shariah_compliance(
    State(state): State<Arc<AppState>>,
    Path((asset_id, profile_id)): Path<(String, String)>,
) -> ApiResult<Json<ShariahScreeningEvaluation>> {
    ensure_enabled(&state)?;
    let profile = state
        .shariah_screening_repository
        .get_profile(&profile_id)
        .map_err(ApiError::from)?;
    let screening = state
        .shariah_screening_repository
        .get_asset_screening_for_profile(&asset_id, &profile_id)
        .map_err(ApiError::from)?
        .ok_or(ApiError::NotFound)?;
    Ok(Json(evaluate_shariah_screening(
        &profile,
        &ShariahScreeningRatios {
            debt_ratio: screening.debt_ratio,
            liquid_assets_ratio: screening.liquid_assets_ratio,
            impure_income_ratio: screening.impure_income_ratio,
        },
    )))
}

async fn upsert_asset_shariah_screening(
    State(state): State<Arc<AppState>>,
    Json(request): Json<UpsertAssetShariahScreeningRequest>,
) -> ApiResult<Json<AssetShariahScreening>> {
    ensure_enabled(&state)?;
    state
        .shariah_screening_repository
        .upsert_asset_screening(request)
        .await
        .map(Json)
        .map_err(ApiError::from)
}

async fn get_asset_shariah_screening(
    State(state): State<Arc<AppState>>,
    Path((asset_id, profile_id)): Path<(String, String)>,
) -> ApiResult<Json<Option<AssetShariahScreening>>> {
    ensure_enabled(&state)?;
    state
        .shariah_screening_repository
        .get_asset_screening_for_profile(&asset_id, &profile_id)
        .map(Json)
        .map_err(ApiError::from)
}

async fn list_shariah_screening_audit(
    State(state): State<Arc<AppState>>,
    Path((asset_id, profile_id)): Path<(String, String)>,
) -> ApiResult<Json<Vec<ShariahScreeningAuditEntry>>> {
    ensure_enabled(&state)?;
    state
        .shariah_screening_repository
        .list_screening_audit(&asset_id, &profile_id)
        .map(Json)
        .map_err(ApiError::from)
}

fn ensure_enabled(state: &AppState) -> ApiResult<()> {
    let settings = state
        .settings_service
        .get_settings()
        .map_err(ApiError::from)?;
    validate_shariah_mode_enabled(settings.shariah_mode_enabled).map_err(ApiError::from)
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route(
            "/shariah-screening/profiles",
            get(list_shariah_screening_profiles),
        )
        .route(
            "/shariah-screening/evaluate",
            post(evaluate_shariah_screening_ratios),
        )
        .route(
            "/shariah-screening/assets/{asset_id}/profiles/{profile_id}/evaluate",
            get(evaluate_shariah_compliance),
        )
        .route(
            "/shariah-screening/assets/{asset_id}/profiles/{profile_id}",
            get(get_asset_shariah_screening),
        )
        .route(
            "/shariah-screening/assets/{asset_id}/profiles/{profile_id}/audit",
            get(list_shariah_screening_audit),
        )
        .route(
            "/shariah-screening/assets",
            post(upsert_asset_shariah_screening),
        )
}
