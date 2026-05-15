use async_trait::async_trait;
use chrono::{NaiveDate, Utc};
use diesel::prelude::*;
use diesel::r2d2::{self, Pool};
use diesel::SqliteConnection;
use rust_decimal::Decimal;
use std::collections::HashSet;
use std::str::FromStr;
use std::sync::Arc;
use uuid::Uuid;

use mizan_core::corporate_actions::{
    preview_stock_split, AppliedCorporateAction, ApplyCorporateActionRequest, CorporateAction,
    CorporateActionPositionPreview, CorporateActionPreview, CorporateActionType,
    CorporateActionsRepositoryTrait,
};
use mizan_core::errors::ValidationError;
use mizan_core::portfolio::snapshot::AccountStateSnapshot;
use mizan_core::{Error, Result};

use crate::db::{get_connection, WriteHandle};
use crate::errors::StorageError;
use crate::portfolio::snapshot::AccountStateSnapshotDB;
use crate::schema::{activities, assets, corporate_actions, holdings_snapshots};

pub struct CorporateActionsRepository {
    pool: Arc<Pool<r2d2::ConnectionManager<SqliteConnection>>>,
    writer: WriteHandle,
}

impl CorporateActionsRepository {
    pub fn new(
        pool: Arc<Pool<r2d2::ConnectionManager<SqliteConnection>>>,
        writer: WriteHandle,
    ) -> Self {
        Self { pool, writer }
    }
}

#[derive(Debug, Clone, Queryable, Insertable)]
#[diesel(table_name = corporate_actions)]
struct CorporateActionRow {
    id: String,
    asset_id: String,
    action_type: String,
    effective_date: NaiveDate,
    ratio_numerator: Option<String>,
    ratio_denominator: Option<String>,
    new_symbol: Option<String>,
    metadata_json: Option<String>,
    source_citation_id: Option<String>,
    created_at: String,
}

impl TryFrom<CorporateActionRow> for CorporateAction {
    type Error = Error;

    fn try_from(row: CorporateActionRow) -> Result<Self> {
        Ok(Self {
            id: row.id,
            asset_id: row.asset_id,
            action_type: CorporateActionType::parse(&row.action_type)
                .ok_or_else(|| invalid("unsupported corporate action type"))?,
            effective_date: row.effective_date,
            ratio_numerator: parse_optional_decimal(row.ratio_numerator)?,
            ratio_denominator: parse_optional_decimal(row.ratio_denominator)?,
            new_symbol: row.new_symbol,
            metadata_json: row
                .metadata_json
                .as_deref()
                .map(serde_json::from_str)
                .transpose()
                .map_err(|err| invalid(format!("invalid corporate action metadata: {err}")))?,
            source_citation_id: row.source_citation_id,
            created_at: row.created_at,
        })
    }
}

#[async_trait]
impl CorporateActionsRepositoryTrait for CorporateActionsRepository {
    async fn preview_action(
        &self,
        request: ApplyCorporateActionRequest,
    ) -> Result<CorporateActionPreview> {
        let mut conn = get_connection(&self.pool)?;
        build_preview(&mut conn, &request)
    }

    async fn apply_action(
        &self,
        request: ApplyCorporateActionRequest,
    ) -> Result<AppliedCorporateAction> {
        request.validate()?;
        self.writer
            .exec_tx(move |tx| -> Result<AppliedCorporateAction> {
                let conn = tx.conn();
                let preview = build_preview(conn, &request)?;
                let now = Utc::now().to_rfc3339();
                let action_id = Uuid::new_v4().to_string();
                let row = CorporateActionRow {
                    id: action_id.clone(),
                    asset_id: request.asset_id.clone(),
                    action_type: request.action_type.as_str().to_string(),
                    effective_date: request.effective_date,
                    ratio_numerator: request
                        .ratio_numerator
                        .map(|value| value.normalize().to_string()),
                    ratio_denominator: request
                        .ratio_denominator
                        .map(|value| value.normalize().to_string()),
                    new_symbol: request
                        .new_symbol
                        .as_ref()
                        .map(|value| value.trim().to_uppercase()),
                    metadata_json: Some(
                        serde_json::json!({
                            "reviewedByUser": true,
                            "preview": preview.clone(),
                        })
                        .to_string(),
                    ),
                    source_citation_id: request.source_citation_id.clone(),
                    created_at: now.clone(),
                };

                diesel::insert_into(corporate_actions::table)
                    .values(&row)
                    .execute(conn)
                    .map_err(StorageError::from)?;

                match request.action_type {
                    CorporateActionType::Split | CorporateActionType::ReverseSplit => {
                        let ratio = request.ratio()?;
                        for position in &preview.positions {
                            insert_split_activity(
                                conn,
                                SplitActivityInsert {
                                    action_id: &action_id,
                                    asset_id: &request.asset_id,
                                    account_id: &position.account_id,
                                    effective_date: request.effective_date,
                                    ratio,
                                    currency: &position.currency,
                                    now: &now,
                                },
                            )?;
                        }
                    }
                    CorporateActionType::SymbolChange => {
                        let new_symbol = request
                            .new_symbol
                            .as_deref()
                            .ok_or_else(|| invalid("new_symbol is required"))?
                            .trim()
                            .to_uppercase();
                        let updated =
                            diesel::update(assets::table.filter(assets::id.eq(&request.asset_id)))
                                .set((
                                    assets::instrument_symbol.eq(Some(new_symbol.clone())),
                                    assets::display_code.eq(Some(new_symbol)),
                                    assets::updated_at.eq(&now),
                                ))
                                .execute(conn)
                                .map_err(StorageError::from)?;
                        if updated == 0 {
                            return Err(invalid("asset not found"));
                        }
                    }
                    _ => return Err(invalid("corporate action type is not implemented yet")),
                }

                Ok(AppliedCorporateAction {
                    action: row.try_into()?,
                    preview,
                })
            })
            .await
    }

    async fn list_actions(&self, asset_id: &str) -> Result<Vec<CorporateAction>> {
        let mut conn = get_connection(&self.pool)?;
        corporate_actions::table
            .filter(corporate_actions::asset_id.eq(asset_id))
            .order(corporate_actions::effective_date.desc())
            .load::<CorporateActionRow>(&mut conn)
            .map_err(StorageError::from)?
            .into_iter()
            .map(CorporateAction::try_from)
            .collect()
    }
}

fn build_preview(
    conn: &mut SqliteConnection,
    request: &ApplyCorporateActionRequest,
) -> Result<CorporateActionPreview> {
    request.validate()?;
    ensure_asset_exists(conn, &request.asset_id)?;

    let mut warnings = Vec::new();
    let positions = match request.action_type {
        CorporateActionType::Split | CorporateActionType::ReverseSplit => {
            let ratio = request.ratio()?;
            let mut previews =
                latest_positions_for_asset(conn, &request.asset_id, request.effective_date)?
                    .into_iter()
                    .map(|position| preview_stock_split(&position, ratio))
                    .collect::<Result<Vec<_>>>()?;
            previews.sort_by(|left, right| left.account_id.cmp(&right.account_id));
            if previews.is_empty() {
                warnings.push(
                    "No dated holdings snapshot currently shows an open position for this asset."
                        .to_string(),
                );
            }
            previews
        }
        CorporateActionType::SymbolChange => Vec::new(),
        _ => return Err(invalid("corporate action type is not implemented yet")),
    };

    Ok(CorporateActionPreview {
        asset_id: request.asset_id.clone(),
        action_type: request.action_type,
        effective_date: request.effective_date,
        ratio: match request.action_type {
            CorporateActionType::Split | CorporateActionType::ReverseSplit => {
                Some(request.ratio()?)
            }
            _ => None,
        },
        new_symbol: request
            .new_symbol
            .as_ref()
            .map(|value| value.trim().to_uppercase()),
        positions,
        warnings,
    })
}

fn latest_positions_for_asset(
    conn: &mut SqliteConnection,
    asset_id: &str,
    effective_date: NaiveDate,
) -> Result<Vec<CorporateActionPositionPreview>> {
    let rows = holdings_snapshots::table
        .filter(holdings_snapshots::snapshot_date.le(effective_date))
        .order((
            holdings_snapshots::account_id.asc(),
            holdings_snapshots::snapshot_date.desc(),
        ))
        .load::<AccountStateSnapshotDB>(conn)
        .map_err(StorageError::from)?;

    let mut seen_accounts = HashSet::new();
    let mut positions = Vec::new();
    for row in rows {
        if !seen_accounts.insert(row.account_id.clone()) {
            continue;
        }
        let snapshot = AccountStateSnapshot::from(row);
        if let Some(position) = snapshot.positions.get(asset_id) {
            if position.quantity <= Decimal::ZERO {
                continue;
            }
            positions.push(CorporateActionPositionPreview {
                account_id: position.account_id.clone(),
                quantity_before: position.quantity,
                quantity_after: position.quantity,
                average_cost_before: position.average_cost,
                average_cost_after: position.average_cost,
                total_cost_basis: position.total_cost_basis,
                currency: position.currency.clone(),
            });
        }
    }

    Ok(positions)
}

fn ensure_asset_exists(conn: &mut SqliteConnection, asset_id: &str) -> Result<()> {
    let exists = assets::table
        .filter(assets::id.eq(asset_id))
        .select(assets::id)
        .first::<String>(conn)
        .optional()
        .map_err(StorageError::from)?
        .is_some();
    if exists {
        Ok(())
    } else {
        Err(invalid("asset not found"))
    }
}

struct SplitActivityInsert<'a> {
    action_id: &'a str,
    asset_id: &'a str,
    account_id: &'a str,
    effective_date: NaiveDate,
    ratio: Decimal,
    currency: &'a str,
    now: &'a str,
}

fn insert_split_activity(
    conn: &mut SqliteConnection,
    input: SplitActivityInsert<'_>,
) -> Result<()> {
    let activity_id = Uuid::new_v4().to_string();
    let metadata = serde_json::json!({
        "corporateActionId": input.action_id,
        "reviewedByUser": true,
    })
    .to_string();
    diesel::insert_into(activities::table)
        .values((
            activities::id.eq(&activity_id),
            activities::account_id.eq(input.account_id),
            activities::asset_id.eq(Some(input.asset_id)),
            activities::activity_type.eq("SPLIT"),
            activities::status.eq("POSTED"),
            activities::activity_date.eq(format!("{}T00:00:00Z", input.effective_date)),
            activities::amount.eq(Some(input.ratio.normalize().to_string())),
            activities::currency.eq(input.currency),
            activities::metadata.eq(Some(metadata)),
            activities::source_system.eq(Some("MANUAL")),
            activities::source_record_id.eq(Some(input.action_id)),
            activities::idempotency_key.eq(Some(format!(
                "corporate-action:{}:{}",
                input.action_id, input.account_id
            ))),
            activities::is_user_modified.eq(1),
            activities::needs_review.eq(0),
            activities::created_at.eq(input.now),
            activities::updated_at.eq(input.now),
        ))
        .execute(conn)
        .map_err(StorageError::from)?;
    Ok(())
}

fn parse_optional_decimal(value: Option<String>) -> Result<Option<Decimal>> {
    value
        .as_deref()
        .map(Decimal::from_str)
        .transpose()
        .map_err(|err| invalid(format!("invalid decimal: {err}")))
}

fn invalid(message: impl Into<String>) -> Error {
    Error::Validation(ValidationError::InvalidInput(message.into()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{create_pool, get_connection, init, run_migrations};
    use crate::portfolio::snapshot::AccountStateSnapshotDB;
    use crate::schema::{accounts, activities, assets, holdings_snapshots};
    use diesel::r2d2;
    use diesel::SqliteConnection;
    use mizan_core::portfolio::snapshot::{AccountStateSnapshot, Position, SnapshotSource};
    use rust_decimal_macros::dec;
    use std::collections::HashMap;
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
        let writer =
            crate::db::write_actor::spawn_writer(pool.as_ref().clone()).expect("spawn writer");
        seed_account(&pool, "acc-1");
        seed_asset(&pool, "asset-1", "AAPL");
        (pool, writer)
    }

    fn seed_account(pool: &Arc<Pool<r2d2::ConnectionManager<SqliteConnection>>>, account_id: &str) {
        let mut conn = get_connection(pool).expect("conn");
        diesel::insert_into(accounts::table)
            .values((
                accounts::id.eq(account_id),
                accounts::name.eq("Taxable"),
                accounts::account_type.eq("brokerage"),
                accounts::currency.eq("USD"),
                accounts::is_default.eq(false),
                accounts::is_active.eq(true),
                accounts::created_at.eq("2026-05-14T00:00:00Z"),
                accounts::updated_at.eq("2026-05-14T00:00:00Z"),
                accounts::is_archived.eq(false),
                accounts::tracking_mode.eq("portfolio"),
            ))
            .execute(&mut conn)
            .expect("seed account");
    }

    fn seed_asset(
        pool: &Arc<Pool<r2d2::ConnectionManager<SqliteConnection>>>,
        asset_id: &str,
        symbol: &str,
    ) {
        let mut conn = get_connection(pool).expect("conn");
        diesel::insert_into(assets::table)
            .values((
                assets::id.eq(asset_id),
                assets::kind.eq("INVESTMENT"),
                assets::name.eq(Some(symbol)),
                assets::display_code.eq(Some(symbol)),
                assets::is_active.eq(1),
                assets::quote_mode.eq("MARKET"),
                assets::quote_ccy.eq("USD"),
                assets::instrument_type.eq(Some("EQUITY")),
                assets::instrument_symbol.eq(Some(symbol)),
                assets::created_at.eq("2026-05-14T00:00:00Z"),
                assets::updated_at.eq("2026-05-14T00:00:00Z"),
            ))
            .execute(&mut conn)
            .expect("seed asset");
    }

    fn seed_snapshot(pool: &Arc<Pool<r2d2::ConnectionManager<SqliteConnection>>>) {
        let mut position = Position::new(
            "acc-1".to_string(),
            "asset-1".to_string(),
            "USD".to_string(),
            Utc::now(),
        );
        position.quantity = dec!(10);
        position.average_cost = dec!(200);
        position.total_cost_basis = dec!(2000);

        let mut positions = HashMap::new();
        positions.insert("asset-1".to_string(), position);

        let snapshot = AccountStateSnapshot {
            id: AccountStateSnapshot::stable_id(
                "acc-1",
                NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            ),
            account_id: "acc-1".to_string(),
            snapshot_date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            currency: "USD".to_string(),
            positions,
            cost_basis: dec!(2000),
            calculated_at: Utc::now().naive_utc(),
            source: SnapshotSource::Calculated,
            ..AccountStateSnapshot::default()
        };

        let mut conn = get_connection(pool).expect("conn");
        diesel::insert_into(holdings_snapshots::table)
            .values(AccountStateSnapshotDB::from(snapshot))
            .execute(&mut conn)
            .expect("seed snapshot");
    }

    fn request(action_type: CorporateActionType) -> ApplyCorporateActionRequest {
        ApplyCorporateActionRequest {
            asset_id: "asset-1".to_string(),
            action_type,
            effective_date: NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
            ratio_numerator: Some(dec!(2)),
            ratio_denominator: Some(dec!(1)),
            new_symbol: None,
            source_citation_id: None,
        }
    }

    #[tokio::test]
    async fn split_apply_writes_audit_and_reviewed_split_activity() {
        let (pool, writer) = setup();
        seed_snapshot(&pool);
        let repo = CorporateActionsRepository::new(pool.clone(), writer);

        let applied = repo
            .apply_action(request(CorporateActionType::Split))
            .await
            .unwrap();

        assert_eq!(applied.preview.positions[0].quantity_after, dec!(20));
        assert_eq!(applied.preview.positions[0].average_cost_after, dec!(100));
        assert_eq!(applied.preview.positions[0].total_cost_basis, dec!(2000));

        let mut conn = get_connection(&pool).expect("conn");
        let split_amount = activities::table
            .filter(activities::source_record_id.eq(Some(applied.action.id.clone())))
            .select(activities::amount)
            .first::<Option<String>>(&mut conn)
            .unwrap();
        assert_eq!(split_amount.as_deref(), Some("2"));

        let actions = repo.list_actions("asset-1").await.unwrap();
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].action_type, CorporateActionType::Split);
    }

    #[tokio::test]
    async fn reverse_split_preview_preserves_cost_basis() {
        let (pool, writer) = setup();
        seed_snapshot(&pool);
        let repo = CorporateActionsRepository::new(pool, writer);
        let mut reverse = request(CorporateActionType::ReverseSplit);
        reverse.ratio_numerator = Some(dec!(1));
        reverse.ratio_denominator = Some(dec!(4));

        let preview = repo.preview_action(reverse).await.unwrap();

        assert_eq!(preview.positions[0].quantity_after, dec!(2.5));
        assert_eq!(preview.positions[0].average_cost_after, dec!(800));
        assert_eq!(preview.positions[0].total_cost_basis, dec!(2000));
    }

    #[tokio::test]
    async fn symbol_change_updates_asset_without_rewriting_activity_history() {
        let (pool, writer) = setup();
        {
            let mut conn = get_connection(&pool).expect("conn");
            diesel::insert_into(activities::table)
                .values((
                    activities::id.eq("activity-1"),
                    activities::account_id.eq("acc-1"),
                    activities::asset_id.eq(Some("asset-1")),
                    activities::activity_type.eq("BUY"),
                    activities::status.eq("POSTED"),
                    activities::activity_date.eq("2026-01-01T00:00:00Z"),
                    activities::quantity.eq(Some("10")),
                    activities::unit_price.eq(Some("200")),
                    activities::amount.eq(Some("2000")),
                    activities::currency.eq("USD"),
                    activities::is_user_modified.eq(1),
                    activities::needs_review.eq(0),
                    activities::created_at.eq("2026-01-01T00:00:00Z"),
                    activities::updated_at.eq("2026-01-01T00:00:00Z"),
                ))
                .execute(&mut conn)
                .expect("seed activity");
        }

        let repo = CorporateActionsRepository::new(pool.clone(), writer);
        let request = ApplyCorporateActionRequest {
            asset_id: "asset-1".to_string(),
            action_type: CorporateActionType::SymbolChange,
            effective_date: NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
            ratio_numerator: None,
            ratio_denominator: None,
            new_symbol: Some("META".to_string()),
            source_citation_id: None,
        };

        repo.apply_action(request).await.unwrap();

        let mut conn = get_connection(&pool).expect("conn");
        let new_symbol = assets::table
            .filter(assets::id.eq("asset-1"))
            .select(assets::instrument_symbol)
            .first::<Option<String>>(&mut conn)
            .unwrap();
        assert_eq!(new_symbol.as_deref(), Some("META"));

        let historical_asset_id = activities::table
            .filter(activities::id.eq("activity-1"))
            .select(activities::asset_id)
            .first::<Option<String>>(&mut conn)
            .unwrap();
        assert_eq!(historical_asset_id.as_deref(), Some("asset-1"));
    }
}
