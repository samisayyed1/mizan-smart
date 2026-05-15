use diesel::prelude::*;
use diesel::r2d2::{self, Pool};
use diesel::SqliteConnection;
use rust_decimal::Decimal;
use std::str::FromStr;
use std::sync::Arc;

use mizan_core::errors::ValidationError;
use mizan_core::islamic_mode::{
    AssetShariahScreening, ShariahScreeningProfile, ShariahScreeningRepositoryTrait,
    ShariahScreeningStatus,
};
use mizan_core::{Error, Result};

use crate::db::get_connection;
use crate::errors::StorageError;
use crate::schema::{asset_shariah_screening, shariah_screening_profiles};

pub struct ShariahScreeningRepository {
    pool: Arc<Pool<r2d2::ConnectionManager<SqliteConnection>>>,
}

impl ShariahScreeningRepository {
    pub fn new(pool: Arc<Pool<r2d2::ConnectionManager<SqliteConnection>>>) -> Self {
        Self { pool }
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

#[derive(Debug, Clone, Queryable)]
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
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

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
        let mut conn = get_connection(&self.pool)?;
        let row = shariah_screening_profiles::table
            .filter(shariah_screening_profiles::is_default.eq(1))
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
        create_pool(&db_path).expect("create pool")
    }

    #[test]
    fn migrations_seed_default_profile_and_disabled_setting() {
        let pool = setup();
        let repo = ShariahScreeningRepository::new(pool.clone());
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

    #[test]
    fn missing_asset_screening_returns_none() {
        let pool = setup();
        let repo = ShariahScreeningRepository::new(pool);
        assert!(repo
            .get_asset_screening("asset-with-no-screening")
            .expect("screening lookup")
            .is_none());
    }
}
