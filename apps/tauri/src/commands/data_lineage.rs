use std::sync::Arc;

use mizan_core::data_lineage::{
    DataLineageEntityType, DataLineageMetricType, DataLineageRepositoryTrait, DataLineageRequest,
    DataLineageResponse,
};
use mizan_core::errors::{Error, ValidationError};
use tauri::State;

use crate::context::ServiceContext;

#[tauri::command]
pub fn get_data_lineage(
    entity_type: String,
    entity_id: String,
    metric_type: String,
    state: State<'_, Arc<ServiceContext>>,
) -> Result<DataLineageResponse, String> {
    let request = DataLineageRequest {
        entity_type: parse_entity_type(&entity_type).map_err(|err| err.to_string())?,
        entity_id,
        metric_type: parse_metric_type(&metric_type).map_err(|err| err.to_string())?,
    };
    state
        .data_lineage_repository()
        .get_data_lineage(request)
        .map_err(|err| format!("Failed to get data lineage: {err}"))
}

fn parse_entity_type(value: &str) -> Result<DataLineageEntityType, Error> {
    DataLineageEntityType::try_from(value)
        .map_err(|message| Error::Validation(ValidationError::InvalidInput(message)))
}

fn parse_metric_type(value: &str) -> Result<DataLineageMetricType, Error> {
    DataLineageMetricType::try_from(value)
        .map_err(|message| Error::Validation(ValidationError::InvalidInput(message)))
}
