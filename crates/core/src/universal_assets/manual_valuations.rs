//! Manual valuation bulk-update domain types.
//!
//! The p6 "Update Values" grid validates every row before storage writes
//! anything. Money remains `Decimal` at the domain boundary; UI/API payloads
//! carry decimal strings so invalid values can be reported per row.

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::universal_assets::{
    parse_value_native, AssetClassification, NewValuation, ValuationSource,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ManualValuationStaleness {
    Current,
    Warning,
    Critical,
}

pub fn manual_valuation_classifications() -> &'static [AssetClassification] {
    &[
        AssetClassification::RealEstate,
        AssetClassification::PrivateEquity,
        AssetClassification::PrivateCredit,
        AssetClassification::HedgeFund,
        AssetClassification::VentureCapital,
        AssetClassification::Commodity,
        AssetClassification::Gold,
        AssetClassification::Silver,
        AssetClassification::Insurance,
        AssetClassification::Ulip,
        AssetClassification::BusinessOwnership,
        AssetClassification::Collectible,
        AssetClassification::Custom,
    ]
}

pub fn is_manual_valuation_class(classification: AssetClassification) -> bool {
    manual_valuation_classifications().contains(&classification)
}

pub fn stale_status(valuation_date: NaiveDate, as_of: NaiveDate) -> ManualValuationStaleness {
    let age_days = (as_of - valuation_date).num_days();
    if age_days > 90 {
        ManualValuationStaleness::Critical
    } else if age_days > 45 {
        ManualValuationStaleness::Warning
    } else {
        ManualValuationStaleness::Current
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManualValuationAsset {
    pub asset_id: String,
    pub name: String,
    pub classification: AssetClassification,
    pub current_value: Option<String>,
    pub valuation_date: Option<NaiveDate>,
    pub currency: String,
    pub notes: Option<String>,
    pub staleness: ManualValuationStaleness,
    pub history_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManualValuationUpdateRow {
    pub asset_id: String,
    pub current_value: String,
    pub valuation_date: NaiveDate,
    pub currency: String,
    #[serde(default)]
    pub notes: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BulkUpdateValuationsRequest {
    pub rows: Vec<ManualValuationUpdateRow>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RowValidationError {
    pub row_index: usize,
    pub asset_id: Option<String>,
    pub field: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BulkUpdateValuationsResult {
    pub updated_count: usize,
    pub errors: Vec<RowValidationError>,
}

pub fn validate_bulk_update_rows(rows: &[ManualValuationUpdateRow]) -> Vec<RowValidationError> {
    let mut errors = Vec::new();
    for (row_index, row) in rows.iter().enumerate() {
        let asset_ref = if row.asset_id.trim().is_empty() {
            None
        } else {
            Some(row.asset_id.clone())
        };

        if row.asset_id.trim().is_empty() {
            errors.push(RowValidationError {
                row_index,
                asset_id: asset_ref.clone(),
                field: "assetId".into(),
                message: "Asset is required".into(),
            });
        }

        if parse_value_native(&row.current_value).is_err() {
            errors.push(RowValidationError {
                row_index,
                asset_id: asset_ref.clone(),
                field: "currentValue".into(),
                message: "Enter a valid decimal amount".into(),
            });
        }

        let currency = row.currency.trim();
        if currency.len() != 3 || !currency.chars().all(|c| c.is_ascii_uppercase()) {
            errors.push(RowValidationError {
                row_index,
                asset_id: asset_ref,
                field: "currency".into(),
                message: "Use a 3-letter uppercase ISO currency code".into(),
            });
        }
    }
    errors
}

pub fn row_to_new_valuation(row: &ManualValuationUpdateRow) -> crate::Result<NewValuation> {
    Ok(NewValuation {
        asset_id: row.asset_id.trim().to_string(),
        valuation_date: row.valuation_date,
        value_native: parse_value_native(&row.current_value)?,
        currency: row.currency.trim().to_uppercase(),
        source_type: ValuationSource::Manual,
        source_id: None,
        confidence: None,
        notes: row
            .notes
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn stale_status_uses_prompt_thresholds() {
        let as_of = NaiveDate::from_ymd_opt(2026, 5, 14).unwrap();
        assert_eq!(
            stale_status(NaiveDate::from_ymd_opt(2026, 4, 1).unwrap(), as_of),
            ManualValuationStaleness::Current
        );
        assert_eq!(
            stale_status(NaiveDate::from_ymd_opt(2026, 3, 29).unwrap(), as_of),
            ManualValuationStaleness::Warning
        );
        assert_eq!(
            stale_status(NaiveDate::from_ymd_opt(2026, 2, 1).unwrap(), as_of),
            ManualValuationStaleness::Critical
        );
    }

    #[test]
    fn decimal_validation_rejects_invalid_strings() {
        let rows = vec![ManualValuationUpdateRow {
            asset_id: "asset-1".into(),
            current_value: "1,234.50".into(),
            valuation_date: NaiveDate::from_ymd_opt(2026, 5, 14).unwrap(),
            currency: "USD".into(),
            notes: None,
        }];
        let errors = validate_bulk_update_rows(&rows);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].field, "currentValue");
    }

    #[test]
    fn valid_row_becomes_manual_new_valuation() {
        let row = ManualValuationUpdateRow {
            asset_id: " asset-1 ".into(),
            current_value: "1234.50".into(),
            valuation_date: NaiveDate::from_ymd_opt(2026, 5, 14).unwrap(),
            currency: "USD".into(),
            notes: Some(" appraisal ".into()),
        };
        let valuation = row_to_new_valuation(&row).unwrap();
        assert_eq!(valuation.asset_id, "asset-1");
        assert_eq!(valuation.value_native, dec!(1234.50));
        assert_eq!(valuation.source_type, ValuationSource::Manual);
        assert_eq!(valuation.notes.as_deref(), Some("appraisal"));
    }
}
