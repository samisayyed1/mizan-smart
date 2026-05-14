//! Typed asset detail structs.
//!
//! One struct per typed extension table. Subtype enums (FixedIncome,
//! private investment, insurance, …) mirror the SQL CHECK constraints
//! in `2026-05-14-000002_universal_asset_model/up.sql` exactly. Each
//! enum provides `as_str` / `parse` so the storage layer can map
//! to/from the persisted snake_case form without ad-hoc strings
//! sprinkled through the codebase.
//!
//! Fields are deliberately optional even where the spec suggests
//! "required" — the universal Add Asset flow (Phase 1 P5) allows
//! partial fills with explicit "I don't know yet" affordances, and
//! the alert engine (Phase 1 P8) surfaces incomplete-setup warnings
//! later. Refusing partial rows at the schema level would block users
//! from saving until every blank is filled in, which the spec
//! explicitly does not want.

use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::fmt;

use crate::errors::{Error, Result, ValidationError};

// ============================================================================
// Public market (equity / ETF / mutual fund)
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PublicMarketSubClass {
    PublicEquity,
    Etf,
    MutualFund,
}

impl PublicMarketSubClass {
    pub const fn as_str(self) -> &'static str {
        match self {
            PublicMarketSubClass::PublicEquity => "public_equity",
            PublicMarketSubClass::Etf => "etf",
            PublicMarketSubClass::MutualFund => "mutual_fund",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "public_equity" => Some(PublicMarketSubClass::PublicEquity),
            "etf" => Some(PublicMarketSubClass::Etf),
            "mutual_fund" => Some(PublicMarketSubClass::MutualFund),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublicMarketDetails {
    pub asset_id: String,
    pub sub_class: Option<PublicMarketSubClass>,
    pub isin: Option<String>,
    pub cusip: Option<String>,
    pub figi: Option<String>,
    pub expense_ratio_bps: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ============================================================================
// Fixed income (bond / sukuk / FD / CD / structured note / treasury bill)
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FixedIncomeSubtype {
    Bond,
    Sukuk,
    TreasuryBill,
    FixedDeposit,
    Cd,
    StructuredNote,
    Other,
}

impl FixedIncomeSubtype {
    pub const fn as_str(self) -> &'static str {
        match self {
            FixedIncomeSubtype::Bond => "bond",
            FixedIncomeSubtype::Sukuk => "sukuk",
            FixedIncomeSubtype::TreasuryBill => "treasury_bill",
            FixedIncomeSubtype::FixedDeposit => "fixed_deposit",
            FixedIncomeSubtype::Cd => "cd",
            FixedIncomeSubtype::StructuredNote => "structured_note",
            FixedIncomeSubtype::Other => "other",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "bond" => Some(FixedIncomeSubtype::Bond),
            "sukuk" => Some(FixedIncomeSubtype::Sukuk),
            "treasury_bill" => Some(FixedIncomeSubtype::TreasuryBill),
            "fixed_deposit" => Some(FixedIncomeSubtype::FixedDeposit),
            "cd" => Some(FixedIncomeSubtype::Cd),
            "structured_note" => Some(FixedIncomeSubtype::StructuredNote),
            "other" => Some(FixedIncomeSubtype::Other),
            _ => None,
        }
    }
}

/// ISO day-count conventions used for accrual maths in Phase 1 P19.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DayCountConvention {
    #[serde(rename = "ACT_360")]
    Act360,
    #[serde(rename = "ACT_365")]
    Act365,
    #[serde(rename = "ACT_ACT")]
    ActAct,
    #[serde(rename = "THIRTY_360")]
    Thirty360,
}

impl DayCountConvention {
    pub const fn as_str(self) -> &'static str {
        match self {
            DayCountConvention::Act360 => "ACT_360",
            DayCountConvention::Act365 => "ACT_365",
            DayCountConvention::ActAct => "ACT_ACT",
            DayCountConvention::Thirty360 => "THIRTY_360",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "ACT_360" => Some(DayCountConvention::Act360),
            "ACT_365" => Some(DayCountConvention::Act365),
            "ACT_ACT" => Some(DayCountConvention::ActAct),
            "THIRTY_360" => Some(DayCountConvention::Thirty360),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FixedIncomeDetails {
    pub asset_id: String,
    pub instrument_subtype: FixedIncomeSubtype,
    pub issuer: Option<String>,
    pub isin: Option<String>,
    pub face_value: Option<Decimal>,
    pub currency: Option<String>,
    pub purchase_date: Option<NaiveDate>,
    pub maturity_date: Option<NaiveDate>,
    pub coupon_or_profit_rate: Option<Decimal>,
    pub payment_frequency: Option<String>,
    pub day_count_convention: Option<DayCountConvention>,
    pub is_sukuk: bool,
    pub source_citation_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ============================================================================
// Real estate
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RealEstateSizeUnit {
    Sqft,
    Sqm,
    Acre,
    Hectare,
}

impl RealEstateSizeUnit {
    pub const fn as_str(self) -> &'static str {
        match self {
            RealEstateSizeUnit::Sqft => "sqft",
            RealEstateSizeUnit::Sqm => "sqm",
            RealEstateSizeUnit::Acre => "acre",
            RealEstateSizeUnit::Hectare => "hectare",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "sqft" => Some(RealEstateSizeUnit::Sqft),
            "sqm" => Some(RealEstateSizeUnit::Sqm),
            "acre" => Some(RealEstateSizeUnit::Acre),
            "hectare" => Some(RealEstateSizeUnit::Hectare),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RealEstateDetails {
    pub asset_id: String,
    pub property_type: Option<String>,
    pub address_approximate: Option<String>,
    pub address_exact: Option<String>,
    pub size_value: Option<Decimal>,
    pub size_unit: Option<RealEstateSizeUnit>,
    pub bedrooms: Option<i32>,
    pub purchase_date: Option<NaiveDate>,
    pub purchase_price: Option<Decimal>,
    pub purchase_currency: Option<String>,
    pub source_citation_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ============================================================================
// Private investments (PE / private credit / hedge fund / VC)
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrivateInvestmentSubtype {
    PrivateEquity,
    PrivateCredit,
    HedgeFund,
    VentureCapital,
}

impl PrivateInvestmentSubtype {
    pub const fn as_str(self) -> &'static str {
        match self {
            PrivateInvestmentSubtype::PrivateEquity => "private_equity",
            PrivateInvestmentSubtype::PrivateCredit => "private_credit",
            PrivateInvestmentSubtype::HedgeFund => "hedge_fund",
            PrivateInvestmentSubtype::VentureCapital => "venture_capital",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "private_equity" => Some(PrivateInvestmentSubtype::PrivateEquity),
            "private_credit" => Some(PrivateInvestmentSubtype::PrivateCredit),
            "hedge_fund" => Some(PrivateInvestmentSubtype::HedgeFund),
            "venture_capital" => Some(PrivateInvestmentSubtype::VentureCapital),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrivateInvestmentDetails {
    pub asset_id: String,
    pub instrument_subtype: PrivateInvestmentSubtype,
    pub manager: Option<String>,
    pub strategy: Option<String>,
    pub vintage_year: Option<i32>,
    pub commitment_amount: Option<Decimal>,
    pub commitment_currency: Option<String>,
    pub inception_date: Option<NaiveDate>,
    pub notes: Option<String>,
    pub source_citation_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ============================================================================
// Insurance / ULIP / Pension
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InsurancePolicyType {
    Insurance,
    Ulip,
    Pension,
}

impl InsurancePolicyType {
    pub const fn as_str(self) -> &'static str {
        match self {
            InsurancePolicyType::Insurance => "insurance",
            InsurancePolicyType::Ulip => "ulip",
            InsurancePolicyType::Pension => "pension",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "insurance" => Some(InsurancePolicyType::Insurance),
            "ulip" => Some(InsurancePolicyType::Ulip),
            "pension" => Some(InsurancePolicyType::Pension),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InsuranceDetails {
    pub asset_id: String,
    pub policy_type: InsurancePolicyType,
    pub provider: Option<String>,
    pub policy_number_hash: Option<String>,
    pub start_date: Option<NaiveDate>,
    pub maturity_date: Option<NaiveDate>,
    pub premium_amount: Option<Decimal>,
    pub premium_currency: Option<String>,
    pub payment_frequency: Option<String>,
    pub source_citation_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ============================================================================
// Commodity (gold / silver / platinum / palladium / other)
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CommodityType {
    Gold,
    Silver,
    Platinum,
    Palladium,
    OtherCommodity,
}

impl CommodityType {
    pub const fn as_str(self) -> &'static str {
        match self {
            CommodityType::Gold => "gold",
            CommodityType::Silver => "silver",
            CommodityType::Platinum => "platinum",
            CommodityType::Palladium => "palladium",
            CommodityType::OtherCommodity => "other_commodity",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "gold" => Some(CommodityType::Gold),
            "silver" => Some(CommodityType::Silver),
            "platinum" => Some(CommodityType::Platinum),
            "palladium" => Some(CommodityType::Palladium),
            "other_commodity" => Some(CommodityType::OtherCommodity),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommodityDetails {
    pub asset_id: String,
    pub commodity_type: CommodityType,
    pub weight_value: Option<Decimal>,
    pub weight_unit: Option<String>,
    pub purity: Option<String>,
    pub storage_location: Option<String>,
    pub source_citation_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ============================================================================
// Business
// ============================================================================

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BusinessDetails {
    pub asset_id: String,
    pub business_name: Option<String>,
    pub ownership_percent: Option<Decimal>,
    pub legal_form: Option<String>,
    pub country: Option<String>,
    pub incorporation_date: Option<NaiveDate>,
    pub notes: Option<String>,
    pub source_citation_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl BusinessDetails {
    /// Reject ownership percentages outside `0..=100`. The schema lets
    /// the column be free-form decimal so this check enforces the
    /// constraint at the domain boundary.
    pub fn validate(&self) -> Result<()> {
        if let Some(percent) = self.ownership_percent {
            if percent < Decimal::ZERO || percent > Decimal::from(100) {
                return Err(Error::Validation(ValidationError::InvalidInput(format!(
                    "ownership_percent {} must be between 0 and 100",
                    percent
                ))));
            }
        }
        Ok(())
    }
}

// ============================================================================
// Collectible
// ============================================================================

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CollectibleDetails {
    pub asset_id: String,
    pub collectible_type: Option<String>,
    pub maker: Option<String>,
    pub model_reference: Option<String>,
    pub year: Option<i32>,
    pub condition: Option<String>,
    pub has_box: bool,
    pub has_papers: bool,
    pub source_citation_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ============================================================================
// Liability
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LiabilityType {
    Mortgage,
    Loan,
    CreditCard,
    LineOfCredit,
    OtherLiability,
}

impl LiabilityType {
    pub const fn as_str(self) -> &'static str {
        match self {
            LiabilityType::Mortgage => "mortgage",
            LiabilityType::Loan => "loan",
            LiabilityType::CreditCard => "credit_card",
            LiabilityType::LineOfCredit => "line_of_credit",
            LiabilityType::OtherLiability => "other_liability",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "mortgage" => Some(LiabilityType::Mortgage),
            "loan" => Some(LiabilityType::Loan),
            "credit_card" => Some(LiabilityType::CreditCard),
            "line_of_credit" => Some(LiabilityType::LineOfCredit),
            "other_liability" => Some(LiabilityType::OtherLiability),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LiabilityDetails {
    pub asset_id: String,
    pub liability_type: LiabilityType,
    pub lender: Option<String>,
    pub principal_original: Option<Decimal>,
    pub principal_currency: Option<String>,
    pub interest_rate: Option<Decimal>,
    pub interest_compounding: Option<String>,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub linked_asset_id: Option<String>,
    pub source_citation_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ============================================================================
// Display impls (debugging + log lines)
// ============================================================================

impl fmt::Display for FixedIncomeSubtype {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl fmt::Display for PrivateInvestmentSubtype {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl fmt::Display for InsurancePolicyType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl fmt::Display for CommodityType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl fmt::Display for LiabilityType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl fmt::Display for PublicMarketSubClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl fmt::Display for DayCountConvention {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl fmt::Display for RealEstateSizeUnit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    /// Cover every subtype enum: each variant round-trips through
    /// `as_str` / `parse` and the JSON form matches the snake_case
    /// SQL value. This is the contract the SQL CHECK constraints rely
    /// on — drift here is a migration-breaking change.
    #[test]
    fn fixed_income_subtypes_round_trip() {
        for v in [
            FixedIncomeSubtype::Bond,
            FixedIncomeSubtype::Sukuk,
            FixedIncomeSubtype::TreasuryBill,
            FixedIncomeSubtype::FixedDeposit,
            FixedIncomeSubtype::Cd,
            FixedIncomeSubtype::StructuredNote,
            FixedIncomeSubtype::Other,
        ] {
            assert_eq!(FixedIncomeSubtype::parse(v.as_str()), Some(v));
        }
    }

    #[test]
    fn private_investment_subtypes_round_trip() {
        for v in [
            PrivateInvestmentSubtype::PrivateEquity,
            PrivateInvestmentSubtype::PrivateCredit,
            PrivateInvestmentSubtype::HedgeFund,
            PrivateInvestmentSubtype::VentureCapital,
        ] {
            assert_eq!(PrivateInvestmentSubtype::parse(v.as_str()), Some(v));
        }
    }

    #[test]
    fn insurance_policy_types_round_trip() {
        for v in [
            InsurancePolicyType::Insurance,
            InsurancePolicyType::Ulip,
            InsurancePolicyType::Pension,
        ] {
            assert_eq!(InsurancePolicyType::parse(v.as_str()), Some(v));
        }
    }

    #[test]
    fn commodity_types_round_trip() {
        for v in [
            CommodityType::Gold,
            CommodityType::Silver,
            CommodityType::Platinum,
            CommodityType::Palladium,
            CommodityType::OtherCommodity,
        ] {
            assert_eq!(CommodityType::parse(v.as_str()), Some(v));
        }
    }

    #[test]
    fn liability_types_round_trip() {
        for v in [
            LiabilityType::Mortgage,
            LiabilityType::Loan,
            LiabilityType::CreditCard,
            LiabilityType::LineOfCredit,
            LiabilityType::OtherLiability,
        ] {
            assert_eq!(LiabilityType::parse(v.as_str()), Some(v));
        }
    }

    #[test]
    fn day_count_conventions_round_trip() {
        for v in [
            DayCountConvention::Act360,
            DayCountConvention::Act365,
            DayCountConvention::ActAct,
            DayCountConvention::Thirty360,
        ] {
            assert_eq!(DayCountConvention::parse(v.as_str()), Some(v));
        }
    }

    #[test]
    fn public_market_subclass_round_trips() {
        for v in [
            PublicMarketSubClass::PublicEquity,
            PublicMarketSubClass::Etf,
            PublicMarketSubClass::MutualFund,
        ] {
            assert_eq!(PublicMarketSubClass::parse(v.as_str()), Some(v));
        }
    }

    #[test]
    fn real_estate_size_units_round_trip() {
        for v in [
            RealEstateSizeUnit::Sqft,
            RealEstateSizeUnit::Sqm,
            RealEstateSizeUnit::Acre,
            RealEstateSizeUnit::Hectare,
        ] {
            assert_eq!(RealEstateSizeUnit::parse(v.as_str()), Some(v));
        }
    }

    #[test]
    fn fixed_income_subtype_parse_rejects_unknown_strings() {
        assert_eq!(FixedIncomeSubtype::parse(""), None);
        assert_eq!(FixedIncomeSubtype::parse("BOND"), None);
        assert_eq!(FixedIncomeSubtype::parse("loan"), None);
    }

    #[test]
    fn private_investment_subtype_parse_rejects_unknown_strings() {
        assert_eq!(PrivateInvestmentSubtype::parse(""), None);
        assert_eq!(PrivateInvestmentSubtype::parse("PE"), None);
    }

    #[test]
    fn business_validate_accepts_zero_and_hundred() {
        let now = chrono::Utc::now();
        for percent in [Decimal::ZERO, dec!(50), dec!(100)] {
            let d = BusinessDetails {
                asset_id: "a1".into(),
                business_name: Some("Co".into()),
                ownership_percent: Some(percent),
                legal_form: None,
                country: None,
                incorporation_date: None,
                notes: None,
                source_citation_id: None,
                created_at: now,
                updated_at: now,
            };
            assert!(d.validate().is_ok(), "percent={} should validate", percent);
        }
    }

    #[test]
    fn business_validate_rejects_out_of_range_ownership() {
        let now = chrono::Utc::now();
        for bad in [dec!(-0.01), dec!(100.01), dec!(150), dec!(-1)] {
            let d = BusinessDetails {
                asset_id: "a1".into(),
                business_name: Some("Co".into()),
                ownership_percent: Some(bad),
                legal_form: None,
                country: None,
                incorporation_date: None,
                notes: None,
                source_citation_id: None,
                created_at: now,
                updated_at: now,
            };
            assert!(d.validate().is_err(), "{} should be rejected", bad);
        }
    }

    #[test]
    fn day_count_convention_json_uses_screaming_snake_case() {
        let json = serde_json::to_string(&DayCountConvention::Act360).unwrap();
        assert_eq!(json, "\"ACT_360\"");
    }
}
