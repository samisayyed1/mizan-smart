//! SQLite persistence for the Smart Alerts engine.
//!
//! Tables created by `migrations/2026-05-14-000001_smart_alerts/up.sql`.
//! See `crates/core/src/alerts` for the domain types and rule trait.

pub mod model;
pub mod repository;

pub use repository::SmartAlertRepository;
