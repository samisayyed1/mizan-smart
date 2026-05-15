use mizan_core::accuracy_invariants::{
    validate_cash_ledger, validate_realized_gain, validate_report_total,
};
use mizan_core::portfolio::snapshot::RealizedGainEntry;
use rust_decimal::Decimal;
use serde::Deserialize;
use std::collections::HashMap;
use std::str::FromStr;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GoldenScenarios {
    realized_gain: GoldenRealizedGain,
    report: GoldenReport,
    cash_ledger: GoldenCashLedger,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GoldenRealizedGain {
    proceeds: String,
    cost_basis: String,
    fees: String,
    expected_gain: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GoldenReport {
    name: String,
    line_amounts: Vec<String>,
    reported_total: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GoldenCashLedger {
    deltas: Vec<(String, String)>,
    balances: HashMap<String, String>,
}

fn decimal(value: &str) -> Decimal {
    Decimal::from_str(value).expect("golden decimal should parse")
}

#[test]
fn golden_accuracy_invariant_scenarios_hold() {
    let scenarios: GoldenScenarios = serde_json::from_str(include_str!(
        "golden/accuracy_invariants/core_scenarios.json"
    ))
    .expect("golden scenarios should parse");

    let gain = RealizedGainEntry {
        proceeds_account_ccy: decimal(&scenarios.realized_gain.proceeds),
        proceeds_base_ccy: decimal(&scenarios.realized_gain.proceeds),
        cost_basis_account_ccy: decimal(&scenarios.realized_gain.cost_basis),
        cost_basis_base_ccy: decimal(&scenarios.realized_gain.cost_basis),
        fees_account_ccy: decimal(&scenarios.realized_gain.fees),
        fees_base_ccy: decimal(&scenarios.realized_gain.fees),
        quantity_sold: Decimal::ONE,
        last_sale_date: None,
    };
    let expected_gain = decimal(&scenarios.realized_gain.expected_gain);
    assert!(validate_realized_gain(&gain, expected_gain, expected_gain).is_empty());

    let line_amounts = scenarios
        .report
        .line_amounts
        .iter()
        .map(|value| decimal(value))
        .collect::<Vec<_>>();
    assert!(validate_report_total(
        &scenarios.report.name,
        &line_amounts,
        decimal(&scenarios.report.reported_total),
    )
    .is_empty());

    let ledger = scenarios
        .cash_ledger
        .deltas
        .iter()
        .map(|(currency, amount)| (currency.clone(), decimal(amount)))
        .collect::<Vec<_>>();
    let balances = scenarios
        .cash_ledger
        .balances
        .iter()
        .map(|(currency, amount)| (currency.clone(), decimal(amount)))
        .collect::<HashMap<_, _>>();
    assert!(validate_cash_ledger(&ledger, &balances).is_empty());
}
