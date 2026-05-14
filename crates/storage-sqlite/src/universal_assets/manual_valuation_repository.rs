//! SQLite repository for the p6 manual valuation grid.
//!
//! Bulk saves append new `valuations` rows, preserving history. The whole
//! batch runs through `WriteHandle::exec_tx`, so an insert failure rolls back
//! every row.

use chrono::{NaiveDate, Utc};
use diesel::prelude::*;
use diesel::r2d2::{self, Pool};
use diesel::SqliteConnection;
use std::collections::HashSet;
use std::sync::Arc;
use uuid::Uuid;

use mizan_core::universal_assets::{
    row_to_new_valuation, stale_status, validate_bulk_update_rows, AssetClassification,
    BulkUpdateValuationsRequest, BulkUpdateValuationsResult, ManualValuationAsset,
    ManualValuationStaleness, RowValidationError, Valuation,
};
use mizan_core::Result;

use super::valuation_model::ValuationDB;
use crate::db::{get_connection, WriteHandle};
use crate::errors::StorageError;
use crate::schema::valuations;
use crate::schema::valuations::dsl as valuation_dsl;

pub struct ManualValuationRepository {
    pool: Arc<Pool<r2d2::ConnectionManager<SqliteConnection>>>,
    writer: WriteHandle,
}

impl ManualValuationRepository {
    pub fn new(
        pool: Arc<Pool<r2d2::ConnectionManager<SqliteConnection>>>,
        writer: WriteHandle,
    ) -> Self {
        Self { pool, writer }
    }

    pub fn list_assets(&self, as_of: NaiveDate) -> Result<Vec<ManualValuationAsset>> {
        let mut conn = get_connection(&self.pool)?;
        let assets = load_manual_asset_rows(&mut conn)?;
        let mut result = Vec::with_capacity(assets.len());

        for asset in assets {
            let latest = latest_valuation(&mut conn, &asset.asset_id)?;
            let history_count = valuation_count(&mut conn, &asset.asset_id)?;
            let staleness = latest
                .as_ref()
                .map(|v| stale_status(v.valuation_date, as_of))
                .unwrap_or(ManualValuationStaleness::Critical);
            result.push(ManualValuationAsset {
                asset_id: asset.asset_id,
                name: asset.name.unwrap_or(asset.fallback_name),
                classification: AssetClassification::parse(&asset.classification)
                    .unwrap_or(AssetClassification::Custom),
                current_value: latest
                    .as_ref()
                    .map(|v| v.value_native.normalize().to_string()),
                valuation_date: latest.as_ref().map(|v| v.valuation_date),
                currency: latest
                    .as_ref()
                    .map(|v| v.currency.clone())
                    .unwrap_or(asset.quote_ccy),
                notes: latest.and_then(|v| v.notes),
                staleness,
                history_count,
            });
        }

        Ok(result)
    }

    pub fn history(&self, target_asset_id: &str) -> Result<Vec<Valuation>> {
        let mut conn = get_connection(&self.pool)?;
        let rows: Vec<ValuationDB> = valuation_dsl::valuations
            .filter(valuation_dsl::asset_id.eq(target_asset_id))
            .order((
                valuation_dsl::valuation_date.desc(),
                valuation_dsl::created_at.desc(),
            ))
            .load::<ValuationDB>(&mut conn)
            .map_err(StorageError::from)?;
        Ok(rows.into_iter().map(Valuation::from).collect())
    }

    pub async fn bulk_update(
        &self,
        request: BulkUpdateValuationsRequest,
    ) -> Result<BulkUpdateValuationsResult> {
        let mut errors = validate_bulk_update_rows(&request.rows);
        if !errors.is_empty() {
            return Ok(BulkUpdateValuationsResult {
                updated_count: 0,
                errors,
            });
        }

        let eligible_ids = self.eligible_asset_ids()?;
        for (row_index, row) in request.rows.iter().enumerate() {
            if !eligible_ids.contains(row.asset_id.trim()) {
                errors.push(RowValidationError {
                    row_index,
                    asset_id: Some(row.asset_id.clone()),
                    field: "assetId".into(),
                    message: "Asset is not a manually valued asset".into(),
                });
            }
        }
        if !errors.is_empty() {
            return Ok(BulkUpdateValuationsResult {
                updated_count: 0,
                errors,
            });
        }

        let now = Utc::now();
        let rows: Vec<ValuationDB> = request
            .rows
            .iter()
            .map(|row| {
                let new = row_to_new_valuation(row)?;
                new.validate()?;
                Ok(ValuationDB::new_row(Uuid::new_v4().to_string(), &new, now))
            })
            .collect::<Result<Vec<_>>>()?;
        let updated_count = rows.len();

        self.writer
            .exec_tx(move |tx| -> Result<()> {
                let conn = tx.conn();
                for row in &rows {
                    diesel::insert_into(valuations::table)
                        .values(row)
                        .execute(conn)
                        .map_err(StorageError::from)?;
                }
                Ok(())
            })
            .await?;

        Ok(BulkUpdateValuationsResult {
            updated_count,
            errors: Vec::new(),
        })
    }

    fn eligible_asset_ids(&self) -> Result<HashSet<String>> {
        let mut conn = get_connection(&self.pool)?;
        Ok(load_manual_asset_rows(&mut conn)?
            .into_iter()
            .map(|row| row.asset_id)
            .collect())
    }
}

#[derive(diesel::QueryableByName, Debug)]
struct ManualAssetRow {
    #[diesel(sql_type = diesel::sql_types::Text)]
    asset_id: String,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    name: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Text)]
    fallback_name: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    classification: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    quote_ccy: String,
}

fn load_manual_asset_rows(conn: &mut SqliteConnection) -> Result<Vec<ManualAssetRow>> {
    let rows = diesel::sql_query(
        "
        SELECT
          id AS asset_id,
          name,
          COALESCE(display_code, id) AS fallback_name,
          classification,
          quote_ccy
        FROM assets
        WHERE is_active = 1
          AND quote_mode = 'MANUAL'
          AND classification IN (
            'real_estate',
            'private_equity',
            'private_credit',
            'hedge_fund',
            'venture_capital',
            'commodity',
            'gold',
            'silver',
            'insurance',
            'ulip',
            'business_ownership',
            'collectible',
            'custom'
          )
        ORDER BY COALESCE(name, display_code, id) COLLATE NOCASE ASC
        ",
    )
    .load::<ManualAssetRow>(conn)
    .map_err(StorageError::from)?;
    Ok(rows)
}

fn latest_valuation(
    conn: &mut SqliteConnection,
    target_asset_id: &str,
) -> Result<Option<Valuation>> {
    let row = valuation_dsl::valuations
        .filter(valuation_dsl::asset_id.eq(target_asset_id))
        .order((
            valuation_dsl::valuation_date.desc(),
            valuation_dsl::created_at.desc(),
        ))
        .first::<ValuationDB>(conn)
        .optional()
        .map_err(StorageError::from)?;
    Ok(row.map(Valuation::from))
}

fn valuation_count(conn: &mut SqliteConnection, target_asset_id: &str) -> Result<usize> {
    let count: i64 = valuation_dsl::valuations
        .filter(valuation_dsl::asset_id.eq(target_asset_id))
        .count()
        .get_result(conn)
        .map_err(StorageError::from)?;
    Ok(count as usize)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::write_actor::spawn_writer;
    use crate::db::{create_pool, init, run_migrations};
    use crate::schema::assets;
    use mizan_core::universal_assets::ManualValuationUpdateRow;
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
        (pool, writer)
    }

    fn seed_manual_asset(pool: &Arc<Pool<r2d2::ConnectionManager<SqliteConnection>>>, aid: &str) {
        let mut conn = get_connection(pool).expect("conn");
        diesel::insert_into(assets::table)
            .values((
                assets::id.eq(aid),
                assets::kind.eq("PROPERTY"),
                assets::name.eq(Some("Manual asset")),
                assets::is_active.eq(1),
                assets::quote_mode.eq("MANUAL"),
                assets::quote_ccy.eq("USD"),
                assets::created_at.eq("2026-05-14T00:00:00Z"),
                assets::updated_at.eq("2026-05-14T00:00:00Z"),
                assets::classification.eq(Some("real_estate")),
            ))
            .execute(&mut conn)
            .expect("seed asset");
    }

    fn update_row(aid: &str, value: &str, date: &str) -> ManualValuationUpdateRow {
        ManualValuationUpdateRow {
            asset_id: aid.into(),
            current_value: value.into(),
            valuation_date: NaiveDate::parse_from_str(date, "%Y-%m-%d").unwrap(),
            currency: "USD".into(),
            notes: Some("manual check".into()),
        }
    }

    #[tokio::test]
    async fn valid_batch_saves_and_history_is_preserved() {
        let (pool, writer) = setup();
        seed_manual_asset(&pool, "asset-1");
        let repo = ManualValuationRepository::new(pool, writer);

        let first = repo
            .bulk_update(BulkUpdateValuationsRequest {
                rows: vec![update_row("asset-1", "1000", "2026-04-01")],
            })
            .await
            .expect("first update");
        assert_eq!(first.updated_count, 1);

        let second = repo
            .bulk_update(BulkUpdateValuationsRequest {
                rows: vec![update_row("asset-1", "1100", "2026-05-01")],
            })
            .await
            .expect("second update");
        assert_eq!(second.updated_count, 1);

        let history = repo.history("asset-1").expect("history");
        assert_eq!(history.len(), 2);
        assert_eq!(history[0].value_native, dec!(1100));
        assert_eq!(history[1].value_native, dec!(1000));
    }

    #[tokio::test]
    async fn invalid_batch_returns_errors_and_rolls_back_everything() {
        let (pool, writer) = setup();
        seed_manual_asset(&pool, "asset-1");
        seed_manual_asset(&pool, "asset-2");
        let repo = ManualValuationRepository::new(pool, writer);

        let result = repo
            .bulk_update(BulkUpdateValuationsRequest {
                rows: vec![
                    update_row("asset-1", "1000", "2026-05-01"),
                    update_row("asset-2", "not-money", "2026-05-01"),
                ],
            })
            .await
            .expect("bulk update");

        assert_eq!(result.updated_count, 0);
        assert_eq!(result.errors.len(), 1);
        assert_eq!(repo.history("asset-1").expect("history").len(), 0);
        assert_eq!(repo.history("asset-2").expect("history").len(), 0);
    }

    #[tokio::test]
    async fn list_assets_marks_stale_rows() {
        let (pool, writer) = setup();
        seed_manual_asset(&pool, "asset-1");
        let repo = ManualValuationRepository::new(pool, writer);
        repo.bulk_update(BulkUpdateValuationsRequest {
            rows: vec![update_row("asset-1", "1000", "2026-01-01")],
        })
        .await
        .expect("bulk update");

        let rows = repo
            .list_assets(NaiveDate::from_ymd_opt(2026, 5, 14).unwrap())
            .expect("list");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].staleness, ManualValuationStaleness::Critical);
        assert_eq!(rows[0].history_count, 1);
    }
}
