//! Universal asset model — Phase 1 / Prompt 4 of `docs/mizan-smart-plan/PLAN.md`.
//!
//! The "universal" model extends the legacy `Asset` row with:
//!
//!   1. A finer classification ([`AssetClassification`]) covering the 22
//!      asset classes from the spec.
//!   2. Typed extension structs ([`details`]) carrying class-specific
//!      fields (fixed income / real estate / private investment / …),
//!      mapped 1:1 to the new SQLite tables created by migration
//!      `2026-05-14-000002_universal_asset_model`.
//!   3. A canonical [`Valuation`] type — the per-asset, per-date,
//!      per-source value that downstream phases (bulk-update grid,
//!      Explain-This-Number, document extraction, web evidence) all
//!      read and write.
//!
//! This module is intentionally additive: the legacy `Asset` /
//! `AssetKind` / `InstrumentType` types in `super::assets` continue to
//! work unchanged. The classification value is a free-form `String`
//! on the assets row guarded by [`AssetClassification`] at the domain
//! boundary, so legacy callers do not need to migrate.

pub mod classification;
pub mod create_request;
pub mod details;
pub mod valuation;

pub use classification::AssetClassification;
pub use create_request::{UniversalAssetCommon, UniversalAssetCreateRequest};
pub use details::{
    BusinessDetails, CollectibleDetails, CommodityDetails, CommodityType, DayCountConvention,
    FixedIncomeDetails, FixedIncomeSubtype, InsuranceDetails, InsurancePolicyType,
    LiabilityDetails, LiabilityType, PrivateInvestmentDetails, PrivateInvestmentSubtype,
    PublicMarketDetails, PublicMarketSubClass, RealEstateDetails, RealEstateSizeUnit,
};
pub use valuation::{NewValuation, Valuation, ValuationSource};
