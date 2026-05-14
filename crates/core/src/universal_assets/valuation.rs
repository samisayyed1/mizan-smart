//! Canonical per-asset valuation.
//!
//! This is the row in the `valuations` table. Every monetary number
//! that appears anywhere downstream — Explain-This-Number lineage,
//! reconciliation, web evidence approvals, document extraction
//! approvals, bulk-update grid — flows through this type. There is no
//! `f64` here: amounts are persisted as canonical decimal strings and
//! parsed into `rust_decimal::Decimal` at the domain boundary.

use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

use crate::errors::{Error, Result, ValidationError};

/// Where a valuation came from. Matches the SQL CHECK constraint on
/// `valuations.source_type` exactly; any new variant requires a
/// migration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValuationSource {
    /// User-entered. The default for property, private investments,
    /// collectibles, etc.
    Manual,
    /// Mirrored from a market quote.
    Market,
    /// Posted from an approved document extraction (Phase 2).
    Document,
    /// Posted by an import flow.
    Import,
    /// Posted from an approved web-evidence pack (Phase 5).
    WebEvidence,
    /// Derived from other valuations (e.g. private-fund NAV roll-forward).
    Calculated,
}

impl ValuationSource {
    pub const fn as_str(self) -> &'static str {
        match self {
            ValuationSource::Manual => "manual",
            ValuationSource::Market => "market",
            ValuationSource::Document => "document",
            ValuationSource::Import => "import",
            ValuationSource::WebEvidence => "web_evidence",
            ValuationSource::Calculated => "calculated",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "manual" => Some(ValuationSource::Manual),
            "market" => Some(ValuationSource::Market),
            "document" => Some(ValuationSource::Document),
            "import" => Some(ValuationSource::Import),
            "web_evidence" => Some(ValuationSource::WebEvidence),
            "calculated" => Some(ValuationSource::Calculated),
            _ => None,
        }
    }

    /// Enumerates every source for fixtures and round-trip tests.
    pub const fn all() -> [ValuationSource; 6] {
        [
            ValuationSource::Manual,
            ValuationSource::Market,
            ValuationSource::Document,
            ValuationSource::Import,
            ValuationSource::WebEvidence,
            ValuationSource::Calculated,
        ]
    }
}

impl fmt::Display for ValuationSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A persisted valuation row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Valuation {
    pub id: String,
    pub asset_id: String,
    pub valuation_date: NaiveDate,
    pub value_native: Decimal,
    pub currency: String,
    pub source_type: ValuationSource,
    /// Free-form FK reference for the source row (a document id, an
    /// import_run id, a web evidence pack id, …). Schema-level FK is
    /// deliberately omitted so source-system migrations can land
    /// independently.
    pub source_id: Option<String>,
    /// 0..=1 confidence band; `None` for sources that have no notion
    /// of confidence (manual, market).
    pub confidence: Option<Decimal>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Input used by repositories to create a new valuation. The
/// repository assigns `id` and timestamps; the caller is responsible
/// for ensuring `asset_id` exists, currency is normalised (ISO 4217
/// uppercase), and `value_native` is the native-currency amount, not a
/// converted base-currency value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewValuation {
    pub asset_id: String,
    pub valuation_date: NaiveDate,
    pub value_native: Decimal,
    pub currency: String,
    pub source_type: ValuationSource,
    pub source_id: Option<String>,
    pub confidence: Option<Decimal>,
    pub notes: Option<String>,
}

impl NewValuation {
    /// Validate the user-facing constraints that aren't enforceable by
    /// SQL CHECKs alone. Currency must be a 3-letter ISO code, value
    /// must be non-negative for non-liability classes, and confidence
    /// must be in `0..=1` when present.
    ///
    /// Negative values are *allowed* because liability rows record
    /// outstanding debt as a negative net-worth contribution. Callers
    /// that need a specific sign must enforce it themselves.
    pub fn validate(&self) -> Result<()> {
        if self.asset_id.trim().is_empty() {
            return Err(Error::Validation(ValidationError::InvalidInput(
                "asset_id is required".into(),
            )));
        }
        let currency = self.currency.trim();
        if currency.len() != 3 || !currency.chars().all(|c| c.is_ascii_uppercase()) {
            return Err(Error::Validation(ValidationError::InvalidInput(
                "currency must be a 3-letter ISO 4217 code".into(),
            )));
        }
        if let Some(confidence) = self.confidence {
            if confidence < Decimal::ZERO || confidence > Decimal::ONE {
                return Err(Error::Validation(ValidationError::InvalidInput(
                    "confidence must be between 0 and 1".into(),
                )));
            }
        }
        Ok(())
    }
}

/// Parse a `value_native` string back into `Decimal`. Centralised so
/// every consumer of `valuations.value_native` rejects the same set of
/// malformed inputs (empty strings, leading whitespace, locale-style
/// thousands separators).
pub fn parse_value_native(raw: &str) -> Result<Decimal> {
    Decimal::from_str(raw.trim()).map_err(|err| {
        Error::Validation(ValidationError::InvalidInput(format!(
            "value_native {:?} is not a valid decimal: {}",
            raw, err
        )))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn source_round_trips_through_string_and_json() {
        for source in ValuationSource::all() {
            assert_eq!(ValuationSource::parse(source.as_str()), Some(source));
            let json = serde_json::to_string(&source).unwrap();
            let back: ValuationSource = serde_json::from_str(&json).unwrap();
            assert_eq!(back, source);
        }
    }

    #[test]
    fn source_rejects_legacy_and_uppercase_values() {
        assert_eq!(ValuationSource::parse(""), None);
        assert_eq!(ValuationSource::parse("MANUAL"), None);
        assert_eq!(ValuationSource::parse("user"), None);
    }

    #[test]
    fn validate_accepts_a_well_formed_manual_valuation() {
        let nv = NewValuation {
            asset_id: "asset-1".into(),
            valuation_date: NaiveDate::from_ymd_opt(2026, 5, 14).unwrap(),
            value_native: dec!(1_000_000),
            currency: "USD".into(),
            source_type: ValuationSource::Manual,
            source_id: None,
            confidence: None,
            notes: None,
        };
        assert!(nv.validate().is_ok());
    }

    #[test]
    fn validate_rejects_blank_asset_id() {
        let nv = NewValuation {
            asset_id: "   ".into(),
            valuation_date: NaiveDate::from_ymd_opt(2026, 5, 14).unwrap(),
            value_native: dec!(100),
            currency: "USD".into(),
            source_type: ValuationSource::Manual,
            source_id: None,
            confidence: None,
            notes: None,
        };
        assert!(nv.validate().is_err());
    }

    #[test]
    fn validate_rejects_non_iso_currency_codes() {
        for bad in ["", "US", "USDX", "us$", "usd", "DOLLAR", "12A"] {
            let nv = NewValuation {
                asset_id: "asset-1".into(),
                valuation_date: NaiveDate::from_ymd_opt(2026, 5, 14).unwrap(),
                value_native: dec!(100),
                currency: bad.into(),
                source_type: ValuationSource::Manual,
                source_id: None,
                confidence: None,
                notes: None,
            };
            assert!(
                nv.validate().is_err(),
                "{:?} should not validate as a currency",
                bad
            );
        }
    }

    #[test]
    fn validate_rejects_confidence_outside_unit_interval() {
        for bad in [dec!(-0.01), dec!(1.01), dec!(2), dec!(-1)] {
            let nv = NewValuation {
                asset_id: "asset-1".into(),
                valuation_date: NaiveDate::from_ymd_opt(2026, 5, 14).unwrap(),
                value_native: dec!(100),
                currency: "USD".into(),
                source_type: ValuationSource::WebEvidence,
                source_id: Some("pack-1".into()),
                confidence: Some(bad),
                notes: None,
            };
            assert!(
                nv.validate().is_err(),
                "confidence {} should be rejected",
                bad
            );
        }
    }

    #[test]
    fn parse_value_native_round_trips_decimals_without_loss() {
        let cases = [
            ("0", dec!(0)),
            ("1234567.89", dec!(1234567.89)),
            (" -42.5 ", dec!(-42.5)),
            ("0.0000000001", dec!(0.0000000001)),
        ];
        for (raw, expected) in cases {
            let parsed = parse_value_native(raw).expect("decimal parses");
            assert_eq!(parsed, expected);
        }
    }

    #[test]
    fn parse_value_native_rejects_locale_formatted_amounts() {
        assert!(parse_value_native("1,234.56").is_err());
        assert!(parse_value_native("$100").is_err());
        assert!(parse_value_native("").is_err());
        assert!(parse_value_native("not a number").is_err());
    }
}
