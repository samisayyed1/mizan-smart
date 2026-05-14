//! SQLite persistence for the universal asset model.
//!
//! Tables created by migration
//! `2026-05-14-000002_universal_asset_model/up.sql`. The companion
//! domain types live in `crates/core/src/universal_assets`.
//!
//! Today this module ships the `Valuation` model and repository (the
//! highest-leverage primitive — used by P6 bulk update grid, P15
//! Explain-This-Number, P35+ web evidence, and the alerts engine).
//! Domain types and Diesel `table!` entries already exist for the nine
//! typed extension tables, so per-class repositories slot in via
//! follow-up prompts (P19 fixed-income, P17 private investments, etc.)
//! without further migration work.

pub mod create_repository;
pub mod details_models;
pub mod valuation_model;
pub mod valuation_repository;

pub use create_repository::{UniversalAssetCreateRepository, UniversalAssetCreated};
pub use valuation_model::ValuationDB;
pub use valuation_repository::ValuationRepository;
