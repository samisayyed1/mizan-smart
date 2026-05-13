//! Smart Alerts engine.
//!
//! Implements the deterministic alert engine described in
//! `docs/mizan-smart-plan/PLAN.md` Phase 1 / Prompt 8.
//!
//! An alert is something Mizan flags about the user's data — a stale
//! manual valuation, a missing FX rate, an unclassified asset, etc.
//! Each alert carries a stable `fingerprint` so re-running the rule
//! engine does not produce duplicate rows; instead the existing alert's
//! `last_seen_at` is bumped.
//!
//! AI never writes to this table directly. Only the rule engine and
//! the explicit user-action commands (snooze, dismiss, resolve) mutate
//! alert state. Every rule's output is structured, deterministic, and
//! traceable to a source entity.

pub mod context;
pub mod fingerprint;
pub mod model;
pub mod rules;
pub mod service;
pub mod traits;

pub use context::AlertContext;
pub use model::{AlertCategory, AlertSeverity, AlertStatus, ProposedAlert, SmartAlert};
pub use rules::stale_manual_valuation::StaleManualValuationRule;
pub use service::AlertEngine;
pub use traits::{AlertRule, AlertStore};
