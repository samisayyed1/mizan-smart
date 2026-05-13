//! Alert engine — drives the deterministic rule run.
//!
//! The engine itself is intentionally tiny: it gathers proposals from
//! each registered rule, then upserts them via the [`AlertStore`].
//! Because rules carry their own input types, the engine is generic
//! over the per-run input shape.
//!
//! Tests cover the rule-evaluation pipeline using an in-memory store.

use std::sync::Arc;

use chrono::{DateTime, Utc};

use super::context::AlertContext;
use super::model::ProposedAlert;
use super::traits::{AlertStore, UpsertOutcome};
use crate::errors::Result;

/// Aggregated outcome of one rule-engine run.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct EngineRunReport {
    pub inserted: usize,
    pub updated: usize,
    pub suppressed: usize,
}

impl EngineRunReport {
    pub fn record(&mut self, outcome: UpsertOutcome) {
        match outcome {
            UpsertOutcome::Inserted => self.inserted += 1,
            UpsertOutcome::Updated => self.updated += 1,
            UpsertOutcome::Suppressed => self.suppressed += 1,
        }
    }

    pub fn total(&self) -> usize {
        self.inserted + self.updated + self.suppressed
    }
}

pub struct AlertEngine {
    store: Arc<dyn AlertStore>,
}

impl AlertEngine {
    pub fn new(store: Arc<dyn AlertStore>) -> Self {
        Self { store }
    }

    /// Persist a batch of proposed alerts as a single deterministic
    /// step. Each proposal flows through [`AlertStore::upsert`] in the
    /// order supplied, so callers retain control over visible ordering
    /// when several rules might produce the same fingerprint (only the
    /// first wins; subsequent calls update `last_seen_at`).
    pub async fn apply(
        &self,
        proposals: &[ProposedAlert],
        now: DateTime<Utc>,
    ) -> Result<EngineRunReport> {
        let mut report = EngineRunReport::default();
        for proposal in proposals {
            let outcome = self.store.upsert(proposal, now).await?;
            report.record(outcome);
        }
        Ok(report)
    }

    /// Convenience used by tests to wrap the system clock.
    pub async fn apply_now(&self, proposals: &[ProposedAlert]) -> Result<EngineRunReport> {
        self.apply(proposals, AlertContext::current().now).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::alerts::model::{
        AlertCategory, AlertSeverity, AlertStatus, ProposedAlert, SmartAlert,
    };
    use async_trait::async_trait;
    use std::sync::Mutex;

    /// Minimal in-memory store used to verify engine bookkeeping.
    /// `crates/storage-sqlite` provides the production implementation.
    #[derive(Default)]
    struct MemStore {
        rows: Mutex<Vec<SmartAlert>>,
    }

    impl MemStore {
        fn snapshot_rows(&self) -> Vec<SmartAlert> {
            self.rows.lock().unwrap().clone()
        }
    }

    #[async_trait]
    impl AlertStore for MemStore {
        async fn upsert(
            &self,
            proposal: &ProposedAlert,
            now: DateTime<Utc>,
        ) -> Result<UpsertOutcome> {
            let mut rows = self.rows.lock().unwrap();
            if let Some(existing) = rows
                .iter_mut()
                .find(|row| row.fingerprint == proposal.fingerprint)
            {
                existing.last_seen_at = now;
                existing.severity = proposal.severity;
                existing.title = proposal.title.clone();
                existing.message = proposal.message.clone();
                existing.category = proposal.category.clone();
                existing.metadata_json = proposal.metadata_json.clone();
                return Ok(UpsertOutcome::Updated);
            }
            let row = SmartAlert {
                id: format!("alert-{}", rows.len() + 1),
                fingerprint: proposal.fingerprint.clone(),
                rule_name: proposal.rule_name.clone(),
                category: proposal.category.clone(),
                severity: proposal.severity,
                title: proposal.title.clone(),
                message: proposal.message.clone(),
                status: AlertStatus::Active,
                source_entity_type: proposal.source_entity_type.clone(),
                source_entity_id: proposal.source_entity_id.clone(),
                action_route: proposal.action_route.clone(),
                first_seen_at: now,
                last_seen_at: now,
                snoozed_until: None,
                dismissed_at: None,
                resolved_at: None,
                metadata_json: proposal.metadata_json.clone(),
            };
            rows.push(row);
            Ok(UpsertOutcome::Inserted)
        }

        async fn list(&self, status: Option<AlertStatus>) -> Result<Vec<SmartAlert>> {
            let rows = self.rows.lock().unwrap().clone();
            Ok(match status {
                Some(s) => rows.into_iter().filter(|r| r.status == s).collect(),
                None => rows,
            })
        }

        async fn get(&self, id: &str) -> Result<Option<SmartAlert>> {
            Ok(self
                .rows
                .lock()
                .unwrap()
                .iter()
                .find(|r| r.id == id)
                .cloned())
        }

        async fn snooze(&self, id: &str, until: DateTime<Utc>, _now: DateTime<Utc>) -> Result<()> {
            if let Some(row) = self.rows.lock().unwrap().iter_mut().find(|r| r.id == id) {
                row.status = AlertStatus::Snoozed;
                row.snoozed_until = Some(until);
            }
            Ok(())
        }

        async fn dismiss(&self, id: &str, now: DateTime<Utc>) -> Result<()> {
            if let Some(row) = self.rows.lock().unwrap().iter_mut().find(|r| r.id == id) {
                row.status = AlertStatus::Dismissed;
                row.dismissed_at = Some(now);
            }
            Ok(())
        }

        async fn resolve(&self, id: &str, now: DateTime<Utc>) -> Result<()> {
            if let Some(row) = self.rows.lock().unwrap().iter_mut().find(|r| r.id == id) {
                row.status = AlertStatus::Resolved;
                row.resolved_at = Some(now);
            }
            Ok(())
        }
    }

    fn proposal(fp: &str) -> ProposedAlert {
        ProposedAlert {
            fingerprint: fp.to_string(),
            rule_name: "TestRule".to_string(),
            category: AlertCategory::valuations(),
            severity: AlertSeverity::Warning,
            title: "T".to_string(),
            message: "M".to_string(),
            source_entity_type: Some("asset".to_string()),
            source_entity_id: Some("a1".to_string()),
            action_route: None,
            metadata_json: None,
        }
    }

    #[tokio::test]
    async fn first_run_inserts_each_unique_fingerprint() {
        let store = Arc::new(MemStore::default());
        let engine = AlertEngine::new(store.clone());
        let now = Utc::now();
        let report = engine
            .apply(&[proposal("R:asset:1"), proposal("R:asset:2")], now)
            .await
            .unwrap();
        assert_eq!(report.inserted, 2);
        assert_eq!(report.updated, 0);
        assert_eq!(store.snapshot_rows().len(), 2);
    }

    #[tokio::test]
    async fn repeated_run_updates_last_seen_at_not_duplicates() {
        let store = Arc::new(MemStore::default());
        let engine = AlertEngine::new(store.clone());
        let now1 = Utc::now();
        engine.apply(&[proposal("R:asset:1")], now1).await.unwrap();

        let now2 = now1 + chrono::Duration::hours(1);
        let report = engine.apply(&[proposal("R:asset:1")], now2).await.unwrap();
        assert_eq!(report.updated, 1);
        assert_eq!(report.inserted, 0);
        let rows = store.snapshot_rows();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].first_seen_at, now1);
        assert_eq!(rows[0].last_seen_at, now2);
    }
}
