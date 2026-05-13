//! Traits for the Smart Alerts engine.
//!
//! `AlertRule` is the unit of work for the deterministic engine; each
//! rule is a pure function from input data + context to a list of
//! `ProposedAlert`s. Rules must not perform IO, must be deterministic
//! given the same inputs, and must not produce duplicate fingerprints
//! within a single run.
//!
//! `AlertStore` is the persistence boundary, implemented by the
//! `storage-sqlite` crate. The engine reconciles proposals against the
//! store and never re-orders or invents data.

use async_trait::async_trait;
use chrono::{DateTime, Utc};

use super::context::AlertContext;
use super::model::{AlertStatus, ProposedAlert, SmartAlert};
use crate::errors::Result;

/// A single deterministic alert rule.
///
/// Implementations live in [`super::rules`] and own their own input
/// type so rules that need different snapshots of the system (holdings,
/// quotes, FX, etc.) can be tested in isolation against fixtures.
pub trait AlertRule {
    /// Input data the rule needs to evaluate.
    type Input;

    /// Stable name used as the prefix of every fingerprint this rule
    /// produces and as `rule_name` on the persisted row. Must be a
    /// PascalCase identifier.
    fn name(&self) -> &'static str;

    /// Evaluate the rule and return any proposed alerts. Rules must
    /// not panic on bad input; they should simply return an empty Vec.
    fn evaluate(&self, ctx: &AlertContext, input: &Self::Input) -> Vec<ProposedAlert>;
}

/// Outcome of upserting a single proposed alert.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpsertOutcome {
    /// A new alert row was created.
    Inserted,
    /// An existing row matched on `fingerprint` and had `last_seen_at`
    /// updated; severity/message/category are also refreshed because
    /// the rule may have re-classified.
    Updated,
    /// The proposal was suppressed because an active dismissal or
    /// snooze window covers it.
    Suppressed,
}

/// Persistence boundary for smart alerts.
///
/// Implementations live in `storage-sqlite`. The trait is async because
/// the SQLite backend uses an async write handle for serialised writes.
#[async_trait]
pub trait AlertStore: Send + Sync {
    /// Upsert a single proposed alert and return what happened.
    ///
    /// The `now` argument lets callers (engine + tests) drive the
    /// `last_seen_at` clock deterministically.
    async fn upsert(&self, proposal: &ProposedAlert, now: DateTime<Utc>) -> Result<UpsertOutcome>;

    /// List alerts filtered by status. Pass `None` to return all rows.
    async fn list(&self, status: Option<AlertStatus>) -> Result<Vec<SmartAlert>>;

    /// Fetch a single alert by id.
    async fn get(&self, id: &str) -> Result<Option<SmartAlert>>;

    /// Mark an alert as snoozed until the given timestamp.
    async fn snooze(&self, id: &str, until: DateTime<Utc>, now: DateTime<Utc>) -> Result<()>;

    /// Mark an alert as dismissed (user choice; not auto).
    async fn dismiss(&self, id: &str, now: DateTime<Utc>) -> Result<()>;

    /// Mark an alert as resolved (condition no longer holds).
    async fn resolve(&self, id: &str, now: DateTime<Utc>) -> Result<()>;
}
