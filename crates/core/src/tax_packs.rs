use async_trait::async_trait;
use chrono::{Datelike, NaiveDate};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, VecDeque};
use std::str::FromStr;

use crate::activities::{
    Activity, ACTIVITY_TYPE_BUY, ACTIVITY_TYPE_DIVIDEND, ACTIVITY_TYPE_FEE, ACTIVITY_TYPE_INTEREST,
    ACTIVITY_TYPE_SELL,
};
use crate::errors::{Error, ValidationError};
use crate::private_investments::PrivateDistribution;
use crate::Result;

pub const TAX_PACK_DISCLAIMER: &str =
    "Data preparation only. Mizan does not provide tax advice or filing guidance.";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaxJurisdiction {
    #[serde(rename = "US")]
    Us,
    #[serde(rename = "UK")]
    Uk,
    Singapore,
    #[serde(rename = "GCC")]
    Gcc,
    General,
}

impl TaxJurisdiction {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Us => "US",
            Self::Uk => "UK",
            Self::Singapore => "Singapore",
            Self::Gcc => "GCC",
            Self::General => "General",
        }
    }
}

impl FromStr for TaxJurisdiction {
    type Err = Error;

    fn from_str(value: &str) -> Result<Self> {
        match value {
            "US" => Ok(Self::Us),
            "UK" => Ok(Self::Uk),
            "Singapore" => Ok(Self::Singapore),
            "GCC" => Ok(Self::Gcc),
            "General" => Ok(Self::General),
            _ => Err(invalid(format!("Unsupported tax jurisdiction: {value}"))),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaxPackStatus {
    Draft,
    Finalized,
    Exported,
}

impl TaxPackStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Finalized => "finalized",
            Self::Exported => "exported",
        }
    }
}

impl FromStr for TaxPackStatus {
    type Err = Error;

    fn from_str(value: &str) -> Result<Self> {
        match value {
            "draft" => Ok(Self::Draft),
            "finalized" => Ok(Self::Finalized),
            "exported" => Ok(Self::Exported),
            _ => Err(invalid(format!("Unsupported tax pack status: {value}"))),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaxPackLineCategory {
    RealizedGain,
    Dividend,
    Interest,
    Coupon,
    Fx,
    PrivateDistribution,
    Fee,
    Other,
}

impl TaxPackLineCategory {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::RealizedGain => "realized_gain",
            Self::Dividend => "dividend",
            Self::Interest => "interest",
            Self::Coupon => "coupon",
            Self::Fx => "fx",
            Self::PrivateDistribution => "private_distribution",
            Self::Fee => "fee",
            Self::Other => "other",
        }
    }
}

impl FromStr for TaxPackLineCategory {
    type Err = Error;

    fn from_str(value: &str) -> Result<Self> {
        match value {
            "realized_gain" => Ok(Self::RealizedGain),
            "dividend" => Ok(Self::Dividend),
            "interest" => Ok(Self::Interest),
            "coupon" => Ok(Self::Coupon),
            "fx" => Ok(Self::Fx),
            "private_distribution" => Ok(Self::PrivateDistribution),
            "fee" => Ok(Self::Fee),
            "other" => Ok(Self::Other),
            _ => Err(invalid(format!(
                "Unsupported tax pack line category: {value}"
            ))),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerateTaxPackRequest {
    pub tax_year: i32,
    pub jurisdiction: TaxJurisdiction,
    pub base_currency: String,
}

impl GenerateTaxPackRequest {
    pub fn validate(&self) -> Result<()> {
        if !(1900..=9999).contains(&self.tax_year) {
            return Err(invalid("tax_year must be between 1900 and 9999"));
        }
        validate_currency(&self.base_currency)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaxPackLine {
    pub id: String,
    pub tax_pack_id: String,
    pub category: TaxPackLineCategory,
    pub asset_id: Option<String>,
    pub activity_id: Option<String>,
    pub amount: Decimal,
    pub currency: String,
    pub taxable_date: NaiveDate,
    pub source_citation_id: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaxPackMissingItem {
    pub id: String,
    pub tax_pack_id: String,
    pub severity: String,
    pub message: String,
    pub related_activity_id: Option<String>,
    pub related_asset_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaxPack {
    pub id: String,
    pub tax_year: i32,
    pub jurisdiction: TaxJurisdiction,
    pub base_currency: String,
    pub status: TaxPackStatus,
    pub created_at: String,
    pub finalized_at: Option<String>,
    pub lines: Vec<TaxPackLine>,
    pub missing_data_checklist: Vec<TaxPackMissingItem>,
    pub disclaimer: String,
}

#[async_trait]
pub trait TaxPackRepositoryTrait: Send + Sync {
    async fn generate_tax_pack(&self, request: GenerateTaxPackRequest) -> Result<TaxPack>;
    fn get_tax_pack(&self, tax_pack_id: &str) -> Result<Option<TaxPack>>;
}

pub fn build_tax_pack_draft(
    pack_id: String,
    request: GenerateTaxPackRequest,
    created_at: String,
    activities: &[Activity],
    private_distributions: &[PrivateDistribution],
) -> Result<TaxPack> {
    request.validate()?;
    let mut generator = TaxPackGenerator {
        pack_id: pack_id.clone(),
        request,
        lines: Vec::new(),
        missing: Vec::new(),
        lots_by_asset: BTreeMap::new(),
    };
    generator.consume_activities(activities)?;
    generator.consume_private_distributions(private_distributions);
    if generator.lines.is_empty() {
        generator.add_missing(
            "info",
            "No taxable ledger activity was found for this tax year. Review account statements before relying on this empty draft.",
            None,
            None,
        );
    }

    generator.lines.sort_by(|a, b| {
        a.taxable_date
            .cmp(&b.taxable_date)
            .then_with(|| a.category.as_str().cmp(b.category.as_str()))
            .then_with(|| a.id.cmp(&b.id))
    });

    Ok(TaxPack {
        id: pack_id,
        tax_year: generator.request.tax_year,
        jurisdiction: generator.request.jurisdiction,
        base_currency: generator.request.base_currency,
        status: TaxPackStatus::Draft,
        created_at,
        finalized_at: None,
        lines: generator.lines,
        missing_data_checklist: generator.missing,
        disclaimer: TAX_PACK_DISCLAIMER.to_string(),
    })
}

struct Lot {
    quantity: Decimal,
    cost_basis: Decimal,
}

struct TaxPackGenerator {
    pack_id: String,
    request: GenerateTaxPackRequest,
    lines: Vec<TaxPackLine>,
    missing: Vec<TaxPackMissingItem>,
    lots_by_asset: BTreeMap<String, VecDeque<Lot>>,
}

impl TaxPackGenerator {
    fn consume_activities(&mut self, activities: &[Activity]) -> Result<()> {
        let mut sorted = activities
            .iter()
            .filter(|activity| activity.is_posted())
            .cloned()
            .collect::<Vec<_>>();
        sorted.sort_by(|a, b| {
            a.activity_date
                .cmp(&b.activity_date)
                .then_with(|| a.created_at.cmp(&b.created_at))
                .then_with(|| a.id.cmp(&b.id))
        });

        for activity in &sorted {
            if activity.effective_date().year() > self.request.tax_year {
                continue;
            }
            match activity.effective_type() {
                ACTIVITY_TYPE_BUY => self.record_buy(activity),
                ACTIVITY_TYPE_SELL => self.record_sell(activity)?,
                ACTIVITY_TYPE_DIVIDEND => {
                    self.record_income(activity, TaxPackLineCategory::Dividend)
                }
                ACTIVITY_TYPE_INTEREST => self.record_income(activity, interest_category(activity)),
                ACTIVITY_TYPE_FEE => self.record_fee(activity),
                _ => {}
            }
        }
        Ok(())
    }

    fn consume_private_distributions(&mut self, distributions: &[PrivateDistribution]) {
        for distribution in distributions {
            if distribution.distribution_date.year() != self.request.tax_year {
                continue;
            }
            let notes = match &distribution.notes {
                Some(notes) if !notes.trim().is_empty() => {
                    Some(format!("Private distribution. {notes}"))
                }
                _ => Some("Private distribution. Classification is not tax advice.".to_string()),
            };
            self.push_line(TaxPackLine {
                id: format!("{}:private-distribution:{}", self.pack_id, distribution.id),
                tax_pack_id: self.pack_id.clone(),
                category: TaxPackLineCategory::PrivateDistribution,
                asset_id: Some(distribution.asset_id.clone()),
                activity_id: None,
                amount: distribution.amount,
                currency: distribution.currency.clone(),
                taxable_date: distribution.distribution_date,
                source_citation_id: distribution.source_citation_id.clone(),
                notes,
            });
        }
    }

    fn record_buy(&mut self, activity: &Activity) {
        let Some(asset_id) = activity.asset_id.clone() else {
            return;
        };
        let quantity = activity.qty();
        if quantity <= Decimal::ZERO {
            return;
        }
        let cost_basis = amount_or_quantity_price(activity) + activity.fee_amt();
        self.lots_by_asset
            .entry(asset_id)
            .or_default()
            .push_back(Lot {
                quantity,
                cost_basis,
            });
    }

    fn record_sell(&mut self, activity: &Activity) -> Result<()> {
        let in_tax_year = activity.effective_date().year() == self.request.tax_year;
        let Some(asset_id) = activity.asset_id.clone() else {
            if in_tax_year {
                self.add_missing(
                    "warning",
                    "Sell activity is missing an asset, so no realized gain line was generated.",
                    Some(activity.id.clone()),
                    None,
                );
            }
            return Ok(());
        };
        let quantity = activity.qty();
        if quantity <= Decimal::ZERO {
            if in_tax_year {
                self.add_missing(
                    "warning",
                    "Sell activity is missing quantity, so no realized gain line was generated.",
                    Some(activity.id.clone()),
                    Some(asset_id),
                );
            }
            return Ok(());
        }

        let Some(cost_basis) = self.consume_fifo_cost(&asset_id, quantity)? else {
            if in_tax_year {
                self.add_missing(
                    "warning",
                    "Sell activity has insufficient FIFO lots, so no realized gain line was generated.",
                    Some(activity.id.clone()),
                    Some(asset_id),
                );
            }
            return Ok(());
        };
        if in_tax_year {
            let proceeds = amount_or_quantity_price(activity) - activity.fee_amt();
            let gain = proceeds - cost_basis;
            self.push_activity_line(
                activity,
                TaxPackLineCategory::RealizedGain,
                gain,
                Some("Realized gain prepared from FIFO lots. Review with a tax professional before filing."),
            );
            self.add_fx_warning(activity);
        }
        Ok(())
    }

    fn record_income(&mut self, activity: &Activity, category: TaxPackLineCategory) {
        if activity.effective_date().year() != self.request.tax_year {
            return;
        }
        let amount = activity.amt();
        if amount <= Decimal::ZERO {
            self.add_missing(
                "warning",
                "Income activity is missing amount, so no tax pack line was generated.",
                Some(activity.id.clone()),
                activity.asset_id.clone(),
            );
            return;
        }
        self.push_activity_line(activity, category, amount, None);
        self.add_fx_warning(activity);
    }

    fn record_fee(&mut self, activity: &Activity) {
        if activity.effective_date().year() != self.request.tax_year {
            return;
        }
        let amount = activity.amt().max(activity.fee_amt());
        if amount <= Decimal::ZERO {
            return;
        }
        self.push_activity_line(
            activity,
            TaxPackLineCategory::Fee,
            amount,
            Some("Fee line included for CPA review; deductibility is not inferred."),
        );
        self.add_fx_warning(activity);
    }

    fn consume_fifo_cost(
        &mut self,
        asset_id: &str,
        mut quantity: Decimal,
    ) -> Result<Option<Decimal>> {
        let lots = self.lots_by_asset.entry(asset_id.to_string()).or_default();
        let mut cost_basis = Decimal::ZERO;
        while quantity > Decimal::ZERO {
            let Some(front) = lots.front_mut() else {
                return Ok(None);
            };
            let consumed = front.quantity.min(quantity);
            cost_basis += front.cost_basis * consumed / front.quantity;
            front.cost_basis -= front.cost_basis * consumed / front.quantity;
            front.quantity -= consumed;
            quantity -= consumed;
            if front.quantity <= Decimal::ZERO {
                lots.pop_front();
            }
        }
        Ok(Some(cost_basis))
    }

    fn push_activity_line(
        &mut self,
        activity: &Activity,
        category: TaxPackLineCategory,
        amount: Decimal,
        notes: Option<&str>,
    ) {
        self.push_line(TaxPackLine {
            id: format!("{}:activity:{}", self.pack_id, activity.id),
            tax_pack_id: self.pack_id.clone(),
            category,
            asset_id: activity.asset_id.clone(),
            activity_id: Some(activity.id.clone()),
            amount,
            currency: activity.currency.clone(),
            taxable_date: activity.effective_date(),
            source_citation_id: None,
            notes: notes.map(ToString::to_string),
        });
    }

    fn push_line(&mut self, line: TaxPackLine) {
        if line.source_citation_id.is_none() {
            self.add_missing(
                "warning",
                "Tax pack line has no source citation. Verify the supporting statement or manual source before export.",
                line.activity_id.clone(),
                line.asset_id.clone(),
            );
        }
        self.lines.push(line);
    }

    fn add_fx_warning(&mut self, activity: &Activity) {
        if activity
            .currency
            .eq_ignore_ascii_case(&self.request.base_currency)
        {
            return;
        }
        let message = if activity.fx_rate.is_some() {
            "Activity has a non-base currency and an FX rate. Tax pack keeps the ledger currency and does not infer jurisdiction-specific FX treatment."
        } else {
            "Activity has a non-base currency and no FX rate. Add FX support before export."
        };
        self.add_missing(
            "warning",
            message,
            Some(activity.id.clone()),
            activity.asset_id.clone(),
        );
    }

    fn add_missing(
        &mut self,
        severity: &str,
        message: &str,
        related_activity_id: Option<String>,
        related_asset_id: Option<String>,
    ) {
        let id = format!("{}:missing:{}", self.pack_id, self.missing.len() + 1);
        self.missing.push(TaxPackMissingItem {
            id,
            tax_pack_id: self.pack_id.clone(),
            severity: severity.to_string(),
            message: message.to_string(),
            related_activity_id,
            related_asset_id,
        });
    }
}

fn amount_or_quantity_price(activity: &Activity) -> Decimal {
    let amount = activity.amt();
    if amount > Decimal::ZERO {
        amount
    } else {
        activity.qty() * activity.price()
    }
}

fn interest_category(activity: &Activity) -> TaxPackLineCategory {
    if activity
        .subtype
        .as_deref()
        .is_some_and(|value| value.eq_ignore_ascii_case("COUPON"))
    {
        TaxPackLineCategory::Coupon
    } else {
        TaxPackLineCategory::Interest
    }
}

fn validate_currency(value: &str) -> Result<()> {
    let trimmed = value.trim();
    if trimmed.len() == 3 && trimmed.chars().all(|c| c.is_ascii_uppercase()) {
        Ok(())
    } else {
        Err(invalid(
            "currency must be a three-letter uppercase ISO code",
        ))
    }
}

fn invalid(message: impl Into<String>) -> Error {
    Error::Validation(ValidationError::InvalidInput(message.into()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::activities::ActivityStatus;
    use chrono::{TimeZone, Utc};
    use rust_decimal_macros::dec;

    #[test]
    fn tax_year_filtering_excludes_other_year_income() {
        let pack = draft(vec![
            activity("div-2025", ACTIVITY_TYPE_DIVIDEND, "2025-12-31", dec!(10)),
            activity("div-2026", ACTIVITY_TYPE_DIVIDEND, "2026-01-01", dec!(20)),
        ]);

        assert_eq!(pack.lines.len(), 1);
        assert_eq!(pack.lines[0].activity_id.as_deref(), Some("div-2026"));
    }

    #[test]
    fn realized_gain_lines_use_fifo_cost_basis() {
        let mut buy1 = activity("buy-1", ACTIVITY_TYPE_BUY, "2025-01-01", dec!(100));
        buy1.quantity = Some(dec!(10));
        buy1.unit_price = Some(dec!(10));
        let mut buy2 = activity("buy-2", ACTIVITY_TYPE_BUY, "2025-06-01", dec!(120));
        buy2.quantity = Some(dec!(10));
        buy2.unit_price = Some(dec!(12));
        let mut sell = activity("sell-1", ACTIVITY_TYPE_SELL, "2026-01-15", dec!(180));
        sell.quantity = Some(dec!(15));

        let pack = draft(vec![buy1, buy2, sell]);

        assert_eq!(pack.lines.len(), 1);
        assert_eq!(pack.lines[0].category, TaxPackLineCategory::RealizedGain);
        assert_eq!(pack.lines[0].amount, dec!(20));
    }

    #[test]
    fn dividend_and_coupon_lines_are_preserved_without_reclassification() {
        let mut coupon = activity("coupon-1", ACTIVITY_TYPE_INTEREST, "2026-02-01", dec!(30));
        coupon.subtype = Some("COUPON".to_string());
        let pack = draft(vec![
            activity("div-1", ACTIVITY_TYPE_DIVIDEND, "2026-01-01", dec!(40)),
            coupon,
        ]);

        assert!(pack
            .lines
            .iter()
            .any(|line| line.category == TaxPackLineCategory::Dividend && line.amount == dec!(40)));
        assert!(pack
            .lines
            .iter()
            .any(|line| line.category == TaxPackLineCategory::Coupon && line.amount == dec!(30)));
    }

    #[test]
    fn missing_citation_warning_is_added_for_ledger_lines() {
        let pack = draft(vec![activity(
            "div-uncited",
            ACTIVITY_TYPE_DIVIDEND,
            "2026-01-01",
            dec!(10),
        )]);

        assert!(pack
            .missing_data_checklist
            .iter()
            .any(|item| item.message.contains("no source citation")));
    }

    #[test]
    fn empty_draft_contains_checklist() {
        let pack = draft(Vec::new());

        assert!(pack.lines.is_empty());
        assert!(pack
            .missing_data_checklist
            .iter()
            .any(|item| item.message.contains("No taxable ledger activity")));
    }

    fn draft(activities: Vec<Activity>) -> TaxPack {
        build_tax_pack_draft(
            "pack-1".to_string(),
            GenerateTaxPackRequest {
                tax_year: 2026,
                jurisdiction: TaxJurisdiction::General,
                base_currency: "USD".to_string(),
            },
            "2026-05-16T00:00:00Z".to_string(),
            &activities,
            &[],
        )
        .expect("draft")
    }

    fn activity(id: &str, activity_type: &str, date: &str, amount: Decimal) -> Activity {
        let date = NaiveDate::parse_from_str(date, "%Y-%m-%d").expect("date");
        let at = Utc
            .with_ymd_and_hms(date.year(), date.month(), date.day(), 0, 0, 0)
            .unwrap();
        Activity {
            id: id.to_string(),
            account_id: "acc-1".to_string(),
            asset_id: Some("asset-1".to_string()),
            activity_type: activity_type.to_string(),
            activity_type_override: None,
            source_type: None,
            subtype: None,
            status: ActivityStatus::Posted,
            activity_date: at,
            settlement_date: None,
            quantity: None,
            unit_price: None,
            amount: Some(amount),
            fee: None,
            currency: "USD".to_string(),
            fx_rate: None,
            notes: None,
            metadata: None,
            source_system: None,
            source_record_id: None,
            source_group_id: None,
            idempotency_key: None,
            import_run_id: None,
            is_user_modified: false,
            needs_review: false,
            created_at: at,
            updated_at: at,
        }
    }
}
