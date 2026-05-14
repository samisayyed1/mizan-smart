//! REST endpoints for citation-backed extracted facts.

use std::sync::Arc;

use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};

use mizan_core::documents::{
    CreateExtractedFactRequest, CreateExtractedFactResult, DeferExtractedFactRequest,
    ExtractedFact, ExtractedFactEntityLink, LinkExtractedFactRequest, ReviewExtractedFactRequest,
    SourceCitation, UpdateExtractedFactRequest,
};

use crate::error::{ApiError, ApiResult};
use crate::main_lib::AppState;

async fn create_extracted_fact(
    State(state): State<Arc<AppState>>,
    Json(request): Json<CreateExtractedFactRequest>,
) -> ApiResult<Json<CreateExtractedFactResult>> {
    let result = state
        .extracted_fact_repository
        .create_extracted_fact(request)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(result))
}

async fn list_pending_extracted_facts(
    State(state): State<Arc<AppState>>,
) -> ApiResult<Json<Vec<ExtractedFact>>> {
    let facts = state
        .extracted_fact_repository
        .list_pending_extracted_facts()
        .map_err(ApiError::from)?;
    Ok(Json(facts))
}

async fn get_source_citation(
    State(state): State<Arc<AppState>>,
    Path(citation_id): Path<String>,
) -> ApiResult<Json<SourceCitation>> {
    let citation = state
        .extracted_fact_repository
        .get_source_citation(&citation_id)
        .map_err(ApiError::from)?;
    Ok(Json(citation))
}

async fn approve_extracted_fact(
    State(state): State<Arc<AppState>>,
    Path(fact_id): Path<String>,
    Json(request): Json<ReviewExtractedFactRequest>,
) -> ApiResult<Json<ExtractedFact>> {
    let fact = state
        .extracted_fact_repository
        .approve_extracted_fact(&fact_id, request)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(fact))
}

async fn update_extracted_fact_before_approval(
    State(state): State<Arc<AppState>>,
    Path(fact_id): Path<String>,
    Json(request): Json<UpdateExtractedFactRequest>,
) -> ApiResult<Json<ExtractedFact>> {
    let fact = state
        .extracted_fact_repository
        .update_extracted_fact_before_approval(&fact_id, request)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(fact))
}

async fn link_extracted_fact_to_entity(
    State(state): State<Arc<AppState>>,
    Path(fact_id): Path<String>,
    Json(request): Json<LinkExtractedFactRequest>,
) -> ApiResult<Json<ExtractedFactEntityLink>> {
    let link = state
        .extracted_fact_repository
        .link_extracted_fact_to_entity(&fact_id, request)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(link))
}

async fn defer_extracted_fact(
    State(state): State<Arc<AppState>>,
    Path(fact_id): Path<String>,
    Json(request): Json<DeferExtractedFactRequest>,
) -> ApiResult<Json<ExtractedFact>> {
    let fact = state
        .extracted_fact_repository
        .defer_extracted_fact(&fact_id, request)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(fact))
}

async fn reject_extracted_fact(
    State(state): State<Arc<AppState>>,
    Path(fact_id): Path<String>,
    Json(request): Json<ReviewExtractedFactRequest>,
) -> ApiResult<Json<ExtractedFact>> {
    let fact = state
        .extracted_fact_repository
        .reject_extracted_fact(&fact_id, request)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(fact))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route(
            "/extracted-facts/pending",
            get(list_pending_extracted_facts),
        )
        .route("/extracted-facts", post(create_extracted_fact))
        .route(
            "/extracted-facts/{fact_id}/approve",
            post(approve_extracted_fact),
        )
        .route(
            "/extracted-facts/{fact_id}/edit",
            post(update_extracted_fact_before_approval),
        )
        .route(
            "/extracted-facts/{fact_id}/link",
            post(link_extracted_fact_to_entity),
        )
        .route(
            "/extracted-facts/{fact_id}/defer",
            post(defer_extracted_fact),
        )
        .route(
            "/extracted-facts/{fact_id}/reject",
            post(reject_extracted_fact),
        )
        .route("/source-citations/{citation_id}", get(get_source_citation))
}
