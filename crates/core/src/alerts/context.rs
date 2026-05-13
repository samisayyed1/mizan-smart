//! Context shared with every alert rule during a single engine run.
//!
//! A `AlertContext` carries the current timestamp and any configuration
//! a rule needs to make deterministic decisions. Rules must derive every
//! threshold from the context (not from `Utc::now()` or other globals)
//! so tests can pin behaviour at any point in time.

use chrono::{DateTime, Utc};

/// Configuration thresholds for the rule engine. Values are kept as
/// integers so they round-trip through JSON without floating-point drift.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AlertEngineConfig {
    /// Number of days after which a manual valuation is flagged as a
    /// warning. Spec: 45 days.
    pub stale_manual_valuation_warning_days: u32,

    /// Number of days after which a manual valuation is flagged as
    /// critical. Spec: 90 days.
    pub stale_manual_valuation_critical_days: u32,
}

impl Default for AlertEngineConfig {
    fn default() -> Self {
        Self {
            stale_manual_valuation_warning_days: 45,
            stale_manual_valuation_critical_days: 90,
        }
    }
}

/// Per-run context handed to every rule. Pinning `now` makes the engine
/// reproducible: the same data + the same context always produces the
/// same fingerprints and severities.
#[derive(Debug, Clone)]
pub struct AlertContext {
    pub now: DateTime<Utc>,
    pub config: AlertEngineConfig,
}

impl AlertContext {
    pub fn new(now: DateTime<Utc>) -> Self {
        Self {
            now,
            config: AlertEngineConfig::default(),
        }
    }

    pub fn with_config(now: DateTime<Utc>, config: AlertEngineConfig) -> Self {
        Self { now, config }
    }

    /// Convenience constructor that pins `now` to the system clock.
    /// Production engine runs use this; tests should always use
    /// [`AlertContext::new`] with an explicit timestamp.
    pub fn current() -> Self {
        Self::new(Utc::now())
    }
}
