//! Data Quality Score computation.
//!
//! Pure function with no IO. Inputs are gathered by Tauri/Axum command
//! handlers from the existing portfolio, market-data, FX, and
//! taxonomy services and passed in via [`DataQualityInput`]. The
//! function is deterministic and side-effect free, making it trivial
//! to verify with fixtures.

use super::model::{
    AssetClassification, AssetValuationFreshness, DataQualityConfig, DataQualityInput,
    DataQualityScore, DataQualityStatus, Deduction, DeductionCategory, DeductionSeverity,
    FxRateAvailability, MarketQuoteFreshness,
};

const MAX_SCORE: u32 = 100;

/// Compute the score from a snapshot of the portfolio.
///
/// An empty portfolio (no assets in any dimension) returns
/// `status: OnboardingRequired` with `score = 0` and no deductions,
/// per the spec's "no fake score if portfolio empty" rule.
pub fn calculate_data_quality(input: &DataQualityInput) -> DataQualityScore {
    if is_empty_portfolio(input) {
        return DataQualityScore {
            score: 0,
            status: DataQualityStatus::OnboardingRequired,
            deductions: Vec::new(),
        };
    }

    let cfg = &input.config;
    let mut deductions: Vec<Deduction> = Vec::new();

    deduct_for_manual_valuations(&input.manual_valuations, cfg, &mut deductions);
    deduct_for_market_quotes(&input.market_priced_assets, cfg, &mut deductions);
    deduct_for_missing_fx(&input.required_fx_rates, cfg, &mut deductions);
    deduct_for_unclassified_assets(&input.asset_classifications, cfg, &mut deductions);

    let total_deducted = sum_with_category_cap(&deductions, cfg.max_points_per_category);
    let score = MAX_SCORE.saturating_sub(total_deducted);
    let status = DataQualityStatus::from_score(score);

    // Sort deductions: highest severity first, then highest points,
    // then by stable category ordering. UI shows them in this order.
    deductions.sort_by(|a, b| {
        b.severity
            .cmp(&a.severity)
            .then_with(|| b.points.cmp(&a.points))
            .then_with(|| (a.category as u8).cmp(&(b.category as u8)))
    });

    DataQualityScore {
        score,
        status,
        deductions,
    }
}

fn is_empty_portfolio(input: &DataQualityInput) -> bool {
    input.manual_valuations.is_empty()
        && input.market_priced_assets.is_empty()
        && input.required_fx_rates.is_empty()
        && input.asset_classifications.is_empty()
}

fn deduct_for_manual_valuations(
    rows: &[AssetValuationFreshness],
    cfg: &DataQualityConfig,
    out: &mut Vec<Deduction>,
) {
    for row in rows {
        let Some(age) = row.manual_valuation_age_days else {
            // No valuation row at all is a separate, heavier deduction.
            out.push(Deduction {
                category: DeductionCategory::MissingCurrentValuation,
                points: cfg.points_missing_current_valuation,
                severity: DeductionSeverity::High,
                explanation: format!("{} has no recorded valuation yet.", row.asset_name),
                action_route: Some(format!("/holdings/{}", row.asset_id)),
                source_entity_type: Some("asset".into()),
                source_entity_id: Some(row.asset_id.clone()),
            });
            continue;
        };

        if age >= cfg.stale_manual_critical_days {
            out.push(Deduction {
                category: DeductionCategory::StaleManualValuation,
                points: cfg.points_stale_manual_critical,
                severity: DeductionSeverity::High,
                explanation: format!("{} has not been revalued in {} days.", row.asset_name, age),
                action_route: Some(format!("/holdings/{}", row.asset_id)),
                source_entity_type: Some("asset".into()),
                source_entity_id: Some(row.asset_id.clone()),
            });
        } else if age >= cfg.stale_manual_warning_days {
            out.push(Deduction {
                category: DeductionCategory::StaleManualValuation,
                points: cfg.points_stale_manual_warning,
                severity: DeductionSeverity::Medium,
                explanation: format!("{} was last valued {} days ago.", row.asset_name, age),
                action_route: Some(format!("/holdings/{}", row.asset_id)),
                source_entity_type: Some("asset".into()),
                source_entity_id: Some(row.asset_id.clone()),
            });
        }
    }
}

fn deduct_for_market_quotes(
    rows: &[MarketQuoteFreshness],
    cfg: &DataQualityConfig,
    out: &mut Vec<Deduction>,
) {
    for row in rows {
        if row.quote_age_days >= cfg.stale_quote_critical_days {
            out.push(Deduction {
                category: DeductionCategory::StaleMarketQuote,
                points: cfg.points_stale_quote_critical,
                severity: DeductionSeverity::High,
                explanation: format!(
                    "Quote for {} is {} days old.",
                    row.asset_name, row.quote_age_days
                ),
                action_route: Some(format!("/holdings/{}", row.asset_id)),
                source_entity_type: Some("asset".into()),
                source_entity_id: Some(row.asset_id.clone()),
            });
        } else if row.quote_age_days >= cfg.stale_quote_warning_days {
            out.push(Deduction {
                category: DeductionCategory::StaleMarketQuote,
                points: cfg.points_stale_quote_warning,
                severity: DeductionSeverity::Low,
                explanation: format!(
                    "Quote for {} is {} days old.",
                    row.asset_name, row.quote_age_days
                ),
                action_route: Some(format!("/holdings/{}", row.asset_id)),
                source_entity_type: Some("asset".into()),
                source_entity_id: Some(row.asset_id.clone()),
            });
        }
    }
}

fn deduct_for_missing_fx(
    rows: &[FxRateAvailability],
    cfg: &DataQualityConfig,
    out: &mut Vec<Deduction>,
) {
    for row in rows {
        if row.available {
            continue;
        }
        out.push(Deduction {
            category: DeductionCategory::MissingFx,
            points: cfg.points_missing_fx,
            severity: DeductionSeverity::High,
            explanation: format!(
                "Missing FX rate {} → {}; net worth in this currency cannot be computed.",
                row.from_currency, row.to_currency
            ),
            action_route: Some("/settings/general/exchange-rates".into()),
            source_entity_type: None,
            source_entity_id: None,
        });
    }
}

fn deduct_for_unclassified_assets(
    rows: &[AssetClassification],
    cfg: &DataQualityConfig,
    out: &mut Vec<Deduction>,
) {
    for row in rows {
        if row.is_classified {
            continue;
        }
        out.push(Deduction {
            category: DeductionCategory::UnclassifiedAsset,
            points: cfg.points_unclassified_asset,
            severity: DeductionSeverity::Low,
            explanation: format!("{} has no taxonomy classification.", row.asset_name),
            action_route: Some(format!("/holdings/{}", row.asset_id)),
            source_entity_type: Some("asset".into()),
            source_entity_id: Some(row.asset_id.clone()),
        });
    }
}

/// Apply the per-category cap to prevent any single dimension from
/// dominating the score. Totals are computed per category, capped, and
/// then summed.
fn sum_with_category_cap(deductions: &[Deduction], cap: u32) -> u32 {
    let mut by_cat: [u32; 9] = [0; 9];
    for d in deductions {
        let idx = d.category as usize;
        by_cat[idx] = by_cat[idx].saturating_add(d.points);
    }
    by_cat
        .iter()
        .map(|points| (*points).min(cap))
        .sum::<u32>()
        .min(MAX_SCORE)
}

// Ordering for severity: High > Medium > Low.
impl PartialOrd for DeductionSeverity {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for DeductionSeverity {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        fn rank(s: DeductionSeverity) -> u8 {
            match s {
                DeductionSeverity::Low => 0,
                DeductionSeverity::Medium => 1,
                DeductionSeverity::High => 2,
            }
        }
        rank(*self).cmp(&rank(*other))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data_quality::model::DataQualityConfig;

    fn empty_input() -> DataQualityInput {
        DataQualityInput::default()
    }

    fn cfg() -> DataQualityConfig {
        DataQualityConfig::default()
    }

    #[test]
    fn empty_portfolio_returns_onboarding_required_with_zero_score() {
        let result = calculate_data_quality(&empty_input());
        assert_eq!(result.score, 0);
        assert_eq!(result.status, DataQualityStatus::OnboardingRequired);
        assert!(result.deductions.is_empty());
    }

    #[test]
    fn clean_portfolio_scores_100_excellent() {
        let input = DataQualityInput {
            manual_valuations: vec![AssetValuationFreshness {
                asset_id: "a1".into(),
                asset_name: "Primary residence".into(),
                manual_valuation_age_days: Some(7),
            }],
            market_priced_assets: vec![MarketQuoteFreshness {
                asset_id: "a2".into(),
                asset_name: "VTI".into(),
                quote_age_days: 1,
            }],
            required_fx_rates: vec![FxRateAvailability {
                from_currency: "EUR".into(),
                to_currency: "USD".into(),
                available: true,
            }],
            asset_classifications: vec![AssetClassification {
                asset_id: "a2".into(),
                asset_name: "VTI".into(),
                is_classified: true,
            }],
            config: cfg(),
        };
        let result = calculate_data_quality(&input);
        assert_eq!(result.score, 100);
        assert_eq!(result.status, DataQualityStatus::Excellent);
        assert!(result.deductions.is_empty());
    }

    #[test]
    fn missing_fx_deducts_8_points_per_pair() {
        let input = DataQualityInput {
            asset_classifications: vec![AssetClassification {
                asset_id: "a2".into(),
                asset_name: "VTI".into(),
                is_classified: true,
            }],
            required_fx_rates: vec![FxRateAvailability {
                from_currency: "EUR".into(),
                to_currency: "USD".into(),
                available: false,
            }],
            ..Default::default()
        };
        let result = calculate_data_quality(&input);
        assert_eq!(result.score, 100 - 8);
        assert_eq!(result.deductions.len(), 1);
        assert_eq!(result.deductions[0].category, DeductionCategory::MissingFx);
    }

    #[test]
    fn stale_manual_valuation_warning_vs_critical_differ_in_severity() {
        let input = DataQualityInput {
            manual_valuations: vec![
                AssetValuationFreshness {
                    asset_id: "a1".into(),
                    asset_name: "House A".into(),
                    manual_valuation_age_days: Some(60),
                },
                AssetValuationFreshness {
                    asset_id: "a2".into(),
                    asset_name: "House B".into(),
                    manual_valuation_age_days: Some(120),
                },
            ],
            ..Default::default()
        };
        let result = calculate_data_quality(&input);
        // Critical (5) + Warning (2) = 7 points.
        assert_eq!(result.score, 100 - 7);
        let critical = result
            .deductions
            .iter()
            .find(|d| d.severity == DeductionSeverity::High)
            .expect("critical deduction present");
        let warning = result
            .deductions
            .iter()
            .find(|d| d.severity == DeductionSeverity::Medium)
            .expect("warning deduction present");
        assert_eq!(critical.category, DeductionCategory::StaleManualValuation);
        assert_eq!(warning.category, DeductionCategory::StaleManualValuation);
    }

    #[test]
    fn missing_current_valuation_is_heavier_than_stale_warning() {
        let input = DataQualityInput {
            manual_valuations: vec![AssetValuationFreshness {
                asset_id: "a1".into(),
                asset_name: "House A".into(),
                manual_valuation_age_days: None,
            }],
            ..Default::default()
        };
        let result = calculate_data_quality(&input);
        // points_missing_current_valuation default is 5.
        assert_eq!(result.score, 100 - 5);
        assert_eq!(
            result.deductions[0].category,
            DeductionCategory::MissingCurrentValuation
        );
    }

    #[test]
    fn category_cap_prevents_one_dimension_from_dominating() {
        let mut classifications = Vec::with_capacity(50);
        for i in 0..50 {
            classifications.push(AssetClassification {
                asset_id: format!("a{}", i),
                asset_name: format!("Asset {}", i),
                is_classified: false,
            });
        }
        let input = DataQualityInput {
            asset_classifications: classifications,
            ..Default::default()
        };
        let result = calculate_data_quality(&input);
        // 50 × 1 = 50, but the per-category cap (default 20) limits it.
        assert_eq!(result.score, 100 - 20);
        assert_eq!(result.deductions.len(), 50);
    }

    #[test]
    fn multiple_categories_deductions_sum_correctly() {
        let input = DataQualityInput {
            manual_valuations: vec![AssetValuationFreshness {
                asset_id: "a1".into(),
                asset_name: "House".into(),
                manual_valuation_age_days: Some(60),
            }],
            market_priced_assets: vec![MarketQuoteFreshness {
                asset_id: "a2".into(),
                asset_name: "VTI".into(),
                quote_age_days: 14,
            }],
            required_fx_rates: vec![FxRateAvailability {
                from_currency: "EUR".into(),
                to_currency: "USD".into(),
                available: false,
            }],
            asset_classifications: vec![AssetClassification {
                asset_id: "a2".into(),
                asset_name: "VTI".into(),
                is_classified: false,
            }],
            config: cfg(),
        };
        let result = calculate_data_quality(&input);
        // 2 (stale manual warning) + 1 (stale quote warning) + 8 (missing
        // FX) + 1 (unclassified) = 12.
        assert_eq!(result.score, 100 - 12);
        assert_eq!(result.deductions.len(), 4);
    }

    #[test]
    fn deductions_are_sorted_severity_first_then_points() {
        let input = DataQualityInput {
            manual_valuations: vec![AssetValuationFreshness {
                asset_id: "a1".into(),
                asset_name: "House".into(),
                manual_valuation_age_days: Some(120),
            }],
            asset_classifications: vec![AssetClassification {
                asset_id: "a2".into(),
                asset_name: "VTI".into(),
                is_classified: false,
            }],
            ..Default::default()
        };
        let result = calculate_data_quality(&input);
        assert_eq!(result.deductions.len(), 2);
        // High severity (stale manual critical, 5 pts) ranks before
        // Low severity (unclassified asset, 1 pt).
        assert_eq!(result.deductions[0].severity, DeductionSeverity::High);
        assert_eq!(result.deductions[1].severity, DeductionSeverity::Low);
    }

    #[test]
    fn action_routes_point_to_real_pages_or_none_for_portfolio_wide_deductions() {
        let input = DataQualityInput {
            manual_valuations: vec![AssetValuationFreshness {
                asset_id: "a1".into(),
                asset_name: "House".into(),
                manual_valuation_age_days: Some(120),
            }],
            required_fx_rates: vec![FxRateAvailability {
                from_currency: "EUR".into(),
                to_currency: "USD".into(),
                available: false,
            }],
            ..Default::default()
        };
        let result = calculate_data_quality(&input);
        let stale = result
            .deductions
            .iter()
            .find(|d| d.category == DeductionCategory::StaleManualValuation)
            .unwrap();
        assert_eq!(stale.action_route.as_deref(), Some("/holdings/a1"));
        let fx = result
            .deductions
            .iter()
            .find(|d| d.category == DeductionCategory::MissingFx)
            .unwrap();
        assert_eq!(
            fx.action_route.as_deref(),
            Some("/settings/general/exchange-rates")
        );
        assert!(fx.source_entity_type.is_none());
    }
}
