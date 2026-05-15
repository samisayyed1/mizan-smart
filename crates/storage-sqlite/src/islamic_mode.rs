use async_trait::async_trait;
use chrono::Utc;
use diesel::prelude::*;
use diesel::r2d2::{self, Pool};
use diesel::SqliteConnection;
use rust_decimal::Decimal;
use std::str::FromStr;
use std::sync::Arc;
use uuid::Uuid;

use mizan_core::errors::ValidationError;
use mizan_core::islamic_mode::{
    evaluate_screening_request, AssetShariahScreening, ShariahScreeningAuditEntry,
    ShariahScreeningProfile, ShariahScreeningRepositoryTrait, ShariahScreeningStatus,
    UpsertAssetShariahScreeningRequest,
};
use mizan_core::{Error, Result};

use crate::db::{get_connection, WriteHandle};
use crate::errors::StorageError;
use crate::schema::{
    asset_shariah_screening, shariah_screening_audit_log, shariah_screening_profiles,
};

pub struct ShariahScreeningRepository {
    pool: Arc<Pool<r2d2::ConnectionManager<SqliteConnection>>>,
    writer: WriteHandle,
}

impl ShariahScreeningRepository {
    pub fn new(
        pool: Arc<Pool<r2d2::ConnectionManager<SqliteConnection>>>,
        writer: WriteHandle,
    ) -> Self {
        Self { pool, writer }
    }
}

#[derive(Debug, Clone, Queryable)]
struct ShariahScreeningProfileRow {
    id: String,
    name: String,
    debt_threshold: String,
    liquid_assets_threshold: String,
    impure_income_threshold: String,
    is_default: i32,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, Clone, Queryable, Insertable)]
#[diesel(table_name = asset_shariah_screening)]
struct AssetShariahScreeningRow {
    id: String,
    asset_id: String,
    profile_id: String,
    status: String,
    debt_ratio: Option<String>,
    liquid_assets_ratio: Option<String>,
    impure_income_ratio: Option<String>,
    source_citation_id: Option<String>,
    manual_override_reason: Option<String>,
    reviewed_at: Option<String>,
    created_at: String,
    updated_at: String,
    notes: Option<String>,
}

#[derive(Debug, Clone, Queryable, Insertable)]
#[diesel(table_name = shariah_screening_audit_log)]
struct ShariahScreeningAuditRow {
    id: String,
    screening_id: String,
    asset_id: String,
    profile_id: String,
    previous_status: Option<String>,
    new_status: String,
    notes: Option<String>,
    created_at: String,
}

impl TryFrom<ShariahScreeningProfileRow> for ShariahScreeningProfile {
    type Error = Error;

    fn try_from(row: ShariahScreeningProfileRow) -> Result<Self> {
        Ok(Self {
            id: row.id,
            name: row.name,
            debt_threshold: parse_decimal("debt_threshold", &row.debt_threshold)?,
            liquid_assets_threshold: parse_decimal(
                "liquid_assets_threshold",
                &row.liquid_assets_threshold,
            )?,
            impure_income_threshold: parse_decimal(
                "impure_income_threshold",
                &row.impure_income_threshold,
            )?,
            is_default: row.is_default == 1,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

impl TryFrom<AssetShariahScreeningRow> for AssetShariahScreening {
    type Error = Error;

    fn try_from(row: AssetShariahScreeningRow) -> Result<Self> {
        Ok(Self {
            id: row.id,
            asset_id: row.asset_id,
            profile_id: row.profile_id,
            status: ShariahScreeningStatus::parse(&row.status)
                .ok_or_else(|| invalid("unsupported Shariah screening status"))?,
            debt_ratio: parse_optional_decimal("debt_ratio", row.debt_ratio)?,
            liquid_assets_ratio: parse_optional_decimal(
                "liquid_assets_ratio",
                row.liquid_assets_ratio,
            )?,
            impure_income_ratio: parse_optional_decimal(
                "impure_income_ratio",
                row.impure_income_ratio,
            )?,
            source_citation_id: row.source_citation_id,
            manual_override_reason: row.manual_override_reason,
            reviewed_at: row.reviewed_at,
            notes: row.notes,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

impl TryFrom<ShariahScreeningAuditRow> for ShariahScreeningAuditEntry {
    type Error = Error;

    fn try_from(row: ShariahScreeningAuditRow) -> Result<Self> {
        Ok(Self {
            id: row.id,
            screening_id: row.screening_id,
            asset_id: row.asset_id,
            profile_id: row.profile_id,
            previous_status: row
                .previous_status
                .as_deref()
                .map(|status| {
                    ShariahScreeningStatus::parse(status)
                        .ok_or_else(|| invalid("unsupported previous Shariah screening status"))
                })
                .transpose()?,
            new_status: ShariahScreeningStatus::parse(&row.new_status)
                .ok_or_else(|| invalid("unsupported new Shariah screening status"))?,
            notes: row.notes,
            created_at: row.created_at,
        })
    }
}

#[async_trait]
impl ShariahScreeningRepositoryTrait for ShariahScreeningRepository {
    fn list_profiles(&self) -> Result<Vec<ShariahScreeningProfile>> {
        let mut conn = get_connection(&self.pool)?;
        let rows = shariah_screening_profiles::table
            .order(shariah_screening_profiles::is_default.desc())
            .then_order_by(shariah_screening_profiles::name.asc())
            .load::<ShariahScreeningProfileRow>(&mut conn)
            .map_err(StorageError::from)?;

        rows.into_iter()
            .map(ShariahScreeningProfile::try_from)
            .collect()
    }

    fn get_default_profile(&self) -> Result<ShariahScreeningProfile> {
        self.get_profile_by_default(true)
    }

    fn get_profile(&self, profile_id: &str) -> Result<ShariahScreeningProfile> {
        let mut conn = get_connection(&self.pool)?;
        let row = shariah_screening_profiles::table
            .filter(shariah_screening_profiles::id.eq(profile_id))
            .first::<ShariahScreeningProfileRow>(&mut conn)
            .map_err(StorageError::from)?;
        ShariahScreeningProfile::try_from(row)
    }

    fn get_asset_screening(&self, asset_id: &str) -> Result<Option<AssetShariahScreening>> {
        let mut conn = get_connection(&self.pool)?;
        let row = asset_shariah_screening::table
            .filter(asset_shariah_screening::asset_id.eq(asset_id))
            .order(asset_shariah_screening::updated_at.desc())
            .first::<AssetShariahScreeningRow>(&mut conn)
            .optional()
            .map_err(StorageError::from)?;

        row.map(AssetShariahScreening::try_from).transpose()
    }

    fn get_asset_screening_for_profile(
        &self,
        asset_id: &str,
        profile_id: &str,
    ) -> Result<Option<AssetShariahScreening>> {
        let mut conn = get_connection(&self.pool)?;
        let row = asset_shariah_screening::table
            .filter(asset_shariah_screening::asset_id.eq(asset_id))
            .filter(asset_shariah_screening::profile_id.eq(profile_id))
            .first::<AssetShariahScreeningRow>(&mut conn)
            .optional()
            .map_err(StorageError::from)?;

        row.map(AssetShariahScreening::try_from).transpose()
    }

    async fn upsert_asset_screening(
        &self,
        request: UpsertAssetShariahScreeningRequest,
    ) -> Result<AssetShariahScreening> {
        let profile = self.get_profile(&request.profile_id)?;
        let evaluation = evaluate_screening_request(&profile, &request)?;
        let now = Utc::now().to_rfc3339();
        let asset_id = request.asset_id.clone();
        let profile_id = request.profile_id.clone();
        let status = evaluation.status.as_str().to_string();

        let row = AssetShariahScreeningRow {
            id: Uuid::new_v4().to_string(),
            asset_id: request.asset_id,
            profile_id: request.profile_id,
            status,
            debt_ratio: request
                .ratios
                .debt_ratio
                .map(|value| value.normalize().to_string()),
            liquid_assets_ratio: request
                .ratios
                .liquid_assets_ratio
                .map(|value| value.normalize().to_string()),
            impure_income_ratio: request
                .ratios
                .impure_income_ratio
                .map(|value| value.normalize().to_string()),
            source_citation_id: request.source_citation_id,
            manual_override_reason: request.manual_override_reason,
            reviewed_at: Some(now.clone()),
            created_at: now.clone(),
            updated_at: now.clone(),
            notes: request.notes,
        };

        let saved_row = self
            .writer
            .exec_tx(move |tx| -> Result<AssetShariahScreeningRow> {
                let conn = tx.conn();
                let existing = asset_shariah_screening::table
                    .filter(asset_shariah_screening::asset_id.eq(&asset_id))
                    .filter(asset_shariah_screening::profile_id.eq(&profile_id))
                    .first::<AssetShariahScreeningRow>(conn)
                    .optional()
                    .map_err(StorageError::from)?;

                let mut write_row = row.clone();
                if let Some(existing_row) = &existing {
                    write_row.id.clone_from(&existing_row.id);
                    write_row.created_at.clone_from(&existing_row.created_at);
                }

                diesel::insert_into(asset_shariah_screening::table)
                    .values(&write_row)
                    .on_conflict((
                        asset_shariah_screening::asset_id,
                        asset_shariah_screening::profile_id,
                    ))
                    .do_update()
                    .set((
                        asset_shariah_screening::status.eq(&write_row.status),
                        asset_shariah_screening::debt_ratio.eq(&write_row.debt_ratio),
                        asset_shariah_screening::liquid_assets_ratio
                            .eq(&write_row.liquid_assets_ratio),
                        asset_shariah_screening::impure_income_ratio
                            .eq(&write_row.impure_income_ratio),
                        asset_shariah_screening::source_citation_id
                            .eq(&write_row.source_citation_id),
                        asset_shariah_screening::manual_override_reason
                            .eq(&write_row.manual_override_reason),
                        asset_shariah_screening::reviewed_at.eq(&write_row.reviewed_at),
                        asset_shariah_screening::updated_at.eq(&write_row.updated_at),
                        asset_shariah_screening::notes.eq(&write_row.notes),
                    ))
                    .execute(conn)
                    .map_err(StorageError::from)?;

                let audit_row = ShariahScreeningAuditRow {
                    id: Uuid::new_v4().to_string(),
                    screening_id: write_row.id.clone(),
                    asset_id: write_row.asset_id.clone(),
                    profile_id: write_row.profile_id.clone(),
                    previous_status: existing.map(|existing_row| existing_row.status),
                    new_status: write_row.status.clone(),
                    notes: write_row.notes.clone(),
                    created_at: now,
                };

                diesel::insert_into(shariah_screening_audit_log::table)
                    .values(&audit_row)
                    .execute(conn)
                    .map_err(StorageError::from)?;

                Ok(write_row)
            })
            .await?;

        AssetShariahScreening::try_from(saved_row)
    }

    fn list_screening_audit(
        &self,
        asset_id: &str,
        profile_id: &str,
    ) -> Result<Vec<ShariahScreeningAuditEntry>> {
        let mut conn = get_connection(&self.pool)?;
        let rows = shariah_screening_audit_log::table
            .filter(shariah_screening_audit_log::asset_id.eq(asset_id))
            .filter(shariah_screening_audit_log::profile_id.eq(profile_id))
            .order(shariah_screening_audit_log::created_at.desc())
            .load::<ShariahScreeningAuditRow>(&mut conn)
            .map_err(StorageError::from)?;

        rows.into_iter()
            .map(ShariahScreeningAuditEntry::try_from)
            .collect()
    }
}

impl ShariahScreeningRepository {
    fn get_profile_by_default(&self, is_default: bool) -> Result<ShariahScreeningProfile> {
        let mut conn = get_connection(&self.pool)?;
        let row = shariah_screening_profiles::table
            .filter(shariah_screening_profiles::is_default.eq(if is_default { 1 } else { 0 }))
            .first::<ShariahScreeningProfileRow>(&mut conn)
            .map_err(StorageError::from)?;
        ShariahScreeningProfile::try_from(row)
    }
}

fn parse_decimal(field: &str, value: &str) -> Result<Decimal> {
    Decimal::from_str(value).map_err(|_| invalid(&format!("{field} is not a valid decimal")))
}

fn parse_optional_decimal(field: &str, value: Option<String>) -> Result<Option<Decimal>> {
    value
        .as_deref()
        .map(|raw| parse_decimal(field, raw))
        .transpose()
}

fn invalid(message: &str) -> Error {
    Error::from(ValidationError::InvalidInput(message.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{create_pool, init, run_migrations};
    use crate::schema::assets;
    use diesel::RunQueryDsl;
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
        let writer =
            crate::db::write_actor::spawn_writer(pool.as_ref().clone()).expect("spawn writer");
        (pool, writer)
    }

    #[tokio::test]
    async fn migrations_seed_default_profile_and_disabled_setting() {
        let (pool, writer) = setup();
        let repo = ShariahScreeningRepository::new(pool.clone(), writer);
        let profile = repo.get_default_profile().expect("default profile");

        assert_eq!(profile.debt_threshold, dec!(0.30));
        assert_eq!(profile.liquid_assets_threshold, dec!(0.30));
        assert_eq!(profile.impure_income_threshold, dec!(0.05));
        assert!(profile.is_default);

        let mut conn = get_connection(&pool).expect("conn");
        let enabled: String = crate::schema::app_settings::table
            .find("shariah_mode_enabled")
            .select(crate::schema::app_settings::setting_value)
            .first(&mut conn)
            .expect("setting row");
        assert_eq!(enabled, "false");
    }

    #[tokio::test]
    async fn missing_asset_screening_returns_none() {
        let (pool, writer) = setup();
        let repo = ShariahScreeningRepository::new(pool, writer);
        assert!(repo
            .get_asset_screening("asset-with-no-screening")
            .expect("screening lookup")
            .is_none());
    }

    #[tokio::test]
    async fn upsert_screening_writes_result_and_audit_atomically() {
        let (pool, writer) = setup();
        seed_asset(&pool, "asset-1");
        let repo = ShariahScreeningRepository::new(pool, writer);

        let saved = repo
            .upsert_asset_screening(request("asset-1", dec!(0.10), dec!(0.10), dec!(0.01)))
            .await
            .expect("upsert screening");

        assert_eq!(saved.status, ShariahScreeningStatus::Compliant);
        assert_eq!(
            saved.notes.as_deref(),
            Some("Reviewed from user-entered ratios")
        );

        let audit = repo
            .list_screening_audit("asset-1", &saved.profile_id)
            .expect("audit");
        assert_eq!(audit.len(), 1);
        assert_eq!(audit[0].previous_status, None);
        assert_eq!(audit[0].new_status, ShariahScreeningStatus::Compliant);
    }

    #[tokio::test]
    async fn manual_override_without_reason_is_rejected_before_write() {
        let (pool, writer) = setup();
        seed_asset(&pool, "asset-1");
        let repo = ShariahScreeningRepository::new(pool.clone(), writer);
        let mut request = request("asset-1", dec!(0.10), dec!(0.10), dec!(0.01));
        request.manual_override_status = Some(ShariahScreeningStatus::NeedsReview);

        assert!(repo.upsert_asset_screening(request).await.is_err());
        let audit = repo
            .list_screening_audit(
                "asset-1",
                mizan_core::islamic_mode::DEFAULT_SHARIAH_PROFILE_ID,
            )
            .expect("audit lookup");
        assert!(audit.is_empty());
    }

    fn request(
        asset_id: &str,
        debt_ratio: Decimal,
        liquid_assets_ratio: Decimal,
        impure_income_ratio: Decimal,
    ) -> UpsertAssetShariahScreeningRequest {
        UpsertAssetShariahScreeningRequest {
            asset_id: asset_id.to_string(),
            profile_id: mizan_core::islamic_mode::DEFAULT_SHARIAH_PROFILE_ID.to_string(),
            ratios: mizan_core::islamic_mode::ShariahScreeningRatios {
                debt_ratio: Some(debt_ratio),
                liquid_assets_ratio: Some(liquid_assets_ratio),
                impure_income_ratio: Some(impure_income_ratio),
            },
            source_citation_id: None,
            notes: Some("Reviewed from user-entered ratios".to_string()),
            manual_override_status: None,
            manual_override_reason: None,
        }
    }

    fn seed_asset(pool: &Arc<Pool<r2d2::ConnectionManager<SqliteConnection>>>, asset_id: &str) {
        let mut conn = get_connection(pool).expect("conn");
        diesel::insert_into(assets::table)
            .values((
                assets::id.eq(asset_id),
                assets::kind.eq("INVESTMENT"),
                assets::name.eq(Some("Apple Inc.")),
                assets::display_code.eq(Some("AAPL")),
                assets::is_active.eq(1),
                assets::quote_mode.eq("MANUAL"),
                assets::quote_ccy.eq("USD"),
                assets::instrument_type.eq(Some("EQUITY")),
                assets::instrument_symbol.eq(Some("AAPL")),
                assets::created_at.eq("2026-05-15T00:00:00Z"),
                assets::updated_at.eq("2026-05-15T00:00:00Z"),
            ))
            .execute(&mut conn)
            .expect("seed asset");
    }
}
