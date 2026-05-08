use std::collections::HashMap;
use std::sync::Arc;

use crate::context::ServiceContext;
use log::debug;
use mizan_core::activities::{
    Activity, ActivityBulkMutationRequest, ActivityBulkMutationResult, ActivityImport,
    ActivitySearchResponse, ActivityUpdate, FdParams, FdPaymentFrequency, ImportActivitiesResult,
    ImportAssetCandidate, ImportAssetPreviewItem, ImportMappingData, ImportTemplateData,
    NewActivity, ParseConfig, ParsedCsvResult, Sort,
};
use rust_decimal::Decimal;
use tauri::State;

#[allow(clippy::too_many_arguments)]
#[tauri::command]
pub async fn search_activities(
    page: i64,                                 // Page number, 0-based
    page_size: i64,                            // Number of items per page
    account_id_filter: Option<Vec<String>>,    // Optional account_id filter
    activity_type_filter: Option<Vec<String>>, // Optional activity_type filter
    asset_id_keyword: Option<String>,          // Optional asset_id keyword for search
    sort: Option<Sort>,
    needs_review_filter: Option<bool>, // Optional needs_review filter for pending review
    date_from: Option<String>,         // Optional start date filter (YYYY-MM-DD, inclusive)
    date_to: Option<String>,           // Optional end date filter (YYYY-MM-DD, inclusive)
    instrument_type_filter: Option<Vec<String>>, // Optional instrument_type filter
    state: State<'_, Arc<ServiceContext>>,
) -> Result<ActivitySearchResponse, String> {
    debug!("Search activities... {}, {}", page, page_size);

    // Parse date strings to NaiveDate
    let date_from_parsed = date_from
        .map(|s| chrono::NaiveDate::parse_from_str(&s, "%Y-%m-%d"))
        .transpose()
        .map_err(|e| format!("Invalid date_from format: {}", e))?;
    let date_to_parsed = date_to
        .map(|s| chrono::NaiveDate::parse_from_str(&s, "%Y-%m-%d"))
        .transpose()
        .map_err(|e| format!("Invalid date_to format: {}", e))?;

    Ok(state.activity_service().search_activities(
        page,
        page_size,
        account_id_filter,
        activity_type_filter,
        asset_id_keyword,
        sort,
        needs_review_filter,
        date_from_parsed,
        date_to_parsed,
        instrument_type_filter,
    )?)
}

#[tauri::command]
pub async fn create_activity(
    activity: NewActivity,
    state: State<'_, Arc<ServiceContext>>,
) -> Result<Activity, String> {
    debug!("Creating activity...");
    // Domain events handle recalculation and asset enrichment automatically
    state
        .activity_service()
        .create_activity(activity)
        .await
        .map_err(|e| e.to_string())
}

/// Frontend-facing payload for `create_fixed_deposit`. Mirrors `FdParams`
/// but takes the start date as an ISO `YYYY-MM-DD` string (the frontend
/// always sends dates as strings) and the payment frequency as a
/// snake_case enum string the JS layer can build trivially.
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateFixedDepositRequest {
    pub account_id: String,
    pub principal: Decimal,
    /// Annual rate as a percent (5.0 = 5%).
    pub annual_rate_percent: Decimal,
    pub term_months: u32,
    /// One of: monthly, quarterly, semi_annual, annual, at_maturity.
    pub payment_frequency: FdPaymentFrequency,
    /// `YYYY-MM-DD`.
    pub start_date: String,
    pub currency: String,
    pub notes: Option<String>,
}

/// Schedule a fixed deposit: emits a series of INTEREST activities at
/// the configured cadence into the chosen account. Returns the inserted
/// activities so the frontend can surface counts / sums in the success
/// toast without an extra round-trip.
#[tauri::command]
pub async fn create_fixed_deposit(
    request: CreateFixedDepositRequest,
    state: State<'_, Arc<ServiceContext>>,
) -> Result<Vec<Activity>, String> {
    debug!(
        "Creating fixed deposit: account={}, principal={}, rate={}%, term={}mo, freq={:?}",
        request.account_id,
        request.principal,
        request.annual_rate_percent,
        request.term_months,
        request.payment_frequency
    );

    let start_date = chrono::NaiveDate::parse_from_str(&request.start_date, "%Y-%m-%d")
        .map_err(|e| format!("Invalid start_date (expected YYYY-MM-DD): {}", e))?;

    let params = FdParams {
        account_id: request.account_id,
        principal: request.principal,
        annual_rate_percent: request.annual_rate_percent,
        term_months: request.term_months,
        payment_frequency: request.payment_frequency,
        start_date,
        currency: request.currency,
        notes: request.notes,
    };

    state
        .activity_service()
        .create_fixed_deposit_schedule(params)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_activity(
    activity: ActivityUpdate,
    state: State<'_, Arc<ServiceContext>>,
) -> Result<Activity, String> {
    debug!("Updating activity...");
    // Domain events handle recalculation and asset enrichment automatically
    state
        .activity_service()
        .update_activity(activity)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_activity(
    activity_id: String,
    state: State<'_, Arc<ServiceContext>>,
) -> Result<Activity, String> {
    debug!("Deleting activity...");
    // Domain events handle recalculation automatically
    state
        .activity_service()
        .delete_activity(activity_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn link_transfer_activities(
    activity_a_id: String,
    activity_b_id: String,
    state: State<'_, Arc<ServiceContext>>,
) -> Result<(Activity, Activity), String> {
    debug!("Linking transfer activities...");
    // Domain events handle recalculation automatically
    state
        .activity_service()
        .link_transfer_activities(activity_a_id, activity_b_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn unlink_transfer_activities(
    activity_a_id: String,
    activity_b_id: String,
    state: State<'_, Arc<ServiceContext>>,
) -> Result<(Activity, Activity), String> {
    debug!("Unlinking transfer activities...");
    // Domain events handle recalculation automatically
    state
        .activity_service()
        .unlink_transfer_activities(activity_a_id, activity_b_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn save_activities(
    request: ActivityBulkMutationRequest,
    state: State<'_, Arc<ServiceContext>>,
) -> Result<ActivityBulkMutationResult, String> {
    let create_count = request.creates.len();
    let update_count = request.updates.len();
    let delete_count = request.delete_ids.len();
    debug!(
        "Bulk activity mutation request: {} creates, {} updates, {} deletes",
        create_count, update_count, delete_count
    );

    // Domain events handle recalculation and asset enrichment automatically
    state
        .activity_service()
        .bulk_mutate_activities(request)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_account_import_mapping(
    account_id: String,
    context_kind: String,
    state: State<'_, Arc<ServiceContext>>,
) -> Result<ImportMappingData, String> {
    debug!("Getting import mapping for account: {}", account_id);
    Ok(state
        .activity_service()
        .get_import_mapping(account_id, context_kind)?)
}

#[tauri::command]
pub async fn save_account_import_mapping(
    mapping: ImportMappingData,
    state: State<'_, Arc<ServiceContext>>,
) -> Result<ImportMappingData, String> {
    debug!("Saving import mapping for account: {}", mapping.account_id);
    state
        .activity_service()
        .save_import_mapping(mapping)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn link_account_template(
    account_id: String,
    template_id: String,
    context_kind: String,
    state: State<'_, Arc<ServiceContext>>,
) -> Result<(), String> {
    debug!("Linking account {} to template {}", account_id, template_id);
    state
        .activity_service()
        .link_account_template(account_id, template_id, context_kind)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_import_templates(
    state: State<'_, Arc<ServiceContext>>,
) -> Result<Vec<ImportTemplateData>, String> {
    Ok(state.activity_service().list_import_templates()?)
}

#[tauri::command]
pub async fn get_import_template(
    id: String,
    state: State<'_, Arc<ServiceContext>>,
) -> Result<ImportTemplateData, String> {
    Ok(state.activity_service().get_import_template(id)?)
}

#[tauri::command]
pub async fn save_import_template(
    template: ImportTemplateData,
    state: State<'_, Arc<ServiceContext>>,
) -> Result<ImportTemplateData, String> {
    state
        .activity_service()
        .save_import_template(template)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_import_template(
    id: String,
    state: State<'_, Arc<ServiceContext>>,
) -> Result<(), String> {
    state
        .activity_service()
        .delete_import_template(id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn check_activities_import(
    activities: Vec<ActivityImport>,
    state: State<'_, Arc<ServiceContext>>,
) -> Result<Vec<ActivityImport>, String> {
    debug!("Checking activities import for {} rows", activities.len());
    let result = state
        .activity_service()
        .check_activities_import(activities)
        .await?;
    Ok(result)
}

#[tauri::command]
pub async fn preview_import_assets(
    candidates: Vec<ImportAssetCandidate>,
    state: State<'_, Arc<ServiceContext>>,
) -> Result<Vec<ImportAssetPreviewItem>, String> {
    let result = state
        .activity_service()
        .preview_import_assets(candidates)
        .await?;
    Ok(result)
}

#[tauri::command]
pub async fn import_activities(
    activities: Vec<ActivityImport>,
    state: State<'_, Arc<ServiceContext>>,
) -> Result<ImportActivitiesResult, String> {
    debug!("Importing {} activities", activities.len());
    // Domain events handle recalculation and asset enrichment automatically
    state
        .activity_service()
        .import_activities(activities)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn check_existing_duplicates(
    idempotency_keys: Vec<String>,
    state: State<'_, Arc<ServiceContext>>,
) -> Result<HashMap<String, String>, String> {
    debug!(
        "Checking for existing duplicates with {} idempotency keys",
        idempotency_keys.len()
    );
    state
        .activity_service()
        .check_existing_duplicates(idempotency_keys)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn parse_csv(
    content: Vec<u8>,
    config: ParseConfig,
    state: State<'_, Arc<ServiceContext>>,
) -> Result<ParsedCsvResult, String> {
    debug!(
        "Parsing CSV with {} bytes, config: {:?}",
        content.len(),
        config
    );
    state
        .activity_service()
        .parse_csv(&content, &config)
        .map_err(|e| {
            debug!("CSV parse error: {}", e);
            e.to_string()
        })
}
