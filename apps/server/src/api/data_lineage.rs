use std::sync::Arc;

use axum::{extract::Query, extract::State, routing::get, Json, Router};
use mizan_core::data_lineage::{
    DataLineageEntityType, DataLineageMetricType, DataLineageRepositoryTrait, DataLineageRequest,
    DataLineageResponse,
};
use mizan_core::errors::{Error as CoreError, ValidationError};
use serde::Deserialize;

use crate::{error::ApiResult, main_lib::AppState};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DataLineageQuery {
    entity_type: String,
    entity_id: String,
    metric_type: String,
}

async fn get_data_lineage(
    State(state): State<Arc<AppState>>,
    Query(query): Query<DataLineageQuery>,
) -> ApiResult<Json<DataLineageResponse>> {
    let request = DataLineageRequest {
        entity_type: DataLineageEntityType::try_from(query.entity_type.as_str())
            .map_err(|message| CoreError::Validation(ValidationError::InvalidInput(message)))?,
        entity_id: query.entity_id,
        metric_type: DataLineageMetricType::try_from(query.metric_type.as_str())
            .map_err(|message| CoreError::Validation(ValidationError::InvalidInput(message)))?,
    };
    let lineage = state.data_lineage_repository.get_data_lineage(request)?;
    Ok(Json(lineage))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/data-lineage", get(get_data_lineage))
}
