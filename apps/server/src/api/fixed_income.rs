use std::sync::Arc;

use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use mizan_core::fixed_income::{
    FixedIncomeProjection, FixedIncomeRepositoryTrait, UpsertFixedIncomeDetailsRequest,
};

use crate::error::{ApiError, ApiResult};
use crate::main_lib::AppState;

async fn upsert_fixed_income_details(
    State(state): State<Arc<AppState>>,
    Json(request): Json<UpsertFixedIncomeDetailsRequest>,
) -> ApiResult<Json<FixedIncomeProjection>> {
    state
        .fixed_income_repository
        .upsert_details(request)
        .await
        .map(Json)
        .map_err(ApiError::from)
}

async fn get_fixed_income_projection(
    State(state): State<Arc<AppState>>,
    Path(asset_id): Path<String>,
) -> ApiResult<Json<Option<FixedIncomeProjection>>> {
    state
        .fixed_income_repository
        .get_projection(&asset_id)
        .await
        .map(Json)
        .map_err(ApiError::from)
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/fixed-income", post(upsert_fixed_income_details))
        .route(
            "/fixed-income/{asset_id}/projection",
            get(get_fixed_income_projection),
        )
}
