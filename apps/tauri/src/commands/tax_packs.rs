use std::sync::Arc;

use log::error;
use mizan_core::tax_packs::{
    GenerateTaxPackRequest, TaxPack, TaxPackExportBundle, TaxPackRepositoryTrait,
};
use tauri::State;

use crate::context::ServiceContext;

#[tauri::command]
pub async fn generate_tax_pack(
    request: GenerateTaxPackRequest,
    state: State<'_, Arc<ServiceContext>>,
) -> Result<TaxPack, String> {
    state
        .tax_pack_repository()
        .generate_tax_pack(request)
        .await
        .map_err(command_error("generate_tax_pack"))
}

#[tauri::command]
pub async fn get_tax_pack(
    tax_pack_id: String,
    state: State<'_, Arc<ServiceContext>>,
) -> Result<Option<TaxPack>, String> {
    state
        .tax_pack_repository()
        .get_tax_pack(&tax_pack_id)
        .map_err(command_error("get_tax_pack"))
}

#[tauri::command]
pub async fn generate_tax_pack_export(
    tax_pack_id: String,
    state: State<'_, Arc<ServiceContext>>,
) -> Result<TaxPackExportBundle, String> {
    state
        .tax_pack_repository()
        .generate_tax_pack_export(&tax_pack_id)
        .map_err(command_error("generate_tax_pack_export"))
}

fn command_error(command: &'static str) -> impl FnOnce(mizan_core::Error) -> String {
    move |err| {
        error!("{command} failed: {err}");
        format!("{command} failed: {err}")
    }
}
