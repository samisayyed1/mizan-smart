use async_trait::async_trait;
use chrono::{Datelike, Months, NaiveDate};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::errors::{Error, ValidationError};
use crate::universal_assets::details::{DayCountConvention, FixedIncomeSubtype};
use crate::Result;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FixedIncomePaymentFrequency {
    Monthly,
    Quarterly,
    SemiAnnual,
    Annual,
    AtMaturity,
}

impl FixedIncomePaymentFrequency {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Monthly => "monthly",
            Self::Quarterly => "quarterly",
            Self::SemiAnnual => "semi_annual",
            Self::Annual => "annual",
            Self::AtMaturity => "at_maturity",
        }
    }

    pub const fn months_between(self) -> Option<u32> {
        match self {
            Self::Monthly => Some(1),
            Self::Quarterly => Some(3),
            Self::SemiAnnual => Some(6),
            Self::Annual => Some(12),
            Self::AtMaturity => None,
        }
    }

    pub const fn payments_per_year(self) -> u32 {
        match self {
            Self::Monthly => 12,
            Self::Quarterly => 4,
            Self::SemiAnnual => 2,
            Self::Annual | Self::AtMaturity => 1,
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "monthly" => Some(Self::Monthly),
            "quarterly" => Some(Self::Quarterly),
            "semi_annual" => Some(Self::SemiAnnual),
            "annual" => Some(Self::Annual),
            "at_maturity" => Some(Self::AtMaturity),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FixedIncomeCashflowType {
    Coupon,
    Profit,
    Principal,
    Maturity,
    Interest,
}

impl FixedIncomeCashflowType {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Coupon => "coupon",
            Self::Profit => "profit",
            Self::Principal => "principal",
            Self::Maturity => "maturity",
            Self::Interest => "interest",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "coupon" => Some(Self::Coupon),
            "profit" => Some(Self::Profit),
            "principal" => Some(Self::Principal),
            "maturity" => Some(Self::Maturity),
            "interest" => Some(Self::Interest),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FixedIncomeCashflowStatus {
    Expected,
    Received,
    Missed,
    Cancelled,
}

impl FixedIncomeCashflowStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Expected => "expected",
            Self::Received => "received",
            Self::Missed => "missed",
            Self::Cancelled => "cancelled",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "expected" => Some(Self::Expected),
            "received" => Some(Self::Received),
            "missed" => Some(Self::Missed),
            "cancelled" => Some(Self::Cancelled),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FixedIncomeDetails {
    pub asset_id: String,
    pub instrument_type: FixedIncomeSubtype,
    pub issuer: String,
    pub isin: Option<String>,
    pub face_value: Decimal,
    pub currency: String,
    pub purchase_date: Option<NaiveDate>,
    pub maturity_date: NaiveDate,
    pub coupon_or_profit_rate: Option<Decimal>,
    pub payment_frequency: Option<FixedIncomePaymentFrequency>,
    pub day_count_convention: DayCountConvention,
    pub is_sukuk: bool,
    pub source_citation_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FixedIncomeCashflow {
    pub id: String,
    pub asset_id: String,
    pub expected_date: NaiveDate,
    pub cashflow_type: FixedIncomeCashflowType,
    pub expected_amount: Decimal,
    pub actual_amount: Option<Decimal>,
    pub currency: String,
    pub status: FixedIncomeCashflowStatus,
    pub source_citation_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectedFixedIncomeCashflow {
    pub expected_date: NaiveDate,
    pub cashflow_type: FixedIncomeCashflowType,
    pub expected_amount: Decimal,
    pub currency: String,
    pub source_citation_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FixedIncomeProjection {
    pub details: FixedIncomeDetails,
    pub accrued_amount: Decimal,
    pub cashflows: Vec<FixedIncomeCashflow>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpsertFixedIncomeDetailsRequest {
    pub asset_id: String,
    pub instrument_type: FixedIncomeSubtype,
    pub issuer: String,
    pub isin: Option<String>,
    pub face_value: Decimal,
    pub currency: String,
    pub purchase_date: Option<NaiveDate>,
    pub maturity_date: NaiveDate,
    pub coupon_or_profit_rate: Option<Decimal>,
    pub payment_frequency: Option<FixedIncomePaymentFrequency>,
    pub day_count_convention: DayCountConvention,
    pub is_sukuk: bool,
    pub source_citation_id: Option<String>,
}

impl UpsertFixedIncomeDetailsRequest {
    pub fn validate(&self) -> Result<()> {
        validate_non_empty("asset_id", &self.asset_id)?;
        validate_non_empty("issuer", &self.issuer)?;
        validate_currency(&self.currency)?;
        if self.face_value <= Decimal::ZERO {
            return Err(invalid("face_value must be positive"));
        }
        if let Some(rate) = self.coupon_or_profit_rate {
            if rate < Decimal::ZERO {
                return Err(invalid("coupon_or_profit_rate must be non-negative"));
            }
        }
        if let Some(purchase_date) = self.purchase_date {
            if purchase_date >= self.maturity_date {
                return Err(invalid("purchase_date must be before maturity_date"));
            }
        }
        Ok(())
    }

    pub fn into_domain(self) -> FixedIncomeDetails {
        FixedIncomeDetails {
            asset_id: self.asset_id,
            instrument_type: self.instrument_type,
            issuer: self.issuer.trim().to_string(),
            isin: self.isin,
            face_value: self.face_value,
            currency: self.currency.trim().to_uppercase(),
            purchase_date: self.purchase_date,
            maturity_date: self.maturity_date,
            coupon_or_profit_rate: self.coupon_or_profit_rate,
            payment_frequency: self.payment_frequency,
            day_count_convention: self.day_count_convention,
            is_sukuk: self.is_sukuk,
            source_citation_id: self.source_citation_id,
        }
    }
}

pub fn day_count_fraction(
    start: NaiveDate,
    end: NaiveDate,
    convention: DayCountConvention,
) -> Decimal {
    if end <= start {
        return Decimal::ZERO;
    }
    match convention {
        DayCountConvention::Act360 => Decimal::from((end - start).num_days()) / Decimal::from(360),
        DayCountConvention::Act365 => Decimal::from((end - start).num_days()) / Decimal::from(365),
        DayCountConvention::Thirty360 => {
            Decimal::from(thirty_360_days(start, end)) / Decimal::from(360)
        }
        DayCountConvention::ActAct => actual_actual_fraction(start, end),
    }
}

pub fn accrued_interest_or_profit(
    face_value: Decimal,
    annual_rate: Decimal,
    accrual_start: NaiveDate,
    accrual_end: NaiveDate,
    convention: DayCountConvention,
) -> Decimal {
    face_value * annual_rate * day_count_fraction(accrual_start, accrual_end, convention)
}

pub fn generate_projected_cashflows(
    details: &FixedIncomeDetails,
) -> Result<(Vec<ProjectedFixedIncomeCashflow>, Vec<String>)> {
    let mut warnings = Vec::new();
    if details.purchase_date.is_none() {
        warnings.push(
            "Purchase date is missing; accrual starts at maturity schedule boundary only."
                .to_string(),
        );
    }
    if details.coupon_or_profit_rate.is_some() && details.payment_frequency.is_none() {
        warnings.push("Payment frequency is missing; only principal or at-maturity cashflow can be projected.".to_string());
    }

    let mut cashflows = Vec::new();
    let start = details.purchase_date.unwrap_or(details.maturity_date);
    let rate = details.coupon_or_profit_rate.unwrap_or(Decimal::ZERO);
    let source_citation_id = details.source_citation_id.clone();

    if details.instrument_type == FixedIncomeSubtype::FixedDeposit
        || details.payment_frequency == Some(FixedIncomePaymentFrequency::AtMaturity)
    {
        let interest = accrued_interest_or_profit(
            details.face_value,
            rate,
            start,
            details.maturity_date,
            details.day_count_convention,
        );
        cashflows.push(ProjectedFixedIncomeCashflow {
            expected_date: details.maturity_date,
            cashflow_type: FixedIncomeCashflowType::Maturity,
            expected_amount: details.face_value + interest,
            currency: details.currency.clone(),
            source_citation_id,
        });
        return Ok((cashflows, warnings));
    }

    if let (Some(frequency), Some(_rate), Some(purchase_date)) = (
        details.payment_frequency,
        details.coupon_or_profit_rate,
        details.purchase_date,
    ) {
        if let Some(months_between) = frequency.months_between() {
            let payment_amount =
                details.face_value * rate / Decimal::from(frequency.payments_per_year());
            let mut months = months_between;
            while let Some(payment_date) = add_months(purchase_date, months) {
                if payment_date >= details.maturity_date {
                    break;
                }
                cashflows.push(ProjectedFixedIncomeCashflow {
                    expected_date: payment_date,
                    cashflow_type: if details.is_sukuk {
                        FixedIncomeCashflowType::Profit
                    } else {
                        FixedIncomeCashflowType::Coupon
                    },
                    expected_amount: payment_amount,
                    currency: details.currency.clone(),
                    source_citation_id: details.source_citation_id.clone(),
                });
                months += months_between;
            }
        }
    }

    cashflows.push(ProjectedFixedIncomeCashflow {
        expected_date: details.maturity_date,
        cashflow_type: FixedIncomeCashflowType::Principal,
        expected_amount: details.face_value,
        currency: details.currency.clone(),
        source_citation_id: details.source_citation_id.clone(),
    });
    Ok((cashflows, warnings))
}

#[async_trait]
pub trait FixedIncomeRepositoryTrait: Send + Sync {
    async fn upsert_details(
        &self,
        request: UpsertFixedIncomeDetailsRequest,
    ) -> Result<FixedIncomeProjection>;
    async fn get_projection(&self, asset_id: &str) -> Result<Option<FixedIncomeProjection>>;
}

fn add_months(start: NaiveDate, months: u32) -> Option<NaiveDate> {
    start.checked_add_months(Months::new(months))
}

fn thirty_360_days(start: NaiveDate, end: NaiveDate) -> i32 {
    let d1 = start.day().min(30) as i32;
    let d2 = if d1 == 30 {
        end.day().min(30)
    } else {
        end.day()
    } as i32;
    ((end.year() - start.year()) * 360) + ((end.month() as i32 - start.month() as i32) * 30) + d2
        - d1
}

fn actual_actual_fraction(start: NaiveDate, end: NaiveDate) -> Decimal {
    let mut cursor = start;
    let mut total = Decimal::ZERO;
    while cursor < end {
        let next_year = NaiveDate::from_ymd_opt(cursor.year() + 1, 1, 1).expect("valid year");
        let period_end = end.min(next_year);
        let days = Decimal::from((period_end - cursor).num_days());
        let year_days = if is_leap_year(cursor.year()) {
            366
        } else {
            365
        };
        total += days / Decimal::from(year_days);
        cursor = period_end;
    }
    total
}

fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

fn validate_non_empty(field: &str, value: &str) -> Result<()> {
    if value.trim().is_empty() {
        return Err(invalid(format!("{field} is required")));
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

    fn date(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day).unwrap()
    }

    fn details(is_sukuk: bool) -> FixedIncomeDetails {
        FixedIncomeDetails {
            asset_id: "asset-1".into(),
            instrument_type: if is_sukuk {
                FixedIncomeSubtype::Sukuk
            } else {
                FixedIncomeSubtype::Bond
            },
            issuer: "Treasury".into(),
            isin: None,
            face_value: dec!(1000),
            currency: "USD".into(),
            purchase_date: Some(date(2026, 1, 1)),
            maturity_date: date(2027, 1, 1),
            coupon_or_profit_rate: Some(dec!(0.06)),
            payment_frequency: Some(FixedIncomePaymentFrequency::SemiAnnual),
            day_count_convention: DayCountConvention::Act365,
            is_sukuk,
            source_citation_id: None,
        }
    }

    #[test]
    fn act_360_fraction_uses_actual_days_over_360() {
        assert_eq!(
            day_count_fraction(
                date(2026, 1, 1),
                date(2026, 7, 1),
                DayCountConvention::Act360
            ),
            Decimal::from(181) / Decimal::from(360)
        );
    }

    #[test]
    fn act_365_fraction_uses_actual_days_over_365() {
        assert_eq!(
            day_count_fraction(
                date(2026, 1, 1),
                date(2026, 7, 1),
                DayCountConvention::Act365
            ),
            Decimal::from(181) / Decimal::from(365)
        );
    }

    #[test]
    fn thirty_360_fraction_uses_bond_calendar() {
        assert_eq!(
            day_count_fraction(
                date(2026, 1, 30),
                date(2026, 2, 28),
                DayCountConvention::Thirty360
            ),
            dec!(0.0777777777777777777777777778)
        );
    }

    #[test]
    fn act_act_splits_leap_and_non_leap_years() {
        let fraction = day_count_fraction(
            date(2024, 7, 1),
            date(2025, 7, 1),
            DayCountConvention::ActAct,
        );
        assert_eq!(fraction.round_dp(6), dec!(0.998623));
    }

    #[test]
    fn coupon_schedule_uses_coupon_label_for_conventional_bond() {
        let (cashflows, warnings) = generate_projected_cashflows(&details(false)).unwrap();
        assert!(warnings.is_empty());
        assert_eq!(cashflows[0].cashflow_type, FixedIncomeCashflowType::Coupon);
        assert_eq!(cashflows[0].expected_amount, dec!(30.00));
        assert_eq!(
            cashflows[1].cashflow_type,
            FixedIncomeCashflowType::Principal
        );
    }

    #[test]
    fn sukuk_schedule_uses_profit_label() {
        let (cashflows, _) = generate_projected_cashflows(&details(true)).unwrap();
        assert_eq!(cashflows[0].cashflow_type, FixedIncomeCashflowType::Profit);
    }

    #[test]
    fn fixed_deposit_projects_single_maturity_cashflow() {
        let mut fd = details(false);
        fd.instrument_type = FixedIncomeSubtype::FixedDeposit;
        fd.payment_frequency = Some(FixedIncomePaymentFrequency::AtMaturity);
        let (cashflows, _) = generate_projected_cashflows(&fd).unwrap();
        assert_eq!(cashflows.len(), 1);
        assert_eq!(
            cashflows[0].cashflow_type,
            FixedIncomeCashflowType::Maturity
        );
        assert_eq!(cashflows[0].expected_amount, dec!(1060.00));
    }

    #[test]
    fn incomplete_setup_reports_warning() {
        let mut incomplete = details(false);
        incomplete.payment_frequency = None;
        let (_, warnings) = generate_projected_cashflows(&incomplete).unwrap();
        assert!(warnings
            .iter()
            .any(|warning| warning.contains("Payment frequency")));
    }
}
