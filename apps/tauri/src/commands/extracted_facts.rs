//! Tauri commands for citation-backed extracted facts.

use std::sync::Arc;

use log::error;
use tauri::State;

use mizan_core::documents::{
    CreateExtractedFactRequest, CreateExtractedFactResult, ExtractedFact,
    ReviewExtractedFactRequest, SourceCitation,
};

use crate::context::ServiceContext;

#[tauri::command]
pub async fn create_extracted_fact(
    request: CreateExtractedFactRequest,
    state: State<'_, Arc<ServiceContext>>,
) -> Result<CreateExtractedFactResult, String> {
    state
        .extracted_fact_repository()
        .create_extracted_fact(request)
        .await
        .map_err(|err| {
            error!("create_extracted_fact failed: {}", err);
            err.to_string()
        })
}

#[tauri::command]
pub fn list_pending_extracted_facts(
    state: State<'_, Arc<ServiceContext>>,
) -> Result<Vec<ExtractedFact>, String> {
    state
        .extracted_fact_repository()
        .list_pending_extracted_facts()
        .map_err(|err| {
            error!("list_pending_extracted_facts failed: {}", err);
            err.to_string()
        })
}

#[tauri::command]
pub fn get_source_citation(
    citation_id: String,
    state: State<'_, Arc<ServiceContext>>,
) -> Result<SourceCitation, String> {
    state
        .extracted_fact_repository()
        .get_source_citation(&citation_id)
        .map_err(|err| {
            error!("get_source_citation failed: {}", err);
            err.to_string()
        })
}

#[tauri::command]
pub async fn approve_extracted_fact(
    fact_id: String,
    request: ReviewExtractedFactRequest,
    state: State<'_, Arc<ServiceContext>>,
) -> Result<ExtractedFact, String> {
    state
        .extracted_fact_repository()
        .approve_extracted_fact(&fact_id, request)
        .await
        .map_err(|err| {
            error!("approve_extracted_fact failed: {}", err);
            err.to_string()
        })
}

#[tauri::command]
pub async fn reject_extracted_fact(
    fact_id: String,
    request: ReviewExtractedFactRequest,
    state: State<'_, Arc<ServiceContext>>,
) -> Result<ExtractedFact, String> {
    state
        .extracted_fact_repository()
        .reject_extracted_fact(&fact_id, request)
        .await
        .map_err(|err| {
            error!("reject_extracted_fact failed: {}", err);
            err.to_string()
        })
}
