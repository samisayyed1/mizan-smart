use async_trait::async_trait;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::errors::Result;

pub const DEFAULT_SHARIAH_PROFILE_ID: &str = "system_default_shariah_screening_profile";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShariahScreeningStatus {
    Compliant,
    NonCompliant,
    Questionable,
    Unknown,
    NeedsReview,
}

impl ShariahScreeningStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Compliant => "compliant",
            Self::NonCompliant => "non_compliant",
            Self::Questionable => "questionable",
            Self::Unknown => "unknown",
            Self::NeedsReview => "needs_review",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "compliant" => Some(Self::Compliant),
            "non_compliant" => Some(Self::NonCompliant),
            "questionable" => Some(Self::Questionable),
            "unknown" => Some(Self::Unknown),
            "needs_review" => Some(Self::NeedsReview),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShariahScreeningProfile {
    pub id: String,
    pub name: String,
    pub debt_threshold: Decimal,
    pub liquid_assets_threshold: Decimal,
    pub impure_income_threshold: Decimal,
    pub is_default: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetShariahScreening {
    pub id: String,
    pub asset_id: String,
    pub profile_id: String,
    pub status: ShariahScreeningStatus,
    pub debt_ratio: Option<Decimal>,
    pub liquid_assets_ratio: Option<Decimal>,
    pub impure_income_ratio: Option<Decimal>,
    pub source_citation_id: Option<String>,
    pub manual_override_reason: Option<String>,
    pub reviewed_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShariahScreeningRatios {
    pub debt_ratio: Option<Decimal>,
    pub liquid_assets_ratio: Option<Decimal>,
    pub impure_income_ratio: Option<Decimal>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShariahScreeningEvaluation {
    pub status: ShariahScreeningStatus,
    pub missing_fields: Vec<String>,
}

#[async_trait]
pub trait ShariahScreeningRepositoryTrait: Send + Sync {
    fn list_profiles(&self) -> Result<Vec<ShariahScreeningProfile>>;

    fn get_default_profile(&self) -> Result<ShariahScreeningProfile>;

    fn get_asset_screening(&self, asset_id: &str) -> Result<Option<AssetShariahScreening>>;
}

pub fn evaluate_shariah_screening(
    profile: &ShariahScreeningProfile,
    ratios: &ShariahScreeningRatios,
) -> ShariahScreeningEvaluation {
    let mut missing_fields = Vec::new();
    if ratios.debt_ratio.is_none() {
        missing_fields.push("debtRatio".to_string());
    }
    if ratios.liquid_assets_ratio.is_none() {
        missing_fields.push("liquidAssetsRatio".to_string());
    }
    if ratios.impure_income_ratio.is_none() {
        missing_fields.push("impureIncomeRatio".to_string());
    }

    if !missing_fields.is_empty() {
        return ShariahScreeningEvaluation {
            status: ShariahScreeningStatus::Unknown,
            missing_fields,
        };
    }

    let Some(debt_ratio) = ratios.debt_ratio else {
        unreachable!("missing fields returned above");
    };
    let Some(liquid_assets_ratio) = ratios.liquid_assets_ratio else {
        unreachable!("missing fields returned above");
    };
    let Some(impure_income_ratio) = ratios.impure_income_ratio else {
        unreachable!("missing fields returned above");
    };

    let status = if debt_ratio.normalize() < profile.debt_threshold.normalize()
        && liquid_assets_ratio.normalize() < profile.liquid_assets_threshold.normalize()
        && impure_income_ratio.normalize() < profile.impure_income_threshold.normalize()
    {
        ShariahScreeningStatus::Compliant
    } else {
        ShariahScreeningStatus::NonCompliant
    };

    ShariahScreeningEvaluation {
        status,
        missing_fields,
    }
}

#[cfg(test)]
mod tests {
    use rust_decimal_macros::dec;

    use super::*;

    fn profile() -> ShariahScreeningProfile {
        ShariahScreeningProfile {
            id: DEFAULT_SHARIAH_PROFILE_ID.to_string(),
            name: "Default".to_string(),
            debt_threshold: dec!(0.30),
            liquid_assets_threshold: dec!(0.30),
            impure_income_threshold: dec!(0.05),
            is_default: true,
            created_at: "2026-05-15T00:00:00Z".to_string(),
            updated_at: "2026-05-15T00:00:00Z".to_string(),
        }
    }

    #[test]
    fn thresholds_evaluate_compliant_and_non_compliant() {
        let compliant = evaluate_shariah_screening(
            &profile(),
            &ShariahScreeningRatios {
                debt_ratio: Some(dec!(0.29)),
                liquid_assets_ratio: Some(dec!(0.20)),
                impure_income_ratio: Some(dec!(0.04)),
            },
        );
        assert_eq!(compliant.status, ShariahScreeningStatus::Compliant);

        let non_compliant = evaluate_shariah_screening(
            &profile(),
            &ShariahScreeningRatios {
                debt_ratio: Some(dec!(0.30)),
                liquid_assets_ratio: Some(dec!(0.20)),
                impure_income_ratio: Some(dec!(0.04)),
            },
        );
        assert_eq!(non_compliant.status, ShariahScreeningStatus::NonCompliant);
    }

    #[test]
    fn missing_ratios_produce_unknown() {
        let evaluation = evaluate_shariah_screening(
            &profile(),
            &ShariahScreeningRatios {
                debt_ratio: Some(dec!(0.20)),
                liquid_assets_ratio: None,
                impure_income_ratio: Some(dec!(0.01)),
            },
        );
        assert_eq!(evaluation.status, ShariahScreeningStatus::Unknown);
        assert_eq!(evaluation.missing_fields, vec!["liquidAssetsRatio"]);
    }
}
