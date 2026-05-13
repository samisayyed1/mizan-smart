//! `StaleManualValuationRule` — Phase 1 / Prompt 8.
//!
//! Flags manual asset valuations whose `valuation_date` is older than
//! the configured warning or critical threshold. Manual valuations
//! cover assets the user prices themselves (property, private holdings,
//! collectibles, etc.) — those are the inputs most likely to drift, so
//! a stale value silently distorting net worth is the highest-value
//! alert we can produce deterministically.
//!
//! Severity is configurable via [`super::super::context::AlertEngineConfig`].
//! Default thresholds (from the spec):
//!   - Warning:   older than 45 days
//!   - Critical:  older than 90 days

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

use crate::alerts::context::AlertContext;
use crate::alerts::fingerprint;
use crate::alerts::model::{AlertCategory, AlertSeverity, ProposedAlert};
use crate::alerts::traits::AlertRule;

/// Minimal projection of a manual valuation row that this rule needs.
/// Stored alongside the rule because the wider `valuations` schema is
/// still in flux (Phase 1 Prompt 4 adds the full typed asset model),
/// and the rule should not depend on those evolving types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManualValuationSnapshot {
    pub asset_id: String,
    pub asset_name: String,
    pub valuation_date: NaiveDate,
}

/// Metadata payload serialised into `smart_alerts.metadata_json` so the
/// UI can render context without re-querying. Kept tiny — additional
/// fields can be appended forwards-compatibly without a migration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct StaleMetadata {
    valuation_date: String,
    age_days: i64,
    threshold_days: u32,
}

pub struct StaleManualValuationRule;

impl StaleManualValuationRule {
    pub const NAME: &'static str = "StaleManualValuation";

    pub fn new() -> Self {
        Self
    }
}

impl Default for StaleManualValuationRule {
    fn default() -> Self {
        Self::new()
    }
}

impl AlertRule for StaleManualValuationRule {
    type Input = Vec<ManualValuationSnapshot>;

    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn evaluate(&self, ctx: &AlertContext, input: &Self::Input) -> Vec<ProposedAlert> {
        let today_utc: DateTime<Utc> = ctx.now;
        let today = today_utc.date_naive();
        let warning_days = ctx.config.stale_manual_valuation_warning_days as i64;
        let critical_days = ctx.config.stale_manual_valuation_critical_days as i64;

        input
            .iter()
            .filter_map(|valuation| {
                let age_days = (today - valuation.valuation_date).num_days();
                // Skip non-stale valuations and any future-dated rows
                // (those are user errors that the data-quality check
                // surfaces separately).
                if age_days < warning_days {
                    return None;
                }

                let severity = if age_days >= critical_days {
                    AlertSeverity::Critical
                } else {
                    AlertSeverity::Warning
                };

                let threshold_days = if severity == AlertSeverity::Critical {
                    critical_days as u32
                } else {
                    warning_days as u32
                };

                let title = match severity {
                    AlertSeverity::Critical => {
                        format!("Valuation is more than {} days old", critical_days)
                    }
                    AlertSeverity::Warning => {
                        format!("Valuation is more than {} days old", warning_days)
                    }
                    AlertSeverity::Info => unreachable!(),
                };

                let message = format!(
                    "{} was last valued on {} ({} days ago). Update it to keep net worth accurate.",
                    valuation.asset_name, valuation.valuation_date, age_days,
                );

                let metadata = StaleMetadata {
                    valuation_date: valuation.valuation_date.to_string(),
                    age_days,
                    threshold_days,
                };
                let metadata_json =
                    serde_json::to_string(&metadata).expect("StaleMetadata serialises");

                Some(ProposedAlert {
                    fingerprint: fingerprint::build(Self::NAME, &["asset", &valuation.asset_id]),
                    rule_name: Self::NAME.to_string(),
                    category: AlertCategory::valuations(),
                    severity,
                    title,
                    message,
                    source_entity_type: Some("asset".to_string()),
                    source_entity_id: Some(valuation.asset_id.clone()),
                    action_route: Some(format!("/holdings/{}", valuation.asset_id)),
                    metadata_json: Some(metadata_json),
                })
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{NaiveDate, TimeZone, Utc};

    fn ctx_at(year: i32, month: u32, day: u32) -> AlertContext {
        AlertContext::new(Utc.with_ymd_and_hms(year, month, day, 12, 0, 0).unwrap())
    }

    fn snapshot(id: &str, name: &str, date: NaiveDate) -> ManualValuationSnapshot {
        ManualValuationSnapshot {
            asset_id: id.to_string(),
            asset_name: name.to_string(),
            valuation_date: date,
        }
    }

    #[test]
    fn fresh_valuation_does_not_alert() {
        let rule = StaleManualValuationRule::new();
        let ctx = ctx_at(2026, 5, 14);
        let input = vec![snapshot(
            "a1",
            "Primary residence",
            NaiveDate::from_ymd_opt(2026, 5, 1).unwrap(),
        )];
        assert!(rule.evaluate(&ctx, &input).is_empty());
    }

    #[test]
    fn valuation_just_past_warning_threshold_emits_warning() {
        let rule = StaleManualValuationRule::new();
        let ctx = ctx_at(2026, 5, 14);
        // 46 days old — past the 45-day warning threshold but well
        // under the 90-day critical threshold.
        let stale_date = NaiveDate::from_ymd_opt(2026, 3, 29).unwrap();
        let input = vec![snapshot("a1", "Primary residence", stale_date)];
        let alerts = rule.evaluate(&ctx, &input);
        assert_eq!(alerts.len(), 1);
        assert_eq!(alerts[0].severity, AlertSeverity::Warning);
        assert_eq!(alerts[0].rule_name, "StaleManualValuation");
        assert_eq!(alerts[0].fingerprint, "StaleManualValuation:asset:a1");
        assert_eq!(alerts[0].source_entity_type.as_deref(), Some("asset"));
        assert_eq!(alerts[0].action_route.as_deref(), Some("/holdings/a1"));
    }

    #[test]
    fn valuation_past_critical_threshold_emits_critical() {
        let rule = StaleManualValuationRule::new();
        let ctx = ctx_at(2026, 5, 14);
        // 100 days old — past the 90-day critical threshold.
        let stale_date = NaiveDate::from_ymd_opt(2026, 2, 3).unwrap();
        let input = vec![snapshot("a2", "Vintage watch", stale_date)];
        let alerts = rule.evaluate(&ctx, &input);
        assert_eq!(alerts.len(), 1);
        assert_eq!(alerts[0].severity, AlertSeverity::Critical);
    }

    #[test]
    fn future_dated_valuation_is_skipped() {
        let rule = StaleManualValuationRule::new();
        let ctx = ctx_at(2026, 5, 14);
        // 30 days in the future — negative age means "not stale".
        let future_date = NaiveDate::from_ymd_opt(2026, 6, 13).unwrap();
        let input = vec![snapshot("a3", "Crystal ball", future_date)];
        assert!(rule.evaluate(&ctx, &input).is_empty());
    }

    #[test]
    fn rule_is_deterministic_for_repeated_runs() {
        let rule = StaleManualValuationRule::new();
        let ctx = ctx_at(2026, 5, 14);
        let stale_date = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
        let input = vec![snapshot("a4", "Apartment", stale_date)];
        let first = rule.evaluate(&ctx, &input);
        let second = rule.evaluate(&ctx, &input);
        assert_eq!(first, second);
    }

    #[test]
    fn fingerprint_separates_distinct_assets() {
        let rule = StaleManualValuationRule::new();
        let ctx = ctx_at(2026, 5, 14);
        let stale_date = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
        let input = vec![
            snapshot("a1", "Apt 1", stale_date),
            snapshot("a2", "Apt 2", stale_date),
        ];
        let alerts = rule.evaluate(&ctx, &input);
        assert_eq!(alerts.len(), 2);
        assert_ne!(alerts[0].fingerprint, alerts[1].fingerprint);
    }

    #[test]
    fn metadata_carries_actual_age_days() {
        let rule = StaleManualValuationRule::new();
        let ctx = ctx_at(2026, 5, 14);
        let stale_date = NaiveDate::from_ymd_opt(2026, 3, 1).unwrap();
        let input = vec![snapshot("a1", "Vault", stale_date)];
        let alerts = rule.evaluate(&ctx, &input);
        assert_eq!(alerts.len(), 1);
        let meta = alerts[0]
            .metadata_json
            .as_ref()
            .expect("metadata is always populated");
        // 74 days from 2026-03-01 to 2026-05-14.
        assert!(meta.contains("\"age_days\":74"));
        assert!(meta.contains("\"valuation_date\":\"2026-03-01\""));
    }
}
