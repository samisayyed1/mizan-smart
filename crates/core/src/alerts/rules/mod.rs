//! Concrete `AlertRule` implementations.
//!
//! Each rule is in its own file so that fixtures and tests live next
//! to the logic they exercise. Adding a new rule: create a new file
//! here, implement `AlertRule`, then register it in
//! `super::service::AlertEngine::default_rules`.

pub mod stale_manual_valuation;
