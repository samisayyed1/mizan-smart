use async_trait::async_trait;
use chrono::{Months, NaiveDate};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::Result;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LiquidityLadderWindow {
    Next30Days,
    Next90Days,
    Next12Months,
}

impl LiquidityLadderWindow {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Next30Days => "next_30_days",
            Self::Next90Days => "next_90_days",
            Self::Next12Months => "next_12_months",
        }
    }

    pub fn end_date(self, as_of: NaiveDate) -> NaiveDate {
        match self {
            Self::Next30Days => as_of + chrono::Duration::days(30),
            Self::Next90Days => as_of + chrono::Duration::days(90),
            Self::Next12Months => as_of.checked_add_months(Months::new(12)).unwrap_or(as_of),
        }
    }

    const fn all() -> [Self; 3] {
        [Self::Next30Days, Self::Next90Days, Self::Next12Months]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LiquidityDirection {
    Incoming,
    Outgoing,
    Balance,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LiquidityConfidence {
    Confirmed,
    Expected,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LiquidityItemType {
    CashBalance,
    FixedIncomeCashflow,
    SukukProfit,
    FixedDepositMaturity,
    PrivateCapitalCall,
    PrivateDistribution,
    ScheduledDividend,
    ScheduledInterest,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LiquidityLadderItem {
    pub id: String,
    pub date: NaiveDate,
    pub currency: String,
    pub amount: Decimal,
    pub direction: LiquidityDirection,
    pub confidence: LiquidityConfidence,
    pub item_type: LiquidityItemType,
    pub label: String,
    pub source_id: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LiquidityCurrencyGroup {
    pub currency: String,
    pub available_cash: Decimal,
    pub confirmed_incoming: Decimal,
    pub expected_incoming: Decimal,
    pub confirmed_outgoing: Decimal,
    pub expected_outgoing: Decimal,
    pub net_confirmed: Decimal,
    pub net_expected: Decimal,
    pub items: Vec<LiquidityLadderItem>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LiquidityLadderView {
    pub window: LiquidityLadderWindow,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub currency_groups: Vec<LiquidityCurrencyGroup>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LiquidityLadderReport {
    pub as_of: NaiveDate,
    pub views: Vec<LiquidityLadderView>,
}

pub fn build_liquidity_ladder(
    as_of: NaiveDate,
    items: Vec<LiquidityLadderItem>,
) -> LiquidityLadderReport {
    let views = LiquidityLadderWindow::all()
        .into_iter()
        .map(|window| build_view(as_of, window, &items))
        .collect();
    LiquidityLadderReport { as_of, views }
}

fn build_view(
    as_of: NaiveDate,
    window: LiquidityLadderWindow,
    items: &[LiquidityLadderItem],
) -> LiquidityLadderView {
    let end_date = window.end_date(as_of);
    let mut grouped = BTreeMap::<String, Vec<LiquidityLadderItem>>::new();
    for item in items {
        let in_window = item.item_type == LiquidityItemType::CashBalance
            || (item.date >= as_of && item.date <= end_date);
        if in_window {
            grouped
                .entry(item.currency.clone())
                .or_default()
                .push(item.clone());
        }
    }

    let mut currency_groups = grouped
        .into_iter()
        .map(|(currency, mut items)| {
            items.sort_by(|a, b| {
                a.date
                    .cmp(&b.date)
                    .then_with(|| a.direction.as_rank().cmp(&b.direction.as_rank()))
                    .then_with(|| a.label.cmp(&b.label))
            });
            group_currency(currency, items)
        })
        .collect::<Vec<_>>();
    currency_groups.sort_by(|a, b| a.currency.cmp(&b.currency));

    let warnings = if currency_groups.iter().all(|group| {
        group
            .items
            .iter()
            .all(|item| item.item_type == LiquidityItemType::CashBalance)
    }) {
        vec![
            "No dated cashflows are scheduled in this window. Future dividends, tax obligations, and insurance premiums are not projected unless they are already recorded."
                .to_string(),
        ]
    } else {
        vec![
            "Future dividends are included only when a dated dividend or interest activity already exists."
                .to_string(),
        ]
    };

    LiquidityLadderView {
        window,
        start_date: as_of,
        end_date,
        currency_groups,
        warnings,
    }
}

fn group_currency(currency: String, items: Vec<LiquidityLadderItem>) -> LiquidityCurrencyGroup {
    let mut available_cash = Decimal::ZERO;
    let mut confirmed_incoming = Decimal::ZERO;
    let mut expected_incoming = Decimal::ZERO;
    let mut confirmed_outgoing = Decimal::ZERO;
    let mut expected_outgoing = Decimal::ZERO;

    for item in &items {
        match (item.direction, item.confidence) {
            (LiquidityDirection::Balance, LiquidityConfidence::Confirmed) => {
                available_cash += item.amount
            }
            (LiquidityDirection::Incoming, LiquidityConfidence::Confirmed) => {
                confirmed_incoming += item.amount
            }
            (LiquidityDirection::Incoming, LiquidityConfidence::Expected) => {
                expected_incoming += item.amount
            }
            (LiquidityDirection::Outgoing, LiquidityConfidence::Confirmed) => {
                confirmed_outgoing += item.amount
            }
            (LiquidityDirection::Outgoing, LiquidityConfidence::Expected) => {
                expected_outgoing += item.amount
            }
            (LiquidityDirection::Balance, LiquidityConfidence::Expected) => {}
        }
    }

    let net_confirmed = available_cash + confirmed_incoming - confirmed_outgoing;
    let net_expected = net_confirmed + expected_incoming - expected_outgoing;

    LiquidityCurrencyGroup {
        currency,
        available_cash,
        confirmed_incoming,
        expected_incoming,
        confirmed_outgoing,
        expected_outgoing,
        net_confirmed,
        net_expected,
        items,
    }
}

impl LiquidityDirection {
    const fn as_rank(self) -> u8 {
        match self {
            Self::Balance => 0,
            Self::Incoming => 1,
            Self::Outgoing => 2,
        }
    }
}

#[async_trait]
pub trait LiquidityLadderRepositoryTrait: Send + Sync {
    async fn get_ladder(&self, as_of: NaiveDate) -> Result<LiquidityLadderReport>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn date(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day).unwrap()
    }

    fn item(
        id: &str,
        date: NaiveDate,
        currency: &str,
        amount: Decimal,
        direction: LiquidityDirection,
        confidence: LiquidityConfidence,
        item_type: LiquidityItemType,
    ) -> LiquidityLadderItem {
        LiquidityLadderItem {
            id: id.into(),
            date,
            currency: currency.into(),
            amount,
            direction,
            confidence,
            item_type,
            label: id.into(),
            source_id: None,
            notes: None,
        }
    }

    #[test]
    fn fixed_income_cashflows_are_included_in_window() {
        let report = build_liquidity_ladder(
            date(2026, 5, 15),
            vec![item(
                "coupon",
                date(2026, 5, 30),
                "USD",
                dec!(25),
                LiquidityDirection::Incoming,
                LiquidityConfidence::Expected,
                LiquidityItemType::FixedIncomeCashflow,
            )],
        );
        let group = &report.views[0].currency_groups[0];
        assert_eq!(group.expected_incoming, dec!(25));
        assert_eq!(
            group.items[0].item_type,
            LiquidityItemType::FixedIncomeCashflow
        );
    }

    #[test]
    fn capital_calls_are_outgoing_expected_cashflows() {
        let report = build_liquidity_ladder(
            date(2026, 5, 15),
            vec![item(
                "call",
                date(2026, 5, 20),
                "USD",
                dec!(100),
                LiquidityDirection::Outgoing,
                LiquidityConfidence::Expected,
                LiquidityItemType::PrivateCapitalCall,
            )],
        );
        let group = &report.views[0].currency_groups[0];
        assert_eq!(group.expected_outgoing, dec!(100));
        assert_eq!(group.net_expected, dec!(-100));
    }

    #[test]
    fn groups_by_currency_without_converting_amounts() {
        let report = build_liquidity_ladder(
            date(2026, 5, 15),
            vec![
                item(
                    "usd",
                    date(2026, 5, 15),
                    "USD",
                    dec!(1000),
                    LiquidityDirection::Balance,
                    LiquidityConfidence::Confirmed,
                    LiquidityItemType::CashBalance,
                ),
                item(
                    "aed",
                    date(2026, 5, 20),
                    "AED",
                    dec!(500),
                    LiquidityDirection::Incoming,
                    LiquidityConfidence::Confirmed,
                    LiquidityItemType::PrivateDistribution,
                ),
            ],
        );
        assert_eq!(report.views[0].currency_groups[0].currency, "AED");
        assert_eq!(report.views[0].currency_groups[1].currency, "USD");
    }

    #[test]
    fn empty_state_is_honest_about_missing_schedules() {
        let report = build_liquidity_ladder(date(2026, 5, 15), Vec::new());
        assert!(report.views[0].currency_groups.is_empty());
        assert!(report.views[0].warnings[0].contains("No dated cashflows"));
    }

    #[test]
    fn confirmed_and_expected_totals_are_kept_separate() {
        let report = build_liquidity_ladder(
            date(2026, 5, 15),
            vec![
                item(
                    "cash",
                    date(2026, 5, 15),
                    "USD",
                    dec!(1000),
                    LiquidityDirection::Balance,
                    LiquidityConfidence::Confirmed,
                    LiquidityItemType::CashBalance,
                ),
                item(
                    "dist",
                    date(2026, 5, 16),
                    "USD",
                    dec!(100),
                    LiquidityDirection::Incoming,
                    LiquidityConfidence::Confirmed,
                    LiquidityItemType::PrivateDistribution,
                ),
                item(
                    "call",
                    date(2026, 5, 17),
                    "USD",
                    dec!(250),
                    LiquidityDirection::Outgoing,
                    LiquidityConfidence::Expected,
                    LiquidityItemType::PrivateCapitalCall,
                ),
            ],
        );
        let group = &report.views[0].currency_groups[0];
        assert_eq!(group.available_cash, dec!(1000));
        assert_eq!(group.confirmed_incoming, dec!(100));
        assert_eq!(group.expected_outgoing, dec!(250));
        assert_eq!(group.net_confirmed, dec!(1100));
        assert_eq!(group.net_expected, dec!(850));
    }
}
