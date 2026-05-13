//! Domain types for the Data Quality Score.
//!
//! The score is computed by [`super::service::calculate_data_quality`].
//! All inputs are plain data so the function is trivially testable from
//! fixtures; no IO inside the score itself.

use serde::{Deserialize, Serialize};

/// Final score result returned to the UI.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DataQualityScore {
    /// 0..=100 integer score. Lower is worse.
    pub score: u32,
    pub status: DataQualityStatus,
    /// Reasons points were deducted. Empty when score is 100 (or when
    /// the portfolio is empty — in that case `status` carries the
    /// distinction).
    pub deductions: Vec<Deduction>,
}

/// Status band derived from `score`. The thresholds match the spec.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DataQualityStatus {
    /// 90..=100. Premium score; no aggressive UI styling.
    Excellent,
    /// 70..=89. Healthy, a few things to tidy up.
    Good,
    /// 40..=69. Several deductions; user attention needed.
    NeedsAttention,
    /// 0..=39. Many gaps or a critical issue; flag prominently.
    Poor,
    /// No assets at all — the score is meaningless. UI renders the
    /// onboarding empty-state rather than a number.
    OnboardingRequired,
}

impl DataQualityStatus {
    /// Compute the band from a `0..=100` score.
    pub fn from_score(score: u32) -> Self {
        match score {
            90..=100 => DataQualityStatus::Excellent,
            70..=89 => DataQualityStatus::Good,
            40..=69 => DataQualityStatus::NeedsAttention,
            _ => DataQualityStatus::Poor,
        }
    }
}

/// Severity hint for a deduction. Distinct from the alert severity in
/// `crates/core/src/alerts` because data-quality deductions are about
/// the *quality of the data*, not about an event that needs the user's
/// immediate attention.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeductionSeverity {
    Low,
    Medium,
    High,
}

/// High-level category of a deduction. Used for grouping in the UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeductionCategory {
    StaleManualValuation,
    StaleMarketQuote,
    MissingFx,
    UnclassifiedAsset,
    MissingCurrentValuation,
    // Reserved for later phases; the score function ignores variants
    // that have no inputs supplied.
    PendingDocumentReview,
    UncitedReportLine,
    MissingFixedIncomeTerms,
    MissingPrivateFundNav,
}

/// Single deduction explanation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Deduction {
    pub category: DeductionCategory,
    pub points: u32,
    pub severity: DeductionSeverity,
    /// Plain-English copy shown directly to the user.
    pub explanation: String,
    /// Optional route the UI uses for "Fix this" buttons.
    pub action_route: Option<String>,
    /// Optional entity reference. Leave `None` for portfolio-wide
    /// deductions (e.g. missing FX rate between two currencies).
    pub source_entity_type: Option<String>,
    pub source_entity_id: Option<String>,
}

// =============================================================================
// Inputs
// =============================================================================

/// Snapshot of an asset's manual-valuation freshness for one asset.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssetValuationFreshness {
    pub asset_id: String,
    pub asset_name: String,
    /// Days since the manual valuation was last recorded. `None` means
    /// the asset is manually valued but has no valuation row at all
    /// (counts as `MissingCurrentValuation`).
    pub manual_valuation_age_days: Option<i64>,
}

/// Snapshot of how stale a market quote is for one priced asset.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarketQuoteFreshness {
    pub asset_id: String,
    pub asset_name: String,
    /// Calendar days since the last quote. Negative or zero means
    /// "today" (fresh) — the rule does not deduct for those.
    pub quote_age_days: i64,
}

/// Whether the FX rate from one currency to the base currency is
/// available *today*. Stale conversion is handled by the quote
/// freshness check above.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FxRateAvailability {
    pub from_currency: String,
    pub to_currency: String,
    pub available: bool,
}

/// Whether an asset has any taxonomy classification at all.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssetClassification {
    pub asset_id: String,
    pub asset_name: String,
    pub is_classified: bool,
}

/// Inputs to [`super::service::calculate_data_quality`]. Every field is
/// optional in the sense that downstream phases may not have
/// implemented their corresponding source yet (e.g. document vault
/// rows do not exist before Phase 2). An empty `Vec` means "this
/// dimension is fully clean", *not* "this dimension is unknown".
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct DataQualityInput {
    /// Includes every manually-valued asset the portfolio contains.
    /// Used to compute total asset count for the `OnboardingRequired`
    /// short-circuit.
    pub manual_valuations: Vec<AssetValuationFreshness>,
    /// Public-market priced assets.
    pub market_priced_assets: Vec<MarketQuoteFreshness>,
    /// FX pairs needed for net-worth aggregation.
    pub required_fx_rates: Vec<FxRateAvailability>,
    /// One row per asset that should carry a classification.
    pub asset_classifications: Vec<AssetClassification>,
    /// Thresholds — pinned in the input so tests are deterministic and
    /// future configuration plumbing is a one-liner.
    pub config: DataQualityConfig,
}

/// Deduction weights and staleness thresholds. The defaults mirror the
/// spec's "no aggressive red" policy: warnings carry mild deductions,
/// criticals carry larger ones, but no single category can blow the
/// score past `MAX_PER_CATEGORY`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataQualityConfig {
    pub stale_manual_warning_days: i64,
    pub stale_manual_critical_days: i64,
    pub stale_quote_warning_days: i64,
    pub stale_quote_critical_days: i64,

    pub points_stale_manual_warning: u32,
    pub points_stale_manual_critical: u32,
    pub points_stale_quote_warning: u32,
    pub points_stale_quote_critical: u32,
    pub points_missing_fx: u32,
    pub points_unclassified_asset: u32,
    pub points_missing_current_valuation: u32,

    /// Cap on points any single category can deduct. Prevents one
    /// noisy dimension from dominating the score.
    pub max_points_per_category: u32,
}

impl Default for DataQualityConfig {
    fn default() -> Self {
        Self {
            stale_manual_warning_days: 45,
            stale_manual_critical_days: 90,
            stale_quote_warning_days: 7,
            stale_quote_critical_days: 30,

            points_stale_manual_warning: 2,
            points_stale_manual_critical: 5,
            points_stale_quote_warning: 1,
            points_stale_quote_critical: 3,
            points_missing_fx: 8,
            points_unclassified_asset: 1,
            points_missing_current_valuation: 5,

            max_points_per_category: 20,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_thresholds_partition_0_to_100() {
        // Spot-check each band boundary.
        assert_eq!(
            DataQualityStatus::from_score(100),
            DataQualityStatus::Excellent
        );
        assert_eq!(
            DataQualityStatus::from_score(90),
            DataQualityStatus::Excellent
        );
        assert_eq!(DataQualityStatus::from_score(89), DataQualityStatus::Good);
        assert_eq!(DataQualityStatus::from_score(70), DataQualityStatus::Good);
        assert_eq!(
            DataQualityStatus::from_score(69),
            DataQualityStatus::NeedsAttention
        );
        assert_eq!(
            DataQualityStatus::from_score(40),
            DataQualityStatus::NeedsAttention
        );
        assert_eq!(DataQualityStatus::from_score(39), DataQualityStatus::Poor);
        assert_eq!(DataQualityStatus::from_score(0), DataQualityStatus::Poor);
    }
}
