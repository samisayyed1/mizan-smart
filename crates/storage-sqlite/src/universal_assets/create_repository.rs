//! Transactional `create_universal_asset` repository.
//!
//! Implements the senior-friendly Add Asset flow described in
//! `docs/mizan-smart-plan/PLAN.md` Phase 1 / Prompt 5.
//!
//! A single transaction inserts:
//!   1. one `assets` row (legacy `kind` from `to_legacy_kind` so
//!      existing portfolio math keeps working, plus `classification`
//!      so the universal flow can be filtered)
//!   2. one row in the matching typed extension table (if the class
//!      has one — `Cash`, `Crypto`, and `Custom` do not)
//!   3. one row in `valuations` with `source_type = manual` so the
//!      asset has a price on day one
//!
//! All universal-flow assets default to `quote_mode = MANUAL`. The
//! spec is explicit: "If symbol lookup fails, allow manual creation."
//! Users can later link a market symbol through a separate action;
//! this flow never performs network lookups during the create path.

use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel::r2d2::{self, Pool};
use diesel::SqliteConnection;
use std::sync::Arc;
use uuid::Uuid;

use mizan_core::universal_assets::create_request::UniversalAssetCreateRequest;
use mizan_core::universal_assets::AssetClassification;
use mizan_core::Result;

use super::details_models::{
    InsertableBusinessDetails, InsertableCollectibleDetails, InsertableCommodityDetails,
    InsertableFixedIncomeDetails, InsertableInsuranceDetails, InsertableLiabilityDetails,
    InsertablePrivateInvestmentDetails, InsertablePublicMarketDetails, InsertableRealEstateDetails,
};
use crate::db::WriteHandle;
use crate::errors::StorageError;
use crate::schema::{
    asset_business_details, asset_collectible_details, asset_commodity_details,
    asset_fixed_income_details, asset_insurance_details, asset_liability_details,
    asset_private_investment_details, asset_public_market_details, asset_real_estate_details,
    assets, valuations,
};

/// Outcome of a successful universal-asset insert. Returned to the
/// caller so the frontend can immediately route to the asset detail
/// page without an extra round trip.
#[derive(Debug, Clone)]
pub struct UniversalAssetCreated {
    pub asset_id: String,
    pub classification: AssetClassification,
    pub valuation_id: String,
}

pub struct UniversalAssetCreateRepository {
    #[allow(dead_code)] // Reserved for future read-back convenience methods.
    pool: Arc<Pool<r2d2::ConnectionManager<SqliteConnection>>>,
    writer: WriteHandle,
}

impl UniversalAssetCreateRepository {
    pub fn new(
        pool: Arc<Pool<r2d2::ConnectionManager<SqliteConnection>>>,
        writer: WriteHandle,
    ) -> Self {
        Self { pool, writer }
    }

    /// Insert a complete universal asset (base + typed detail +
    /// initial valuation) in a single transaction.
    pub async fn create(
        &self,
        request: UniversalAssetCreateRequest,
    ) -> Result<UniversalAssetCreated> {
        request.validate()?;

        let classification = request.classification();
        let legacy_kind = classification.to_legacy_kind();
        let now = Utc::now();
        let now_rfc = now.to_rfc3339();
        let asset_id = Uuid::new_v4().to_string();
        let valuation_id = Uuid::new_v4().to_string();

        let common = request.common().clone();
        let normalised_currency = common.currency.trim().to_uppercase();
        // The value is serialised normalised so the canonical decimal
        // representation matches what every other read path expects.
        let value_native = common.initial_value.normalize().to_string();
        let initial_value_date = common.initial_value_date.to_string();

        let asset_id_for_tx = asset_id.clone();
        let valuation_id_for_tx = valuation_id.clone();
        let request_for_tx = request.clone();
        let classification_str = classification.as_str().to_string();
        let legacy_kind_str = legacy_kind.as_db_str().to_string();
        let name = common.name.trim().to_string();
        let notes = common.notes.clone();
        let now_for_tx = now;

        self.writer
            .exec_tx(move |tx| -> Result<()> {
                let conn = tx.conn();

                // 1. Base assets row. quote_mode is MANUAL for the
                //    entire universal flow — see module docs.
                diesel::insert_into(assets::table)
                    .values((
                        assets::id.eq(&asset_id_for_tx),
                        assets::kind.eq(&legacy_kind_str),
                        assets::name.eq(Some(&name)),
                        assets::notes.eq(notes.as_ref()),
                        assets::is_active.eq(1),
                        assets::quote_mode.eq("MANUAL"),
                        assets::quote_ccy.eq(&normalised_currency),
                        assets::classification.eq(Some(classification_str.clone())),
                        assets::created_at.eq(&now_rfc),
                        assets::updated_at.eq(&now_rfc),
                    ))
                    .execute(conn)
                    .map_err(StorageError::from)?;

                // 2. Typed extension row.
                insert_detail_row(conn, &asset_id_for_tx, &request_for_tx, now_for_tx)?;

                // 3. Initial valuation.
                diesel::insert_into(valuations::table)
                    .values((
                        valuations::id.eq(&valuation_id_for_tx),
                        valuations::asset_id.eq(&asset_id_for_tx),
                        valuations::valuation_date.eq(&initial_value_date),
                        valuations::value_native.eq(&value_native),
                        valuations::currency.eq(&normalised_currency),
                        valuations::source_type.eq("manual"),
                        valuations::created_at.eq(&now_rfc),
                        valuations::updated_at.eq(&now_rfc),
                    ))
                    .execute(conn)
                    .map_err(StorageError::from)?;

                Ok(())
            })
            .await?;

        Ok(UniversalAssetCreated {
            asset_id,
            classification,
            valuation_id,
        })
    }
}

/// Insert the matching typed-extension-table row. Cash, Crypto, and
/// Custom classifications have no detail table — they are stored as
/// pure base-asset rows. Every other variant lands exactly one row.
fn insert_detail_row(
    conn: &mut SqliteConnection,
    asset_id: &str,
    request: &UniversalAssetCreateRequest,
    now: DateTime<Utc>,
) -> Result<()> {
    use UniversalAssetCreateRequest::*;

    match request {
        PublicEquity {
            sub_class, isin, ..
        } => {
            let row = InsertablePublicMarketDetails::new(
                asset_id.to_string(),
                sub_class.map(|s| s.as_str().to_string()),
                isin.clone(),
                now,
            );
            diesel::insert_into(asset_public_market_details::table)
                .values(&row)
                .execute(conn)
                .map_err(StorageError::from)?;
        }
        Etf { isin, .. } => {
            let row = InsertablePublicMarketDetails::new(
                asset_id.to_string(),
                Some("etf".into()),
                isin.clone(),
                now,
            );
            diesel::insert_into(asset_public_market_details::table)
                .values(&row)
                .execute(conn)
                .map_err(StorageError::from)?;
        }
        MutualFund { isin, .. } => {
            let row = InsertablePublicMarketDetails::new(
                asset_id.to_string(),
                Some("mutual_fund".into()),
                isin.clone(),
                now,
            );
            diesel::insert_into(asset_public_market_details::table)
                .values(&row)
                .execute(conn)
                .map_err(StorageError::from)?;
        }
        FixedIncome {
            instrument_subtype,
            issuer,
            maturity_date,
            common,
        } => {
            let row = InsertableFixedIncomeDetails::new(
                asset_id.to_string(),
                instrument_subtype.as_str().to_string(),
                issuer.clone(),
                Some(common.currency.trim().to_uppercase()),
                maturity_date.map(|d| d.to_string()),
                false,
                now,
            );
            diesel::insert_into(asset_fixed_income_details::table)
                .values(&row)
                .execute(conn)
                .map_err(StorageError::from)?;
        }
        Sukuk {
            issuer,
            maturity_date,
            common,
        } => {
            let row = InsertableFixedIncomeDetails::new(
                asset_id.to_string(),
                "sukuk".into(),
                issuer.clone(),
                Some(common.currency.trim().to_uppercase()),
                maturity_date.map(|d| d.to_string()),
                true, // is_sukuk
                now,
            );
            diesel::insert_into(asset_fixed_income_details::table)
                .values(&row)
                .execute(conn)
                .map_err(StorageError::from)?;
        }
        FixedDeposit {
            issuer,
            maturity_date,
            common,
        } => {
            let row = InsertableFixedIncomeDetails::new(
                asset_id.to_string(),
                "fixed_deposit".into(),
                issuer.clone(),
                Some(common.currency.trim().to_uppercase()),
                maturity_date.map(|d| d.to_string()),
                false,
                now,
            );
            diesel::insert_into(asset_fixed_income_details::table)
                .values(&row)
                .execute(conn)
                .map_err(StorageError::from)?;
        }
        RealEstate {
            property_type,
            address_approximate,
            ..
        } => {
            let row = InsertableRealEstateDetails::new(
                asset_id.to_string(),
                property_type.clone(),
                address_approximate.clone(),
                now,
            );
            diesel::insert_into(asset_real_estate_details::table)
                .values(&row)
                .execute(conn)
                .map_err(StorageError::from)?;
        }
        PrivateEquity { manager, .. } => {
            insert_private(conn, asset_id, "private_equity", manager.clone(), now)?;
        }
        PrivateCredit { manager, .. } => {
            insert_private(conn, asset_id, "private_credit", manager.clone(), now)?;
        }
        HedgeFund { manager, .. } => {
            insert_private(conn, asset_id, "hedge_fund", manager.clone(), now)?;
        }
        VentureCapital { manager, .. } => {
            insert_private(conn, asset_id, "venture_capital", manager.clone(), now)?;
        }
        Insurance { provider, .. } => {
            insert_insurance(conn, asset_id, "insurance", provider.clone(), now)?;
        }
        Ulip { provider, .. } => {
            insert_insurance(conn, asset_id, "ulip", provider.clone(), now)?;
        }
        Pension { provider, .. } => {
            insert_insurance(conn, asset_id, "pension", provider.clone(), now)?;
        }
        Commodity {
            commodity_type,
            weight_value,
            weight_unit,
            purity,
            ..
        } => {
            let row = InsertableCommodityDetails::new(
                asset_id.to_string(),
                commodity_type.as_str().to_string(),
                weight_value.map(|d| d.normalize().to_string()),
                weight_unit.clone(),
                purity.clone(),
                now,
            );
            diesel::insert_into(asset_commodity_details::table)
                .values(&row)
                .execute(conn)
                .map_err(StorageError::from)?;
        }
        Gold {
            weight_value,
            weight_unit,
            purity,
            ..
        } => {
            let row = InsertableCommodityDetails::new(
                asset_id.to_string(),
                "gold".into(),
                weight_value.map(|d| d.normalize().to_string()),
                weight_unit.clone(),
                purity.clone(),
                now,
            );
            diesel::insert_into(asset_commodity_details::table)
                .values(&row)
                .execute(conn)
                .map_err(StorageError::from)?;
        }
        Silver {
            weight_value,
            weight_unit,
            purity,
            ..
        } => {
            let row = InsertableCommodityDetails::new(
                asset_id.to_string(),
                "silver".into(),
                weight_value.map(|d| d.normalize().to_string()),
                weight_unit.clone(),
                purity.clone(),
                now,
            );
            diesel::insert_into(asset_commodity_details::table)
                .values(&row)
                .execute(conn)
                .map_err(StorageError::from)?;
        }
        BusinessOwnership {
            business_name,
            ownership_percent,
            ..
        } => {
            let row = InsertableBusinessDetails::new(
                asset_id.to_string(),
                business_name.clone(),
                ownership_percent.map(|d| d.normalize().to_string()),
                now,
            );
            diesel::insert_into(asset_business_details::table)
                .values(&row)
                .execute(conn)
                .map_err(StorageError::from)?;
        }
        Collectible {
            collectible_type,
            maker,
            ..
        } => {
            let row = InsertableCollectibleDetails::new(
                asset_id.to_string(),
                collectible_type.clone(),
                maker.clone(),
                now,
            );
            diesel::insert_into(asset_collectible_details::table)
                .values(&row)
                .execute(conn)
                .map_err(StorageError::from)?;
        }
        Liability {
            liability_type,
            lender,
            ..
        } => {
            let row = InsertableLiabilityDetails::new(
                asset_id.to_string(),
                liability_type.as_str().to_string(),
                lender.clone(),
                now,
            );
            diesel::insert_into(asset_liability_details::table)
                .values(&row)
                .execute(conn)
                .map_err(StorageError::from)?;
        }
        // No typed extension table for these classes — they live as
        // pure base-asset rows.
        Cash { .. } | Crypto { .. } | Custom { .. } => {}
    }
    Ok(())
}

fn insert_private(
    conn: &mut SqliteConnection,
    asset_id: &str,
    subtype: &str,
    manager: Option<String>,
    now: DateTime<Utc>,
) -> Result<()> {
    let row = InsertablePrivateInvestmentDetails::new(
        asset_id.to_string(),
        subtype.to_string(),
        manager,
        now,
    );
    diesel::insert_into(asset_private_investment_details::table)
        .values(&row)
        .execute(conn)
        .map_err(StorageError::from)?;
    Ok(())
}

fn insert_insurance(
    conn: &mut SqliteConnection,
    asset_id: &str,
    policy_type: &str,
    provider: Option<String>,
    now: DateTime<Utc>,
) -> Result<()> {
    let row = InsertableInsuranceDetails::new(
        asset_id.to_string(),
        policy_type.to_string(),
        provider,
        now,
    );
    diesel::insert_into(asset_insurance_details::table)
        .values(&row)
        .execute(conn)
        .map_err(StorageError::from)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::write_actor::spawn_writer;
    use crate::db::{create_pool, get_connection, init, run_migrations};
    use chrono::NaiveDate;
    use mizan_core::universal_assets::create_request::UniversalAssetCommon;
    use mizan_core::universal_assets::details::{CommodityType, FixedIncomeSubtype, LiabilityType};
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

    fn common(name: &str, value: rust_decimal::Decimal) -> UniversalAssetCommon {
        UniversalAssetCommon {
            name: name.into(),
            currency: "USD".into(),
            notes: None,
            initial_value: value,
            initial_value_date: NaiveDate::from_ymd_opt(2026, 5, 14).unwrap(),
        }
    }

    /// Count rows in an arbitrary table for verification.
    fn count_rows(pool: &Arc<Pool<r2d2::ConnectionManager<SqliteConnection>>>, sql: &str) -> i64 {
        let mut conn = get_connection(pool).expect("conn");
        diesel::sql_query(sql)
            .get_result::<RowCount>(&mut conn)
            .expect("count")
            .n
    }

    #[derive(diesel::QueryableByName, Debug)]
    struct RowCount {
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        n: i64,
    }

    #[tokio::test]
    async fn real_estate_creates_base_typed_and_valuation_rows() {
        let (pool, writer) = setup();
        let repo = UniversalAssetCreateRepository::new(pool.clone(), writer);

        let result = repo
            .create(UniversalAssetCreateRequest::RealEstate {
                common: common("Primary residence", dec!(750_000)),
                property_type: Some("apartment".into()),
                address_approximate: Some("London, UK".into()),
            })
            .await
            .expect("create real estate");
        assert_eq!(result.classification, AssetClassification::RealEstate);

        assert_eq!(count_rows(&pool, "SELECT COUNT(*) AS n FROM assets"), 1);
        assert_eq!(
            count_rows(&pool, "SELECT COUNT(*) AS n FROM asset_real_estate_details"),
            1
        );
        assert_eq!(count_rows(&pool, "SELECT COUNT(*) AS n FROM valuations"), 1);
    }

    #[tokio::test]
    async fn sukuk_marks_is_sukuk_and_fills_fixed_income_table() {
        let (pool, writer) = setup();
        let repo = UniversalAssetCreateRepository::new(pool.clone(), writer);

        repo.create(UniversalAssetCreateRequest::Sukuk {
            common: common("UAE Sovereign Sukuk", dec!(100_000)),
            issuer: Some("Government of UAE".into()),
            maturity_date: NaiveDate::from_ymd_opt(2030, 12, 31),
        })
        .await
        .expect("create sukuk");

        let mut conn = get_connection(&pool).expect("conn");
        let (subtype, is_sukuk): (String, i32) = asset_fixed_income_details::table
            .select((
                asset_fixed_income_details::instrument_subtype,
                asset_fixed_income_details::is_sukuk,
            ))
            .first(&mut conn)
            .expect("fixed income row exists");
        assert_eq!(subtype, "sukuk");
        assert_eq!(is_sukuk, 1);
    }

    #[tokio::test]
    async fn cash_creates_base_and_valuation_but_no_detail_row() {
        let (pool, writer) = setup();
        let repo = UniversalAssetCreateRepository::new(pool.clone(), writer);

        repo.create(UniversalAssetCreateRequest::Cash {
            common: common("Checking account", dec!(5_000)),
        })
        .await
        .expect("create cash");

        assert_eq!(count_rows(&pool, "SELECT COUNT(*) AS n FROM assets"), 1);
        assert_eq!(count_rows(&pool, "SELECT COUNT(*) AS n FROM valuations"), 1);
        // Cash has no typed extension table; verify none of the 9
        // extension tables got populated.
        for table in [
            "asset_public_market_details",
            "asset_fixed_income_details",
            "asset_real_estate_details",
            "asset_private_investment_details",
            "asset_insurance_details",
            "asset_commodity_details",
            "asset_business_details",
            "asset_collectible_details",
            "asset_liability_details",
        ] {
            let sql = format!("SELECT COUNT(*) AS n FROM {}", table);
            assert_eq!(count_rows(&pool, &sql), 0, "{} should be empty", table);
        }
    }

    #[tokio::test]
    async fn create_persists_classification_on_assets_row() {
        let (pool, writer) = setup();
        let repo = UniversalAssetCreateRepository::new(pool.clone(), writer);

        repo.create(UniversalAssetCreateRequest::Gold {
            common: common("Gold bullion 10oz", dec!(20_000)),
            weight_value: Some(dec!(10)),
            weight_unit: Some("oz".into()),
            purity: Some("999".into()),
        })
        .await
        .unwrap();

        let mut conn = get_connection(&pool).expect("conn");
        let row: (Option<String>, String) = assets::table
            .select((assets::classification, assets::kind))
            .first(&mut conn)
            .expect("asset row");
        assert_eq!(row.0.as_deref(), Some("gold"));
        // gold → legacy PreciousMetal so existing portfolio math
        // treats it as an alternative asset rather than an
        // Investment.
        assert_eq!(row.1, "PRECIOUS_METAL");
    }

    #[tokio::test]
    async fn invalid_request_is_rejected_before_any_insert() {
        let (pool, writer) = setup();
        let repo = UniversalAssetCreateRepository::new(pool.clone(), writer);

        let result = repo
            .create(UniversalAssetCreateRequest::Custom {
                common: UniversalAssetCommon {
                    name: "  ".into(), // blank — rejected by validate()
                    currency: "USD".into(),
                    notes: None,
                    initial_value: dec!(0),
                    initial_value_date: NaiveDate::from_ymd_opt(2026, 5, 14).unwrap(),
                },
            })
            .await;
        assert!(result.is_err());
        assert_eq!(count_rows(&pool, "SELECT COUNT(*) AS n FROM assets"), 0);
    }

    #[tokio::test]
    async fn private_equity_writes_subtype_into_extension_table() {
        let (pool, writer) = setup();
        let repo = UniversalAssetCreateRepository::new(pool.clone(), writer);

        repo.create(UniversalAssetCreateRequest::HedgeFund {
            common: common("Macro fund", dec!(500_000)),
            manager: Some("Acme Capital".into()),
        })
        .await
        .unwrap();

        let mut conn = get_connection(&pool).expect("conn");
        let subtype: String = asset_private_investment_details::table
            .select(asset_private_investment_details::instrument_subtype)
            .first(&mut conn)
            .expect("private investment row");
        assert_eq!(subtype, "hedge_fund");
    }

    #[tokio::test]
    async fn liability_persists_liability_type_check_constraint() {
        let (pool, writer) = setup();
        let repo = UniversalAssetCreateRepository::new(pool.clone(), writer);

        repo.create(UniversalAssetCreateRequest::Liability {
            common: common("Home mortgage", dec!(400_000)),
            liability_type: LiabilityType::Mortgage,
            lender: Some("Big Bank".into()),
        })
        .await
        .unwrap();

        let mut conn = get_connection(&pool).expect("conn");
        let row: (String, Option<String>) = asset_liability_details::table
            .select((
                asset_liability_details::liability_type,
                asset_liability_details::lender,
            ))
            .first(&mut conn)
            .expect("liability row");
        assert_eq!(row.0, "mortgage");
        assert_eq!(row.1.as_deref(), Some("Big Bank"));
    }

    #[tokio::test]
    async fn fixed_income_supports_treasury_bill_subtype() {
        let (pool, writer) = setup();
        let repo = UniversalAssetCreateRepository::new(pool.clone(), writer);

        repo.create(UniversalAssetCreateRequest::FixedIncome {
            common: common("US T-Bill 13W", dec!(10_000)),
            instrument_subtype: FixedIncomeSubtype::TreasuryBill,
            issuer: Some("US Treasury".into()),
            maturity_date: NaiveDate::from_ymd_opt(2026, 8, 14),
        })
        .await
        .unwrap();

        let mut conn = get_connection(&pool).expect("conn");
        let (subtype, is_sukuk): (String, i32) = asset_fixed_income_details::table
            .select((
                asset_fixed_income_details::instrument_subtype,
                asset_fixed_income_details::is_sukuk,
            ))
            .first(&mut conn)
            .expect("fi row");
        assert_eq!(subtype, "treasury_bill");
        assert_eq!(is_sukuk, 0);
    }

    #[tokio::test]
    async fn commodity_records_weight_unit_and_purity() {
        let (pool, writer) = setup();
        let repo = UniversalAssetCreateRepository::new(pool.clone(), writer);

        repo.create(UniversalAssetCreateRequest::Commodity {
            common: common("Palladium ingot", dec!(15_000)),
            commodity_type: CommodityType::Palladium,
            weight_value: Some(dec!(5)),
            weight_unit: Some("oz".into()),
            purity: Some("999".into()),
        })
        .await
        .unwrap();

        let mut conn = get_connection(&pool).expect("conn");
        let (ctype, weight, unit, purity): (
            String,
            Option<String>,
            Option<String>,
            Option<String>,
        ) = asset_commodity_details::table
            .select((
                asset_commodity_details::commodity_type,
                asset_commodity_details::weight_value,
                asset_commodity_details::weight_unit,
                asset_commodity_details::purity,
            ))
            .first(&mut conn)
            .expect("commodity row");
        assert_eq!(ctype, "palladium");
        assert_eq!(weight.as_deref(), Some("5"));
        assert_eq!(unit.as_deref(), Some("oz"));
        assert_eq!(purity.as_deref(), Some("999"));
    }
}
