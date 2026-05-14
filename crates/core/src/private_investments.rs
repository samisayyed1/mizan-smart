use async_trait::async_trait;
use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

use crate::errors::{Error, ValidationError};
use crate::Result;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CapitalCallStatus {
    Expected,
    Due,
    Paid,
    Cancelled,
}

impl CapitalCallStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Expected => "expected",
            Self::Due => "due",
            Self::Paid => "paid",
            Self::Cancelled => "cancelled",
        }
    }
}

impl FromStr for CapitalCallStatus {
    type Err = Error;

    fn from_str(value: &str) -> Result<Self> {
        match value {
            "expected" => Ok(Self::Expected),
            "due" => Ok(Self::Due),
            "paid" => Ok(Self::Paid),
            "cancelled" => Ok(Self::Cancelled),
            _ => Err(invalid(format!("Unsupported capital call status: {value}"))),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrivateInvestment {
    pub asset_id: String,
    pub manager: String,
    pub strategy: String,
    pub vintage_year: Option<i32>,
    pub commitment_amount: Decimal,
    pub commitment_currency: String,
    pub inception_date: Option<NaiveDate>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrivateInvestmentValuation {
    pub id: String,
    pub asset_id: String,
    pub valuation_date: NaiveDate,
    pub nav: Decimal,
    pub currency: String,
    pub source_citation_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CapitalCall {
    pub id: String,
    pub asset_id: String,
    pub notice_date: NaiveDate,
    pub due_date: NaiveDate,
    pub amount: Decimal,
    pub currency: String,
    pub status: CapitalCallStatus,
    pub source_citation_id: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrivateDistribution {
    pub id: String,
    pub asset_id: String,
    pub distribution_date: NaiveDate,
    pub amount: Decimal,
    pub currency: String,
    pub recallable: bool,
    pub source_citation_id: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrivateInvestmentSummary {
    pub investment: PrivateInvestment,
    pub commitment: Decimal,
    pub paid_in_capital: Decimal,
    pub unfunded_commitment: Decimal,
    pub total_distributions: Decimal,
    pub recallable_distributions: Decimal,
    pub current_nav: Decimal,
    pub dpi: Option<Decimal>,
    pub rvpi: Option<Decimal>,
    pub tvpi: Option<Decimal>,
    pub moic: Option<Decimal>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpsertPrivateInvestmentRequest {
    pub asset_id: String,
    pub manager: String,
    pub strategy: String,
    pub vintage_year: Option<i32>,
    pub commitment_amount: Decimal,
    pub commitment_currency: String,
    pub inception_date: Option<NaiveDate>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatePrivateInvestmentValuationRequest {
    pub asset_id: String,
    pub valuation_date: NaiveDate,
    pub nav: Decimal,
    pub currency: String,
    pub source_citation_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateCapitalCallRequest {
    pub asset_id: String,
    pub notice_date: NaiveDate,
    pub due_date: NaiveDate,
    pub amount: Decimal,
    pub currency: String,
    pub status: CapitalCallStatus,
    pub source_citation_id: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCapitalCallStatusRequest {
    pub id: String,
    pub status: CapitalCallStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatePrivateDistributionRequest {
    pub asset_id: String,
    pub distribution_date: NaiveDate,
    pub amount: Decimal,
    pub currency: String,
    pub recallable: bool,
    pub source_citation_id: Option<String>,
    pub notes: Option<String>,
}

impl UpsertPrivateInvestmentRequest {
    pub fn validate(&self) -> Result<()> {
        validate_asset_id(&self.asset_id)?;
        validate_non_empty("manager", &self.manager)?;
        validate_non_empty("strategy", &self.strategy)?;
        validate_non_negative("commitment_amount", self.commitment_amount)?;
        validate_currency(&self.commitment_currency)?;
        Ok(())
    }

    pub fn into_domain(self) -> PrivateInvestment {
        PrivateInvestment {
            asset_id: self.asset_id,
            manager: self.manager.trim().to_string(),
            strategy: self.strategy.trim().to_string(),
            vintage_year: self.vintage_year,
            commitment_amount: self.commitment_amount,
            commitment_currency: self.commitment_currency.trim().to_uppercase(),
            inception_date: self.inception_date,
            notes: self.notes,
        }
    }
}

impl CreatePrivateInvestmentValuationRequest {
    pub fn validate(&self) -> Result<()> {
        validate_asset_id(&self.asset_id)?;
        validate_non_negative("nav", self.nav)?;
        validate_currency(&self.currency)
    }
}

impl CreateCapitalCallRequest {
    pub fn validate(&self) -> Result<()> {
        validate_asset_id(&self.asset_id)?;
        validate_non_negative("amount", self.amount)?;
        validate_currency(&self.currency)
    }
}

impl CreatePrivateDistributionRequest {
    pub fn validate(&self) -> Result<()> {
        validate_asset_id(&self.asset_id)?;
        validate_non_negative("amount", self.amount)?;
        validate_currency(&self.currency)
    }
}

pub fn calculate_private_investment_summary(
    investment: PrivateInvestment,
    valuations: &[PrivateInvestmentValuation],
    capital_calls: &[CapitalCall],
    distributions: &[PrivateDistribution],
) -> PrivateInvestmentSummary {
    let commitment = investment.commitment_amount;
    let paid_in_capital = capital_calls
        .iter()
        .filter(|call| call.status == CapitalCallStatus::Paid)
        .map(|call| call.amount)
        .sum::<Decimal>();
    let total_distributions = distributions
        .iter()
        .map(|distribution| distribution.amount)
        .sum::<Decimal>();
    let recallable_distributions = distributions
        .iter()
        .filter(|distribution| distribution.recallable)
        .map(|distribution| distribution.amount)
        .sum::<Decimal>();
    let current_nav = valuations
        .iter()
        .max_by_key(|valuation| valuation.valuation_date)
        .map(|valuation| valuation.nav)
        .unwrap_or(Decimal::ZERO);
    let unfunded_commitment = commitment - paid_in_capital + recallable_distributions;
    let denominator = if paid_in_capital.is_zero() {
        None
    } else {
        Some(paid_in_capital)
    };
    let dpi = denominator.map(|value| total_distributions / value);
    let rvpi = denominator.map(|value| current_nav / value);
    let tvpi = denominator.map(|value| (total_distributions + current_nav) / value);
    let moic = tvpi;
    let mut warnings = Vec::new();
    if paid_in_capital > commitment {
        warnings.push("Paid-in capital exceeds commitment.".to_string());
    }
    if total_distributions > paid_in_capital && !paid_in_capital.is_zero() {
        warnings.push("Total distributions exceed paid-in capital.".to_string());
    }
    if unfunded_commitment < Decimal::ZERO {
        warnings.push("Unfunded commitment is negative.".to_string());
    }

    PrivateInvestmentSummary {
        investment,
        commitment,
        paid_in_capital,
        unfunded_commitment,
        total_distributions,
        recallable_distributions,
        current_nav,
        dpi,
        rvpi,
        tvpi,
        moic,
        warnings,
    }
}

#[async_trait]
pub trait PrivateInvestmentRepositoryTrait: Send + Sync {
    async fn upsert_investment(
        &self,
        request: UpsertPrivateInvestmentRequest,
    ) -> Result<PrivateInvestment>;
    async fn get_investment(&self, asset_id: &str) -> Result<Option<PrivateInvestment>>;
    async fn delete_investment(&self, asset_id: &str) -> Result<()>;
    async fn add_valuation(
        &self,
        request: CreatePrivateInvestmentValuationRequest,
    ) -> Result<PrivateInvestmentValuation>;
    async fn add_capital_call(&self, request: CreateCapitalCallRequest) -> Result<CapitalCall>;
    async fn update_capital_call_status(
        &self,
        request: UpdateCapitalCallStatusRequest,
    ) -> Result<CapitalCall>;
    async fn add_distribution(
        &self,
        request: CreatePrivateDistributionRequest,
    ) -> Result<PrivateDistribution>;
    async fn get_summary(&self, asset_id: &str) -> Result<Option<PrivateInvestmentSummary>>;
}

fn validate_asset_id(value: &str) -> Result<()> {
    validate_non_empty("asset_id", value)
}

fn validate_non_empty(field: &str, value: &str) -> Result<()> {
    if value.trim().is_empty() {
        return Err(invalid(format!("{field} is required")));
    }
    Ok(())
}

fn validate_non_negative(field: &str, value: Decimal) -> Result<()> {
    if value < Decimal::ZERO {
        return Err(invalid(format!("{field} must be non-negative")));
    }
    Ok(())
}

fn validate_currency(value: &str) -> Result<()> {
    let value = value.trim();
    if value.len() != 3 || !value.chars().all(|c| c.is_ascii_uppercase()) {
        return Err(invalid("currency must be a 3-letter ISO 4217 code"));
    }
    Ok(())
}

fn invalid(message: impl Into<String>) -> Error {
    Error::Validation(ValidationError::InvalidInput(message.into()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn investment() -> PrivateInvestment {
        PrivateInvestment {
            asset_id: "asset-1".into(),
            manager: "Acme Capital".into(),
            strategy: "Buyout".into(),
            vintage_year: Some(2024),
            commitment_amount: dec!(1000),
            commitment_currency: "USD".into(),
            inception_date: None,
            notes: None,
        }
    }

    fn call(amount: Decimal, status: CapitalCallStatus) -> CapitalCall {
        CapitalCall {
            id: "call".into(),
            asset_id: "asset-1".into(),
            notice_date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            due_date: NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
            amount,
            currency: "USD".into(),
            status,
            source_citation_id: None,
            notes: None,
        }
    }

    fn distribution(amount: Decimal, recallable: bool) -> PrivateDistribution {
        PrivateDistribution {
            id: "dist".into(),
            asset_id: "asset-1".into(),
            distribution_date: NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(),
            amount,
            currency: "USD".into(),
            recallable,
            source_citation_id: None,
            notes: None,
        }
    }

    fn valuation(nav: Decimal, day: u32) -> PrivateInvestmentValuation {
        PrivateInvestmentValuation {
            id: format!("nav-{day}"),
            asset_id: "asset-1".into(),
            valuation_date: NaiveDate::from_ymd_opt(2026, 6, day).unwrap(),
            nav,
            currency: "USD".into(),
            source_citation_id: None,
        }
    }

    #[test]
    fn commitment_math_uses_declared_commitment() {
        let summary = calculate_private_investment_summary(investment(), &[], &[], &[]);
        assert_eq!(summary.commitment, dec!(1000));
        assert_eq!(summary.unfunded_commitment, dec!(1000));
    }

    #[test]
    fn capital_call_paid_in_counts_paid_only() {
        let summary = calculate_private_investment_summary(
            investment(),
            &[],
            &[
                call(dec!(300), CapitalCallStatus::Paid),
                call(dec!(200), CapitalCallStatus::Due),
            ],
            &[],
        );
        assert_eq!(summary.paid_in_capital, dec!(300));
    }

    #[test]
    fn unfunded_commitment_subtracts_paid_in() {
        let summary = calculate_private_investment_summary(
            investment(),
            &[],
            &[call(dec!(400), CapitalCallStatus::Paid)],
            &[],
        );
        assert_eq!(summary.unfunded_commitment, dec!(600));
    }

    #[test]
    fn recallable_distribution_increases_unfunded_commitment() {
        let summary = calculate_private_investment_summary(
            investment(),
            &[],
            &[call(dec!(400), CapitalCallStatus::Paid)],
            &[distribution(dec!(50), true), distribution(dec!(25), false)],
        );
        assert_eq!(summary.total_distributions, dec!(75));
        assert_eq!(summary.recallable_distributions, dec!(50));
        assert_eq!(summary.unfunded_commitment, dec!(650));
    }

    #[test]
    fn private_fund_ratios_use_paid_in_denominator() {
        let summary = calculate_private_investment_summary(
            investment(),
            &[valuation(dec!(300), 1), valuation(dec!(450), 30)],
            &[call(dec!(500), CapitalCallStatus::Paid)],
            &[distribution(dec!(100), false)],
        );
        assert_eq!(summary.current_nav, dec!(450));
        assert_eq!(summary.dpi, Some(dec!(0.2)));
        assert_eq!(summary.rvpi, Some(dec!(0.9)));
        assert_eq!(summary.tvpi, Some(dec!(1.1)));
        assert_eq!(summary.moic, Some(dec!(1.1)));
    }
}
