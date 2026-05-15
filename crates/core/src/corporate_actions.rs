use async_trait::async_trait;
use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::errors::ValidationError;
use crate::{Error, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CorporateActionType {
    Split,
    ReverseSplit,
    Merger,
    Spinoff,
    SymbolChange,
    ReturnOfCapital,
    StockDividend,
}

impl CorporateActionType {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Split => "split",
            Self::ReverseSplit => "reverse_split",
            Self::Merger => "merger",
            Self::Spinoff => "spinoff",
            Self::SymbolChange => "symbol_change",
            Self::ReturnOfCapital => "return_of_capital",
            Self::StockDividend => "stock_dividend",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "split" => Some(Self::Split),
            "reverse_split" => Some(Self::ReverseSplit),
            "merger" => Some(Self::Merger),
            "spinoff" => Some(Self::Spinoff),
            "symbol_change" => Some(Self::SymbolChange),
            "return_of_capital" => Some(Self::ReturnOfCapital),
            "stock_dividend" => Some(Self::StockDividend),
            _ => None,
        }
    }

    pub const fn is_implemented(self) -> bool {
        matches!(self, Self::Split | Self::ReverseSplit | Self::SymbolChange)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CorporateAction {
    pub id: String,
    pub asset_id: String,
    pub action_type: CorporateActionType,
    pub effective_date: NaiveDate,
    pub ratio_numerator: Option<Decimal>,
    pub ratio_denominator: Option<Decimal>,
    pub new_symbol: Option<String>,
    pub metadata_json: Option<serde_json::Value>,
    pub source_citation_id: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CorporateActionPositionPreview {
    pub account_id: String,
    pub quantity_before: Decimal,
    pub quantity_after: Decimal,
    pub average_cost_before: Decimal,
    pub average_cost_after: Decimal,
    pub total_cost_basis: Decimal,
    pub currency: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CorporateActionPreview {
    pub asset_id: String,
    pub action_type: CorporateActionType,
    pub effective_date: NaiveDate,
    pub ratio: Option<Decimal>,
    pub new_symbol: Option<String>,
    pub positions: Vec<CorporateActionPositionPreview>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplyCorporateActionRequest {
    pub asset_id: String,
    pub action_type: CorporateActionType,
    pub effective_date: NaiveDate,
    pub ratio_numerator: Option<Decimal>,
    pub ratio_denominator: Option<Decimal>,
    pub new_symbol: Option<String>,
    pub source_citation_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppliedCorporateAction {
    pub action: CorporateAction,
    pub preview: CorporateActionPreview,
}

impl ApplyCorporateActionRequest {
    pub fn validate(&self) -> Result<()> {
        if self.asset_id.trim().is_empty() {
            return Err(invalid("asset_id is required"));
        }
        if !self.action_type.is_implemented() {
            return Err(invalid("corporate action type is not implemented yet"));
        }

        match self.action_type {
            CorporateActionType::Split | CorporateActionType::ReverseSplit => {
                let ratio = self.ratio()?;
                if ratio <= Decimal::ZERO {
                    return Err(invalid("split ratio must be greater than zero"));
                }
                if self.action_type == CorporateActionType::Split && ratio <= Decimal::ONE {
                    return Err(invalid("split ratio must be greater than one"));
                }
                if self.action_type == CorporateActionType::ReverseSplit && ratio >= Decimal::ONE {
                    return Err(invalid("reverse split ratio must be less than one"));
                }
            }
            CorporateActionType::SymbolChange => {
                let new_symbol = self.new_symbol.as_deref().unwrap_or("").trim();
                if new_symbol.is_empty() {
                    return Err(invalid("new_symbol is required for symbol changes"));
                }
            }
            _ => {}
        }

        Ok(())
    }

    pub fn ratio(&self) -> Result<Decimal> {
        let numerator = self
            .ratio_numerator
            .ok_or_else(|| invalid("ratio_numerator is required"))?;
        let denominator = self
            .ratio_denominator
            .ok_or_else(|| invalid("ratio_denominator is required"))?;
        if numerator <= Decimal::ZERO || denominator <= Decimal::ZERO {
            return Err(invalid("ratio parts must be greater than zero"));
        }
        Ok(numerator / denominator)
    }
}

pub fn preview_stock_split(
    position: &CorporateActionPositionPreview,
    ratio: Decimal,
) -> Result<CorporateActionPositionPreview> {
    if ratio <= Decimal::ZERO {
        return Err(invalid("split ratio must be greater than zero"));
    }

    Ok(CorporateActionPositionPreview {
        account_id: position.account_id.clone(),
        quantity_before: position.quantity_before,
        quantity_after: position.quantity_before * ratio,
        average_cost_before: position.average_cost_before,
        average_cost_after: if position.average_cost_before.is_zero() {
            Decimal::ZERO
        } else {
            position.average_cost_before / ratio
        },
        total_cost_basis: position.total_cost_basis,
        currency: position.currency.clone(),
    })
}

#[async_trait]
pub trait CorporateActionsRepositoryTrait: Send + Sync {
    async fn preview_action(
        &self,
        request: ApplyCorporateActionRequest,
    ) -> Result<CorporateActionPreview>;
    async fn apply_action(
        &self,
        request: ApplyCorporateActionRequest,
    ) -> Result<AppliedCorporateAction>;
    async fn list_actions(&self, asset_id: &str) -> Result<Vec<CorporateAction>>;
}

fn invalid(message: impl Into<String>) -> Error {
    Error::Validation(ValidationError::InvalidInput(message.into()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn position() -> CorporateActionPositionPreview {
        CorporateActionPositionPreview {
            account_id: "acc_1".to_string(),
            quantity_before: dec!(10),
            quantity_after: dec!(10),
            average_cost_before: dec!(200),
            average_cost_after: dec!(200),
            total_cost_basis: dec!(2000),
            currency: "USD".to_string(),
        }
    }

    #[test]
    fn two_for_one_split_preserves_cost_basis_and_adjusts_quantity() {
        let preview = preview_stock_split(&position(), dec!(2)).unwrap();

        assert_eq!(preview.quantity_after, dec!(20));
        assert_eq!(preview.average_cost_after, dec!(100));
        assert_eq!(preview.total_cost_basis, dec!(2000));
    }

    #[test]
    fn reverse_split_preserves_cost_basis_and_adjusts_quantity() {
        let preview = preview_stock_split(&position(), dec!(0.25)).unwrap();

        assert_eq!(preview.quantity_after, dec!(2.5));
        assert_eq!(preview.average_cost_after, dec!(800));
        assert_eq!(preview.total_cost_basis, dec!(2000));
    }

    #[test]
    fn invalid_ratio_is_rejected() {
        assert!(preview_stock_split(&position(), dec!(0)).is_err());

        let request = ApplyCorporateActionRequest {
            asset_id: "asset_1".to_string(),
            action_type: CorporateActionType::Split,
            effective_date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            ratio_numerator: Some(dec!(1)),
            ratio_denominator: Some(dec!(1)),
            new_symbol: None,
            source_citation_id: None,
        };

        assert!(request.validate().is_err());
    }

    #[test]
    fn symbol_change_requires_new_symbol() {
        let request = ApplyCorporateActionRequest {
            asset_id: "asset_1".to_string(),
            action_type: CorporateActionType::SymbolChange,
            effective_date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            ratio_numerator: None,
            ratio_denominator: None,
            new_symbol: Some(" ".to_string()),
            source_citation_id: None,
        };

        assert!(request.validate().is_err());
    }
}
