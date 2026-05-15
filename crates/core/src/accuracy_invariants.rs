//! Deterministic financial accuracy invariants.
//!
//! These helpers keep arithmetic assertions close to the domain types that
//! produce them. They intentionally use `Decimal` throughout; presentation
//! rounding belongs in UI/export code.

use crate::fixed_income::{
    accrued_interest_or_profit, FixedIncomeCashflowType, FixedIncomeDetails,
    FixedIncomePaymentFrequency, ProjectedFixedIncomeCashflow,
};
use crate::portfolio::snapshot::{AccountStateSnapshot, Position, RealizedGainEntry};
use crate::private_investments::PrivateInvestmentSummary;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, HashMap};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccuracyInvariantViolation {
    pub code: String,
    pub message: String,
}

impl AccuracyInvariantViolation {
    fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
        }
    }
}

pub fn validate_position_lots(position: &Position) -> Vec<AccuracyInvariantViolation> {
    let mut violations = Vec::new();
    let lot_quantity = position
        .lots
        .iter()
        .map(|lot| lot.quantity)
        .sum::<Decimal>();
    let lot_cost_basis = position
        .lots
        .iter()
        .map(|lot| lot.cost_basis)
        .sum::<Decimal>();

    if lot_quantity != position.quantity {
        violations.push(AccuracyInvariantViolation::new(
            "position_lot_quantity_mismatch",
            format!(
                "Position {} quantity {} does not equal open lots {}.",
                position.asset_id, position.quantity, lot_quantity
            ),
        ));
    }
    if lot_cost_basis != position.total_cost_basis {
        violations.push(AccuracyInvariantViolation::new(
            "position_lot_cost_basis_mismatch",
            format!(
                "Position {} cost basis {} does not equal open lots {}.",
                position.asset_id, position.total_cost_basis, lot_cost_basis
            ),
        ));
    }
    violations
}

pub fn validate_snapshot_lots(snapshot: &AccountStateSnapshot) -> Vec<AccuracyInvariantViolation> {
    snapshot
        .positions
        .values()
        .flat_map(validate_position_lots)
        .collect()
}

pub fn realized_gain_account(entry: &RealizedGainEntry) -> Decimal {
    entry.proceeds_account_ccy - entry.cost_basis_account_ccy - entry.fees_account_ccy
}

pub fn realized_gain_base(entry: &RealizedGainEntry) -> Decimal {
    entry.proceeds_base_ccy - entry.cost_basis_base_ccy - entry.fees_base_ccy
}

pub fn validate_realized_gain(
    entry: &RealizedGainEntry,
    expected_account_ccy: Decimal,
    expected_base_ccy: Decimal,
) -> Vec<AccuracyInvariantViolation> {
    let mut violations = Vec::new();
    let account_gain = realized_gain_account(entry);
    let base_gain = realized_gain_base(entry);

    if account_gain != expected_account_ccy || account_gain != entry.realized_gain_account_ccy() {
        violations.push(AccuracyInvariantViolation::new(
            "realized_gain_account_formula_mismatch",
            format!(
                "Account realized gain {} does not equal expected {}.",
                account_gain, expected_account_ccy
            ),
        ));
    }
    if base_gain != expected_base_ccy || base_gain != entry.realized_gain_base_ccy() {
        violations.push(AccuracyInvariantViolation::new(
            "realized_gain_base_formula_mismatch",
            format!(
                "Base realized gain {} does not equal expected {}.",
                base_gain, expected_base_ccy
            ),
        ));
    }

    violations
}

pub fn validate_cash_ledger(
    ledger_deltas: &[(String, Decimal)],
    balances: &HashMap<String, Decimal>,
) -> Vec<AccuracyInvariantViolation> {
    let mut expected = HashMap::<String, Decimal>::new();
    for (currency, delta) in ledger_deltas {
        *expected.entry(currency.clone()).or_insert(Decimal::ZERO) += *delta;
    }

    let currencies = expected
        .keys()
        .chain(balances.keys())
        .cloned()
        .collect::<BTreeSet<_>>();

    currencies
        .into_iter()
        .filter_map(|currency| {
            let expected_amount = expected.get(&currency).copied().unwrap_or(Decimal::ZERO);
            let actual_amount = balances.get(&currency).copied().unwrap_or(Decimal::ZERO);
            (expected_amount != actual_amount).then(|| {
                AccuracyInvariantViolation::new(
                    "cash_ledger_balance_mismatch",
                    format!(
                        "Cash ledger for {} sums to {}, but balance is {}.",
                        currency, expected_amount, actual_amount
                    ),
                )
            })
        })
        .collect()
}

pub fn validate_report_total(
    report_name: &str,
    line_amounts: &[Decimal],
    reported_total: Decimal,
) -> Vec<AccuracyInvariantViolation> {
    let line_sum = line_amounts.iter().copied().sum::<Decimal>();
    if line_sum == reported_total {
        Vec::new()
    } else {
        vec![AccuracyInvariantViolation::new(
            "report_total_line_sum_mismatch",
            format!(
                "Report {} total {} does not equal line sum {}.",
                report_name, reported_total, line_sum
            ),
        )]
    }
}

pub fn validate_private_investment_summary(
    summary: &PrivateInvestmentSummary,
) -> Vec<AccuracyInvariantViolation> {
    let expected_unfunded =
        summary.commitment - summary.paid_in_capital + summary.recallable_distributions;
    if summary.unfunded_commitment == expected_unfunded {
        Vec::new()
    } else {
        vec![AccuracyInvariantViolation::new(
            "private_investment_unfunded_mismatch",
            format!(
                "Private investment {} unfunded commitment {} does not equal commitment {} minus paid-in {} plus recallable {}.",
                summary.investment.asset_id,
                summary.unfunded_commitment,
                summary.commitment,
                summary.paid_in_capital,
                summary.recallable_distributions
            ),
        )]
    }
}

pub fn validate_projected_fixed_income_cashflows(
    details: &FixedIncomeDetails,
    cashflows: &[ProjectedFixedIncomeCashflow],
) -> Vec<AccuracyInvariantViolation> {
    let mut violations = Vec::new();
    for cashflow in cashflows {
        if cashflow.currency != details.currency {
            violations.push(AccuracyInvariantViolation::new(
                "fixed_income_cashflow_currency_mismatch",
                format!(
                    "Cashflow on {} uses {}, expected {}.",
                    cashflow.expected_date, cashflow.currency, details.currency
                ),
            ));
        }
    }

    let principal_total = cashflows
        .iter()
        .filter(|cashflow| cashflow.cashflow_type == FixedIncomeCashflowType::Principal)
        .map(|cashflow| cashflow.expected_amount)
        .sum::<Decimal>();
    let maturity_total = cashflows
        .iter()
        .filter(|cashflow| cashflow.cashflow_type == FixedIncomeCashflowType::Maturity)
        .map(|cashflow| cashflow.expected_amount)
        .sum::<Decimal>();
    let income_total = cashflows
        .iter()
        .filter(|cashflow| {
            matches!(
                cashflow.cashflow_type,
                FixedIncomeCashflowType::Coupon
                    | FixedIncomeCashflowType::Profit
                    | FixedIncomeCashflowType::Interest
            )
        })
        .map(|cashflow| cashflow.expected_amount)
        .sum::<Decimal>();

    if details.instrument_type == crate::universal_assets::details::FixedIncomeSubtype::FixedDeposit
        || details.payment_frequency == Some(FixedIncomePaymentFrequency::AtMaturity)
    {
        let start = details.purchase_date.unwrap_or(details.maturity_date);
        let expected_maturity = details.face_value
            + accrued_interest_or_profit(
                details.face_value,
                details.coupon_or_profit_rate.unwrap_or(Decimal::ZERO),
                start,
                details.maturity_date,
                details.day_count_convention,
            );
        if maturity_total != expected_maturity {
            violations.push(AccuracyInvariantViolation::new(
                "fixed_income_maturity_total_mismatch",
                format!(
                    "Maturity cashflows {} do not equal face value plus accrued amount {}.",
                    maturity_total, expected_maturity
                ),
            ));
        }
        return violations;
    }

    if principal_total != details.face_value {
        violations.push(AccuracyInvariantViolation::new(
            "fixed_income_principal_total_mismatch",
            format!(
                "Principal cashflows {} do not equal face value {}.",
                principal_total, details.face_value
            ),
        ));
    }

    if let (Some(rate), Some(frequency), Some(purchase_date)) = (
        details.coupon_or_profit_rate,
        details.payment_frequency,
        details.purchase_date,
    ) {
        if frequency.months_between().is_some() {
            let expected_payment =
                details.face_value * rate / Decimal::from(frequency.payments_per_year());
            let coupon_count = cashflows
                .iter()
                .filter(|cashflow| {
                    matches!(
                        cashflow.cashflow_type,
                        FixedIncomeCashflowType::Coupon | FixedIncomeCashflowType::Profit
                    ) && cashflow.expected_date > purchase_date
                        && cashflow.expected_date < details.maturity_date
                })
                .count();
            let expected_income = expected_payment * Decimal::from(coupon_count);
            if income_total != expected_income {
                violations.push(AccuracyInvariantViolation::new(
                    "fixed_income_income_total_mismatch",
                    format!(
                        "Income cashflows {} do not equal scheduled income {}.",
                        income_total, expected_income
                    ),
                ));
            }
        }
    }

    violations
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixed_income::{
        generate_projected_cashflows, FixedIncomeDetails, FixedIncomePaymentFrequency,
    };
    use crate::portfolio::snapshot::{Lot, Position};
    use crate::private_investments::{
        calculate_private_investment_summary, CapitalCall, CapitalCallStatus, PrivateDistribution,
        PrivateInvestment,
    };
    use crate::universal_assets::details::{DayCountConvention, FixedIncomeSubtype};
    use chrono::{NaiveDate, TimeZone, Utc};
    use proptest::prelude::*;
    use rust_decimal_macros::dec;
    use std::collections::{HashMap, VecDeque};

    fn date(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day).unwrap()
    }

    fn lot(id: &str, quantity: Decimal, cost_basis: Decimal) -> Lot {
        Lot {
            id: id.to_string(),
            position_id: "pos-asset-1-account-1".into(),
            acquisition_date: Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap(),
            quantity,
            cost_basis,
            acquisition_price: if quantity.is_zero() {
                Decimal::ZERO
            } else {
                cost_basis / quantity
            },
            acquisition_fees: Decimal::ZERO,
            fx_rate_to_position: None,
        }
    }

    fn position(lots: Vec<Lot>) -> Position {
        let mut position = Position::new(
            "account-1".into(),
            "asset-1".into(),
            "USD".into(),
            Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap(),
        );
        position.lots = VecDeque::from(lots);
        position.recalculate_aggregates();
        position
    }

    #[test]
    fn position_lot_totals_match_open_holding() {
        let position = position(vec![
            lot("lot-1", dec!(3), dec!(30)),
            lot("lot-2", dec!(2), dec!(22)),
        ]);
        assert!(validate_position_lots(&position).is_empty());
    }

    #[test]
    fn position_lot_cost_basis_mismatch_is_reported() {
        let mut position = position(vec![lot("lot-1", dec!(3), dec!(30))]);
        position.total_cost_basis += dec!(0.01);
        let violations = validate_position_lots(&position);
        assert_eq!(violations[0].code, "position_lot_cost_basis_mismatch");
    }

    proptest! {
        #[test]
        fn prop_position_lot_totals_survive_recalculation(
            quantities in proptest::collection::vec(1i64..1_000_000, 1..10),
            cents in proptest::collection::vec(1i64..1_000_000, 1..10),
        ) {
            let lots = quantities
                .iter()
                .zip(cents.iter().cycle())
                .enumerate()
                .map(|(idx, (quantity, cents))| {
                    lot(
                        &format!("lot-{idx}"),
                        Decimal::from(*quantity) / dec!(10000),
                        Decimal::from(*cents) / dec!(100),
                    )
                })
                .collect::<Vec<_>>();
            let position = position(lots);
            prop_assert!(validate_position_lots(&position).is_empty());
        }
    }

    #[test]
    fn realized_gain_equals_proceeds_minus_cost_basis_minus_fees() {
        let entry = RealizedGainEntry {
            proceeds_account_ccy: dec!(1250),
            proceeds_base_ccy: dec!(1250),
            cost_basis_account_ccy: dec!(900),
            cost_basis_base_ccy: dec!(900),
            fees_account_ccy: dec!(12.50),
            fees_base_ccy: dec!(12.50),
            quantity_sold: dec!(10),
            last_sale_date: Some(date(2026, 5, 1)),
        };
        assert!(validate_realized_gain(&entry, dec!(337.50), dec!(337.50)).is_empty());
    }

    #[test]
    fn cash_ledger_must_equal_cash_balances() {
        let ledger = vec![("USD".into(), dec!(1000)), ("USD".into(), dec!(-125.25))];
        let mut balances = HashMap::new();
        balances.insert("USD".into(), dec!(874.75));
        assert!(validate_cash_ledger(&ledger, &balances).is_empty());
    }

    #[test]
    fn report_total_must_equal_line_sum() {
        assert!(
            validate_report_total("tax-pack", &[dec!(10.10), dec!(20.20)], dec!(30.30)).is_empty()
        );
    }

    #[test]
    fn private_investment_paid_in_unfunded_invariant() {
        let investment = PrivateInvestment {
            asset_id: "fund-1".into(),
            manager: "Manager".into(),
            strategy: "Credit".into(),
            vintage_year: Some(2026),
            commitment_amount: dec!(1000),
            commitment_currency: "USD".into(),
            inception_date: None,
            notes: None,
        };
        let calls = vec![CapitalCall {
            id: "call-1".into(),
            asset_id: "fund-1".into(),
            notice_date: date(2026, 1, 1),
            due_date: date(2026, 1, 15),
            amount: dec!(300),
            currency: "USD".into(),
            status: CapitalCallStatus::Paid,
            source_citation_id: None,
            notes: None,
        }];
        let distributions = vec![PrivateDistribution {
            id: "dist-1".into(),
            asset_id: "fund-1".into(),
            distribution_date: date(2026, 8, 1),
            amount: dec!(50),
            currency: "USD".into(),
            recallable: true,
            source_citation_id: None,
            notes: None,
        }];
        let summary = calculate_private_investment_summary(investment, &[], &calls, &distributions);
        assert!(validate_private_investment_summary(&summary).is_empty());
    }

    #[test]
    fn fixed_income_cashflow_totals_match_schedule() {
        let details = FixedIncomeDetails {
            asset_id: "bond-1".into(),
            instrument_type: FixedIncomeSubtype::Bond,
            issuer: "Treasury".into(),
            isin: None,
            face_value: dec!(1000),
            currency: "USD".into(),
            purchase_date: Some(date(2026, 1, 1)),
            maturity_date: date(2027, 1, 1),
            coupon_or_profit_rate: Some(dec!(0.06)),
            payment_frequency: Some(FixedIncomePaymentFrequency::SemiAnnual),
            day_count_convention: DayCountConvention::Act365,
            is_sukuk: false,
            source_citation_id: None,
        };
        let (cashflows, warnings) = generate_projected_cashflows(&details).unwrap();
        assert!(warnings.is_empty());
        assert!(validate_projected_fixed_income_cashflows(&details, &cashflows).is_empty());
    }
}
