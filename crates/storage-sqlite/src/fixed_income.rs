use async_trait::async_trait;
use chrono::{NaiveDate, Utc};
use diesel::prelude::*;
use diesel::r2d2::{self, Pool};
use diesel::SqliteConnection;
use rust_decimal::Decimal;
use std::str::FromStr;
use std::sync::Arc;
use uuid::Uuid;

use mizan_core::errors::ValidationError;
use mizan_core::fixed_income::{
    generate_projected_cashflows, FixedIncomeCashflow, FixedIncomeCashflowStatus,
    FixedIncomeCashflowType, FixedIncomeDetails, FixedIncomePaymentFrequency,
    FixedIncomeProjection, FixedIncomeRepositoryTrait, UpsertFixedIncomeDetailsRequest,
};
use mizan_core::universal_assets::details::{DayCountConvention, FixedIncomeSubtype};
use mizan_core::{Error, Result};

use crate::db::{get_connection, WriteHandle};
use crate::errors::StorageError;
use crate::schema::{asset_fixed_income_details, fixed_income_cashflows};

pub struct FixedIncomeRepository {
    pool: Arc<Pool<r2d2::ConnectionManager<SqliteConnection>>>,
    writer: WriteHandle,
}

impl FixedIncomeRepository {
    pub fn new(
        pool: Arc<Pool<r2d2::ConnectionManager<SqliteConnection>>>,
        writer: WriteHandle,
    ) -> Self {
        Self { pool, writer }
    }
}

#[derive(Debug, Clone, Queryable, Insertable)]
#[diesel(table_name = asset_fixed_income_details)]
struct FixedIncomeDetailsRow {
    asset_id: String,
    instrument_subtype: String,
    issuer: Option<String>,
    isin: Option<String>,
    face_value: Option<String>,
    currency: Option<String>,
    purchase_date: Option<String>,
    maturity_date: Option<String>,
    coupon_or_profit_rate: Option<String>,
    payment_frequency: Option<String>,
    day_count_convention: Option<String>,
    is_sukuk: i32,
    source_citation_id: Option<String>,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, Clone, Queryable, Insertable)]
#[diesel(table_name = fixed_income_cashflows)]
struct FixedIncomeCashflowRow {
    id: String,
    asset_id: String,
    expected_date: String,
    cashflow_type: String,
    expected_amount: String,
    actual_amount: Option<String>,
    currency: String,
    status: String,
    source_citation_id: Option<String>,
    created_at: String,
    updated_at: String,
}

impl TryFrom<FixedIncomeDetailsRow> for FixedIncomeDetails {
    type Error = Error;

    fn try_from(row: FixedIncomeDetailsRow) -> Result<Self> {
        Ok(Self {
            asset_id: row.asset_id,
            instrument_type: FixedIncomeSubtype::parse(&row.instrument_subtype)
                .ok_or_else(|| invalid("unsupported fixed income instrument type"))?,
            issuer: row.issuer.unwrap_or_default(),
            isin: row.isin,
            face_value: parse_required_decimal("face_value", row.face_value)?,
            currency: row.currency.unwrap_or_default(),
            purchase_date: parse_optional_date(row.purchase_date)?,
            maturity_date: parse_required_date("maturity_date", row.maturity_date)?,
            coupon_or_profit_rate: parse_optional_decimal(
                "coupon_or_profit_rate",
                row.coupon_or_profit_rate,
            )?,
            payment_frequency: row
                .payment_frequency
                .as_deref()
                .map(|value| {
                    FixedIncomePaymentFrequency::parse(value)
                        .ok_or_else(|| invalid("unsupported payment frequency"))
                })
                .transpose()?,
            day_count_convention: row
                .day_count_convention
                .as_deref()
                .and_then(DayCountConvention::parse)
                .unwrap_or(DayCountConvention::Act365),
            is_sukuk: row.is_sukuk == 1,
            source_citation_id: row.source_citation_id,
        })
    }
}

impl TryFrom<FixedIncomeCashflowRow> for FixedIncomeCashflow {
    type Error = Error;

    fn try_from(row: FixedIncomeCashflowRow) -> Result<Self> {
        Ok(Self {
            id: row.id,
            asset_id: row.asset_id,
            expected_date: parse_date(&row.expected_date)?,
            cashflow_type: FixedIncomeCashflowType::parse(&row.cashflow_type)
                .ok_or_else(|| invalid("unsupported fixed income cashflow type"))?,
            expected_amount: parse_decimal("expected_amount", &row.expected_amount)?,
            actual_amount: row
                .actual_amount
                .as_deref()
                .map(|value| parse_decimal("actual_amount", value))
                .transpose()?,
            currency: row.currency,
            status: FixedIncomeCashflowStatus::parse(&row.status)
                .ok_or_else(|| invalid("unsupported fixed income cashflow status"))?,
            source_citation_id: row.source_citation_id,
        })
    }
}

#[async_trait]
impl FixedIncomeRepositoryTrait for FixedIncomeRepository {
    async fn upsert_details(
        &self,
        request: UpsertFixedIncomeDetailsRequest,
    ) -> Result<FixedIncomeProjection> {
        request.validate()?;
        let details = request.into_domain();
        let (projected, warnings) = generate_projected_cashflows(&details)?;
        let now = Utc::now().to_rfc3339();
        let detail_row = FixedIncomeDetailsRow {
            asset_id: details.asset_id.clone(),
            instrument_subtype: details.instrument_type.as_str().to_string(),
            issuer: Some(details.issuer.clone()),
            isin: details.isin.clone(),
            face_value: Some(details.face_value.normalize().to_string()),
            currency: Some(details.currency.clone()),
            purchase_date: details.purchase_date.map(|date| date.to_string()),
            maturity_date: Some(details.maturity_date.to_string()),
            coupon_or_profit_rate: details
                .coupon_or_profit_rate
                .map(|rate| rate.normalize().to_string()),
            payment_frequency: details
                .payment_frequency
                .map(|frequency| frequency.as_str().to_string()),
            day_count_convention: Some(details.day_count_convention.as_str().to_string()),
            is_sukuk: if details.is_sukuk { 1 } else { 0 },
            source_citation_id: details.source_citation_id.clone(),
            created_at: now.clone(),
            updated_at: now.clone(),
        };
        let cashflow_rows = projected
            .into_iter()
            .map(|cashflow| FixedIncomeCashflowRow {
                id: Uuid::new_v4().to_string(),
                asset_id: details.asset_id.clone(),
                expected_date: cashflow.expected_date.to_string(),
                cashflow_type: cashflow.cashflow_type.as_str().to_string(),
                expected_amount: cashflow.expected_amount.normalize().to_string(),
                actual_amount: None,
                currency: cashflow.currency,
                status: FixedIncomeCashflowStatus::Expected.as_str().to_string(),
                source_citation_id: cashflow.source_citation_id,
                created_at: now.clone(),
                updated_at: now.clone(),
            })
            .collect::<Vec<_>>();
        let projection_rows = cashflow_rows.clone();
        self.writer
            .exec_tx(move |tx| -> Result<()> {
                let conn = tx.conn();
                diesel::insert_into(asset_fixed_income_details::table)
                    .values(&detail_row)
                    .on_conflict(asset_fixed_income_details::asset_id)
                    .do_update()
                    .set((
                        asset_fixed_income_details::instrument_subtype
                            .eq(&detail_row.instrument_subtype),
                        asset_fixed_income_details::issuer.eq(&detail_row.issuer),
                        asset_fixed_income_details::isin.eq(&detail_row.isin),
                        asset_fixed_income_details::face_value.eq(&detail_row.face_value),
                        asset_fixed_income_details::currency.eq(&detail_row.currency),
                        asset_fixed_income_details::purchase_date.eq(&detail_row.purchase_date),
                        asset_fixed_income_details::maturity_date.eq(&detail_row.maturity_date),
                        asset_fixed_income_details::coupon_or_profit_rate
                            .eq(&detail_row.coupon_or_profit_rate),
                        asset_fixed_income_details::payment_frequency
                            .eq(&detail_row.payment_frequency),
                        asset_fixed_income_details::day_count_convention
                            .eq(&detail_row.day_count_convention),
                        asset_fixed_income_details::is_sukuk.eq(detail_row.is_sukuk),
                        asset_fixed_income_details::source_citation_id
                            .eq(&detail_row.source_citation_id),
                        asset_fixed_income_details::updated_at.eq(&detail_row.updated_at),
                    ))
                    .execute(conn)
                    .map_err(StorageError::from)?;

                diesel::delete(
                    fixed_income_cashflows::table
                        .filter(fixed_income_cashflows::asset_id.eq(&detail_row.asset_id))
                        .filter(fixed_income_cashflows::status.eq("expected")),
                )
                .execute(conn)
                .map_err(StorageError::from)?;
                diesel::insert_into(fixed_income_cashflows::table)
                    .values(&cashflow_rows)
                    .execute(conn)
                    .map_err(StorageError::from)?;
                Ok(())
            })
            .await?;

        let cashflows = projection_rows
            .into_iter()
            .map(FixedIncomeCashflow::try_from)
            .collect::<Result<Vec<_>>>()?;
        Ok(FixedIncomeProjection {
            accrued_amount: Decimal::ZERO,
            details,
            cashflows,
            warnings,
        })
    }

    async fn get_projection(&self, asset_id: &str) -> Result<Option<FixedIncomeProjection>> {
        let mut conn = get_connection(&self.pool)?;
        let Some(detail_row) = asset_fixed_income_details::table
            .find(asset_id)
            .first::<FixedIncomeDetailsRow>(&mut conn)
            .optional()
            .map_err(StorageError::from)?
        else {
            return Ok(None);
        };
        let cashflow_rows = fixed_income_cashflows::table
            .filter(fixed_income_cashflows::asset_id.eq(asset_id))
            .order(fixed_income_cashflows::expected_date.asc())
            .load::<FixedIncomeCashflowRow>(&mut conn)
            .map_err(StorageError::from)?;
        let details = FixedIncomeDetails::try_from(detail_row)?;
        let cashflows = cashflow_rows
            .into_iter()
            .map(FixedIncomeCashflow::try_from)
            .collect::<Result<Vec<_>>>()?;
        Ok(Some(FixedIncomeProjection {
            details,
            accrued_amount: Decimal::ZERO,
            cashflows,
            warnings: Vec::new(),
        }))
    }
}

fn parse_required_decimal(field: &str, value: Option<String>) -> Result<Decimal> {
    value
        .as_deref()
        .ok_or_else(|| invalid(format!("{field} is required")))
        .and_then(|value| parse_decimal(field, value))
}

fn parse_optional_decimal(field: &str, value: Option<String>) -> Result<Option<Decimal>> {
    value
        .as_deref()
        .map(|value| parse_decimal(field, value))
        .transpose()
}

fn parse_decimal(field: &str, value: &str) -> Result<Decimal> {
    Decimal::from_str(value).map_err(|err| {
        Error::Validation(ValidationError::InvalidInput(format!(
            "{field} {value:?} is not a valid decimal: {err}"
        )))
    })
}

fn parse_required_date(field: &str, value: Option<String>) -> Result<NaiveDate> {
    value
        .as_deref()
        .ok_or_else(|| invalid(format!("{field} is required")))
        .and_then(parse_date)
}

fn parse_date(value: &str) -> Result<NaiveDate> {
    NaiveDate::parse_from_str(value, "%Y-%m-%d").map_err(|err| {
        Error::Validation(ValidationError::InvalidInput(format!(
            "date {value:?} is invalid: {err}"
        )))
    })
}

fn parse_optional_date(value: Option<String>) -> Result<Option<NaiveDate>> {
    value.as_deref().map(parse_date).transpose()
}

fn invalid(message: impl Into<String>) -> Error {
    Error::Validation(ValidationError::InvalidInput(message.into()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::write_actor::spawn_writer;
    use crate::db::{create_pool, init, run_migrations};
    use crate::schema::assets;
    use rust_decimal_macros::dec;
    use tempfile::tempdir;

    fn setup() -> (
        Arc<Pool<r2d2::ConnectionManager<SqliteConnection>>>,
        WriteHandle,
    ) {
        std::env::set_var("CONNECT_API_URL", "http://test.local");
        let app_data = tempdir()
            .expect("tempdir")
            .keep()
            .to_string_lossy()
            .to_string();
        let db_path = init(&app_data).expect("init db");
        run_migrations(&db_path).expect("run migrations");
        let pool = create_pool(&db_path).expect("create pool");
        let writer = spawn_writer(pool.as_ref().clone()).expect("spawn writer");
        seed_asset(&pool);
        (pool, writer)
    }

    fn seed_asset(pool: &Arc<Pool<r2d2::ConnectionManager<SqliteConnection>>>) {
        let mut conn = get_connection(pool).expect("conn");
        diesel::insert_into(assets::table)
            .values((
                assets::id.eq("asset-1"),
                assets::kind.eq("INVESTMENT"),
                assets::name.eq(Some("Bond")),
                assets::is_active.eq(1),
                assets::quote_mode.eq("MANUAL"),
                assets::quote_ccy.eq("USD"),
                assets::classification.eq(Some("fixed_income")),
                assets::created_at.eq("2026-05-14T00:00:00Z"),
                assets::updated_at.eq("2026-05-14T00:00:00Z"),
            ))
            .execute(&mut conn)
            .expect("seed asset");
    }

    fn request() -> UpsertFixedIncomeDetailsRequest {
        UpsertFixedIncomeDetailsRequest {
            asset_id: "asset-1".into(),
            instrument_type: FixedIncomeSubtype::Bond,
            issuer: "Treasury".into(),
            isin: None,
            face_value: dec!(1000),
            currency: "USD".into(),
            purchase_date: Some(NaiveDate::from_ymd_opt(2026, 1, 1).unwrap()),
            maturity_date: NaiveDate::from_ymd_opt(2027, 1, 1).unwrap(),
            coupon_or_profit_rate: Some(dec!(0.06)),
            payment_frequency: Some(FixedIncomePaymentFrequency::SemiAnnual),
            day_count_convention: DayCountConvention::Act365,
            is_sukuk: false,
            source_citation_id: None,
        }
    }

    #[tokio::test]
    async fn upsert_generates_expected_cashflows_atomically() {
        let (pool, writer) = setup();
        let repo = FixedIncomeRepository::new(pool, writer);
        let projection = repo.upsert_details(request()).await.unwrap();
        assert_eq!(projection.cashflows.len(), 2);
        assert_eq!(
            projection.cashflows[0].cashflow_type,
            FixedIncomeCashflowType::Coupon
        );
        assert_eq!(projection.cashflows[0].expected_amount, dec!(30));

        let loaded = repo
            .get_projection("asset-1")
            .await
            .unwrap()
            .expect("projection");
        assert_eq!(loaded.cashflows.len(), 2);
        assert_eq!(loaded.details.face_value, dec!(1000));
    }
}
