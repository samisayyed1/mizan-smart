use std::sync::Arc;

use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use mizan_core::tax_packs::{GenerateTaxPackRequest, TaxPack, TaxPackRepositoryTrait};

use crate::error::{ApiError, ApiResult};
use crate::main_lib::AppState;

async fn generate_tax_pack(
    State(state): State<Arc<AppState>>,
    Json(request): Json<GenerateTaxPackRequest>,
) -> ApiResult<Json<TaxPack>> {
    state
        .tax_pack_repository
        .generate_tax_pack(request)
        .await
        .map(Json)
        .map_err(ApiError::from)
}

async fn get_tax_pack(
    State(state): State<Arc<AppState>>,
    Path(tax_pack_id): Path<String>,
) -> ApiResult<Json<Option<TaxPack>>> {
    state
        .tax_pack_repository
        .get_tax_pack(&tax_pack_id)
        .map(Json)
        .map_err(ApiError::from)
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/tax-packs", post(generate_tax_pack))
        .route("/tax-packs/{tax_pack_id}", get(get_tax_pack))
}
