use async_trait::async_trait;
use chrono::NaiveDate;
use diesel::prelude::*;
use diesel::r2d2::{self, Pool};
use diesel::SqliteConnection;
use rust_decimal::Decimal;
use std::collections::BTreeMap;
use std::str::FromStr;
use std::sync::Arc;

use mizan_core::errors::ValidationError;
use mizan_core::fixed_income::{FixedIncomeCashflowStatus, FixedIncomeCashflowType};
use mizan_core::liquidity_ladder::{
    build_liquidity_ladder, LiquidityConfidence, LiquidityDirection, LiquidityItemType,
    LiquidityLadderItem, LiquidityLadderReport, LiquidityLadderRepositoryTrait,
};
use mizan_core::private_investments::CapitalCallStatus;
use mizan_core::{Error, Result};

use crate::db::get_connection;
use crate::errors::StorageError;
use crate::schema::{
    activities, capital_calls, fixed_income_cashflows, holdings_snapshots, private_distributions,
};

pub struct LiquidityLadderRepository {
    pool: Arc<Pool<r2d2::ConnectionManager<SqliteConnection>>>,
}

impl LiquidityLadderRepository {
    pub fn new(pool: Arc<Pool<r2d2::ConnectionManager<SqliteConnection>>>) -> Self {
        Self { pool }
    }
}

#[derive(Debug, Queryable)]
struct SnapshotCashRow {
    account_id: String,
    snapshot_date: NaiveDate,
    cash_balances: String,
}

#[derive(Debug, Queryable)]
struct FixedIncomeCashflowRow {
    id: String,
    asset_id: String,
    expected_date: String,
    cashflow_type: String,
    expected_amount: String,
    actual_amount: Option<String>,
    currency: String,
    status: String,
}

#[derive(Debug, Queryable)]
struct CapitalCallRow {
    id: String,
    asset_id: String,
    due_date: String,
    amount: String,
    currency: String,
    status: String,
}

#[derive(Debug, Queryable)]
struct PrivateDistributionRow {
    id: String,
    asset_id: String,
    distribution_date: String,
    amount: String,
    currency: String,
}

#[derive(Debug, Queryable)]
struct ScheduledIncomeRow {
    id: String,
    activity_type: String,
    activity_type_override: Option<String>,
    status: String,
    activity_date: String,
    amount: Option<String>,
    unit_price: Option<String>,
    currency: String,
}

#[async_trait]
impl LiquidityLadderRepositoryTrait for LiquidityLadderRepository {
    async fn get_ladder(&self, as_of: NaiveDate) -> Result<LiquidityLadderReport> {
        let mut conn = get_connection(&self.pool)?;
        let max_date = as_of
            .checked_add_months(chrono::Months::new(12))
            .unwrap_or(as_of)
            .to_string();
        let as_of_text = as_of.to_string();
        let mut items = Vec::new();

        items.extend(load_cash_balances(&mut conn, as_of)?);
        items.extend(load_fixed_income_cashflows(
            &mut conn,
            &as_of_text,
            &max_date,
        )?);
        items.extend(load_capital_calls(&mut conn, &as_of_text, &max_date)?);
        items.extend(load_private_distributions(
            &mut conn,
            &as_of_text,
            &max_date,
        )?);
        items.extend(load_scheduled_income(&mut conn, &as_of_text, &max_date)?);

        Ok(build_liquidity_ladder(as_of, items))
    }
}

fn load_cash_balances(
    conn: &mut SqliteConnection,
    as_of: NaiveDate,
) -> Result<Vec<LiquidityLadderItem>> {
    let rows = holdings_snapshots::table
        .select((
            holdings_snapshots::account_id,
            holdings_snapshots::snapshot_date,
            holdings_snapshots::cash_balances,
        ))
        .load::<SnapshotCashRow>(conn)
        .map_err(StorageError::from)?;
    let mut latest = BTreeMap::<String, SnapshotCashRow>::new();
    for row in rows {
        let replace = latest
            .get(&row.account_id)
            .map(|existing| row.snapshot_date > existing.snapshot_date)
            .unwrap_or(true);
        if replace {
            latest.insert(row.account_id.clone(), row);
        }
    }

    let mut items = Vec::new();
    for row in latest.into_values() {
        let balances: BTreeMap<String, Decimal> =
            serde_json::from_str(&row.cash_balances).unwrap_or_default();
        for (currency, amount) in balances {
            if amount == Decimal::ZERO {
                continue;
            }
            items.push(LiquidityLadderItem {
                id: format!("cash:{}:{currency}", row.account_id),
                date: as_of,
                currency,
                amount,
                direction: LiquidityDirection::Balance,
                confidence: LiquidityConfidence::Confirmed,
                item_type: LiquidityItemType::CashBalance,
                label: "Available cash balance".into(),
                source_id: Some(row.account_id.clone()),
                notes: Some(format!("Latest cash snapshot from {}", row.snapshot_date)),
            });
        }
    }
    Ok(items)
}

fn load_fixed_income_cashflows(
    conn: &mut SqliteConnection,
    as_of: &str,
    max_date: &str,
) -> Result<Vec<LiquidityLadderItem>> {
    let rows = fixed_income_cashflows::table
        .select((
            fixed_income_cashflows::id,
            fixed_income_cashflows::asset_id,
            fixed_income_cashflows::expected_date,
            fixed_income_cashflows::cashflow_type,
            fixed_income_cashflows::expected_amount,
            fixed_income_cashflows::actual_amount,
            fixed_income_cashflows::currency,
            fixed_income_cashflows::status,
        ))
        .filter(fixed_income_cashflows::expected_date.ge(as_of))
        .filter(fixed_income_cashflows::expected_date.le(max_date))
        .filter(fixed_income_cashflows::status.ne("cancelled"))
        .load::<FixedIncomeCashflowRow>(conn)
        .map_err(StorageError::from)?;

    rows.into_iter()
        .map(|row| {
            let status = FixedIncomeCashflowStatus::parse(&row.status)
                .ok_or_else(|| invalid("unsupported fixed income cashflow status"))?;
            let cashflow_type = FixedIncomeCashflowType::parse(&row.cashflow_type)
                .ok_or_else(|| invalid("unsupported fixed income cashflow type"))?;
            let amount = row
                .actual_amount
                .as_deref()
                .map(|value| parse_decimal("actual_amount", value))
                .transpose()?
                .unwrap_or(parse_decimal("expected_amount", &row.expected_amount)?);
            Ok(LiquidityLadderItem {
                id: format!("fixed-income:{}", row.id),
                date: parse_date(&row.expected_date)?,
                currency: row.currency,
                amount,
                direction: LiquidityDirection::Incoming,
                confidence: if status == FixedIncomeCashflowStatus::Received {
                    LiquidityConfidence::Confirmed
                } else {
                    LiquidityConfidence::Expected
                },
                item_type: match cashflow_type {
                    FixedIncomeCashflowType::Profit => LiquidityItemType::SukukProfit,
                    FixedIncomeCashflowType::Maturity => LiquidityItemType::FixedDepositMaturity,
                    _ => LiquidityItemType::FixedIncomeCashflow,
                },
                label: match cashflow_type {
                    FixedIncomeCashflowType::Profit => "Sukuk profit payment".into(),
                    FixedIncomeCashflowType::Maturity => "Fixed deposit maturity".into(),
                    FixedIncomeCashflowType::Principal => "Fixed income principal".into(),
                    FixedIncomeCashflowType::Interest => "Fixed income interest".into(),
                    FixedIncomeCashflowType::Coupon => "Fixed income coupon".into(),
                },
                source_id: Some(row.asset_id),
                notes: None,
            })
        })
        .collect()
}

fn load_capital_calls(
    conn: &mut SqliteConnection,
    as_of: &str,
    max_date: &str,
) -> Result<Vec<LiquidityLadderItem>> {
    let rows = capital_calls::table
        .select((
            capital_calls::id,
            capital_calls::asset_id,
            capital_calls::due_date,
            capital_calls::amount,
            capital_calls::currency,
            capital_calls::status,
        ))
        .filter(capital_calls::due_date.ge(as_of))
        .filter(capital_calls::due_date.le(max_date))
        .filter(capital_calls::status.ne("cancelled"))
        .load::<CapitalCallRow>(conn)
        .map_err(StorageError::from)?;

    rows.into_iter()
        .map(|row| {
            let status = CapitalCallStatus::from_str(&row.status)?;
            Ok(LiquidityLadderItem {
                id: format!("capital-call:{}", row.id),
                date: parse_date(&row.due_date)?,
                currency: row.currency,
                amount: parse_decimal("amount", &row.amount)?,
                direction: LiquidityDirection::Outgoing,
                confidence: if status == CapitalCallStatus::Paid {
                    LiquidityConfidence::Confirmed
                } else {
                    LiquidityConfidence::Expected
                },
                item_type: LiquidityItemType::PrivateCapitalCall,
                label: "Private capital call".into(),
                source_id: Some(row.asset_id),
                notes: None,
            })
        })
        .collect()
}

fn load_private_distributions(
    conn: &mut SqliteConnection,
    as_of: &str,
    max_date: &str,
) -> Result<Vec<LiquidityLadderItem>> {
    let rows = private_distributions::table
        .select((
            private_distributions::id,
            private_distributions::asset_id,
            private_distributions::distribution_date,
            private_distributions::amount,
            private_distributions::currency,
        ))
        .filter(private_distributions::distribution_date.ge(as_of))
        .filter(private_distributions::distribution_date.le(max_date))
        .load::<PrivateDistributionRow>(conn)
        .map_err(StorageError::from)?;

    rows.into_iter()
        .map(|row| {
            Ok(LiquidityLadderItem {
                id: format!("private-distribution:{}", row.id),
                date: parse_date(&row.distribution_date)?,
                currency: row.currency,
                amount: parse_decimal("amount", &row.amount)?,
                direction: LiquidityDirection::Incoming,
                confidence: LiquidityConfidence::Confirmed,
                item_type: LiquidityItemType::PrivateDistribution,
                label: "Private distribution".into(),
                source_id: Some(row.asset_id),
                notes: None,
            })
        })
        .collect()
}

fn load_scheduled_income(
    conn: &mut SqliteConnection,
    as_of: &str,
    max_date: &str,
) -> Result<Vec<LiquidityLadderItem>> {
    let rows = activities::table
        .select((
            activities::id,
            activities::activity_type,
            activities::activity_type_override,
            activities::status,
            activities::activity_date,
            activities::amount,
            activities::unit_price,
            activities::currency,
        ))
        .filter(activities::activity_date.ge(as_of))
        .filter(activities::activity_date.le(max_date))
        .filter(activities::status.ne("VOID"))
        .load::<ScheduledIncomeRow>(conn)
        .map_err(StorageError::from)?;

    rows.into_iter()
        .filter_map(|row| {
            let activity_type = row
                .activity_type_override
                .as_deref()
                .unwrap_or(&row.activity_type);
            let item_type = match activity_type {
                "DIVIDEND" => LiquidityItemType::ScheduledDividend,
                "INTEREST" => LiquidityItemType::ScheduledInterest,
                _ => return None,
            };
            let amount = row
                .amount
                .as_deref()
                .or(row.unit_price.as_deref())
                .map(|value| parse_decimal("amount", value));
            amount.map(|amount| {
                let amount = amount?;
                Ok(LiquidityLadderItem {
                    id: format!("scheduled-income:{}", row.id),
                    date: parse_date(&row.activity_date)?,
                    currency: row.currency,
                    amount,
                    direction: LiquidityDirection::Incoming,
                    confidence: if row.status == "POSTED" {
                        LiquidityConfidence::Confirmed
                    } else {
                        LiquidityConfidence::Expected
                    },
                    item_type,
                    label: match item_type {
                        LiquidityItemType::ScheduledDividend => "Scheduled dividend".into(),
                        LiquidityItemType::ScheduledInterest => "Scheduled interest".into(),
                        _ => "Scheduled income".into(),
                    },
                    source_id: None,
                    notes: Some("Recorded activity; no future income was projected.".into()),
                })
            })
        })
        .collect::<Result<Vec<_>>>()
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

fn invalid(message: impl Into<String>) -> Error {
    Error::Validation(ValidationError::InvalidInput(message.into()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::write_actor::spawn_writer;
    use crate::db::{create_pool, init, run_migrations};
    use crate::schema::{assets, holdings_snapshots};
    use diesel::RunQueryDsl;
    use rust_decimal_macros::dec;
    use tempfile::tempdir;

    fn setup() -> Arc<Pool<r2d2::ConnectionManager<SqliteConnection>>> {
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
        drop(writer);
        seed_rows(&pool);
        pool
    }

    fn seed_rows(pool: &Arc<Pool<r2d2::ConnectionManager<SqliteConnection>>>) {
        let mut conn = get_connection(pool).expect("conn");
        diesel::insert_into(assets::table)
            .values((
                assets::id.eq("asset-1"),
                assets::kind.eq("INVESTMENT"),
                assets::name.eq(Some("Income Asset")),
                assets::is_active.eq(1),
                assets::quote_mode.eq("MANUAL"),
                assets::quote_ccy.eq("USD"),
                assets::classification.eq(Some("fixed_income")),
                assets::created_at.eq("2026-05-14T00:00:00Z"),
                assets::updated_at.eq("2026-05-14T00:00:00Z"),
            ))
            .execute(&mut conn)
            .expect("seed asset");

        diesel::insert_into(holdings_snapshots::table)
            .values((
                holdings_snapshots::id.eq("snapshot-1"),
                holdings_snapshots::account_id.eq("account-1"),
                holdings_snapshots::snapshot_date.eq(NaiveDate::from_ymd_opt(2026, 5, 14).unwrap()),
                holdings_snapshots::currency.eq("USD"),
                holdings_snapshots::positions.eq("{}"),
                holdings_snapshots::cash_balances.eq(r#"{"USD":"1000","AED":"500"}"#),
                holdings_snapshots::cost_basis.eq("0"),
                holdings_snapshots::net_contribution.eq("0"),
                holdings_snapshots::calculated_at.eq("2026-05-14T00:00:00Z"),
                holdings_snapshots::net_contribution_base.eq("0"),
                holdings_snapshots::cash_total_account_currency.eq("1000"),
                holdings_snapshots::cash_total_base_currency.eq("1000"),
                holdings_snapshots::source.eq("MANUAL"),
                holdings_snapshots::realized_gains.eq("{}"),
            ))
            .execute(&mut conn)
            .expect("seed snapshot");

        diesel::insert_into(fixed_income_cashflows::table)
            .values((
                fixed_income_cashflows::id.eq("fi-1"),
                fixed_income_cashflows::asset_id.eq("asset-1"),
                fixed_income_cashflows::expected_date.eq("2026-05-30"),
                fixed_income_cashflows::cashflow_type.eq("profit"),
                fixed_income_cashflows::expected_amount.eq("25"),
                fixed_income_cashflows::actual_amount.eq::<Option<String>>(None),
                fixed_income_cashflows::currency.eq("USD"),
                fixed_income_cashflows::status.eq("expected"),
                fixed_income_cashflows::source_citation_id.eq::<Option<String>>(None),
                fixed_income_cashflows::created_at.eq("2026-05-14T00:00:00Z"),
                fixed_income_cashflows::updated_at.eq("2026-05-14T00:00:00Z"),
            ))
            .execute(&mut conn)
            .expect("seed fixed income");

        diesel::insert_into(capital_calls::table)
            .values((
                capital_calls::id.eq("call-1"),
                capital_calls::asset_id.eq("asset-1"),
                capital_calls::notice_date.eq("2026-05-15"),
                capital_calls::due_date.eq("2026-05-25"),
                capital_calls::amount.eq("200"),
                capital_calls::currency.eq("USD"),
                capital_calls::status.eq("due"),
                capital_calls::source_citation_id.eq::<Option<String>>(None),
                capital_calls::notes.eq::<Option<String>>(None),
                capital_calls::created_at.eq("2026-05-14T00:00:00Z"),
                capital_calls::updated_at.eq("2026-05-14T00:00:00Z"),
            ))
            .execute(&mut conn)
            .expect("seed call");
    }

    #[tokio::test]
    async fn ladder_includes_cash_fixed_income_and_capital_calls_grouped_by_currency() {
        let pool = setup();
        let repo = LiquidityLadderRepository::new(pool);
        let report = repo
            .get_ladder(NaiveDate::from_ymd_opt(2026, 5, 15).unwrap())
            .await
            .unwrap();
        let view = &report.views[0];
        assert_eq!(view.currency_groups.len(), 2);
        let usd = view
            .currency_groups
            .iter()
            .find(|group| group.currency == "USD")
            .unwrap();
        assert_eq!(usd.available_cash, dec!(1000));
        assert_eq!(usd.expected_incoming, dec!(25));
        assert_eq!(usd.expected_outgoing, dec!(200));
        assert!(usd
            .items
            .iter()
            .any(|item| item.item_type == LiquidityItemType::SukukProfit));
        assert!(usd
            .items
            .iter()
            .any(|item| item.item_type == LiquidityItemType::PrivateCapitalCall));
    }
}
