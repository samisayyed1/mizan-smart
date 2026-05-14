//! Wealth Inbox normalized view model.
//!
//! The inbox is an aggregation layer over real, deterministic sources:
//! persisted smart alerts and stale manual valuation rows. Future prompt
//! sources can map into the same [`InboxItem`] shape without changing the UI.

use chrono::{DateTime, Duration, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

use crate::alerts::{AlertCategory, AlertSeverity, AlertStatus, SmartAlert};
use crate::universal_assets::{ManualValuationAsset, ManualValuationStaleness};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InboxItemType {
    Alert,
    Document,
    Valuation,
    Tax,
    Income,
    PrivateInvestment,
    Security,
    AiSuggestion,
    WebEvidence,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum InboxSeverity {
    Info,
    Warning,
    Critical,
}

impl InboxSeverity {
    pub fn rank(self) -> u8 {
        match self {
            Self::Critical => 0,
            Self::Warning => 1,
            Self::Info => 2,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum InboxStatus {
    Active,
    Snoozed,
    Dismissed,
    Resolved,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InboxItem {
    pub id: String,
    pub item_type: InboxItemType,
    pub title: String,
    pub description: String,
    pub severity: InboxSeverity,
    pub due_date: Option<NaiveDate>,
    pub source_entity_type: String,
    pub source_entity_id: String,
    pub action_route: String,
    pub status: InboxStatus,
    pub created_at: DateTime<Utc>,
}

pub fn build_wealth_inbox(
    alerts: Vec<SmartAlert>,
    manual_valuations: Vec<ManualValuationAsset>,
    now: DateTime<Utc>,
) -> Vec<InboxItem> {
    let mut items: Vec<InboxItem> = alerts
        .into_iter()
        .filter(|alert| alert.status == AlertStatus::Active)
        .map(alert_to_inbox_item)
        .collect();

    items.extend(
        manual_valuations
            .into_iter()
            .filter_map(|row| stale_valuation_to_inbox_item(row, now)),
    );

    sort_critical_first(&mut items);
    items
}

pub fn sort_critical_first(items: &mut [InboxItem]) {
    items.sort_by(|a, b| {
        a.severity
            .rank()
            .cmp(&b.severity.rank())
            .then_with(|| compare_due_dates(a.due_date, b.due_date))
            .then_with(|| b.created_at.cmp(&a.created_at))
            .then_with(|| a.id.cmp(&b.id))
    });
}

fn compare_due_dates(a: Option<NaiveDate>, b: Option<NaiveDate>) -> std::cmp::Ordering {
    match (a, b) {
        (Some(left), Some(right)) => left.cmp(&right),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => std::cmp::Ordering::Equal,
    }
}

fn alert_to_inbox_item(alert: SmartAlert) -> InboxItem {
    let item_type = alert_category_to_type(&alert.category);
    let source_entity_type = alert
        .source_entity_type
        .clone()
        .unwrap_or_else(|| "alert".to_string());
    let source_entity_id = alert
        .source_entity_id
        .clone()
        .unwrap_or_else(|| alert.id.clone());
    let action_route = alert
        .action_route
        .clone()
        .unwrap_or_else(|| default_action_route(&source_entity_type, &source_entity_id));

    InboxItem {
        id: format!("alert:{}", alert.id),
        item_type,
        title: alert.title,
        description: alert.message,
        severity: alert_severity_to_inbox(alert.severity),
        due_date: None,
        source_entity_type,
        source_entity_id,
        action_route,
        status: alert_status_to_inbox(alert.status),
        created_at: alert.first_seen_at,
    }
}

fn stale_valuation_to_inbox_item(
    row: ManualValuationAsset,
    now: DateTime<Utc>,
) -> Option<InboxItem> {
    let severity = match row.staleness {
        ManualValuationStaleness::Current => return None,
        ManualValuationStaleness::Warning => InboxSeverity::Warning,
        ManualValuationStaleness::Critical => InboxSeverity::Critical,
    };
    let due_date = row
        .valuation_date
        .map(|date| date + Duration::days(45))
        .or_else(|| Some(now.date_naive()));
    let description = match row.staleness {
        ManualValuationStaleness::Warning => "Manual valuation is over 45 days old.",
        ManualValuationStaleness::Critical => "Manual valuation is over 90 days old or missing.",
        ManualValuationStaleness::Current => "Manual valuation is current.",
    };

    Some(InboxItem {
        id: format!("valuation:{}", row.asset_id),
        item_type: InboxItemType::Valuation,
        title: format!("Update value for {}", row.name),
        description: description.to_string(),
        severity,
        due_date,
        source_entity_type: "asset".to_string(),
        source_entity_id: row.asset_id,
        action_route: "/holdings/update-values".to_string(),
        status: InboxStatus::Active,
        created_at: now,
    })
}

fn alert_category_to_type(category: &AlertCategory) -> InboxItemType {
    match category.0.as_str() {
        AlertCategory::DOCUMENTS => InboxItemType::Document,
        AlertCategory::VALUATIONS => InboxItemType::Valuation,
        AlertCategory::PRIVATE_INVESTMENTS => InboxItemType::PrivateInvestment,
        AlertCategory::FIXED_INCOME => InboxItemType::Income,
        "tax" => InboxItemType::Tax,
        "ai_suggestions" => InboxItemType::AiSuggestion,
        "web_evidence" => InboxItemType::WebEvidence,
        AlertCategory::FX
        | AlertCategory::MARKET_DATA
        | AlertCategory::CLASSIFICATION
        | AlertCategory::CONCENTRATION => InboxItemType::Security,
        _ => InboxItemType::Alert,
    }
}

fn alert_severity_to_inbox(severity: AlertSeverity) -> InboxSeverity {
    match severity {
        AlertSeverity::Info => InboxSeverity::Info,
        AlertSeverity::Warning => InboxSeverity::Warning,
        AlertSeverity::Critical => InboxSeverity::Critical,
    }
}

fn alert_status_to_inbox(status: AlertStatus) -> InboxStatus {
    match status {
        AlertStatus::Active => InboxStatus::Active,
        AlertStatus::Snoozed => InboxStatus::Snoozed,
        AlertStatus::Dismissed => InboxStatus::Dismissed,
        AlertStatus::Resolved => InboxStatus::Resolved,
    }
}

fn default_action_route(source_entity_type: &str, source_entity_id: &str) -> String {
    match source_entity_type {
        "asset" => format!("/holdings/{source_entity_id}"),
        "document" => "/documents".to_string(),
        "health" => "/health".to_string(),
        _ => "/inbox".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::alerts::{AlertCategory, AlertSeverity, AlertStatus, SmartAlert};
    use crate::universal_assets::{AssetClassification, ManualValuationAsset};

    fn now() -> DateTime<Utc> {
        DateTime::parse_from_rfc3339("2026-05-14T12:00:00Z")
            .unwrap()
            .with_timezone(&Utc)
    }

    fn alert(id: &str, severity: AlertSeverity, category: AlertCategory) -> SmartAlert {
        SmartAlert {
            id: id.to_string(),
            fingerprint: format!("fp:{id}"),
            rule_name: "TestRule".to_string(),
            category,
            severity,
            title: format!("Alert {id}"),
            message: "Real alert message".to_string(),
            status: AlertStatus::Active,
            source_entity_type: Some("asset".to_string()),
            source_entity_id: Some(format!("asset-{id}")),
            action_route: Some(format!("/holdings/asset-{id}")),
            first_seen_at: now(),
            last_seen_at: now(),
            snoozed_until: None,
            dismissed_at: None,
            resolved_at: None,
            metadata_json: None,
        }
    }

    fn manual_asset(id: &str, staleness: ManualValuationStaleness) -> ManualValuationAsset {
        ManualValuationAsset {
            asset_id: id.to_string(),
            name: format!("Asset {id}"),
            classification: AssetClassification::RealEstate,
            current_value: Some("1000".to_string()),
            valuation_date: Some(NaiveDate::from_ymd_opt(2026, 2, 1).unwrap()),
            currency: "USD".to_string(),
            notes: None,
            staleness,
            history_count: 1,
        }
    }

    #[test]
    fn active_alerts_become_inbox_items() {
        let items = build_wealth_inbox(
            vec![alert(
                "1",
                AlertSeverity::Warning,
                AlertCategory::market_data(),
            )],
            vec![],
            now(),
        );
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].id, "alert:1");
        assert_eq!(items[0].item_type, InboxItemType::Security);
        assert_eq!(items[0].action_route, "/holdings/asset-1");
    }

    #[test]
    fn stale_manual_valuations_become_real_tasks() {
        let items = build_wealth_inbox(
            vec![],
            vec![
                manual_asset("fresh", ManualValuationStaleness::Current),
                manual_asset("stale", ManualValuationStaleness::Critical),
            ],
            now(),
        );
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].id, "valuation:stale");
        assert_eq!(items[0].item_type, InboxItemType::Valuation);
        assert_eq!(items[0].severity, InboxSeverity::Critical);
    }

    #[test]
    fn sort_orders_critical_due_then_newest() {
        let older = DateTime::parse_from_rfc3339("2026-05-01T00:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let newer = DateTime::parse_from_rfc3339("2026-05-02T00:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let mut items = vec![
            InboxItem {
                id: "warning".to_string(),
                item_type: InboxItemType::Alert,
                title: "Warning".to_string(),
                description: "Warning".to_string(),
                severity: InboxSeverity::Warning,
                due_date: None,
                source_entity_type: "alert".to_string(),
                source_entity_id: "warning".to_string(),
                action_route: "/inbox".to_string(),
                status: InboxStatus::Active,
                created_at: newer,
            },
            InboxItem {
                id: "critical-later".to_string(),
                item_type: InboxItemType::Alert,
                title: "Critical later".to_string(),
                description: "Critical later".to_string(),
                severity: InboxSeverity::Critical,
                due_date: Some(NaiveDate::from_ymd_opt(2026, 5, 10).unwrap()),
                source_entity_type: "alert".to_string(),
                source_entity_id: "critical-later".to_string(),
                action_route: "/inbox".to_string(),
                status: InboxStatus::Active,
                created_at: newer,
            },
            InboxItem {
                id: "critical-sooner".to_string(),
                item_type: InboxItemType::Alert,
                title: "Critical sooner".to_string(),
                description: "Critical sooner".to_string(),
                severity: InboxSeverity::Critical,
                due_date: Some(NaiveDate::from_ymd_opt(2026, 5, 1).unwrap()),
                source_entity_type: "alert".to_string(),
                source_entity_id: "critical-sooner".to_string(),
                action_route: "/inbox".to_string(),
                status: InboxStatus::Active,
                created_at: older,
            },
        ];
        sort_critical_first(&mut items);
        assert_eq!(
            items
                .iter()
                .map(|item| item.id.as_str())
                .collect::<Vec<_>>(),
            vec!["critical-sooner", "critical-later", "warning"]
        );
    }
}
