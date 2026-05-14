//! Deterministic reconciliation runs for statements, documents, and import previews.

use std::str::FromStr;
use std::sync::Arc;

use chrono::Utc;
use diesel::prelude::*;
use diesel::r2d2::{self, Pool};
use diesel::sql_query;
use diesel::sql_types::{Nullable, Text};
use diesel::sqlite::SqliteConnection;
use serde_json::Value;
use uuid::Uuid;

use mizan_core::errors::{DatabaseError, Error, ValidationError};
use mizan_core::reconciliation::{
    build_reconciliation_matches, normalized_hash, validate_date_tolerance,
    AcceptReconciliationAdjustmentRequest, AcceptReconciliationAdjustmentResult,
    IgnoreReconciliationMatchRequest, ManualReconciliationMatchRequest, ReconcileAccountRequest,
    ReconcileDocumentFactsRequest, ReconcileImportPreviewRequest, ReconciliationInputItem,
    ReconciliationItem, ReconciliationItemStatus, ReconciliationMatch, ReconciliationMatchStatus,
    ReconciliationRepositoryTrait, ReconciliationRun, ReconciliationRunDetail,
    ReconciliationRunStatus, ReconciliationScopeType, ReconciliationSourceSide,
};
use mizan_core::Result;

use crate::db::{get_connection, WriteHandle};
use crate::errors::StorageError;
use crate::schema::{
    activities, reconciliation_items, reconciliation_matches, reconciliation_runs,
};

#[derive(Clone)]
pub struct ReconciliationRepository {
    pool: Arc<Pool<r2d2::ConnectionManager<SqliteConnection>>>,
    writer: WriteHandle,
}

impl ReconciliationRepository {
    pub fn new(
        pool: Arc<Pool<r2d2::ConnectionManager<SqliteConnection>>>,
        writer: WriteHandle,
    ) -> Self {
        Self { pool, writer }
    }
}

#[derive(Debug, Clone, Insertable, Queryable)]
#[diesel(table_name = reconciliation_runs)]
struct RunRow {
    id: String,
    scope_type: String,
    scope_id: String,
    status: String,
    date_tolerance_days: i32,
    created_at: String,
    completed_at: Option<String>,
}

#[derive(Debug, Clone, Insertable, Queryable)]
#[diesel(table_name = reconciliation_items)]
struct ItemRow {
    id: String,
    run_id: String,
    item_type: String,
    source_side: String,
    raw_json: String,
    normalized_hash: String,
    amount: Option<String>,
    currency: Option<String>,
    effective_date: Option<String>,
    status: String,
}

#[derive(Debug, Clone, Insertable, Queryable)]
#[diesel(table_name = reconciliation_matches)]
struct MatchRow {
    id: String,
    run_id: String,
    mizan_item_id: Option<String>,
    external_item_id: Option<String>,
    match_status: String,
    confidence: String,
    reason: String,
    created_at: String,
}

#[derive(Debug, QueryableByName)]
struct ActivityReconRow {
    #[diesel(sql_type = Text)]
    id: String,
    #[diesel(sql_type = Text)]
    activity_type: String,
    #[diesel(sql_type = Nullable<Text>)]
    amount: Option<String>,
    #[diesel(sql_type = Text)]
    currency: String,
    #[diesel(sql_type = Text)]
    effective_date: String,
}

#[derive(Debug, QueryableByName)]
struct FactReconRow {
    #[diesel(sql_type = Text)]
    id: String,
    #[diesel(sql_type = Text)]
    fact_type: String,
    #[diesel(sql_type = Nullable<Text>)]
    normalized_value: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    currency: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    date_value: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    source_citation_id: Option<String>,
}

#[async_trait::async_trait]
impl ReconciliationRepositoryTrait for ReconciliationRepository {
    async fn reconcile_import_preview(
        &self,
        request: ReconcileImportPreviewRequest,
    ) -> Result<ReconciliationRunDetail> {
        self.create_run(
            request.scope_type,
            request.scope_id,
            request.mizan_items,
            request.external_items,
            request.date_tolerance_days,
        )
        .await
    }

    async fn reconcile_account(
        &self,
        request: ReconcileAccountRequest,
    ) -> Result<ReconciliationRunDetail> {
        let mizan_items = self.load_account_items(&request.account_id)?;
        self.create_run(
            ReconciliationScopeType::Account,
            request.account_id,
            mizan_items,
            request.external_items,
            request.date_tolerance_days,
        )
        .await
    }

    async fn reconcile_document_facts(
        &self,
        request: ReconcileDocumentFactsRequest,
    ) -> Result<ReconciliationRunDetail> {
        let mizan_items = match request.account_id.as_deref() {
            Some(account_id) => self.load_account_items(account_id)?,
            None => Vec::new(),
        };
        let external_items = self.load_document_fact_items(&request.document_id)?;
        self.create_run(
            ReconciliationScopeType::Document,
            request.document_id,
            mizan_items,
            external_items,
            request.date_tolerance_days,
        )
        .await
    }

    fn get_reconciliation_run(&self, run_id: &str) -> Result<ReconciliationRunDetail> {
        self.load_run(run_id)
    }

    async fn accept_adjustment(
        &self,
        request: AcceptReconciliationAdjustmentRequest,
    ) -> Result<AcceptReconciliationAdjustmentResult> {
        validate_accept_request(&request)?;
        let activity_id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();

        let activity_id_for_tx = activity_id.clone();
        self.writer
            .exec_tx(move |tx| -> Result<()> {
                let conn = tx.conn();
                let match_row = reconciliation_matches::table
                    .find(&request.match_id)
                    .first::<MatchRow>(conn)
                    .map_err(StorageError::from)?;
                if match_row.match_status != ReconciliationMatchStatus::MissingInMizan.as_str() {
                    return Err(invalid(
                        "Only missing_in_mizan matches can be accepted as adjustments",
                    ));
                }
                let external_item_id = match_row.external_item_id.clone().ok_or_else(|| {
                    invalid("Accepted adjustment requires an external reconciliation item")
                })?;
                let item = reconciliation_items::table
                    .find(&external_item_id)
                    .first::<ItemRow>(conn)
                    .map_err(StorageError::from)?;
                let metadata = adjustment_metadata(&match_row, &item, &request.reason)?;
                let activity_date = item
                    .effective_date
                    .clone()
                    .map(|date| format!("{date}T00:00:00Z"))
                    .unwrap_or_else(|| now.clone());

                diesel::insert_into(activities::table)
                    .values((
                        activities::id.eq(&activity_id_for_tx),
                        activities::account_id.eq(&request.account_id),
                        activities::asset_id.eq::<Option<String>>(None),
                        activities::activity_type.eq(request.activity_type.trim().to_uppercase()),
                        activities::activity_type_override.eq::<Option<String>>(None),
                        activities::source_type.eq(Some("reconciliation".to_string())),
                        activities::subtype.eq::<Option<String>>(None),
                        activities::status.eq("POSTED"),
                        activities::activity_date.eq(activity_date),
                        activities::settlement_date.eq::<Option<String>>(None),
                        activities::quantity.eq::<Option<String>>(None),
                        activities::unit_price.eq::<Option<String>>(None),
                        activities::amount.eq(item.amount),
                        activities::fee.eq::<Option<String>>(None),
                        activities::currency.eq(item.currency.unwrap_or_else(|| "USD".into())),
                        activities::fx_rate.eq::<Option<String>>(None),
                        activities::notes.eq(Some(format!(
                            "Accepted reconciliation adjustment: {}",
                            request.reason.trim()
                        ))),
                        activities::metadata.eq(Some(metadata)),
                        activities::source_system.eq(Some("reconciliation".to_string())),
                        activities::source_record_id.eq(Some(match_row.id.clone())),
                        activities::source_group_id.eq(Some(match_row.run_id.clone())),
                        activities::idempotency_key
                            .eq(Some(format!("reconciliation-adjustment:{}", match_row.id))),
                        activities::import_run_id.eq::<Option<String>>(None),
                        activities::is_user_modified.eq(1),
                        activities::needs_review.eq(1),
                        activities::created_at.eq(&now),
                        activities::updated_at.eq(&now),
                    ))
                    .execute(conn)
                    .map_err(StorageError::from)?;
                Ok(())
            })
            .await?;

        Ok(AcceptReconciliationAdjustmentResult { activity_id })
    }

    async fn ignore_match(&self, request: IgnoreReconciliationMatchRequest) -> Result<()> {
        if request.reason.trim().is_empty() {
            return Err(invalid("ignore reason is required"));
        }
        self.writer
            .exec_tx(move |tx| -> Result<()> {
                let conn = tx.conn();
                let match_row = reconciliation_matches::table
                    .find(&request.match_id)
                    .first::<MatchRow>(conn)
                    .map_err(StorageError::from)?;
                if let Some(item_id) = match_row.mizan_item_id {
                    diesel::update(reconciliation_items::table.find(item_id))
                        .set(
                            reconciliation_items::status
                                .eq(ReconciliationItemStatus::Ignored.as_str()),
                        )
                        .execute(conn)
                        .map_err(StorageError::from)?;
                }
                if let Some(item_id) = match_row.external_item_id {
                    diesel::update(reconciliation_items::table.find(item_id))
                        .set(
                            reconciliation_items::status
                                .eq(ReconciliationItemStatus::Ignored.as_str()),
                        )
                        .execute(conn)
                        .map_err(StorageError::from)?;
                }
                Ok(())
            })
            .await
    }

    async fn manual_match(
        &self,
        request: ManualReconciliationMatchRequest,
    ) -> Result<ReconciliationMatch> {
        if request.reason.trim().is_empty() {
            return Err(invalid("manual match reason is required"));
        }
        let created_at = Utc::now().to_rfc3339();
        let match_row = MatchRow {
            id: Uuid::new_v4().to_string(),
            run_id: request.run_id,
            mizan_item_id: Some(request.mizan_item_id),
            external_item_id: Some(request.external_item_id),
            match_status: ReconciliationMatchStatus::Matched.as_str().to_string(),
            confidence: "1.00".into(),
            reason: format!("Manual match: {}", request.reason.trim()),
            created_at,
        };
        let return_row = match_row.clone();
        self.writer
            .exec_tx(move |tx| -> Result<()> {
                diesel::insert_into(reconciliation_matches::table)
                    .values(&match_row)
                    .execute(tx.conn())
                    .map_err(StorageError::from)?;
                Ok(())
            })
            .await?;
        match_from_row(return_row)
    }
}

impl ReconciliationRepository {
    async fn create_run(
        &self,
        scope_type: ReconciliationScopeType,
        scope_id: String,
        mizan_inputs: Vec<ReconciliationInputItem>,
        external_inputs: Vec<ReconciliationInputItem>,
        date_tolerance_days: i64,
    ) -> Result<ReconciliationRunDetail> {
        validate_date_tolerance(date_tolerance_days)?;
        if scope_id.trim().is_empty() {
            return Err(invalid("scope_id is required"));
        }
        let run_id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        let run = ReconciliationRun {
            id: run_id.clone(),
            scope_type,
            scope_id: scope_id.trim().to_string(),
            status: ReconciliationRunStatus::Completed,
            date_tolerance_days,
            created_at: now.clone(),
            completed_at: Some(now.clone()),
        };
        let mizan_items = to_items(&run_id, ReconciliationSourceSide::Mizan, mizan_inputs)?;
        let external_items =
            to_items(&run_id, ReconciliationSourceSide::External, external_inputs)?;
        let matches = build_reconciliation_matches(
            &run_id,
            &mizan_items,
            &external_items,
            date_tolerance_days,
            &now,
        );
        let all_items = mizan_items
            .iter()
            .chain(external_items.iter())
            .cloned()
            .collect::<Vec<_>>();
        let detail = ReconciliationRunDetail {
            run: run.clone(),
            items: all_items.clone(),
            matches: matches.clone(),
        };
        let run_row = run_to_row(run)?;
        let item_rows = all_items
            .into_iter()
            .map(item_to_row)
            .collect::<Result<Vec<_>>>()?;
        let match_rows = matches.into_iter().map(match_to_row).collect::<Vec<_>>();

        self.writer
            .exec_tx(move |tx| -> Result<()> {
                let conn = tx.conn();
                diesel::insert_into(reconciliation_runs::table)
                    .values(&run_row)
                    .execute(conn)
                    .map_err(StorageError::from)?;
                diesel::insert_into(reconciliation_items::table)
                    .values(&item_rows)
                    .execute(conn)
                    .map_err(StorageError::from)?;
                diesel::insert_into(reconciliation_matches::table)
                    .values(&match_rows)
                    .execute(conn)
                    .map_err(StorageError::from)?;
                Ok(())
            })
            .await?;

        Ok(detail)
    }

    fn load_run(&self, run_id: &str) -> Result<ReconciliationRunDetail> {
        let mut conn = get_connection(&self.pool)?;
        let run = reconciliation_runs::table
            .find(run_id)
            .first::<RunRow>(&mut conn)
            .optional()
            .map_err(StorageError::from)?
            .ok_or_else(|| Error::Database(DatabaseError::NotFound("run not found".into())))?;
        let items = reconciliation_items::table
            .filter(reconciliation_items::run_id.eq(run_id))
            .load::<ItemRow>(&mut conn)
            .map_err(StorageError::from)?
            .into_iter()
            .map(item_from_row)
            .collect::<Result<Vec<_>>>()?;
        let matches = reconciliation_matches::table
            .filter(reconciliation_matches::run_id.eq(run_id))
            .load::<MatchRow>(&mut conn)
            .map_err(StorageError::from)?
            .into_iter()
            .map(match_from_row)
            .collect::<Result<Vec<_>>>()?;
        Ok(ReconciliationRunDetail {
            run: run_from_row(run)?,
            items,
            matches,
        })
    }

    fn load_account_items(&self, account_id: &str) -> Result<Vec<ReconciliationInputItem>> {
        let mut conn = get_connection(&self.pool)?;
        let rows: Vec<ActivityReconRow> = sql_query(
            "
            SELECT id, activity_type, amount, currency, substr(activity_date, 1, 10) AS effective_date
            FROM activities
            WHERE account_id = ? AND status != 'VOID'
            ORDER BY activity_date ASC, id ASC
            ",
        )
        .bind::<Text, _>(account_id)
        .load(&mut conn)
        .map_err(StorageError::from)?;
        Ok(rows
            .into_iter()
            .map(|row| ReconciliationInputItem {
                id: Some(row.id.clone()),
                item_type: row.activity_type,
                raw_json: serde_json::json!({ "activityId": row.id }),
                amount: row.amount,
                currency: Some(row.currency),
                effective_date: Some(row.effective_date),
            })
            .collect())
    }

    fn load_document_fact_items(&self, document_id: &str) -> Result<Vec<ReconciliationInputItem>> {
        let mut conn = get_connection(&self.pool)?;
        let rows: Vec<FactReconRow> = sql_query(
            "
            SELECT
                extracted_facts.id,
                extracted_facts.fact_type,
                extracted_facts.normalized_value,
                extracted_facts.currency,
                extracted_facts.date_value,
                source_citations.id AS source_citation_id
            FROM extracted_facts
            LEFT JOIN source_citations ON source_citations.extracted_fact_id = extracted_facts.id
            WHERE extracted_facts.document_id = ?
              AND extracted_facts.status = 'approved'
              AND extracted_facts.normalized_value IS NOT NULL
            ORDER BY extracted_facts.created_at ASC
            ",
        )
        .bind::<Text, _>(document_id)
        .load(&mut conn)
        .map_err(StorageError::from)?;
        Ok(rows
            .into_iter()
            .map(|row| ReconciliationInputItem {
                id: Some(row.id.clone()),
                item_type: row.fact_type,
                raw_json: serde_json::json!({
                    "extractedFactId": row.id,
                    "sourceCitationId": row.source_citation_id
                }),
                amount: row.normalized_value,
                currency: row.currency,
                effective_date: row.date_value,
            })
            .collect())
    }
}

fn to_items(
    run_id: &str,
    side: ReconciliationSourceSide,
    inputs: Vec<ReconciliationInputItem>,
) -> Result<Vec<ReconciliationItem>> {
    inputs
        .into_iter()
        .map(|input| {
            let id = input
                .id
                .clone()
                .unwrap_or_else(|| Uuid::new_v4().to_string());
            Ok(ReconciliationItem {
                id,
                run_id: run_id.to_string(),
                item_type: input.item_type.trim().to_string(),
                source_side: side,
                raw_json: input.raw_json.clone(),
                normalized_hash: normalized_hash(&input)?,
                amount: input.amount.map(|value| value.trim().to_string()),
                currency: input.currency.map(|value| value.trim().to_uppercase()),
                effective_date: input.effective_date.map(|value| value.trim().to_string()),
                status: ReconciliationItemStatus::Open,
            })
        })
        .collect()
}

fn run_to_row(run: ReconciliationRun) -> Result<RunRow> {
    Ok(RunRow {
        id: run.id,
        scope_type: run.scope_type.as_str().into(),
        scope_id: run.scope_id,
        status: run.status.as_str().into(),
        date_tolerance_days: i32::try_from(run.date_tolerance_days)
            .map_err(|_| invalid("date tolerance is too large"))?,
        created_at: run.created_at,
        completed_at: run.completed_at,
    })
}

fn run_from_row(row: RunRow) -> Result<ReconciliationRun> {
    Ok(ReconciliationRun {
        id: row.id,
        scope_type: ReconciliationScopeType::from_str(&row.scope_type)?,
        scope_id: row.scope_id,
        status: run_status_from_str(&row.status)?,
        date_tolerance_days: i64::from(row.date_tolerance_days),
        created_at: row.created_at,
        completed_at: row.completed_at,
    })
}

fn item_to_row(item: ReconciliationItem) -> Result<ItemRow> {
    Ok(ItemRow {
        id: item.id,
        run_id: item.run_id,
        item_type: item.item_type,
        source_side: item.source_side.as_str().into(),
        raw_json: serde_json::to_string(&item.raw_json)
            .map_err(|err| invalid(format!("invalid item raw_json: {err}")))?,
        normalized_hash: item.normalized_hash,
        amount: item.amount,
        currency: item.currency,
        effective_date: item.effective_date,
        status: item.status.as_str().into(),
    })
}

fn item_from_row(row: ItemRow) -> Result<ReconciliationItem> {
    Ok(ReconciliationItem {
        id: row.id,
        run_id: row.run_id,
        item_type: row.item_type,
        source_side: ReconciliationSourceSide::from_str(&row.source_side)?,
        raw_json: serde_json::from_str(&row.raw_json)
            .map_err(|err| invalid(format!("invalid stored raw_json: {err}")))?,
        normalized_hash: row.normalized_hash,
        amount: row.amount,
        currency: row.currency,
        effective_date: row.effective_date,
        status: item_status_from_str(&row.status)?,
    })
}

fn match_to_row(value: ReconciliationMatch) -> MatchRow {
    MatchRow {
        id: value.id,
        run_id: value.run_id,
        mizan_item_id: value.mizan_item_id,
        external_item_id: value.external_item_id,
        match_status: value.match_status.as_str().into(),
        confidence: value.confidence,
        reason: value.reason,
        created_at: value.created_at,
    }
}

fn match_from_row(row: MatchRow) -> Result<ReconciliationMatch> {
    Ok(ReconciliationMatch {
        id: row.id,
        run_id: row.run_id,
        mizan_item_id: row.mizan_item_id,
        external_item_id: row.external_item_id,
        match_status: ReconciliationMatchStatus::from_str(&row.match_status)?,
        confidence: row.confidence,
        reason: row.reason,
        created_at: row.created_at,
    })
}

fn adjustment_metadata(match_row: &MatchRow, item: &ItemRow, reason: &str) -> Result<String> {
    let raw_json: Value = serde_json::from_str(&item.raw_json)
        .map_err(|err| invalid(format!("invalid stored raw_json: {err}")))?;
    Ok(serde_json::json!({
        "source": "reconciliation",
        "runId": match_row.run_id,
        "matchId": match_row.id,
        "externalItemId": item.id,
        "sourceCitationId": raw_json.get("sourceCitationId").cloned().unwrap_or(Value::Null),
        "acceptedReason": reason.trim(),
        "externalRaw": raw_json
    })
    .to_string())
}

fn validate_accept_request(request: &AcceptReconciliationAdjustmentRequest) -> Result<()> {
    if request.match_id.trim().is_empty() {
        return Err(invalid("match_id is required"));
    }
    if request.account_id.trim().is_empty() {
        return Err(invalid("account_id is required"));
    }
    if request.activity_type.trim().is_empty() {
        return Err(invalid("activity_type is required"));
    }
    if request.reason.trim().is_empty() {
        return Err(invalid("acceptance reason is required"));
    }
    Ok(())
}

fn run_status_from_str(value: &str) -> Result<ReconciliationRunStatus> {
    match value {
        "open" => Ok(ReconciliationRunStatus::Open),
        "completed" => Ok(ReconciliationRunStatus::Completed),
        "failed" => Ok(ReconciliationRunStatus::Failed),
        _ => Err(invalid(format!("unknown run status: {value}"))),
    }
}

fn item_status_from_str(value: &str) -> Result<ReconciliationItemStatus> {
    match value {
        "open" => Ok(ReconciliationItemStatus::Open),
        "ignored" => Ok(ReconciliationItemStatus::Ignored),
        "accepted_adjustment" => Ok(ReconciliationItemStatus::AcceptedAdjustment),
        _ => Err(invalid(format!("unknown item status: {value}"))),
    }
}

fn invalid(message: impl Into<String>) -> Error {
    Error::Validation(ValidationError::InvalidInput(message.into()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accounts::AccountDB;
    use crate::db::write_actor::spawn_writer;
    use crate::db::{create_pool, init, run_migrations};
    use crate::schema::accounts;
    use tempfile::tempdir;

    struct TestDb {
        pool: Arc<Pool<r2d2::ConnectionManager<SqliteConnection>>>,
        repo: ReconciliationRepository,
        _app_data: tempfile::TempDir,
    }

    fn setup() -> TestDb {
        let app_data = tempdir().expect("tempdir");
        let db_path = init(app_data.path().to_str().expect("path")).expect("init");
        run_migrations(&db_path).expect("migrate");
        let pool = create_pool(&db_path).expect("pool");
        let writer = spawn_writer(pool.as_ref().clone()).expect("writer");
        let repo = ReconciliationRepository::new(pool.clone(), writer);
        TestDb {
            pool,
            repo,
            _app_data: app_data,
        }
    }

    fn input(id: &str, amount: &str, date: &str) -> ReconciliationInputItem {
        ReconciliationInputItem {
            id: Some(id.into()),
            item_type: "activity".into(),
            raw_json: serde_json::json!({ "id": id }),
            amount: Some(amount.into()),
            currency: Some("USD".into()),
            effective_date: Some(date.into()),
        }
    }

    fn seed_account(pool: &Arc<Pool<r2d2::ConnectionManager<SqliteConnection>>>) {
        let mut conn = get_connection(pool).expect("conn");
        diesel::insert_into(accounts::table)
            .values(&AccountDB {
                id: "account-1".into(),
                name: "Checking".into(),
                account_type: "BANK".into(),
                group: None,
                currency: "USD".into(),
                is_default: false,
                is_active: true,
                created_at: Utc::now().naive_utc(),
                updated_at: Utc::now().naive_utc(),
                platform_id: None,
                account_number: None,
                meta: None,
                provider: None,
                provider_account_id: None,
                is_archived: false,
                tracking_mode: "TRANSACTIONS".into(),
            })
            .execute(&mut conn)
            .expect("seed account");
    }

    #[tokio::test]
    async fn reconcile_import_preview_persists_exact_match() {
        let db = setup();
        let detail = db
            .repo
            .reconcile_import_preview(ReconcileImportPreviewRequest {
                scope_type: ReconciliationScopeType::Import,
                scope_id: "import-1".into(),
                mizan_items: vec![input("m1", "10.00", "2026-05-14")],
                external_items: vec![input("e1", "10.0", "2026-05-14")],
                date_tolerance_days: 0,
            })
            .await
            .expect("run");

        assert_eq!(
            detail.matches[0].match_status,
            ReconciliationMatchStatus::Matched
        );
        let loaded = db
            .repo
            .get_reconciliation_run(&detail.run.id)
            .expect("loaded");
        assert_eq!(loaded.items.len(), 2);
    }

    #[tokio::test]
    async fn accept_adjustment_writes_one_activity_row() {
        let db = setup();
        seed_account(&db.pool);
        let detail = db
            .repo
            .reconcile_import_preview(ReconcileImportPreviewRequest {
                scope_type: ReconciliationScopeType::Import,
                scope_id: "import-1".into(),
                mizan_items: Vec::new(),
                external_items: vec![input("e1", "25.00", "2026-05-14")],
                date_tolerance_days: 0,
            })
            .await
            .expect("run");
        let result = db
            .repo
            .accept_adjustment(AcceptReconciliationAdjustmentRequest {
                match_id: detail.matches[0].id.clone(),
                account_id: "account-1".into(),
                activity_type: "deposit".into(),
                reason: "Statement has a cash deposit".into(),
            })
            .await
            .expect("adjustment");

        let mut conn = get_connection(&db.pool).expect("conn");
        let count: i64 = activities::table
            .filter(activities::id.eq(result.activity_id))
            .count()
            .get_result(&mut conn)
            .expect("count");
        assert_eq!(count, 1);
    }
}
