use std::sync::Arc;

use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use mizan_core::islamic_mode::{
    evaluate_shariah_screening, ShariahScreeningEvaluation, ShariahScreeningProfile,
    ShariahScreeningRatios, ShariahScreeningRepositoryTrait,
};

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
}
