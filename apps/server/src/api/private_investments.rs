use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post, put},
    Json, Router,
};
use mizan_core::private_investments::{
    CapitalCall, CreateCapitalCallRequest, CreatePrivateDistributionRequest,
    CreatePrivateInvestmentValuationRequest, PrivateDistribution, PrivateInvestment,
    PrivateInvestmentRepositoryTrait, PrivateInvestmentSummary, PrivateInvestmentValuation,
    UpdateCapitalCallStatusRequest, UpsertPrivateInvestmentRequest,
};

use crate::error::{ApiError, ApiResult};
use crate::main_lib::AppState;

async fn upsert_private_investment(
    State(state): State<Arc<AppState>>,
    Json(request): Json<UpsertPrivateInvestmentRequest>,
) -> ApiResult<Json<PrivateInvestment>> {
    state
        .private_investment_repository
        .upsert_investment(request)
        .await
        .map(Json)
        .map_err(ApiError::from)
}

async fn get_private_investment(
    State(state): State<Arc<AppState>>,
    Path(asset_id): Path<String>,
) -> ApiResult<Json<Option<PrivateInvestment>>> {
    state
        .private_investment_repository
        .get_investment(&asset_id)
        .await
        .map(Json)
        .map_err(ApiError::from)
}

async fn delete_private_investment(
    State(state): State<Arc<AppState>>,
    Path(asset_id): Path<String>,
) -> ApiResult<StatusCode> {
    state
        .private_investment_repository
        .delete_investment(&asset_id)
        .await
        .map_err(ApiError::from)?;
    Ok(StatusCode::NO_CONTENT)
}

async fn add_private_investment_valuation(
    State(state): State<Arc<AppState>>,
    Json(request): Json<CreatePrivateInvestmentValuationRequest>,
) -> ApiResult<Json<PrivateInvestmentValuation>> {
    state
        .private_investment_repository
        .add_valuation(request)
        .await
        .map(Json)
        .map_err(ApiError::from)
}

async fn add_capital_call(
    State(state): State<Arc<AppState>>,
    Json(request): Json<CreateCapitalCallRequest>,
) -> ApiResult<Json<CapitalCall>> {
    state
        .private_investment_repository
        .add_capital_call(request)
        .await
        .map(Json)
        .map_err(ApiError::from)
}

async fn update_capital_call_status(
    State(state): State<Arc<AppState>>,
    Json(request): Json<UpdateCapitalCallStatusRequest>,
) -> ApiResult<Json<CapitalCall>> {
    state
        .private_investment_repository
        .update_capital_call_status(request)
        .await
        .map(Json)
        .map_err(ApiError::from)
}

async fn add_private_distribution(
    State(state): State<Arc<AppState>>,
    Json(request): Json<CreatePrivateDistributionRequest>,
) -> ApiResult<Json<PrivateDistribution>> {
    state
        .private_investment_repository
        .add_distribution(request)
        .await
        .map(Json)
        .map_err(ApiError::from)
}

async fn get_private_investment_summary(
    State(state): State<Arc<AppState>>,
    Path(asset_id): Path<String>,
) -> ApiResult<Json<Option<PrivateInvestmentSummary>>> {
    state
        .private_investment_repository
        .get_summary(&asset_id)
        .await
        .map(Json)
        .map_err(ApiError::from)
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route(
            "/private-investments",
            post(upsert_private_investment).put(upsert_private_investment),
        )
        .route(
            "/private-investments/{asset_id}",
            get(get_private_investment).delete(delete_private_investment),
        )
        .route(
            "/private-investments/{asset_id}/summary",
            get(get_private_investment_summary),
        )
        .route(
            "/private-investments/valuations",
            post(add_private_investment_valuation),
        )
        .route("/private-investments/capital-calls", post(add_capital_call))
        .route(
            "/private-investments/capital-calls/status",
            put(update_capital_call_status),
        )
        .route(
            "/private-investments/distributions",
            post(add_private_distribution),
        )
}
