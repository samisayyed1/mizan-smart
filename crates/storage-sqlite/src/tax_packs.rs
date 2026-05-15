use async_trait::async_trait;
use chrono::{NaiveDate, Utc};
use diesel::prelude::*;
use diesel::r2d2::{self, Pool};
use diesel::SqliteConnection;
use rust_decimal::Decimal;
use std::str::FromStr;
use std::sync::Arc;
use uuid::Uuid;

use mizan_core::activities::Activity;
use mizan_core::private_investments::PrivateDistribution;
use mizan_core::tax_packs::{
    build_tax_pack_draft, GenerateTaxPackRequest, TaxJurisdiction, TaxPack, TaxPackLine,
    TaxPackLineCategory, TaxPackMissingItem, TaxPackRepositoryTrait, TaxPackStatus,
    TAX_PACK_DISCLAIMER,
};
use mizan_core::{Error, Result};

use crate::activities::ActivityDB;
use crate::db::{get_connection, WriteHandle};
use crate::errors::StorageError;
use crate::schema::{
    accounts, activities, private_distributions, tax_pack_lines, tax_pack_missing_items, tax_packs,
};

pub struct TaxPackRepository {
    pool: Arc<Pool<r2d2::ConnectionManager<SqliteConnection>>>,
    writer: WriteHandle,
}

impl TaxPackRepository {
    pub fn new(
        pool: Arc<Pool<r2d2::ConnectionManager<SqliteConnection>>>,
        writer: WriteHandle,
    ) -> Self {
        Self { pool, writer }
    }
}

#[derive(Debug, Clone, Queryable, Insertable, Selectable)]
#[diesel(table_name = tax_packs)]
struct TaxPackRow {
    id: String,
    tax_year: i32,
    jurisdiction: String,
    base_currency: String,
    status: String,
    created_at: String,
    finalized_at: Option<String>,
}

#[derive(Debug, Clone, Queryable, Insertable, Selectable)]
#[diesel(table_name = tax_pack_lines)]
struct TaxPackLineRow {
    id: String,
    tax_pack_id: String,
    category: String,
    asset_id: Option<String>,
    activity_id: Option<String>,
    amount: String,
    currency: String,
    taxable_date: String,
    source_citation_id: Option<String>,
    notes: Option<String>,
}

#[derive(Debug, Clone, Queryable, Insertable, Selectable)]
#[diesel(table_name = tax_pack_missing_items)]
struct TaxPackMissingItemRow {
    id: String,
    tax_pack_id: String,
    severity: String,
    message: String,
    related_activity_id: Option<String>,
    related_asset_id: Option<String>,
}

#[derive(Debug, Clone, Queryable)]
struct PrivateDistributionRow {
    id: String,
    asset_id: String,
    distribution_date: String,
    amount: String,
    currency: String,
    recallable: i32,
    source_citation_id: Option<String>,
    notes: Option<String>,
}

#[async_trait]
impl TaxPackRepositoryTrait for TaxPackRepository {
    async fn generate_tax_pack(&self, request: GenerateTaxPackRequest) -> Result<TaxPack> {
        request.validate()?;
        let pack_id = Uuid::new_v4().to_string();
        let created_at = Utc::now().to_rfc3339();
        let activities = self.load_activities_for_tax_pack(request.tax_year)?;
        let distributions = self.load_private_distributions_for_year(request.tax_year)?;
        let pack = build_tax_pack_draft(pack_id, request, created_at, &activities, &distributions)?;
        self.persist_pack(pack.clone()).await?;
        Ok(pack)
    }

    fn get_tax_pack(&self, tax_pack_id: &str) -> Result<Option<TaxPack>> {
        let mut conn = get_connection(&self.pool)?;
        let row = tax_packs::table
            .find(tax_pack_id)
            .select(TaxPackRow::as_select())
            .first::<TaxPackRow>(&mut conn)
            .optional()
            .map_err(StorageError::from)?;
        let Some(row) = row else {
            return Ok(None);
        };

        let line_rows = tax_pack_lines::table
            .filter(tax_pack_lines::tax_pack_id.eq(tax_pack_id))
            .order((tax_pack_lines::taxable_date.asc(), tax_pack_lines::id.asc()))
            .select(TaxPackLineRow::as_select())
            .load::<TaxPackLineRow>(&mut conn)
            .map_err(StorageError::from)?;
        let missing_rows = tax_pack_missing_items::table
            .filter(tax_pack_missing_items::tax_pack_id.eq(tax_pack_id))
            .order(tax_pack_missing_items::id.asc())
            .select(TaxPackMissingItemRow::as_select())
            .load::<TaxPackMissingItemRow>(&mut conn)
            .map_err(StorageError::from)?;

        Ok(Some(row_to_pack(row, line_rows, missing_rows)?))
    }
}

impl TaxPackRepository {
    fn load_activities_for_tax_pack(&self, tax_year: i32) -> Result<Vec<Activity>> {
        let mut conn = get_connection(&self.pool)?;
        let end = format!("{tax_year}-12-31T23:59:59");
        let rows = activities::table
            .inner_join(accounts::table.on(accounts::id.eq(activities::account_id)))
            .filter(accounts::is_archived.eq(false))
            .filter(activities::activity_date.le(end))
            .select(ActivityDB::as_select())
            .order((
                activities::activity_date.asc(),
                activities::created_at.asc(),
            ))
            .load::<ActivityDB>(&mut conn)
            .map_err(StorageError::from)?;
        Ok(rows.into_iter().map(Activity::from).collect())
    }

    fn load_private_distributions_for_year(
        &self,
        tax_year: i32,
    ) -> Result<Vec<PrivateDistribution>> {
        let mut conn = get_connection(&self.pool)?;
        let start = format!("{tax_year}-01-01");
        let end = format!("{tax_year}-12-31");
        let rows = private_distributions::table
            .filter(private_distributions::distribution_date.ge(start))
            .filter(private_distributions::distribution_date.le(end))
            .select((
                private_distributions::id,
                private_distributions::asset_id,
                private_distributions::distribution_date,
                private_distributions::amount,
                private_distributions::currency,
                private_distributions::recallable,
                private_distributions::source_citation_id,
                private_distributions::notes,
            ))
            .order(private_distributions::distribution_date.asc())
            .load::<PrivateDistributionRow>(&mut conn)
            .map_err(StorageError::from)?;
        rows.into_iter().map(TryInto::try_into).collect()
    }

    async fn persist_pack(&self, pack: TaxPack) -> Result<()> {
        let pack_row = TaxPackRow::from(&pack);
        let line_rows = pack
            .lines
            .iter()
            .map(TaxPackLineRow::from)
            .collect::<Vec<_>>();
        let missing_rows = pack
            .missing_data_checklist
            .iter()
            .map(TaxPackMissingItemRow::from)
            .collect::<Vec<_>>();

        self.writer
            .exec_tx(move |tx| -> Result<()> {
                let conn = tx.conn();
                diesel::insert_into(tax_packs::table)
                    .values(&pack_row)
                    .execute(conn)
                    .map_err(StorageError::from)?;
                if !line_rows.is_empty() {
                    diesel::insert_into(tax_pack_lines::table)
                        .values(&line_rows)
                        .execute(conn)
                        .map_err(StorageError::from)?;
                }
                if !missing_rows.is_empty() {
                    diesel::insert_into(tax_pack_missing_items::table)
                        .values(&missing_rows)
                        .execute(conn)
                        .map_err(StorageError::from)?;
                }
                Ok(())
            })
            .await
    }
}

impl From<&TaxPack> for TaxPackRow {
    fn from(pack: &TaxPack) -> Self {
        Self {
            id: pack.id.clone(),
            tax_year: pack.tax_year,
            jurisdiction: pack.jurisdiction.as_str().to_string(),
            base_currency: pack.base_currency.clone(),
            status: pack.status.as_str().to_string(),
            created_at: pack.created_at.clone(),
            finalized_at: pack.finalized_at.clone(),
        }
    }
}

impl From<&TaxPackLine> for TaxPackLineRow {
    fn from(line: &TaxPackLine) -> Self {
        Self {
            id: line.id.clone(),
            tax_pack_id: line.tax_pack_id.clone(),
            category: line.category.as_str().to_string(),
            asset_id: line.asset_id.clone(),
            activity_id: line.activity_id.clone(),
            amount: line.amount.normalize().to_string(),
            currency: line.currency.clone(),
            taxable_date: line.taxable_date.to_string(),
            source_citation_id: line.source_citation_id.clone(),
            notes: line.notes.clone(),
        }
    }
}

impl From<&TaxPackMissingItem> for TaxPackMissingItemRow {
    fn from(item: &TaxPackMissingItem) -> Self {
        Self {
            id: item.id.clone(),
            tax_pack_id: item.tax_pack_id.clone(),
            severity: item.severity.clone(),
            message: item.message.clone(),
            related_activity_id: item.related_activity_id.clone(),
            related_asset_id: item.related_asset_id.clone(),
        }
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

fn row_to_pack(
    row: TaxPackRow,
    line_rows: Vec<TaxPackLineRow>,
    missing_rows: Vec<TaxPackMissingItemRow>,
) -> Result<TaxPack> {
    Ok(TaxPack {
        id: row.id,
        tax_year: row.tax_year,
        jurisdiction: TaxJurisdiction::from_str(&row.jurisdiction)?,
        base_currency: row.base_currency,
        status: TaxPackStatus::from_str(&row.status)?,
        created_at: row.created_at,
        finalized_at: row.finalized_at,
        lines: line_rows
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<_>>()?,
        missing_data_checklist: missing_rows.into_iter().map(Into::into).collect(),
        disclaimer: TAX_PACK_DISCLAIMER.to_string(),
    })
}

impl TryFrom<TaxPackLineRow> for TaxPackLine {
    type Error = Error;

    fn try_from(row: TaxPackLineRow) -> Result<Self> {
        Ok(Self {
            id: row.id,
            tax_pack_id: row.tax_pack_id,
            category: TaxPackLineCategory::from_str(&row.category)?,
            asset_id: row.asset_id,
            activity_id: row.activity_id,
            amount: parse_decimal("amount", &row.amount)?,
            currency: row.currency,
            taxable_date: parse_date(&row.taxable_date)?,
            source_citation_id: row.source_citation_id,
            notes: row.notes,
        })
    }
}

impl From<TaxPackMissingItemRow> for TaxPackMissingItem {
    fn from(row: TaxPackMissingItemRow) -> Self {
        Self {
            id: row.id,
            tax_pack_id: row.tax_pack_id,
            severity: row.severity,
            message: row.message,
            related_activity_id: row.related_activity_id,
            related_asset_id: row.related_asset_id,
        }
    }
}

fn parse_decimal(field: &str, value: &str) -> Result<Decimal> {
    Decimal::from_str(value).map_err(|err| {
        Error::Validation(mizan_core::errors::ValidationError::InvalidInput(format!(
            "{field} {value:?} is not a valid decimal: {err}"
        )))
    })
}

fn parse_date(value: &str) -> Result<NaiveDate> {
    NaiveDate::parse_from_str(value, "%Y-%m-%d").map_err(|err| {
        Error::Validation(mizan_core::errors::ValidationError::InvalidInput(format!(
            "date {value:?} is invalid: {err}"
        )))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::write_actor::spawn_writer;
    use crate::db::{create_pool, init, run_migrations};
    use crate::schema::{accounts, assets};
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
        seed_account(&pool);
        seed_asset(&pool, "asset-1");
        (pool, writer)
    }

    #[tokio::test]
    async fn generated_pack_filters_tax_year_and_persists_lines() {
        let (pool, writer) = setup();
        seed_activity(
            &pool,
            "div-old",
            "DIVIDEND",
            "2025-12-31",
            None,
            None,
            Some(dec!(10)),
            None,
        );
        seed_activity(
            &pool,
            "div-new",
            "DIVIDEND",
            "2026-01-01",
            None,
            None,
            Some(dec!(20)),
            None,
        );
        let repo = TaxPackRepository::new(pool, writer);

        let pack = repo
            .generate_tax_pack(request())
            .await
            .expect("generate tax pack");
        let reloaded = repo.get_tax_pack(&pack.id).expect("lookup").expect("pack");

        assert_eq!(reloaded.lines.len(), 1);
        assert_eq!(reloaded.lines[0].activity_id.as_deref(), Some("div-new"));
        assert_eq!(reloaded.lines[0].amount, dec!(20));
    }

    #[tokio::test]
    async fn generated_pack_includes_fifo_gain_coupon_and_missing_citation_warning() {
        let (pool, writer) = setup();
        seed_activity(
            &pool,
            "buy-1",
            "BUY",
            "2025-01-01",
            Some(dec!(10)),
            Some(dec!(10)),
            Some(dec!(100)),
            None,
        );
        seed_activity(
            &pool,
            "sell-1",
            "SELL",
            "2026-01-10",
            Some(dec!(5)),
            None,
            Some(dec!(80)),
            None,
        );
        seed_activity(
            &pool,
            "coupon-1",
            "INTEREST",
            "2026-01-11",
            None,
            None,
            Some(dec!(12)),
            Some("COUPON"),
        );
        let repo = TaxPackRepository::new(pool, writer);

        let pack = repo.generate_tax_pack(request()).await.expect("generate");

        assert!(pack
            .lines
            .iter()
            .any(|line| line.category == TaxPackLineCategory::RealizedGain
                && line.amount == dec!(30)));
        assert!(pack
            .lines
            .iter()
            .any(|line| line.category == TaxPackLineCategory::Coupon && line.amount == dec!(12)));
        assert!(pack
            .missing_data_checklist
            .iter()
            .any(|item| item.message.contains("no source citation")));
    }

    #[tokio::test]
    async fn empty_pack_is_persisted_with_checklist() {
        let (pool, writer) = setup();
        let repo = TaxPackRepository::new(pool, writer);

        let pack = repo.generate_tax_pack(request()).await.expect("generate");

        assert!(pack.lines.is_empty());
        assert!(repo
            .get_tax_pack(&pack.id)
            .expect("lookup")
            .expect("pack")
            .missing_data_checklist
            .iter()
            .any(|item| item.message.contains("No taxable ledger activity")));
    }

    fn request() -> GenerateTaxPackRequest {
        GenerateTaxPackRequest {
            tax_year: 2026,
            jurisdiction: TaxJurisdiction::General,
            base_currency: "USD".to_string(),
        }
    }

    fn seed_account(pool: &Arc<Pool<r2d2::ConnectionManager<SqliteConnection>>>) {
        let mut conn = get_connection(pool).expect("conn");
        diesel::insert_into(accounts::table)
            .values((
                accounts::id.eq("acc-1"),
                accounts::name.eq("Taxable"),
                accounts::account_type.eq("brokerage"),
                accounts::currency.eq("USD"),
                accounts::is_default.eq(false),
                accounts::is_active.eq(true),
                accounts::created_at.eq("2026-05-16T00:00:00Z"),
                accounts::updated_at.eq("2026-05-16T00:00:00Z"),
                accounts::is_archived.eq(false),
                accounts::tracking_mode.eq("portfolio"),
            ))
            .execute(&mut conn)
            .expect("seed account");
    }

    fn seed_asset(pool: &Arc<Pool<r2d2::ConnectionManager<SqliteConnection>>>, asset_id: &str) {
        let mut conn = get_connection(pool).expect("conn");
        diesel::insert_into(assets::table)
            .values((
                assets::id.eq(asset_id),
                assets::kind.eq("INVESTMENT"),
                assets::name.eq(Some("Asset")),
                assets::display_code.eq(Some("AST")),
                assets::is_active.eq(1),
                assets::quote_mode.eq("MANUAL"),
                assets::quote_ccy.eq("USD"),
                assets::classification.eq(Some("public_equity")),
                assets::created_at.eq("2026-05-16T00:00:00Z"),
                assets::updated_at.eq("2026-05-16T00:00:00Z"),
            ))
            .execute(&mut conn)
            .expect("seed asset");
    }

    #[allow(clippy::too_many_arguments)]
    fn seed_activity(
        pool: &Arc<Pool<r2d2::ConnectionManager<SqliteConnection>>>,
        id: &str,
        activity_type: &str,
        date: &str,
        quantity: Option<Decimal>,
        unit_price: Option<Decimal>,
        amount: Option<Decimal>,
        subtype: Option<&str>,
    ) {
        let mut conn = get_connection(pool).expect("conn");
        diesel::insert_into(activities::table)
            .values((
                activities::id.eq(id),
                activities::account_id.eq("acc-1"),
                activities::asset_id.eq(Some("asset-1")),
                activities::activity_type.eq(activity_type),
                activities::status.eq("POSTED"),
                activities::activity_date.eq(format!("{date}T00:00:00Z")),
                activities::quantity.eq(quantity.map(|value| value.to_string())),
                activities::unit_price.eq(unit_price.map(|value| value.to_string())),
                activities::amount.eq(amount.map(|value| value.to_string())),
                activities::currency.eq("USD"),
                activities::subtype.eq(subtype),
                activities::is_user_modified.eq(0),
                activities::needs_review.eq(0),
                activities::created_at.eq(format!("{date}T00:00:00Z")),
                activities::updated_at.eq(format!("{date}T00:00:00Z")),
            ))
            .execute(&mut conn)
            .expect("seed activity");
    }
}
