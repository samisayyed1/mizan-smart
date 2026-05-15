use std::sync::Arc;

use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use mizan_core::corporate_actions::{
    AppliedCorporateAction, ApplyCorporateActionRequest, CorporateAction, CorporateActionPreview,
    CorporateActionsRepositoryTrait,
};

use crate::error::{ApiError, ApiResult};
use crate::main_lib::AppState;

async fn preview_corporate_action(
    State(state): State<Arc<AppState>>,
    Json(request): Json<ApplyCorporateActionRequest>,
) -> ApiResult<Json<CorporateActionPreview>> {
    state
        .corporate_actions_repository
        .preview_action(request)
        .await
        .map(Json)
        .map_err(ApiError::from)
}

async fn apply_corporate_action(
    State(state): State<Arc<AppState>>,
    Json(request): Json<ApplyCorporateActionRequest>,
) -> ApiResult<Json<AppliedCorporateAction>> {
    state
        .corporate_actions_repository
        .apply_action(request)
        .await
        .map(Json)
        .map_err(ApiError::from)
}

async fn list_corporate_actions(
    State(state): State<Arc<AppState>>,
    Path(asset_id): Path<String>,
) -> ApiResult<Json<Vec<CorporateAction>>> {
    state
        .corporate_actions_repository
        .list_actions(&asset_id)
        .await
        .map(Json)
        .map_err(ApiError::from)
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/corporate-actions/preview", post(preview_corporate_action))
        .route("/corporate-actions/apply", post(apply_corporate_action))
        .route("/corporate-actions/{asset_id}", get(list_corporate_actions))
}
