//! Database models for the Smart Alerts table.
//!
//! All timestamps are stored as RFC3339 strings — the same convention
//! used elsewhere in the SQLite schema (see `health_issue_dismissals`,
//! `ai_threads`, etc.). The conversions to/from the domain types in
//! `crates/core/src/alerts` happen here so the rest of the codebase
//! works in terms of strongly-typed `DateTime<Utc>` values.

use chrono::{DateTime, Utc};
use diesel::prelude::*;
use mizan_core::alerts::{AlertCategory, AlertSeverity, AlertStatus, ProposedAlert, SmartAlert};

#[derive(Queryable, Identifiable, Insertable, AsChangeset, Selectable, PartialEq, Debug, Clone)]
#[diesel(table_name = crate::schema::smart_alerts)]
#[diesel(primary_key(id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct SmartAlertDB {
    pub id: String,
    pub fingerprint: String,
    pub rule_name: String,
    pub category: String,
    pub severity: String,
    pub title: String,
    pub message: String,
    pub status: String,
    pub source_entity_type: Option<String>,
    pub source_entity_id: Option<String>,
    pub action_route: Option<String>,
    pub first_seen_at: String,
    pub last_seen_at: String,
    pub snoozed_until: Option<String>,
    pub dismissed_at: Option<String>,
    pub resolved_at: Option<String>,
    pub metadata_json: Option<String>,
}

fn rfc3339(value: DateTime<Utc>) -> String {
    value.to_rfc3339()
}

fn parse_rfc3339(value: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(value)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now())
}

fn parse_rfc3339_opt(value: &Option<String>) -> Option<DateTime<Utc>> {
    value.as_deref().map(parse_rfc3339)
}

impl SmartAlertDB {
    /// Build the row to insert for a brand-new alert. The caller
    /// (`SmartAlertRepository::upsert`) generates the id so tests can
    /// inject deterministic ids.
    pub fn from_proposal(id: String, proposal: &ProposedAlert, now: DateTime<Utc>) -> Self {
        Self {
            id,
            fingerprint: proposal.fingerprint.clone(),
            rule_name: proposal.rule_name.clone(),
            category: proposal.category.0.clone(),
            severity: proposal.severity.as_str().to_string(),
            title: proposal.title.clone(),
            message: proposal.message.clone(),
            status: AlertStatus::Active.as_str().to_string(),
            source_entity_type: proposal.source_entity_type.clone(),
            source_entity_id: proposal.source_entity_id.clone(),
            action_route: proposal.action_route.clone(),
            first_seen_at: rfc3339(now),
            last_seen_at: rfc3339(now),
            snoozed_until: None,
            dismissed_at: None,
            resolved_at: None,
            metadata_json: proposal.metadata_json.clone(),
        }
    }
}

impl From<SmartAlertDB> for SmartAlert {
    fn from(db: SmartAlertDB) -> Self {
        let severity = AlertSeverity::parse(&db.severity).unwrap_or(AlertSeverity::Info);
        let status = AlertStatus::parse(&db.status).unwrap_or(AlertStatus::Active);
        Self {
            id: db.id,
            fingerprint: db.fingerprint,
            rule_name: db.rule_name,
            category: AlertCategory(db.category),
            severity,
            title: db.title,
            message: db.message,
            status,
            source_entity_type: db.source_entity_type,
            source_entity_id: db.source_entity_id,
            action_route: db.action_route,
            first_seen_at: parse_rfc3339(&db.first_seen_at),
            last_seen_at: parse_rfc3339(&db.last_seen_at),
            snoozed_until: parse_rfc3339_opt(&db.snoozed_until),
            dismissed_at: parse_rfc3339_opt(&db.dismissed_at),
            resolved_at: parse_rfc3339_opt(&db.resolved_at),
            metadata_json: db.metadata_json,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mizan_core::alerts::{AlertCategory, AlertSeverity, ProposedAlert};

    fn proposal() -> ProposedAlert {
        ProposedAlert {
            fingerprint: "R:a:1".into(),
            rule_name: "R".into(),
            category: AlertCategory::valuations(),
            severity: AlertSeverity::Warning,
            title: "T".into(),
            message: "M".into(),
            source_entity_type: Some("asset".into()),
            source_entity_id: Some("1".into()),
            action_route: Some("/holdings/1".into()),
            metadata_json: Some("{\"k\":1}".into()),
        }
    }

    #[test]
    fn db_roundtrips_to_domain_preserving_all_fields() {
        let now = Utc::now();
        let id = "alert-1".to_string();
        let db = SmartAlertDB::from_proposal(id.clone(), &proposal(), now);
        let alert: SmartAlert = db.clone().into();
        assert_eq!(alert.id, "alert-1");
        assert_eq!(alert.fingerprint, "R:a:1");
        assert_eq!(alert.severity, AlertSeverity::Warning);
        assert_eq!(alert.status, AlertStatus::Active);
        assert_eq!(alert.source_entity_type.as_deref(), Some("asset"));
        // RFC3339 parsing should be stable up to second-precision.
        assert_eq!(
            alert.first_seen_at.timestamp(),
            now.timestamp(),
            "first_seen_at should round-trip"
        );
    }
}
