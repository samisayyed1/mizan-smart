use std::sync::Arc;

use axum::{
    extract::{Query, State},
    routing::get,
    Json, Router,
};
use chrono::{NaiveDate, Utc};
use mizan_core::liquidity_ladder::{LiquidityLadderReport, LiquidityLadderRepositoryTrait};
use serde::Deserialize;

use crate::error::{ApiError, ApiResult};
use crate::main_lib::AppState;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LiquidityLadderQuery {
    as_of: Option<NaiveDate>,
}

async fn get_liquidity_ladder(
    State(state): State<Arc<AppState>>,
    Query(query): Query<LiquidityLadderQuery>,
) -> ApiResult<Json<LiquidityLadderReport>> {
    let as_of = query.as_of.unwrap_or_else(|| Utc::now().date_naive());
    state
        .liquidity_ladder_repository
        .get_ladder(as_of)
        .await
        .map(Json)
        .map_err(ApiError::from)
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/liquidity-ladder", get(get_liquidity_ladder))
}
