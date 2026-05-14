//! SQLite-backed valuation CRUD.
//!
//! Phase 1 P6 (bulk update grid), P15 (Explain-This-Number), and Phase
//! 5 web evidence approvals all read and write through this
//! repository. Writes flow through the project `WriteHandle` so SQLite
//! never sees parallel writes from different async tasks.

use async_trait::async_trait;
use chrono::{NaiveDate, Utc};
use diesel::prelude::*;
use diesel::r2d2::{self, Pool};
use diesel::SqliteConnection;
use std::sync::Arc;
use uuid::Uuid;

use mizan_core::universal_assets::{NewValuation, Valuation};
use mizan_core::Result;

use super::valuation_model::ValuationDB;
use crate::db::{get_connection, WriteHandle};
use crate::errors::StorageError;
use crate::schema::valuations;
use crate::schema::valuations::dsl::*;

/// Abstract API the universal asset domain depends on. Implemented by
/// [`ValuationRepository`] in production; tests can supply their own
/// fake without spinning up SQLite.
#[async_trait]
pub trait ValuationStore: Send + Sync {
    async fn insert(&self, valuation: &NewValuation) -> Result<Valuation>;
    async fn list_for_asset(&self, asset_id: &str) -> Result<Vec<Valuation>>;
    async fn latest_for_asset(&self, asset_id: &str) -> Result<Option<Valuation>>;
    async fn list_between(
        &self,
        asset_id: &str,
        from: NaiveDate,
        to: NaiveDate,
    ) -> Result<Vec<Valuation>>;
    async fn delete(&self, valuation_id: &str) -> Result<()>;
}

pub struct ValuationRepository {
    pool: Arc<Pool<r2d2::ConnectionManager<SqliteConnection>>>,
    writer: WriteHandle,
}

impl ValuationRepository {
    pub fn new(
        pool: Arc<Pool<r2d2::ConnectionManager<SqliteConnection>>>,
        writer: WriteHandle,
    ) -> Self {
        Self { pool, writer }
    }
}

#[async_trait]
impl ValuationStore for ValuationRepository {
    async fn insert(&self, valuation: &NewValuation) -> Result<Valuation> {
        // Validate at the boundary — every row in the table must come
        // through this gate, so callers cannot accidentally bypass the
        // domain checks (negative confidence, junk currency code, etc.).
        valuation.validate()?;

        let new_id = Uuid::new_v4().to_string();
        let row = ValuationDB::new_row(new_id, valuation, Utc::now());
        let row_clone = row.clone();

        self.writer
            .exec(move |conn: &mut SqliteConnection| -> Result<()> {
                diesel::insert_into(valuations::table)
                    .values(&row_clone)
                    .execute(conn)
                    .map_err(StorageError::from)?;
                Ok(())
            })
            .await?;
        Ok(row.into())
    }

    async fn list_for_asset(&self, target_asset_id: &str) -> Result<Vec<Valuation>> {
        let mut conn = get_connection(&self.pool)?;
        let rows: Vec<ValuationDB> = valuations
            .filter(asset_id.eq(target_asset_id))
            .order((valuation_date.desc(), created_at.desc()))
            .load::<ValuationDB>(&mut conn)
            .map_err(StorageError::from)?;
        Ok(rows.into_iter().map(Valuation::from).collect())
    }

    async fn latest_for_asset(&self, target_asset_id: &str) -> Result<Option<Valuation>> {
        let mut conn = get_connection(&self.pool)?;
        let row = valuations
            .filter(asset_id.eq(target_asset_id))
            .order((valuation_date.desc(), created_at.desc()))
            .first::<ValuationDB>(&mut conn)
            .optional()
            .map_err(StorageError::from)?;
        Ok(row.map(Valuation::from))
    }

    async fn list_between(
        &self,
        target_asset_id: &str,
        from: NaiveDate,
        to: NaiveDate,
    ) -> Result<Vec<Valuation>> {
        let from_s = from.to_string();
        let to_s = to.to_string();
        let mut conn = get_connection(&self.pool)?;
        let rows: Vec<ValuationDB> = valuations
            .filter(asset_id.eq(target_asset_id))
            .filter(valuation_date.ge(from_s))
            .filter(valuation_date.le(to_s))
            .order(valuation_date.asc())
            .load::<ValuationDB>(&mut conn)
            .map_err(StorageError::from)?;
        Ok(rows.into_iter().map(Valuation::from).collect())
    }

    async fn delete(&self, valuation_id: &str) -> Result<()> {
        let valuation_id = valuation_id.to_string();
        self.writer
            .exec(move |conn: &mut SqliteConnection| -> Result<()> {
                diesel::delete(valuations.find(&valuation_id))
                    .execute(conn)
                    .map_err(StorageError::from)?;
                Ok(())
            })
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::write_actor::spawn_writer;
    use crate::db::{create_pool, init, run_migrations};
    use crate::schema::assets;
    use mizan_core::universal_assets::{AssetClassification, NewValuation, ValuationSource};
    use rust_decimal_macros::dec;
    use tempfile::tempdir;

    fn setup_db() -> (
        Arc<Pool<r2d2::ConnectionManager<SqliteConnection>>>,
        WriteHandle,
    ) {
        // The codebase's outbox writes consult CONNECT_API_URL; setting
        // a placeholder keeps the writer side happy in test env.
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

    /// Seed a minimal asset row. The universal-asset flow normally
    /// inserts through the assets repository, but for valuation-level
    /// integration tests a raw INSERT is sufficient and avoids
    /// dragging in the wider asset service.
    fn seed_asset(
        pool: &Arc<Pool<r2d2::ConnectionManager<SqliteConnection>>>,
        seeded_id: &str,
        classification_value: Option<&str>,
    ) {
        let mut conn = get_connection(pool).expect("conn");
        diesel::insert_into(assets::table)
            .values((
                assets::id.eq(seeded_id),
                assets::kind.eq("PROPERTY"),
                assets::name.eq(Some("Test asset")),
                assets::is_active.eq(1),
                assets::quote_mode.eq("MANUAL"),
                assets::quote_ccy.eq("USD"),
                assets::created_at.eq("2026-05-14T00:00:00Z"),
                assets::updated_at.eq("2026-05-14T00:00:00Z"),
                assets::classification.eq(classification_value.map(str::to_string)),
            ))
            .execute(&mut conn)
            .expect("seed asset");
    }

    fn new_valuation(aid: &str, date: &str, value: rust_decimal::Decimal) -> NewValuation {
        NewValuation {
            asset_id: aid.to_string(),
            valuation_date: NaiveDate::parse_from_str(date, "%Y-%m-%d").unwrap(),
            value_native: value,
            currency: "USD".into(),
            source_type: ValuationSource::Manual,
            source_id: None,
            confidence: None,
            notes: None,
        }
    }

    #[tokio::test]
    async fn migration_creates_valuations_table_and_universal_asset_columns() {
        let (pool, _writer) = setup_db();
        // assets.classification must exist on the live table.
        let mut conn = get_connection(&pool).expect("conn");
        let count: i64 = assets::table
            .filter(assets::classification.is_not_null())
            .count()
            .get_result(&mut conn)
            .expect("classification column queryable");
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn insert_and_list_round_trips_a_valuation() {
        let (pool, writer) = setup_db();
        let repo = ValuationRepository::new(pool.clone(), writer);
        seed_asset(
            &pool,
            "asset-1",
            Some(AssetClassification::RealEstate.as_str()),
        );

        let inserted = repo
            .insert(&new_valuation("asset-1", "2026-05-14", dec!(1_500_000)))
            .await
            .expect("insert");
        assert_eq!(inserted.asset_id, "asset-1");
        assert_eq!(inserted.value_native, dec!(1_500_000));
        assert_eq!(inserted.currency, "USD");

        let listed = repo.list_for_asset("asset-1").await.expect("list");
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].id, inserted.id);
    }

    #[tokio::test]
    async fn list_for_asset_orders_newest_first() {
        let (pool, writer) = setup_db();
        let repo = ValuationRepository::new(pool.clone(), writer);
        seed_asset(&pool, "asset-2", None);

        for (date, value) in [
            ("2026-01-01", dec!(900_000)),
            ("2026-03-01", dec!(1_000_000)),
            ("2026-05-01", dec!(1_100_000)),
        ] {
            repo.insert(&new_valuation("asset-2", date, value))
                .await
                .expect("insert");
        }

        let listed = repo.list_for_asset("asset-2").await.expect("list");
        let dates: Vec<String> = listed
            .iter()
            .map(|v| v.valuation_date.to_string())
            .collect();
        assert_eq!(dates, vec!["2026-05-01", "2026-03-01", "2026-01-01"]);
    }

    #[tokio::test]
    async fn latest_for_asset_returns_most_recent_row() {
        let (pool, writer) = setup_db();
        let repo = ValuationRepository::new(pool.clone(), writer);
        seed_asset(&pool, "asset-3", None);

        repo.insert(&new_valuation("asset-3", "2026-01-01", dec!(1000)))
            .await
            .unwrap();
        repo.insert(&new_valuation("asset-3", "2026-04-01", dec!(2000)))
            .await
            .unwrap();

        let latest = repo
            .latest_for_asset("asset-3")
            .await
            .expect("latest")
            .expect("at least one row");
        assert_eq!(latest.value_native, dec!(2000));
    }

    #[tokio::test]
    async fn list_between_filters_by_date_window_inclusive() {
        let (pool, writer) = setup_db();
        let repo = ValuationRepository::new(pool.clone(), writer);
        seed_asset(&pool, "asset-4", None);

        for date in ["2026-01-15", "2026-02-10", "2026-03-20", "2026-04-30"] {
            repo.insert(&new_valuation("asset-4", date, dec!(100)))
                .await
                .unwrap();
        }

        let from = NaiveDate::from_ymd_opt(2026, 2, 1).unwrap();
        let to = NaiveDate::from_ymd_opt(2026, 3, 31).unwrap();
        let result = repo.list_between("asset-4", from, to).await.expect("range");
        let dates: Vec<String> = result
            .iter()
            .map(|v| v.valuation_date.to_string())
            .collect();
        assert_eq!(dates, vec!["2026-02-10", "2026-03-20"]);
    }

    #[tokio::test]
    async fn delete_removes_the_row_and_leaves_siblings_untouched() {
        let (pool, writer) = setup_db();
        let repo = ValuationRepository::new(pool.clone(), writer);
        seed_asset(&pool, "asset-5", None);

        let v1 = repo
            .insert(&new_valuation("asset-5", "2026-01-01", dec!(1)))
            .await
            .unwrap();
        let v2 = repo
            .insert(&new_valuation("asset-5", "2026-02-01", dec!(2)))
            .await
            .unwrap();
        repo.delete(&v1.id).await.expect("delete");

        let remaining = repo.list_for_asset("asset-5").await.expect("list");
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].id, v2.id);
    }

    #[tokio::test]
    async fn insert_rejects_invalid_valuation_at_the_domain_boundary() {
        let (_pool, writer) = setup_db();
        let pool = setup_db().0;
        let repo = ValuationRepository::new(pool.clone(), writer);

        let bad = NewValuation {
            asset_id: "asset-x".into(),
            valuation_date: NaiveDate::from_ymd_opt(2026, 5, 14).unwrap(),
            value_native: dec!(1),
            currency: "usd".into(), // lowercase rejected as non-ISO
            source_type: ValuationSource::Manual,
            source_id: None,
            confidence: None,
            notes: None,
        };
        let result = repo.insert(&bad).await;
        assert!(
            result.is_err(),
            "insert must reject invalid currency at the domain boundary"
        );
    }

    #[tokio::test]
    async fn check_constraint_rejects_unknown_source_type_on_raw_insert() {
        // Defence in depth: even if a future caller bypasses the
        // NewValuation::validate gate, the SQL CHECK on source_type
        // must still reject unknown values.
        let (pool, _writer) = setup_db();
        seed_asset(&pool, "asset-6", None);
        let mut conn = get_connection(&pool).expect("conn");
        let bad_row = ValuationDB {
            id: "v-bad".into(),
            asset_id: "asset-6".into(),
            valuation_date: "2026-05-14".into(),
            value_native: "1".into(),
            currency: "USD".into(),
            source_type: "nonexistent_source".into(),
            source_id: None,
            confidence: None,
            notes: None,
            created_at: "2026-05-14T00:00:00Z".into(),
            updated_at: "2026-05-14T00:00:00Z".into(),
        };
        let result = diesel::insert_into(valuations::table)
            .values(&bad_row)
            .execute(&mut conn);
        assert!(
            result.is_err(),
            "SQLite CHECK constraint must reject unknown source_type"
        );
    }
}
