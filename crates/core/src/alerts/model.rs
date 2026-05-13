//! Smart Alerts domain types.
//!
//! These types are persisted by `crates/storage-sqlite` and consumed by
//! the rule engine, Tauri/Axum commands, and the frontend.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Severity of a smart alert. Mirrors the SQLite CHECK constraint in
/// `2026-05-14-000001_smart_alerts/up.sql`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

impl AlertSeverity {
    /// Lower rank = higher priority. Used for sorting.
    pub fn rank(self) -> u8 {
        match self {
            AlertSeverity::Critical => 0,
            AlertSeverity::Warning => 1,
            AlertSeverity::Info => 2,
        }
    }

    /// String form used in the database and on the wire.
    pub fn as_str(self) -> &'static str {
        match self {
            AlertSeverity::Info => "info",
            AlertSeverity::Warning => "warning",
            AlertSeverity::Critical => "critical",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "info" => Some(AlertSeverity::Info),
            "warning" => Some(AlertSeverity::Warning),
            "critical" => Some(AlertSeverity::Critical),
            _ => None,
        }
    }
}

impl fmt::Display for AlertSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Lifecycle status of an alert.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AlertStatus {
    Active,
    Snoozed,
    Dismissed,
    Resolved,
}

impl AlertStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            AlertStatus::Active => "active",
            AlertStatus::Snoozed => "snoozed",
            AlertStatus::Dismissed => "dismissed",
            AlertStatus::Resolved => "resolved",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "active" => Some(AlertStatus::Active),
            "snoozed" => Some(AlertStatus::Snoozed),
            "dismissed" => Some(AlertStatus::Dismissed),
            "resolved" => Some(AlertStatus::Resolved),
            _ => None,
        }
    }
}

impl fmt::Display for AlertStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// High-level category of an alert. Used for filtering in the Inbox.
/// Stored as a free-form text column so future rules can extend without
/// a schema migration.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AlertCategory(pub String);

impl AlertCategory {
    pub const VALUATIONS: &'static str = "valuations";
    pub const MARKET_DATA: &'static str = "market_data";
    pub const FX: &'static str = "fx";
    pub const CLASSIFICATION: &'static str = "classification";
    pub const CONCENTRATION: &'static str = "concentration";
    pub const DOCUMENTS: &'static str = "documents";
    pub const PRIVATE_INVESTMENTS: &'static str = "private_investments";
    pub const FIXED_INCOME: &'static str = "fixed_income";

    pub fn valuations() -> Self {
        Self(Self::VALUATIONS.to_string())
    }

    pub fn market_data() -> Self {
        Self(Self::MARKET_DATA.to_string())
    }
}

impl fmt::Display for AlertCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// A new alert proposed by a rule. The engine reconciles proposals
/// against existing rows using `fingerprint` so re-running rules is
/// idempotent.
///
/// `metadata_json` carries rule-specific context (e.g. the actual
/// staleness in days) so the Inbox can render context without the
/// engine needing to re-query.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProposedAlert {
    pub fingerprint: String,
    pub rule_name: String,
    pub category: AlertCategory,
    pub severity: AlertSeverity,
    pub title: String,
    pub message: String,
    pub source_entity_type: Option<String>,
    pub source_entity_id: Option<String>,
    pub action_route: Option<String>,
    pub metadata_json: Option<String>,
}

/// A persisted smart alert row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SmartAlert {
    pub id: String,
    pub fingerprint: String,
    pub rule_name: String,
    pub category: AlertCategory,
    pub severity: AlertSeverity,
    pub title: String,
    pub message: String,
    pub status: AlertStatus,
    pub source_entity_type: Option<String>,
    pub source_entity_id: Option<String>,
    pub action_route: Option<String>,
    pub first_seen_at: DateTime<Utc>,
    pub last_seen_at: DateTime<Utc>,
    pub snoozed_until: Option<DateTime<Utc>>,
    pub dismissed_at: Option<DateTime<Utc>>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub metadata_json: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn severity_rank_orders_critical_first() {
        assert!(AlertSeverity::Critical.rank() < AlertSeverity::Warning.rank());
        assert!(AlertSeverity::Warning.rank() < AlertSeverity::Info.rank());
    }

    #[test]
    fn severity_round_trips_through_string() {
        for severity in [
            AlertSeverity::Info,
            AlertSeverity::Warning,
            AlertSeverity::Critical,
        ] {
            assert_eq!(AlertSeverity::parse(severity.as_str()), Some(severity));
        }
    }

    #[test]
    fn status_round_trips_through_string() {
        for status in [
            AlertStatus::Active,
            AlertStatus::Snoozed,
            AlertStatus::Dismissed,
            AlertStatus::Resolved,
        ] {
            assert_eq!(AlertStatus::parse(status.as_str()), Some(status));
        }
    }

    #[test]
    fn severity_rejects_unknown_strings() {
        assert_eq!(AlertSeverity::parse(""), None);
        assert_eq!(AlertSeverity::parse("WARNING"), None);
        assert_eq!(AlertSeverity::parse("urgent"), None);
    }
}
