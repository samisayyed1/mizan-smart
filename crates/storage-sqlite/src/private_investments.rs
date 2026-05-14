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
use mizan_core::private_investments::{
    calculate_private_investment_summary, CapitalCall, CapitalCallStatus, CreateCapitalCallRequest,
    CreatePrivateDistributionRequest, CreatePrivateInvestmentValuationRequest, PrivateDistribution,
    PrivateInvestment, PrivateInvestmentRepositoryTrait, PrivateInvestmentSummary,
    PrivateInvestmentValuation, UpdateCapitalCallStatusRequest, UpsertPrivateInvestmentRequest,
};
use mizan_core::{Error, Result};

use crate::db::{get_connection, WriteHandle};
use crate::errors::StorageError;
use crate::schema::{
    asset_private_investment_details, capital_calls, private_distributions,
    private_investment_valuations, private_investments,
};

pub struct PrivateInvestmentRepository {
    pool: Arc<Pool<r2d2::ConnectionManager<SqliteConnection>>>,
    writer: WriteHandle,
}

impl PrivateInvestmentRepository {
    pub fn new(
        pool: Arc<Pool<r2d2::ConnectionManager<SqliteConnection>>>,
        writer: WriteHandle,
    ) -> Self {
        Self { pool, writer }
    }
}

#[derive(Debug, Clone, Queryable, Insertable)]
#[diesel(table_name = private_investments)]
struct PrivateInvestmentRow {
    asset_id: String,
    manager: String,
    strategy: String,
    vintage_year: Option<i32>,
    commitment_amount: String,
    commitment_currency: String,
    inception_date: Option<String>,
    notes: Option<String>,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, Clone, Queryable, Insertable)]
#[diesel(table_name = private_investment_valuations)]
struct PrivateInvestmentValuationRow {
    id: String,
    asset_id: String,
    valuation_date: String,
    nav: String,
    currency: String,
    source_citation_id: Option<String>,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, Clone, Queryable, Insertable)]
#[diesel(table_name = capital_calls)]
struct CapitalCallRow {
    id: String,
    asset_id: String,
    notice_date: String,
    due_date: String,
    amount: String,
    currency: String,
    status: String,
    source_citation_id: Option<String>,
    notes: Option<String>,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, Clone, Queryable, Insertable)]
#[diesel(table_name = private_distributions)]
struct PrivateDistributionRow {
    id: String,
    asset_id: String,
    distribution_date: String,
    amount: String,
    currency: String,
    recallable: i32,
    source_citation_id: Option<String>,
    notes: Option<String>,
    created_at: String,
    updated_at: String,
}

impl TryFrom<PrivateInvestmentRow> for PrivateInvestment {
    type Error = Error;

    fn try_from(row: PrivateInvestmentRow) -> Result<Self> {
        Ok(Self {
            asset_id: row.asset_id,
            manager: row.manager,
            strategy: row.strategy,
            vintage_year: row.vintage_year,
            commitment_amount: parse_decimal("commitment_amount", &row.commitment_amount)?,
            commitment_currency: row.commitment_currency,
            inception_date: parse_optional_date(row.inception_date)?,
            notes: row.notes,
        })
    }
}

impl TryFrom<PrivateInvestmentValuationRow> for PrivateInvestmentValuation {
    type Error = Error;

    fn try_from(row: PrivateInvestmentValuationRow) -> Result<Self> {
        Ok(Self {
            id: row.id,
            asset_id: row.asset_id,
            valuation_date: parse_date(&row.valuation_date)?,
            nav: parse_decimal("nav", &row.nav)?,
            currency: row.currency,
            source_citation_id: row.source_citation_id,
        })
    }
}

impl TryFrom<CapitalCallRow> for CapitalCall {
    type Error = Error;

    fn try_from(row: CapitalCallRow) -> Result<Self> {
        Ok(Self {
            id: row.id,
            asset_id: row.asset_id,
            notice_date: parse_date(&row.notice_date)?,
            due_date: parse_date(&row.due_date)?,
            amount: parse_decimal("amount", &row.amount)?,
            currency: row.currency,
            status: CapitalCallStatus::from_str(&row.status)?,
            source_citation_id: row.source_citation_id,
            notes: row.notes,
        })
    }
}

impl TryFrom<PrivateDistributionRow> for PrivateDistribution {
    type Error = Error;

    fn try_from(row: PrivateDistributionRow) -> Result<Self> {
        Ok(Self {
            id: row.id,
            asset_id: row.asset_id,
            distribution_date: parse_date(&row.distribution_date)?,
            amount: parse_decimal("amount", &row.amount)?,
            currency: row.currency,
            recallable: row.recallable == 1,
            source_citation_id: row.source_citation_id,
            notes: row.notes,
        })
    }
}

#[async_trait]
impl PrivateInvestmentRepositoryTrait for PrivateInvestmentRepository {
    async fn upsert_investment(
        &self,
        request: UpsertPrivateInvestmentRequest,
    ) -> Result<PrivateInvestment> {
        request.validate()?;
        let investment = request.into_domain();
        let now = Utc::now().to_rfc3339();
        let row = PrivateInvestmentRow {
            asset_id: investment.asset_id.clone(),
            manager: investment.manager.clone(),
            strategy: investment.strategy.clone(),
            vintage_year: investment.vintage_year,
            commitment_amount: investment.commitment_amount.normalize().to_string(),
            commitment_currency: investment.commitment_currency.clone(),
            inception_date: investment.inception_date.map(|date| date.to_string()),
            notes: investment.notes.clone(),
            created_at: now.clone(),
            updated_at: now.clone(),
        };
        let details_asset_id = row.asset_id.clone();
        let details_manager = Some(row.manager.clone());
        let details_strategy = Some(row.strategy.clone());
        let details_vintage_year = row.vintage_year;
        let details_commitment_amount = Some(row.commitment_amount.clone());
        let details_commitment_currency = Some(row.commitment_currency.clone());
        let details_inception_date = row.inception_date.clone();
        let details_notes = row.notes.clone();
        let details_now = now.clone();

        self.writer
            .exec_tx(move |tx| -> Result<()> {
                let conn = tx.conn();
                diesel::insert_into(private_investments::table)
                    .values(&row)
                    .on_conflict(private_investments::asset_id)
                    .do_update()
                    .set((
                        private_investments::manager.eq(&row.manager),
                        private_investments::strategy.eq(&row.strategy),
                        private_investments::vintage_year.eq(row.vintage_year),
                        private_investments::commitment_amount.eq(&row.commitment_amount),
                        private_investments::commitment_currency.eq(&row.commitment_currency),
                        private_investments::inception_date.eq(&row.inception_date),
                        private_investments::notes.eq(&row.notes),
                        private_investments::updated_at.eq(&row.updated_at),
                    ))
                    .execute(conn)
                    .map_err(StorageError::from)?;

                diesel::update(
                    asset_private_investment_details::table
                        .filter(asset_private_investment_details::asset_id.eq(&details_asset_id)),
                )
                .set((
                    asset_private_investment_details::manager.eq(details_manager.as_ref()),
                    asset_private_investment_details::strategy.eq(details_strategy.as_ref()),
                    asset_private_investment_details::vintage_year.eq(details_vintage_year),
                    asset_private_investment_details::commitment_amount
                        .eq(details_commitment_amount.as_ref()),
                    asset_private_investment_details::commitment_currency
                        .eq(details_commitment_currency.as_ref()),
                    asset_private_investment_details::inception_date
                        .eq(details_inception_date.as_ref()),
                    asset_private_investment_details::notes.eq(details_notes.as_ref()),
                    asset_private_investment_details::updated_at.eq(&details_now),
                ))
                .execute(conn)
                .map_err(StorageError::from)?;
                Ok(())
            })
            .await?;

        Ok(investment)
    }

    async fn get_investment(&self, target_asset_id: &str) -> Result<Option<PrivateInvestment>> {
        let mut conn = get_connection(&self.pool)?;
        let row = private_investments::table
            .find(target_asset_id)
            .first::<PrivateInvestmentRow>(&mut conn)
            .optional()
            .map_err(StorageError::from)?;
        row.map(PrivateInvestment::try_from).transpose()
    }

    async fn delete_investment(&self, target_asset_id: &str) -> Result<()> {
        let target_asset_id = target_asset_id.to_string();
        self.writer
            .exec(move |conn| -> Result<()> {
                diesel::delete(private_investments::table.find(&target_asset_id))
                    .execute(conn)
                    .map_err(StorageError::from)?;
                Ok(())
            })
            .await
    }

    async fn add_valuation(
        &self,
        request: CreatePrivateInvestmentValuationRequest,
    ) -> Result<PrivateInvestmentValuation> {
        request.validate()?;
        let row = PrivateInvestmentValuationRow {
            id: Uuid::new_v4().to_string(),
            asset_id: request.asset_id,
            valuation_date: request.valuation_date.to_string(),
            nav: request.nav.normalize().to_string(),
            currency: request.currency,
            source_citation_id: request.source_citation_id,
            created_at: Utc::now().to_rfc3339(),
            updated_at: Utc::now().to_rfc3339(),
        };
        let inserted = row.clone();
        self.writer
            .exec(move |conn| -> Result<()> {
                diesel::insert_into(private_investment_valuations::table)
                    .values(&row)
                    .execute(conn)
                    .map_err(StorageError::from)?;
                Ok(())
            })
            .await?;
        PrivateInvestmentValuation::try_from(inserted)
    }

    async fn add_capital_call(&self, request: CreateCapitalCallRequest) -> Result<CapitalCall> {
        request.validate()?;
        let now = Utc::now().to_rfc3339();
        let row = CapitalCallRow {
            id: Uuid::new_v4().to_string(),
            asset_id: request.asset_id,
            notice_date: request.notice_date.to_string(),
            due_date: request.due_date.to_string(),
            amount: request.amount.normalize().to_string(),
            currency: request.currency,
            status: request.status.as_str().to_string(),
            source_citation_id: request.source_citation_id,
            notes: request.notes,
            created_at: now.clone(),
            updated_at: now,
        };
        let inserted = row.clone();
        self.writer
            .exec(move |conn| -> Result<()> {
                diesel::insert_into(capital_calls::table)
                    .values(&row)
                    .execute(conn)
                    .map_err(StorageError::from)?;
                Ok(())
            })
            .await?;
        CapitalCall::try_from(inserted)
    }

    async fn update_capital_call_status(
        &self,
        request: UpdateCapitalCallStatusRequest,
    ) -> Result<CapitalCall> {
        let call_id = request.id;
        let call_id_for_read = call_id.clone();
        let status = request.status.as_str().to_string();
        let now = Utc::now().to_rfc3339();
        self.writer
            .exec(move |conn| -> Result<()> {
                diesel::update(capital_calls::table.find(&call_id))
                    .set((
                        capital_calls::status.eq(&status),
                        capital_calls::updated_at.eq(&now),
                    ))
                    .execute(conn)
                    .map_err(StorageError::from)?;
                Ok(())
            })
            .await?;
        let mut conn = get_connection(&self.pool)?;
        let row = capital_calls::table
            .find(call_id_for_read)
            .first::<CapitalCallRow>(&mut conn)
            .map_err(StorageError::from)?;
        CapitalCall::try_from(row)
    }

    async fn add_distribution(
        &self,
        request: CreatePrivateDistributionRequest,
    ) -> Result<PrivateDistribution> {
        request.validate()?;
        let now = Utc::now().to_rfc3339();
        let row = PrivateDistributionRow {
            id: Uuid::new_v4().to_string(),
            asset_id: request.asset_id,
            distribution_date: request.distribution_date.to_string(),
            amount: request.amount.normalize().to_string(),
            currency: request.currency,
            recallable: if request.recallable { 1 } else { 0 },
            source_citation_id: request.source_citation_id,
            notes: request.notes,
            created_at: now.clone(),
            updated_at: now,
        };
        let inserted = row.clone();
        self.writer
            .exec(move |conn| -> Result<()> {
                diesel::insert_into(private_distributions::table)
                    .values(&row)
                    .execute(conn)
                    .map_err(StorageError::from)?;
                Ok(())
            })
            .await?;
        PrivateDistribution::try_from(inserted)
    }

    async fn get_summary(&self, target_asset_id: &str) -> Result<Option<PrivateInvestmentSummary>> {
        let Some(investment) = self.get_investment(target_asset_id).await? else {
            return Ok(None);
        };
        let mut conn = get_connection(&self.pool)?;
        let valuation_rows = private_investment_valuations::table
            .filter(private_investment_valuations::asset_id.eq(target_asset_id))
            .load::<PrivateInvestmentValuationRow>(&mut conn)
            .map_err(StorageError::from)?;
        let call_rows = capital_calls::table
            .filter(capital_calls::asset_id.eq(target_asset_id))
            .load::<CapitalCallRow>(&mut conn)
            .map_err(StorageError::from)?;
        let distribution_rows = private_distributions::table
            .filter(private_distributions::asset_id.eq(target_asset_id))
            .load::<PrivateDistributionRow>(&mut conn)
            .map_err(StorageError::from)?;
        let valuations = valuation_rows
            .into_iter()
            .map(PrivateInvestmentValuation::try_from)
            .collect::<Result<Vec<_>>>()?;
        let calls = call_rows
            .into_iter()
            .map(CapitalCall::try_from)
            .collect::<Result<Vec<_>>>()?;
        let distributions = distribution_rows
            .into_iter()
            .map(PrivateDistribution::try_from)
            .collect::<Result<Vec<_>>>()?;
        Ok(Some(calculate_private_investment_summary(
            investment,
            &valuations,
            &calls,
            &distributions,
        )))
    }
}

fn parse_decimal(field: &str, value: &str) -> Result<Decimal> {
    Decimal::from_str(value).map_err(|err| {
        Error::Validation(ValidationError::InvalidInput(format!(
            "{field} {value:?} is not a valid decimal: {err}"
        )))
    })
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::write_actor::spawn_writer;
    use crate::db::{create_pool, init, run_migrations};
    use crate::schema::{asset_private_investment_details, assets};
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
        seed_asset(&pool, "asset-1");
        (pool, writer)
    }

    fn seed_asset(pool: &Arc<Pool<r2d2::ConnectionManager<SqliteConnection>>>, asset_id: &str) {
        let mut conn = get_connection(pool).expect("conn");
        diesel::insert_into(assets::table)
            .values((
                assets::id.eq(asset_id),
                assets::kind.eq("INVESTMENT"),
                assets::name.eq(Some("Fund I")),
                assets::is_active.eq(1),
                assets::quote_mode.eq("MANUAL"),
                assets::quote_ccy.eq("USD"),
                assets::classification.eq(Some("private_equity")),
                assets::created_at.eq("2026-05-14T00:00:00Z"),
                assets::updated_at.eq("2026-05-14T00:00:00Z"),
            ))
            .execute(&mut conn)
            .expect("seed asset");
        diesel::insert_into(asset_private_investment_details::table)
            .values((
                asset_private_investment_details::asset_id.eq(asset_id),
                asset_private_investment_details::instrument_subtype.eq("private_equity"),
                asset_private_investment_details::created_at.eq("2026-05-14T00:00:00Z"),
                asset_private_investment_details::updated_at.eq("2026-05-14T00:00:00Z"),
            ))
            .execute(&mut conn)
            .expect("seed private detail");
    }

    fn upsert_request() -> UpsertPrivateInvestmentRequest {
        UpsertPrivateInvestmentRequest {
            asset_id: "asset-1".into(),
            manager: "Acme Capital".into(),
            strategy: "Buyout".into(),
            vintage_year: Some(2024),
            commitment_amount: dec!(1000),
            commitment_currency: "USD".into(),
            inception_date: Some(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()),
            notes: None,
        }
    }

    #[tokio::test]
    async fn upsert_private_investment_updates_detail_table() {
        let (pool, writer) = setup();
        let repo = PrivateInvestmentRepository::new(pool.clone(), writer);
        let investment = repo
            .upsert_investment(upsert_request())
            .await
            .expect("upsert investment");
        assert_eq!(investment.commitment_amount, dec!(1000));

        let mut conn = get_connection(&pool).expect("conn");
        let details: (Option<String>, Option<String>) = asset_private_investment_details::table
            .select((
                asset_private_investment_details::manager,
                asset_private_investment_details::commitment_amount,
            ))
            .first(&mut conn)
            .expect("detail row");
        assert_eq!(details.0.as_deref(), Some("Acme Capital"));
        assert_eq!(details.1.as_deref(), Some("1000"));
    }

    #[tokio::test]
    async fn summary_counts_paid_in_and_recallable_distribution() {
        let (_pool, writer) = setup();
        let repo = PrivateInvestmentRepository::new(_pool.clone(), writer);
        repo.upsert_investment(upsert_request()).await.unwrap();
        repo.add_capital_call(CreateCapitalCallRequest {
            asset_id: "asset-1".into(),
            notice_date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            due_date: NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
            amount: dec!(400),
            currency: "USD".into(),
            status: CapitalCallStatus::Paid,
            source_citation_id: None,
            notes: None,
        })
        .await
        .unwrap();
        repo.add_distribution(CreatePrivateDistributionRequest {
            asset_id: "asset-1".into(),
            distribution_date: NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(),
            amount: dec!(50),
            currency: "USD".into(),
            recallable: true,
            source_citation_id: None,
            notes: None,
        })
        .await
        .unwrap();
        repo.add_valuation(CreatePrivateInvestmentValuationRequest {
            asset_id: "asset-1".into(),
            valuation_date: NaiveDate::from_ymd_opt(2026, 6, 30).unwrap(),
            nav: dec!(450),
            currency: "USD".into(),
            source_citation_id: None,
        })
        .await
        .unwrap();

        let summary = repo.get_summary("asset-1").await.unwrap().expect("summary");
        assert_eq!(summary.paid_in_capital, dec!(400));
        assert_eq!(summary.recallable_distributions, dec!(50));
        assert_eq!(summary.unfunded_commitment, dec!(650));
        assert_eq!(summary.rvpi, Some(dec!(1.125)));
    }
}
