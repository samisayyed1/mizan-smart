//! Deterministic "Explain This Number" lineage from persisted SQLite rows.

use std::collections::BTreeSet;
use std::str::FromStr;
use std::sync::Arc;

use chrono::{Datelike, NaiveDate, Utc};
use diesel::prelude::*;
use diesel::r2d2::{self, Pool};
use diesel::sql_query;
use diesel::sql_types::{Integer, Nullable, Text};
use diesel::sqlite::SqliteConnection;
use rust_decimal::Decimal;

use mizan_core::data_lineage::{
    DataLineageEntityType, DataLineageFxRate, DataLineageInputRow, DataLineageMetricType,
    DataLineageRepositoryTrait, DataLineageRequest, DataLineageResponse, DataLineageSourceCitation,
    DataLineageSourceDocument,
};
use mizan_core::errors::{Error, ValidationError};
use mizan_core::Result;

use crate::db::{get_connection, WriteHandle};
use crate::errors::StorageError;

const ROUNDING_POLICY: &str =
    "Stored Decimal values are returned without display rounding; UI/export code may round.";
const STALE_VALUATION_DAYS: i64 = 90;

#[derive(Clone)]
pub struct DataLineageRepository {
    pool: Arc<Pool<r2d2::ConnectionManager<SqliteConnection>>>,
}

impl DataLineageRepository {
    pub fn new(
        pool: Arc<Pool<r2d2::ConnectionManager<SqliteConnection>>>,
        _writer: WriteHandle,
    ) -> Self {
        Self { pool }
    }
}

impl DataLineageRepositoryTrait for DataLineageRepository {
    fn get_data_lineage(&self, request: DataLineageRequest) -> Result<DataLineageResponse> {
        match (&request.entity_type, &request.metric_type) {
            (DataLineageEntityType::Portfolio, DataLineageMetricType::NetWorth) => {
                self.net_worth_lineage(request)
            }
            (DataLineageEntityType::Valuation, DataLineageMetricType::Valuation) => {
                self.valuation_lineage(request, ValuationLookup::ById)
            }
            (DataLineageEntityType::Asset, DataLineageMetricType::AssetValue)
            | (DataLineageEntityType::Asset, DataLineageMetricType::Valuation) => {
                self.valuation_lineage(request, ValuationLookup::LatestForAsset)
            }
            (DataLineageEntityType::Portfolio, DataLineageMetricType::IncomeThisMonth)
            | (DataLineageEntityType::Account, DataLineageMetricType::IncomeThisMonth) => {
                self.income_this_month_lineage(request)
            }
            (DataLineageEntityType::Portfolio, DataLineageMetricType::DataQualityScore) => {
                Ok(static_not_persisted_lineage(
                    request,
                    "Data Quality Score",
                    "The Data Quality Score is computed deterministically from current portfolio state.",
                    "No persisted data-quality lineage snapshot exists yet.",
                ))
            }
            (DataLineageEntityType::Alert, DataLineageMetricType::AlertReason) => {
                self.alert_reason_lineage(request)
            }
            (
                _,
                DataLineageMetricType::PrivateInvestmentMetric
                | DataLineageMetricType::TaxPackLine
                | DataLineageMetricType::ZakatLine,
            ) => Ok(static_not_persisted_lineage(
                request,
                "Future lineage metric",
                "This metric is reserved for a later Mizan prompt.",
                "No lineage exists yet because this metric is not implemented.",
            )),
            _ => Err(Error::Validation(ValidationError::InvalidInput(format!(
                "Unsupported lineage request: {} / {}",
                request.entity_type.as_str(),
                request.metric_type.as_str()
            )))),
        }
    }
}

#[derive(Debug, QueryableByName)]
struct LatestAccountValuationRow {
    #[diesel(sql_type = Text)]
    id: String,
    #[diesel(sql_type = Text)]
    account_id: String,
    #[diesel(sql_type = Text)]
    account_name: String,
    #[diesel(sql_type = Text)]
    valuation_date: String,
    #[diesel(sql_type = Text)]
    account_currency: String,
    #[diesel(sql_type = Text)]
    base_currency: String,
    #[diesel(sql_type = Text)]
    fx_rate_to_base: String,
    #[diesel(sql_type = Text)]
    total_value: String,
    #[diesel(sql_type = Text)]
    calculated_at: String,
}

#[derive(Debug, QueryableByName)]
struct ValuationLineageRow {
    #[diesel(sql_type = Text)]
    id: String,
    #[diesel(sql_type = Text)]
    asset_id: String,
    #[diesel(sql_type = Text)]
    asset_label: String,
    #[diesel(sql_type = Text)]
    valuation_date: String,
    #[diesel(sql_type = Text)]
    value_native: String,
    #[diesel(sql_type = Text)]
    currency: String,
    #[diesel(sql_type = Text)]
    source_type: String,
    #[diesel(sql_type = Nullable<Text>)]
    source_id: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    confidence: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    notes: Option<String>,
    #[diesel(sql_type = Text)]
    updated_at: String,
    #[diesel(sql_type = Nullable<Text>)]
    source_citation_id: Option<String>,
}

#[derive(Debug, QueryableByName)]
struct CitationJoinRow {
    #[diesel(sql_type = Text)]
    id: String,
    #[diesel(sql_type = Text)]
    source_type: String,
    #[diesel(sql_type = Nullable<Text>)]
    source_id: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    document_id: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    extracted_fact_id: Option<String>,
    #[diesel(sql_type = Nullable<Integer>)]
    page_number: Option<i32>,
    #[diesel(sql_type = Nullable<Text>)]
    bounding_box_json: Option<String>,
    #[diesel(sql_type = Text)]
    citation_label: String,
    #[diesel(sql_type = Nullable<Text>)]
    document_name: Option<String>,
}

#[derive(Debug, QueryableByName)]
struct IncomeActivityRow {
    #[diesel(sql_type = Text)]
    id: String,
    #[diesel(sql_type = Text)]
    activity_type: String,
    #[diesel(sql_type = Text)]
    activity_date: String,
    #[diesel(sql_type = Nullable<Text>)]
    amount: Option<String>,
    #[diesel(sql_type = Text)]
    currency: String,
}

#[derive(Debug, QueryableByName)]
struct AlertRow {
    #[diesel(sql_type = Text)]
    id: String,
    #[diesel(sql_type = Text)]
    rule_name: String,
    #[diesel(sql_type = Text)]
    severity: String,
    #[diesel(sql_type = Text)]
    title: String,
    #[diesel(sql_type = Text)]
    message: String,
    #[diesel(sql_type = Text)]
    last_seen_at: String,
    #[diesel(sql_type = Nullable<Text>)]
    source_entity_type: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    source_entity_id: Option<String>,
}

#[derive(Clone, Copy)]
enum ValuationLookup {
    ById,
    LatestForAsset,
}

impl DataLineageRepository {
    fn net_worth_lineage(&self, request: DataLineageRequest) -> Result<DataLineageResponse> {
        let mut conn = get_connection(&self.pool)?;
        let rows: Vec<LatestAccountValuationRow> = sql_query(
            "
            WITH ranked AS (
                SELECT
                    dav.id,
                    dav.account_id,
                    COALESCE(accounts.name, dav.account_id) AS account_name,
                    CAST(dav.valuation_date AS TEXT) AS valuation_date,
                    dav.account_currency,
                    dav.base_currency,
                    dav.fx_rate_to_base,
                    dav.total_value,
                    dav.calculated_at,
                    ROW_NUMBER() OVER (
                        PARTITION BY dav.account_id
                        ORDER BY dav.valuation_date DESC, dav.calculated_at DESC
                    ) AS rn
                FROM daily_account_valuation dav
                JOIN accounts ON accounts.id = dav.account_id
                WHERE accounts.is_active = 1 AND accounts.is_archived = 0
            )
            SELECT
                id,
                account_id,
                account_name,
                valuation_date,
                account_currency,
                base_currency,
                fx_rate_to_base,
                total_value,
                calculated_at
            FROM ranked
            WHERE rn = 1
            ORDER BY account_name ASC
            ",
        )
        .load(&mut conn)
        .map_err(StorageError::from)?;

        let mut total = Decimal::ZERO;
        let mut input_rows = Vec::new();
        let mut valuation_dates = BTreeSet::new();
        let mut fx_rates_used = Vec::new();
        let mut last_updated: Option<String> = None;
        let currency = rows.first().map(|row| row.base_currency.clone());

        for row in rows {
            let value = decimal_or_zero(&row.total_value);
            total += value;
            valuation_dates.insert(row.valuation_date.clone());
            last_updated = max_string(last_updated, row.calculated_at.clone());
            input_rows.push(DataLineageInputRow {
                source_table: "daily_account_valuation".into(),
                source_id: row.id,
                label: row.account_name,
                value: row.total_value,
                currency: Some(row.base_currency.clone()),
                as_of_date: Some(row.valuation_date.clone()),
                notes: Some(format!("account_id={}", row.account_id)),
            });
            if row.account_currency != row.base_currency || row.fx_rate_to_base != "1" {
                fx_rates_used.push(DataLineageFxRate {
                    from_currency: row.account_currency,
                    to_currency: row.base_currency,
                    rate: row.fx_rate_to_base,
                    as_of_date: Some(row.valuation_date),
                });
            }
        }

        Ok(DataLineageResponse {
            entity_type: request.entity_type,
            entity_id: request.entity_id,
            metric_type: request.metric_type,
            displayed_value: total.to_string(),
            currency,
            formula_name: "Net worth".into(),
            formula_description: "Sum of latest active, non-archived account valuations.".into(),
            input_rows,
            source_citations: Vec::new(),
            source_documents: Vec::new(),
            fx_rates_used,
            valuation_dates: valuation_dates.into_iter().collect(),
            rounding_policy: ROUNDING_POLICY.into(),
            warnings: Vec::new(),
            confidence: Some("deterministic".into()),
            freshness: None,
            last_updated,
        })
    }

    fn valuation_lineage(
        &self,
        request: DataLineageRequest,
        lookup: ValuationLookup,
    ) -> Result<DataLineageResponse> {
        let mut conn = get_connection(&self.pool)?;
        let row = match lookup {
            ValuationLookup::ById => sql_query(valuation_by_id_sql())
                .bind::<Text, _>(&request.entity_id)
                .get_result::<ValuationLineageRow>(&mut conn)
                .optional()
                .map_err(StorageError::from)?,
            ValuationLookup::LatestForAsset => sql_query(latest_valuation_for_asset_sql())
                .bind::<Text, _>(&request.entity_id)
                .get_result::<ValuationLineageRow>(&mut conn)
                .optional()
                .map_err(StorageError::from)?,
        }
        .ok_or_else(|| {
            Error::Database(mizan_core::errors::DatabaseError::NotFound(format!(
                "No valuation lineage for {}",
                request.entity_id
            )))
        })?;

        let mut warnings = Vec::new();
        let mut source_citations = Vec::new();
        let mut source_documents = Vec::new();
        if let Some(citation_id) = row.source_citation_id.as_deref() {
            let citation = self.load_citation(citation_id)?;
            if let Some(citation) = citation {
                if let (Some(document_id), Some(document_name)) =
                    (citation.document_id.clone(), citation.document_name.clone())
                {
                    source_documents.push(DataLineageSourceDocument {
                        id: document_id,
                        name: document_name,
                        page_number: citation.page_number,
                    });
                }
                source_citations.push(DataLineageSourceCitation {
                    id: citation.id,
                    label: citation.citation_label,
                    source_type: citation.source_type,
                    source_id: citation.source_id,
                    document_id: citation.document_id,
                    extracted_fact_id: citation.extracted_fact_id,
                    page_number: citation.page_number,
                    bounding_box_json: citation.bounding_box_json,
                });
            } else {
                warnings.push("Linked source citation row was not found.".into());
            }
        } else {
            warnings.push("No source document linked yet.".into());
        }

        let freshness = freshness_for_date(&row.valuation_date);
        if freshness.as_deref() == Some("stale") {
            warnings.push(format!(
                "Valuation date is at least {STALE_VALUATION_DAYS} days old."
            ));
        }

        Ok(DataLineageResponse {
            entity_type: request.entity_type,
            entity_id: request.entity_id,
            metric_type: request.metric_type,
            displayed_value: row.value_native.clone(),
            currency: Some(row.currency.clone()),
            formula_name: "Valuation".into(),
            formula_description: "Stored asset valuation value from the valuations table.".into(),
            input_rows: vec![DataLineageInputRow {
                source_table: "valuations".into(),
                source_id: row.id.clone(),
                label: row.asset_label.clone(),
                value: row.value_native.clone(),
                currency: Some(row.currency.clone()),
                as_of_date: Some(row.valuation_date.clone()),
                notes: Some(valuation_notes(&row)),
            }],
            source_citations,
            source_documents,
            fx_rates_used: Vec::new(),
            valuation_dates: vec![row.valuation_date],
            rounding_policy: ROUNDING_POLICY.into(),
            warnings,
            confidence: row.confidence,
            freshness,
            last_updated: Some(row.updated_at),
        })
    }

    fn income_this_month_lineage(
        &self,
        request: DataLineageRequest,
    ) -> Result<DataLineageResponse> {
        let today = Utc::now().date_naive();
        let month_start = NaiveDate::from_ymd_opt(today.year(), today.month(), 1)
            .expect("valid first day for current month");
        let mut conn = get_connection(&self.pool)?;

        let sql = if request.entity_type == DataLineageEntityType::Account {
            "
            SELECT id, activity_type, activity_date, amount, currency
            FROM activities
            WHERE lower(activity_type) IN ('dividend', 'interest')
              AND activity_date >= ?
              AND account_id = ?
            ORDER BY activity_date ASC
            "
        } else {
            "
            SELECT id, activity_type, activity_date, amount, currency
            FROM activities
            WHERE lower(activity_type) IN ('dividend', 'interest')
              AND activity_date >= ?
            ORDER BY activity_date ASC
            "
        };
        let rows: Vec<IncomeActivityRow> = if request.entity_type == DataLineageEntityType::Account
        {
            sql_query(sql)
                .bind::<Text, _>(month_start.to_string())
                .bind::<Text, _>(&request.entity_id)
                .load(&mut conn)
                .map_err(StorageError::from)?
        } else {
            sql_query(sql)
                .bind::<Text, _>(month_start.to_string())
                .load(&mut conn)
                .map_err(StorageError::from)?
        };

        let mut total = Decimal::ZERO;
        let mut currencies = BTreeSet::new();
        let mut input_rows = Vec::new();
        for row in rows {
            let value = row
                .amount
                .as_deref()
                .map(decimal_or_zero)
                .unwrap_or_default();
            total += value;
            currencies.insert(row.currency.clone());
            input_rows.push(DataLineageInputRow {
                source_table: "activities".into(),
                source_id: row.id,
                label: row.activity_type,
                value: row.amount.unwrap_or_else(|| "0".into()),
                currency: Some(row.currency),
                as_of_date: Some(row.activity_date),
                notes: None,
            });
        }

        Ok(DataLineageResponse {
            entity_type: request.entity_type,
            entity_id: request.entity_id,
            metric_type: request.metric_type,
            displayed_value: total.to_string(),
            currency: single_currency(currencies),
            formula_name: "Income this month".into(),
            formula_description: "Sum of dividend and interest activity amounts dated this month."
                .into(),
            input_rows,
            source_citations: Vec::new(),
            source_documents: Vec::new(),
            fx_rates_used: Vec::new(),
            valuation_dates: Vec::new(),
            rounding_policy: ROUNDING_POLICY.into(),
            warnings: Vec::new(),
            confidence: Some("deterministic".into()),
            freshness: Some("current_month".into()),
            last_updated: None,
        })
    }

    fn alert_reason_lineage(&self, request: DataLineageRequest) -> Result<DataLineageResponse> {
        let mut conn = get_connection(&self.pool)?;
        let row: AlertRow = sql_query(
            "
            SELECT
                id,
                rule_name,
                severity,
                title,
                message,
                last_seen_at,
                source_entity_type,
                source_entity_id
            FROM smart_alerts
            WHERE id = ?
            ",
        )
        .bind::<Text, _>(&request.entity_id)
        .get_result(&mut conn)
        .map_err(StorageError::from)?;

        Ok(DataLineageResponse {
            entity_type: request.entity_type,
            entity_id: request.entity_id,
            metric_type: request.metric_type,
            displayed_value: row.title.clone(),
            currency: None,
            formula_name: row.rule_name.clone(),
            formula_description: "Stored Smart Alert rule output.".into(),
            input_rows: vec![DataLineageInputRow {
                source_table: "smart_alerts".into(),
                source_id: row.id,
                label: row.title,
                value: row.message,
                currency: None,
                as_of_date: None,
                notes: Some(format!(
                    "severity={}; source={}/{}",
                    row.severity,
                    row.source_entity_type.as_deref().unwrap_or("none"),
                    row.source_entity_id.as_deref().unwrap_or("none")
                )),
            }],
            source_citations: Vec::new(),
            source_documents: Vec::new(),
            fx_rates_used: Vec::new(),
            valuation_dates: Vec::new(),
            rounding_policy: ROUNDING_POLICY.into(),
            warnings: Vec::new(),
            confidence: Some("deterministic".into()),
            freshness: Some("latest_alert_state".into()),
            last_updated: Some(row.last_seen_at),
        })
    }

    fn load_citation(&self, citation_id: &str) -> Result<Option<CitationJoinRow>> {
        let mut conn = get_connection(&self.pool)?;
        sql_query(
            "
            SELECT
                source_citations.id,
                source_citations.source_type,
                source_citations.source_id,
                source_citations.document_id,
                source_citations.extracted_fact_id,
                source_citations.page_number,
                source_citations.bounding_box_json,
                source_citations.citation_label,
                documents.original_name AS document_name
            FROM source_citations
            LEFT JOIN documents ON documents.id = source_citations.document_id
            WHERE source_citations.id = ?
            ",
        )
        .bind::<Text, _>(citation_id)
        .get_result::<CitationJoinRow>(&mut conn)
        .optional()
        .map_err(StorageError::from)
        .map_err(Into::into)
    }
}

fn valuation_by_id_sql() -> &'static str {
    "
    SELECT
        valuations.id,
        valuations.asset_id,
        COALESCE(assets.name, assets.display_code, assets.id) AS asset_label,
        valuations.valuation_date,
        valuations.value_native,
        valuations.currency,
        valuations.source_type,
        valuations.source_id,
        valuations.confidence,
        valuations.notes,
        valuations.updated_at,
        valuations.source_citation_id
    FROM valuations
    JOIN assets ON assets.id = valuations.asset_id
    WHERE valuations.id = ?
    "
}

fn latest_valuation_for_asset_sql() -> &'static str {
    "
    SELECT
        valuations.id,
        valuations.asset_id,
        COALESCE(assets.name, assets.display_code, assets.id) AS asset_label,
        valuations.valuation_date,
        valuations.value_native,
        valuations.currency,
        valuations.source_type,
        valuations.source_id,
        valuations.confidence,
        valuations.notes,
        valuations.updated_at,
        valuations.source_citation_id
    FROM valuations
    JOIN assets ON assets.id = valuations.asset_id
    WHERE valuations.asset_id = ?
    ORDER BY valuations.valuation_date DESC, valuations.updated_at DESC
    LIMIT 1
    "
}

fn decimal_or_zero(value: &str) -> Decimal {
    Decimal::from_str(value).unwrap_or_default()
}

fn max_string(current: Option<String>, candidate: String) -> Option<String> {
    match current {
        Some(existing) if existing >= candidate => Some(existing),
        _ => Some(candidate),
    }
}

fn single_currency(currencies: BTreeSet<String>) -> Option<String> {
    if currencies.len() == 1 {
        currencies.into_iter().next()
    } else {
        None
    }
}

fn freshness_for_date(date: &str) -> Option<String> {
    let valuation_date = NaiveDate::parse_from_str(date, "%Y-%m-%d").ok()?;
    let age_days = (Utc::now().date_naive() - valuation_date).num_days();
    Some(
        if age_days >= STALE_VALUATION_DAYS {
            "stale"
        } else {
            "fresh"
        }
        .into(),
    )
}

fn static_not_persisted_lineage(
    request: DataLineageRequest,
    formula_name: &str,
    formula_description: &str,
    warning: &str,
) -> DataLineageResponse {
    DataLineageResponse {
        entity_type: request.entity_type,
        entity_id: request.entity_id,
        metric_type: request.metric_type,
        displayed_value: "not_available".into(),
        currency: None,
        formula_name: formula_name.into(),
        formula_description: formula_description.into(),
        input_rows: Vec::new(),
        source_citations: Vec::new(),
        source_documents: Vec::new(),
        fx_rates_used: Vec::new(),
        valuation_dates: Vec::new(),
        rounding_policy: ROUNDING_POLICY.into(),
        warnings: vec![warning.into()],
        confidence: Some("deterministic".into()),
        freshness: None,
        last_updated: None,
    }
}

fn valuation_notes(row: &ValuationLineageRow) -> String {
    let mut parts = vec![
        format!("asset_id={}", row.asset_id),
        format!("source_type={}", row.source_type),
        format!("source_id={}", row.source_id.as_deref().unwrap_or("none")),
    ];
    if let Some(notes) = row
        .notes
        .as_deref()
        .filter(|value| !value.trim().is_empty())
    {
        parts.push(format!("notes={notes}"));
    }
    parts.join("; ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accounts::AccountDB;
    use crate::db::write_actor::spawn_writer;
    use crate::db::{create_pool, init, run_migrations};
    use crate::schema::{
        accounts, assets, daily_account_valuation, documents, source_citations, valuations,
    };
    use tempfile::tempdir;

    struct TestDb {
        pool: Arc<Pool<r2d2::ConnectionManager<SqliteConnection>>>,
        repo: DataLineageRepository,
        _app_data: tempfile::TempDir,
    }

    fn setup() -> TestDb {
        let app_data = tempdir().expect("tempdir");
        let db_path = init(app_data.path().to_str().expect("path")).expect("init");
        run_migrations(&db_path).expect("migrate");
        let pool = create_pool(&db_path).expect("pool");
        let writer = spawn_writer(pool.as_ref().clone()).expect("writer");
        let repo = DataLineageRepository::new(pool.clone(), writer);
        TestDb {
            pool,
            repo,
            _app_data: app_data,
        }
    }

    fn seed_account(pool: &Arc<Pool<r2d2::ConnectionManager<SqliteConnection>>>, id: &str) {
        let mut conn = get_connection(pool).expect("conn");
        diesel::insert_into(accounts::table)
            .values(&AccountDB {
                id: id.to_string(),
                name: format!("Account {id}"),
                account_type: "BROKERAGE".into(),
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
                tracking_mode: "HOLDINGS".into(),
            })
            .execute(&mut conn)
            .expect("seed account");
    }

    fn seed_daily_valuation(
        pool: &Arc<Pool<r2d2::ConnectionManager<SqliteConnection>>>,
        id: &str,
        account_id: &str,
        value: &str,
    ) {
        let mut conn = get_connection(pool).expect("conn");
        diesel::insert_into(daily_account_valuation::table)
            .values((
                daily_account_valuation::id.eq(id),
                daily_account_valuation::account_id.eq(account_id),
                daily_account_valuation::valuation_date
                    .eq(NaiveDate::from_ymd_opt(2026, 5, 14).unwrap()),
                daily_account_valuation::account_currency.eq("USD"),
                daily_account_valuation::base_currency.eq("USD"),
                daily_account_valuation::fx_rate_to_base.eq("1"),
                daily_account_valuation::cash_balance.eq("0"),
                daily_account_valuation::investment_market_value.eq(value),
                daily_account_valuation::total_value.eq(value),
                daily_account_valuation::cost_basis.eq("0"),
                daily_account_valuation::net_contribution.eq("0"),
                daily_account_valuation::calculated_at.eq("2026-05-14T00:00:00Z"),
            ))
            .execute(&mut conn)
            .expect("seed daily valuation");
    }

    fn seed_asset(pool: &Arc<Pool<r2d2::ConnectionManager<SqliteConnection>>>, id: &str) {
        let mut conn = get_connection(pool).expect("conn");
        diesel::insert_into(assets::table)
            .values((
                assets::id.eq(id),
                assets::kind.eq("PROPERTY"),
                assets::name.eq(Some("Villa")),
                assets::is_active.eq(1),
                assets::quote_mode.eq("MANUAL"),
                assets::quote_ccy.eq("USD"),
                assets::created_at.eq("2026-05-14T00:00:00Z"),
                assets::updated_at.eq("2026-05-14T00:00:00Z"),
                assets::classification.eq(Some("real_estate".to_string())),
            ))
            .execute(&mut conn)
            .expect("seed asset");
    }

    fn seed_document_and_citation(pool: &Arc<Pool<r2d2::ConnectionManager<SqliteConnection>>>) {
        let mut conn = get_connection(pool).expect("conn");
        diesel::insert_into(documents::table)
            .values((
                documents::id.eq("doc-1"),
                documents::file_hash.eq("hash-doc-1"),
                documents::original_name.eq("statement.pdf"),
                documents::mime_type.eq("application/pdf"),
                documents::file_size_bytes.eq(100_i64),
                documents::encrypted_storage_path.eq("doc-1.mizdoc"),
                documents::status.eq("processed"),
                documents::source_type.eq::<Option<String>>(None),
                documents::error_message.eq::<Option<String>>(None),
                documents::created_at.eq("2026-05-14T00:00:00Z"),
                documents::updated_at.eq("2026-05-14T00:00:00Z"),
            ))
            .execute(&mut conn)
            .expect("seed document");
        diesel::insert_into(source_citations::table)
            .values((
                source_citations::id.eq("citation-1"),
                source_citations::source_type.eq("document"),
                source_citations::source_id.eq(Some("doc-1")),
                source_citations::document_id.eq(Some("doc-1")),
                source_citations::extracted_fact_id.eq::<Option<String>>(None),
                source_citations::page_number.eq(Some(3)),
                source_citations::bounding_box_json.eq::<Option<String>>(None),
                source_citations::citation_label.eq("statement.pdf p.3"),
                source_citations::created_at.eq("2026-05-14T00:00:00Z"),
            ))
            .execute(&mut conn)
            .expect("seed citation");
    }

    fn seed_valuation(
        pool: &Arc<Pool<r2d2::ConnectionManager<SqliteConnection>>>,
        source_citation_id: Option<&str>,
        date: &str,
    ) {
        let mut conn = get_connection(pool).expect("conn");
        diesel::insert_into(valuations::table)
            .values((
                valuations::id.eq("valuation-1"),
                valuations::asset_id.eq("asset-1"),
                valuations::valuation_date.eq(date),
                valuations::value_native.eq("1250.00"),
                valuations::currency.eq("USD"),
                valuations::source_type.eq("manual"),
                valuations::source_id.eq::<Option<String>>(None),
                valuations::confidence.eq(Some("0.90")),
                valuations::notes.eq::<Option<String>>(None),
                valuations::created_at.eq("2026-05-14T00:00:00Z"),
                valuations::updated_at.eq("2026-05-14T00:00:00Z"),
                valuations::source_citation_id.eq(source_citation_id),
            ))
            .execute(&mut conn)
            .expect("seed valuation");
    }

    #[tokio::test]
    async fn net_worth_lineage_sums_latest_account_valuations() {
        let db = setup();
        seed_account(&db.pool, "account-1");
        seed_account(&db.pool, "account-2");
        seed_daily_valuation(&db.pool, "daily-1", "account-1", "100.50");
        seed_daily_valuation(&db.pool, "daily-2", "account-2", "200.25");

        let lineage = db
            .repo
            .get_data_lineage(DataLineageRequest {
                entity_type: DataLineageEntityType::Portfolio,
                entity_id: "total".into(),
                metric_type: DataLineageMetricType::NetWorth,
            })
            .expect("lineage");

        assert_eq!(lineage.displayed_value, "300.75");
        assert_eq!(lineage.input_rows.len(), 2);
        assert_eq!(lineage.formula_name, "Net worth");
    }

    #[tokio::test]
    async fn valuation_lineage_includes_source_document_link() {
        let db = setup();
        seed_asset(&db.pool, "asset-1");
        seed_document_and_citation(&db.pool);
        seed_valuation(&db.pool, Some("citation-1"), "2026-05-14");

        let lineage = db
            .repo
            .get_data_lineage(DataLineageRequest {
                entity_type: DataLineageEntityType::Valuation,
                entity_id: "valuation-1".into(),
                metric_type: DataLineageMetricType::Valuation,
            })
            .expect("lineage");

        assert_eq!(lineage.source_citations[0].label, "statement.pdf p.3");
        assert_eq!(lineage.source_documents[0].name, "statement.pdf");
        assert_eq!(lineage.source_documents[0].page_number, Some(3));
    }

    #[tokio::test]
    async fn valuation_lineage_reports_missing_citation_and_stale_warning() {
        let db = setup();
        seed_asset(&db.pool, "asset-1");
        seed_valuation(&db.pool, None, "2020-01-01");

        let lineage = db
            .repo
            .get_data_lineage(DataLineageRequest {
                entity_type: DataLineageEntityType::Valuation,
                entity_id: "valuation-1".into(),
                metric_type: DataLineageMetricType::Valuation,
            })
            .expect("lineage");

        assert!(lineage
            .warnings
            .iter()
            .any(|warning| warning == "No source document linked yet."));
        assert!(lineage
            .warnings
            .iter()
            .any(|warning| warning.contains("Valuation date is at least")));
        assert_eq!(lineage.freshness.as_deref(), Some("stale"));
    }
}
