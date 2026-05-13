//! Data Quality Score.
//!
//! Phase 1 / Prompt 7 of `docs/mizan-smart-plan/PLAN.md`. The score is a
//! single deterministic function from a portfolio snapshot to a 0-100
//! integer plus a list of `Deduction`s the user can act on.
//!
//! No fake numbers: an empty portfolio scores 0 with status
//! `OnboardingRequired` so the UI can show an onboarding state rather
//! than fabricate a "perfect" empty score. A populated portfolio with no
//! issues scores 100 with status `Excellent`.
//!
//! Higher phases of the build plan (Phase 2 document vault, Phase 4
//! reports/tax packs, Phase 5 web evidence) extend the score by feeding
//! additional inputs into the same struct — no new top-level service is
//! required.

pub mod model;
pub mod service;

pub use model::{
    AssetClassification, AssetValuationFreshness, DataQualityInput, DataQualityScore,
    DataQualityStatus, Deduction, DeductionCategory, DeductionSeverity, FxRateAvailability,
    MarketQuoteFreshness,
};
pub use service::calculate_data_quality;
