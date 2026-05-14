//! Diesel model for the `valuations` table.
//!
//! Timestamps, dates, decimal amounts, and confidence are persisted as
//! RFC3339 / ISO 8601 / canonical decimal strings — the same TEXT-only
//! convention every other table in this repo uses.

use chrono::{DateTime, NaiveDate, Utc};
use diesel::prelude::*;
use mizan_core::universal_assets::{NewValuation, Valuation, ValuationSource};
use rust_decimal::Decimal;
use std::str::FromStr;

#[derive(Queryable, Identifiable, Insertable, AsChangeset, Selectable, PartialEq, Debug, Clone)]
#[diesel(table_name = crate::schema::valuations)]
#[diesel(primary_key(id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct ValuationDB {
    pub id: String,
    pub asset_id: String,
    pub valuation_date: String,
    pub value_native: String,
    pub currency: String,
    pub source_type: String,
    pub source_id: Option<String>,
    pub confidence: Option<String>,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl ValuationDB {
    /// Build a row to insert from a [`NewValuation`]. The caller is
    /// responsible for producing a stable `id` (the repository uses a
    /// fresh uuid v4) and a `now` timestamp (so tests can pin).
    pub fn new_row(id: String, new: &NewValuation, now: DateTime<Utc>) -> Self {
        Self {
            id,
            asset_id: new.asset_id.clone(),
            valuation_date: new.valuation_date.to_string(),
            value_native: new.value_native.normalize().to_string(),
            currency: new.currency.trim().to_uppercase(),
            source_type: new.source_type.as_str().to_string(),
            source_id: new.source_id.clone(),
            confidence: new.confidence.map(|c| c.normalize().to_string()),
            notes: new.notes.clone(),
            created_at: now.to_rfc3339(),
            updated_at: now.to_rfc3339(),
        }
    }
}

fn parse_rfc3339(value: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(value)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now())
}

fn parse_date(value: &str) -> NaiveDate {
    NaiveDate::parse_from_str(value, "%Y-%m-%d").unwrap_or_else(|_| Utc::now().date_naive())
}

impl From<ValuationDB> for Valuation {
    fn from(db: ValuationDB) -> Self {
        let source = ValuationSource::parse(&db.source_type).unwrap_or(ValuationSource::Manual);
        let value = Decimal::from_str(&db.value_native).unwrap_or_default();
        let confidence = db
            .confidence
            .as_deref()
            .and_then(|s| Decimal::from_str(s).ok());
        Self {
            id: db.id,
            asset_id: db.asset_id,
            valuation_date: parse_date(&db.valuation_date),
            value_native: value,
            currency: db.currency,
            source_type: source,
            source_id: db.source_id,
            confidence,
            notes: db.notes,
            created_at: parse_rfc3339(&db.created_at),
            updated_at: parse_rfc3339(&db.updated_at),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use rust_decimal_macros::dec;

    fn nv() -> NewValuation {
        NewValuation {
            asset_id: "asset-1".into(),
            valuation_date: NaiveDate::from_ymd_opt(2026, 5, 14).unwrap(),
            value_native: dec!(1_234_567.89),
            currency: "  usd  ".into(),
            source_type: ValuationSource::Manual,
            source_id: Some("source-1".into()),
            confidence: Some(dec!(0.75)),
            notes: Some("annual revaluation".into()),
        }
    }

    #[test]
    fn new_row_normalises_currency_to_uppercase_iso_code() {
        let now = Utc.with_ymd_and_hms(2026, 5, 14, 12, 0, 0).unwrap();
        let row = ValuationDB::new_row("v1".into(), &nv(), now);
        assert_eq!(row.currency, "USD");
    }

    #[test]
    fn new_row_persists_decimal_amount_as_string_without_loss() {
        let now = Utc.with_ymd_and_hms(2026, 5, 14, 12, 0, 0).unwrap();
        let row = ValuationDB::new_row("v1".into(), &nv(), now);
        // Decimal::normalize trims trailing zeros but keeps full
        // precision; verify the round-trip still equals the input.
        let parsed = Decimal::from_str(&row.value_native).unwrap();
        assert_eq!(parsed, dec!(1_234_567.89));
    }

    #[test]
    fn round_trip_db_to_domain_preserves_all_fields() {
        let now = Utc.with_ymd_and_hms(2026, 5, 14, 12, 0, 0).unwrap();
        let row = ValuationDB::new_row("v1".into(), &nv(), now);
        let domain: Valuation = row.clone().into();
        assert_eq!(domain.id, "v1");
        assert_eq!(domain.asset_id, "asset-1");
        assert_eq!(
            domain.valuation_date,
            NaiveDate::from_ymd_opt(2026, 5, 14).unwrap()
        );
        assert_eq!(domain.value_native, dec!(1_234_567.89));
        assert_eq!(domain.currency, "USD");
        assert_eq!(domain.source_type, ValuationSource::Manual);
        assert_eq!(domain.source_id.as_deref(), Some("source-1"));
        assert_eq!(domain.confidence, Some(dec!(0.75)));
    }
}
