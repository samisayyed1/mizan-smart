use crate::errors::ValidationError;
use async_trait::async_trait;
use chrono::NaiveDate;
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
    pub notes: Option<String>,
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpsertAssetShariahScreeningRequest {
    pub asset_id: String,
    pub profile_id: String,
    pub ratios: ShariahScreeningRatios,
    pub source_citation_id: Option<String>,
    pub notes: Option<String>,
    pub manual_override_status: Option<ShariahScreeningStatus>,
    pub manual_override_reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShariahScreeningAuditEntry {
    pub id: String,
    pub screening_id: String,
    pub asset_id: String,
    pub profile_id: String,
    pub previous_status: Option<ShariahScreeningStatus>,
    pub new_status: ShariahScreeningStatus,
    pub notes: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ZakatInputLine {
    pub asset_id: Option<String>,
    pub category: String,
    pub amount: Option<Decimal>,
    pub included: bool,
    pub explanation: Option<String>,
    pub source_citation_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CalculateZakatSnapshotRequest {
    pub snapshot_date: NaiveDate,
    pub base_currency: String,
    pub nisab_value: Decimal,
    pub notes: Option<String>,
    pub lines: Vec<ZakatInputLine>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ZakatLine {
    pub id: String,
    pub snapshot_id: String,
    pub asset_id: Option<String>,
    pub category: String,
    pub amount: Decimal,
    pub included: bool,
    pub explanation: String,
    pub source_citation_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ZakatSnapshot {
    pub id: String,
    pub snapshot_date: NaiveDate,
    pub base_currency: String,
    pub total_zakatable_assets: Decimal,
    pub deductible_liabilities: Decimal,
    pub net_zakatable_wealth: Decimal,
    pub nisab_value: Decimal,
    pub zakat_due: Decimal,
    pub notes: Option<String>,
    pub created_at: String,
    pub lines: Vec<ZakatLine>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PurificationCalculationMethod {
    ImpureIncomePerShare,
    DividendRatio,
    NeedsReview,
}

impl PurificationCalculationMethod {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ImpureIncomePerShare => "impure_income_per_share",
            Self::DividendRatio => "dividend_ratio",
            Self::NeedsReview => "needs_review",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "impure_income_per_share" => Some(Self::ImpureIncomePerShare),
            "dividend_ratio" => Some(Self::DividendRatio),
            "needs_review" => Some(Self::NeedsReview),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PurificationStatus {
    Calculated,
    Paid,
    Waived,
}

impl PurificationStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Calculated => "calculated",
            Self::Paid => "paid",
            Self::Waived => "waived",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "calculated" => Some(Self::Calculated),
            "paid" => Some(Self::Paid),
            "waived" => Some(Self::Waived),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpsertPurificationEntryRequest {
    pub asset_id: String,
    pub period_start: NaiveDate,
    pub period_end: NaiveDate,
    pub total_impure_income: Option<Decimal>,
    pub outstanding_shares: Option<Decimal>,
    pub user_shares: Option<Decimal>,
    pub dividend_received: Option<Decimal>,
    pub impure_income_ratio: Option<Decimal>,
    pub status: Option<PurificationStatus>,
    pub source_citation_id: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PurificationEntry {
    pub id: String,
    pub asset_id: String,
    pub period_start: NaiveDate,
    pub period_end: NaiveDate,
    pub total_impure_income: Option<Decimal>,
    pub outstanding_shares: Option<Decimal>,
    pub user_shares: Option<Decimal>,
    pub dividend_received: Option<Decimal>,
    pub impure_income_ratio: Option<Decimal>,
    pub purification_amount: Decimal,
    pub calculation_method: PurificationCalculationMethod,
    pub status: PurificationStatus,
    pub source_citation_id: Option<String>,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PurificationPeriodSummary {
    pub period_start: NaiveDate,
    pub period_end: NaiveDate,
    pub total_calculated: Decimal,
    pub total_paid: Decimal,
    pub entries: Vec<PurificationEntry>,
}

#[async_trait]
pub trait ShariahScreeningRepositoryTrait: Send + Sync {
    fn list_profiles(&self) -> Result<Vec<ShariahScreeningProfile>>;

    fn get_default_profile(&self) -> Result<ShariahScreeningProfile>;

    fn get_profile(&self, profile_id: &str) -> Result<ShariahScreeningProfile>;

    fn get_asset_screening(&self, asset_id: &str) -> Result<Option<AssetShariahScreening>>;

    fn get_asset_screening_for_profile(
        &self,
        asset_id: &str,
        profile_id: &str,
    ) -> Result<Option<AssetShariahScreening>>;

    async fn upsert_asset_screening(
        &self,
        request: UpsertAssetShariahScreeningRequest,
    ) -> Result<AssetShariahScreening>;

    fn list_screening_audit(
        &self,
        asset_id: &str,
        profile_id: &str,
    ) -> Result<Vec<ShariahScreeningAuditEntry>>;

    async fn calculate_zakat_snapshot(
        &self,
        request: CalculateZakatSnapshotRequest,
    ) -> Result<ZakatSnapshot>;

    async fn upsert_purification_entry(
        &self,
        request: UpsertPurificationEntryRequest,
    ) -> Result<PurificationEntry>;

    async fn mark_purification_paid(&self, entry_id: &str) -> Result<PurificationEntry>;

    fn list_purification_entries(
        &self,
        period_start: NaiveDate,
        period_end: NaiveDate,
    ) -> Result<Vec<PurificationEntry>>;

    fn purification_period_summary(
        &self,
        period_start: NaiveDate,
        period_end: NaiveDate,
    ) -> Result<PurificationPeriodSummary>;
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

pub fn validate_shariah_mode_enabled(enabled: bool) -> Result<()> {
    if enabled {
        Ok(())
    } else {
        Err(ValidationError::InvalidInput(
            "Islamic finance tools are disabled for this profile".to_string(),
        )
        .into())
    }
}

pub fn evaluate_screening_request(
    profile: &ShariahScreeningProfile,
    request: &UpsertAssetShariahScreeningRequest,
) -> Result<ShariahScreeningEvaluation> {
    validate_screening_request(request)?;
    if let Some(status) = request.manual_override_status {
        return Ok(ShariahScreeningEvaluation {
            status,
            missing_fields: Vec::new(),
        });
    }
    Ok(evaluate_shariah_screening(profile, &request.ratios))
}

pub fn validate_screening_request(request: &UpsertAssetShariahScreeningRequest) -> Result<()> {
    if request.asset_id.trim().is_empty() {
        return Err(ValidationError::InvalidInput("asset_id is required".to_string()).into());
    }
    if request.profile_id.trim().is_empty() {
        return Err(ValidationError::InvalidInput("profile_id is required".to_string()).into());
    }
    if request.manual_override_status.is_some()
        && request
            .manual_override_reason
            .as_deref()
            .map(str::trim)
            .unwrap_or_default()
            .is_empty()
    {
        return Err(
            ValidationError::InvalidInput("manual override requires a reason".to_string()).into(),
        );
    }
    Ok(())
}

pub fn validate_zakat_request(request: &CalculateZakatSnapshotRequest) -> Result<()> {
    let currency = request.base_currency.trim();
    if currency.len() != 3 || !currency.chars().all(|c| c.is_ascii_uppercase()) {
        return Err(ValidationError::InvalidInput(
            "base_currency must be a 3-letter ISO code".to_string(),
        )
        .into());
    }
    if request.nisab_value <= Decimal::ZERO {
        return Err(ValidationError::InvalidInput(
            "manual nisab value must be greater than zero".to_string(),
        )
        .into());
    }
    if request.lines.is_empty() {
        return Err(ValidationError::InvalidInput(
            "at least one zakat line is required".to_string(),
        )
        .into());
    }
    for line in &request.lines {
        if line.category.trim().is_empty() {
            return Err(
                ValidationError::InvalidInput("line category is required".to_string()).into(),
            );
        }
        if line
            .asset_id
            .as_deref()
            .map(str::trim)
            .unwrap_or_default()
            .is_empty()
            && line.amount.is_none()
        {
            return Err(ValidationError::InvalidInput(
                "line amount is required when asset_id is not provided".to_string(),
            )
            .into());
        }
        if let Some(amount) = line.amount {
            if amount < Decimal::ZERO {
                return Err(ValidationError::InvalidInput(
                    "line amount cannot be negative".to_string(),
                )
                .into());
            }
        }
    }
    Ok(())
}

pub fn is_zakat_liability_category(category: &str) -> bool {
    let normalised = category
        .trim()
        .to_ascii_lowercase()
        .replace([' ', '-'], "_");
    matches!(
        normalised.as_str(),
        "liability" | "deductible_liability" | "short_term_liability"
    )
}

pub fn calculate_zakat_totals(
    lines: &[ZakatLine],
    nisab_value: Decimal,
) -> (Decimal, Decimal, Decimal, Decimal) {
    let total_zakatable_assets = lines
        .iter()
        .filter(|line| line.included && !is_zakat_liability_category(&line.category))
        .map(|line| line.amount)
        .sum::<Decimal>();
    let deductible_liabilities = lines
        .iter()
        .filter(|line| line.included && is_zakat_liability_category(&line.category))
        .map(|line| line.amount)
        .sum::<Decimal>();
    let net_zakatable_wealth = (total_zakatable_assets - deductible_liabilities).max(Decimal::ZERO);
    let zakat_due = if net_zakatable_wealth >= nisab_value {
        net_zakatable_wealth * Decimal::new(25, 3)
    } else {
        Decimal::ZERO
    };
    (
        total_zakatable_assets,
        deductible_liabilities,
        net_zakatable_wealth,
        zakat_due,
    )
}

pub fn calculate_purification(
    request: &UpsertPurificationEntryRequest,
) -> Result<(Decimal, PurificationCalculationMethod)> {
    validate_purification_request(request)?;
    if let (Some(total_impure_income), Some(outstanding_shares), Some(user_shares)) = (
        request.total_impure_income,
        request.outstanding_shares,
        request.user_shares,
    ) {
        if outstanding_shares > Decimal::ZERO {
            return Ok((
                (total_impure_income / outstanding_shares) * user_shares,
                PurificationCalculationMethod::ImpureIncomePerShare,
            ));
        }
    }
    if let (Some(dividend_received), Some(impure_income_ratio)) =
        (request.dividend_received, request.impure_income_ratio)
    {
        return Ok((
            dividend_received * impure_income_ratio,
            PurificationCalculationMethod::DividendRatio,
        ));
    }
    Ok((Decimal::ZERO, PurificationCalculationMethod::NeedsReview))
}

pub fn validate_purification_request(request: &UpsertPurificationEntryRequest) -> Result<()> {
    if request.asset_id.trim().is_empty() {
        return Err(ValidationError::InvalidInput("asset_id is required".to_string()).into());
    }
    if request.period_end < request.period_start {
        return Err(ValidationError::InvalidInput(
            "period_end must be on or after period_start".to_string(),
        )
        .into());
    }
    for (field, value) in [
        ("total_impure_income", request.total_impure_income),
        ("outstanding_shares", request.outstanding_shares),
        ("user_shares", request.user_shares),
        ("dividend_received", request.dividend_received),
        ("impure_income_ratio", request.impure_income_ratio),
    ] {
        if let Some(amount) = value {
            if amount < Decimal::ZERO {
                return Err(
                    ValidationError::InvalidInput(format!("{field} cannot be negative")).into(),
                );
            }
        }
    }
    Ok(())
}

pub fn summarize_purification_period(
    period_start: NaiveDate,
    period_end: NaiveDate,
    entries: Vec<PurificationEntry>,
) -> PurificationPeriodSummary {
    let total_calculated = entries
        .iter()
        .filter(|entry| entry.calculation_method != PurificationCalculationMethod::NeedsReview)
        .map(|entry| entry.purification_amount)
        .sum::<Decimal>();
    let total_paid = entries
        .iter()
        .filter(|entry| entry.status == PurificationStatus::Paid)
        .map(|entry| entry.purification_amount)
        .sum::<Decimal>();
    PurificationPeriodSummary {
        period_start,
        period_end,
        total_calculated,
        total_paid,
        entries,
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

    fn upsert_request() -> UpsertAssetShariahScreeningRequest {
        UpsertAssetShariahScreeningRequest {
            asset_id: "asset-1".to_string(),
            profile_id: DEFAULT_SHARIAH_PROFILE_ID.to_string(),
            ratios: ShariahScreeningRatios {
                debt_ratio: Some(dec!(0.10)),
                liquid_assets_ratio: Some(dec!(0.10)),
                impure_income_ratio: Some(dec!(0.01)),
            },
            source_citation_id: None,
            notes: None,
            manual_override_status: None,
            manual_override_reason: None,
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

    #[test]
    fn manual_override_without_reason_is_rejected() {
        let request = UpsertAssetShariahScreeningRequest {
            manual_override_status: Some(ShariahScreeningStatus::Compliant),
            ..upsert_request()
        };
        assert!(validate_screening_request(&request).is_err());
    }

    #[test]
    fn disabled_mode_is_rejected() {
        assert!(validate_shariah_mode_enabled(false).is_err());
        assert!(validate_shariah_mode_enabled(true).is_ok());
    }

    #[test]
    fn all_threshold_failures_are_non_compliant() {
        for ratios in [
            ShariahScreeningRatios {
                debt_ratio: Some(dec!(0.31)),
                liquid_assets_ratio: Some(dec!(0.10)),
                impure_income_ratio: Some(dec!(0.01)),
            },
            ShariahScreeningRatios {
                debt_ratio: Some(dec!(0.10)),
                liquid_assets_ratio: Some(dec!(0.31)),
                impure_income_ratio: Some(dec!(0.01)),
            },
            ShariahScreeningRatios {
                debt_ratio: Some(dec!(0.10)),
                liquid_assets_ratio: Some(dec!(0.10)),
                impure_income_ratio: Some(dec!(0.06)),
            },
        ] {
            let evaluation = evaluate_shariah_screening(&profile(), &ratios);
            assert_eq!(evaluation.status, ShariahScreeningStatus::NonCompliant);
        }
    }

    #[test]
    fn zakat_totals_include_assets_and_deduct_liabilities() {
        let lines = vec![
            ZakatLine {
                id: "line-1".to_string(),
                snapshot_id: "snapshot-1".to_string(),
                asset_id: Some("asset-1".to_string()),
                category: "short_term_asset".to_string(),
                amount: dec!(10000),
                included: true,
                explanation: "Included from latest market value".to_string(),
                source_citation_id: None,
            },
            ZakatLine {
                id: "line-2".to_string(),
                snapshot_id: "snapshot-1".to_string(),
                asset_id: None,
                category: "liability".to_string(),
                amount: dec!(2500),
                included: true,
                explanation: "Deductible short-term liability".to_string(),
                source_citation_id: None,
            },
            ZakatLine {
                id: "line-3".to_string(),
                snapshot_id: "snapshot-1".to_string(),
                asset_id: Some("asset-2".to_string()),
                category: "investment".to_string(),
                amount: dec!(999),
                included: false,
                explanation: "Excluded by user".to_string(),
                source_citation_id: None,
            },
        ];

        let (assets, liabilities, net, due) = calculate_zakat_totals(&lines, dec!(5000));

        assert_eq!(assets, dec!(10000));
        assert_eq!(liabilities, dec!(2500));
        assert_eq!(net, dec!(7500));
        assert_eq!(due, dec!(187.500));
    }

    #[test]
    fn manual_nisab_is_required() {
        let request = CalculateZakatSnapshotRequest {
            snapshot_date: NaiveDate::from_ymd_opt(2026, 5, 15).unwrap(),
            base_currency: "USD".to_string(),
            nisab_value: Decimal::ZERO,
            notes: None,
            lines: vec![ZakatInputLine {
                asset_id: None,
                category: "cash".to_string(),
                amount: Some(dec!(100)),
                included: true,
                explanation: Some("Manual cash balance".to_string()),
                source_citation_id: None,
            }],
        };

        assert!(validate_zakat_request(&request).is_err());
    }

    #[test]
    fn purification_impure_income_per_share_method() {
        let request = purification_request(
            Some(dec!(1_000)),
            Some(dec!(10_000)),
            Some(dec!(50)),
            None,
            None,
        );

        let (amount, method) = calculate_purification(&request).expect("calculation");

        assert_eq!(amount, dec!(5));
        assert_eq!(method, PurificationCalculationMethod::ImpureIncomePerShare);
    }

    #[test]
    fn purification_dividend_ratio_method() {
        let request = purification_request(None, None, None, Some(dec!(400)), Some(dec!(0.05)));

        let (amount, method) = calculate_purification(&request).expect("calculation");

        assert_eq!(amount, dec!(20.00));
        assert_eq!(method, PurificationCalculationMethod::DividendRatio);
    }

    #[test]
    fn purification_missing_data_needs_review() {
        let request = purification_request(None, None, None, None, None);

        let (amount, method) = calculate_purification(&request).expect("calculation");

        assert_eq!(amount, Decimal::ZERO);
        assert_eq!(method, PurificationCalculationMethod::NeedsReview);
    }

    #[test]
    fn purification_period_summary_totals_paid_separately() {
        let start = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2026, 12, 31).unwrap();
        let entries = vec![
            purification_entry("entry-1", dec!(10), PurificationStatus::Calculated),
            purification_entry("entry-2", dec!(20), PurificationStatus::Paid),
        ];

        let summary = summarize_purification_period(start, end, entries);

        assert_eq!(summary.total_calculated, dec!(30));
        assert_eq!(summary.total_paid, dec!(20));
    }

    fn purification_request(
        total_impure_income: Option<Decimal>,
        outstanding_shares: Option<Decimal>,
        user_shares: Option<Decimal>,
        dividend_received: Option<Decimal>,
        impure_income_ratio: Option<Decimal>,
    ) -> UpsertPurificationEntryRequest {
        UpsertPurificationEntryRequest {
            asset_id: "asset-1".to_string(),
            period_start: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            period_end: NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
            total_impure_income,
            outstanding_shares,
            user_shares,
            dividend_received,
            impure_income_ratio,
            status: None,
            source_citation_id: None,
            notes: None,
        }
    }

    fn purification_entry(
        id: &str,
        purification_amount: Decimal,
        status: PurificationStatus,
    ) -> PurificationEntry {
        PurificationEntry {
            id: id.to_string(),
            asset_id: "asset-1".to_string(),
            period_start: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            period_end: NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
            total_impure_income: None,
            outstanding_shares: None,
            user_shares: None,
            dividend_received: None,
            impure_income_ratio: None,
            purification_amount,
            calculation_method: PurificationCalculationMethod::DividendRatio,
            status,
            source_citation_id: None,
            notes: None,
            created_at: "2026-05-15T00:00:00Z".to_string(),
            updated_at: "2026-05-15T00:00:00Z".to_string(),
        }
    }
}
