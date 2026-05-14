use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::str::FromStr;

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::errors::{Error, ValidationError};
use crate::Result;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReconciliationScopeType {
    Account,
    Asset,
    Document,
    Import,
}

impl ReconciliationScopeType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Account => "account",
            Self::Asset => "asset",
            Self::Document => "document",
            Self::Import => "import",
        }
    }
}

impl FromStr for ReconciliationScopeType {
    type Err = Error;

    fn from_str(value: &str) -> Result<Self> {
        match value {
            "account" => Ok(Self::Account),
            "asset" => Ok(Self::Asset),
            "document" => Ok(Self::Document),
            "import" => Ok(Self::Import),
            _ => Err(invalid(format!(
                "Unsupported reconciliation scope: {value}"
            ))),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReconciliationRunStatus {
    Open,
    Completed,
    Failed,
}

impl ReconciliationRunStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::Completed => "completed",
            Self::Failed => "failed",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReconciliationSourceSide {
    Mizan,
    External,
}

impl ReconciliationSourceSide {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Mizan => "mizan",
            Self::External => "external",
        }
    }
}

impl FromStr for ReconciliationSourceSide {
    type Err = Error;

    fn from_str(value: &str) -> Result<Self> {
        match value {
            "mizan" => Ok(Self::Mizan),
            "external" => Ok(Self::External),
            _ => Err(invalid(format!("Unsupported reconciliation side: {value}"))),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReconciliationItemStatus {
    Open,
    Ignored,
    AcceptedAdjustment,
}

impl ReconciliationItemStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::Ignored => "ignored",
            Self::AcceptedAdjustment => "accepted_adjustment",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReconciliationMatchStatus {
    Matched,
    PossibleMatch,
    MissingInMizan,
    MissingInExternal,
    Duplicate,
    Mismatch,
}

impl ReconciliationMatchStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Matched => "matched",
            Self::PossibleMatch => "possible_match",
            Self::MissingInMizan => "missing_in_mizan",
            Self::MissingInExternal => "missing_in_external",
            Self::Duplicate => "duplicate",
            Self::Mismatch => "mismatch",
        }
    }
}

impl FromStr for ReconciliationMatchStatus {
    type Err = Error;

    fn from_str(value: &str) -> Result<Self> {
        match value {
            "matched" => Ok(Self::Matched),
            "possible_match" => Ok(Self::PossibleMatch),
            "missing_in_mizan" => Ok(Self::MissingInMizan),
            "missing_in_external" => Ok(Self::MissingInExternal),
            "duplicate" => Ok(Self::Duplicate),
            "mismatch" => Ok(Self::Mismatch),
            _ => Err(invalid(format!(
                "Unsupported reconciliation match: {value}"
            ))),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReconciliationInputItem {
    pub id: Option<String>,
    pub item_type: String,
    pub raw_json: Value,
    pub amount: Option<String>,
    pub currency: Option<String>,
    pub effective_date: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReconciliationItem {
    pub id: String,
    pub run_id: String,
    pub item_type: String,
    pub source_side: ReconciliationSourceSide,
    pub raw_json: Value,
    pub normalized_hash: String,
    pub amount: Option<String>,
    pub currency: Option<String>,
    pub effective_date: Option<String>,
    pub status: ReconciliationItemStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReconciliationMatch {
    pub id: String,
    pub run_id: String,
    pub mizan_item_id: Option<String>,
    pub external_item_id: Option<String>,
    pub match_status: ReconciliationMatchStatus,
    pub confidence: String,
    pub reason: String,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReconciliationRun {
    pub id: String,
    pub scope_type: ReconciliationScopeType,
    pub scope_id: String,
    pub status: ReconciliationRunStatus,
    pub date_tolerance_days: i64,
    pub created_at: String,
    pub completed_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReconciliationRunDetail {
    pub run: ReconciliationRun,
    pub items: Vec<ReconciliationItem>,
    pub matches: Vec<ReconciliationMatch>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReconcileImportPreviewRequest {
    pub scope_type: ReconciliationScopeType,
    pub scope_id: String,
    pub mizan_items: Vec<ReconciliationInputItem>,
    pub external_items: Vec<ReconciliationInputItem>,
    pub date_tolerance_days: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReconcileAccountRequest {
    pub account_id: String,
    pub external_items: Vec<ReconciliationInputItem>,
    pub date_tolerance_days: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReconcileDocumentFactsRequest {
    pub document_id: String,
    pub account_id: Option<String>,
    pub date_tolerance_days: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AcceptReconciliationAdjustmentRequest {
    pub match_id: String,
    pub account_id: String,
    pub activity_type: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AcceptReconciliationAdjustmentResult {
    pub activity_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IgnoreReconciliationMatchRequest {
    pub match_id: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManualReconciliationMatchRequest {
    pub run_id: String,
    pub mizan_item_id: String,
    pub external_item_id: String,
    pub reason: String,
}

#[async_trait::async_trait]
pub trait ReconciliationRepositoryTrait: Send + Sync {
    async fn reconcile_import_preview(
        &self,
        request: ReconcileImportPreviewRequest,
    ) -> Result<ReconciliationRunDetail>;
    async fn reconcile_account(
        &self,
        request: ReconcileAccountRequest,
    ) -> Result<ReconciliationRunDetail>;
    async fn reconcile_document_facts(
        &self,
        request: ReconcileDocumentFactsRequest,
    ) -> Result<ReconciliationRunDetail>;
    fn get_reconciliation_run(&self, run_id: &str) -> Result<ReconciliationRunDetail>;
    async fn accept_adjustment(
        &self,
        request: AcceptReconciliationAdjustmentRequest,
    ) -> Result<AcceptReconciliationAdjustmentResult>;
    async fn ignore_match(&self, request: IgnoreReconciliationMatchRequest) -> Result<()>;
    async fn manual_match(
        &self,
        request: ManualReconciliationMatchRequest,
    ) -> Result<ReconciliationMatch>;
}

pub fn normalized_hash(item: &ReconciliationInputItem) -> Result<String> {
    validate_item(item)?;
    let amount = item.amount.as_deref().map(parse_decimal).transpose()?;
    let mut hasher = Sha256::new();
    hasher.update(item.item_type.trim().as_bytes());
    hasher.update(b"|");
    hasher.update(
        item.currency
            .as_deref()
            .map(str::to_uppercase)
            .unwrap_or_default()
            .as_bytes(),
    );
    hasher.update(b"|");
    hasher.update(
        item.effective_date
            .as_deref()
            .unwrap_or_default()
            .trim()
            .as_bytes(),
    );
    hasher.update(b"|");
    hasher.update(
        amount
            .map(|value| value.normalize().to_string())
            .unwrap_or_default(),
    );
    Ok(hex::encode(hasher.finalize()))
}

pub fn build_reconciliation_matches(
    run_id: &str,
    mizan_items: &[ReconciliationItem],
    external_items: &[ReconciliationItem],
    date_tolerance_days: i64,
    created_at: &str,
) -> Vec<ReconciliationMatch> {
    let mut matches = Vec::new();
    let mut used_mizan = HashSet::new();
    let mut used_external = HashSet::new();
    let duplicates = duplicate_item_ids(mizan_items, external_items);

    for item_id in duplicates {
        let (mizan_item_id, external_item_id) = split_duplicate_id(&item_id);
        matches.push(match_row(
            run_id,
            mizan_item_id,
            external_item_id,
            ReconciliationMatchStatus::Duplicate,
            "0.00",
            "Duplicate normalized reconciliation item.",
            created_at,
        ));
    }

    for external in external_items {
        if used_external.contains(&external.id) {
            continue;
        }
        if let Some(mizan) = mizan_items
            .iter()
            .find(|mizan| !used_mizan.contains(&mizan.id) && exact_match(mizan, external))
        {
            used_mizan.insert(mizan.id.clone());
            used_external.insert(external.id.clone());
            matches.push(match_row(
                run_id,
                Some(mizan.id.clone()),
                Some(external.id.clone()),
                ReconciliationMatchStatus::Matched,
                "1.00",
                "Amount, currency, and effective date match exactly.",
                created_at,
            ));
        }
    }

    for external in external_items {
        if used_external.contains(&external.id) {
            continue;
        }
        if let Some(mizan) = mizan_items.iter().find(|mizan| {
            !used_mizan.contains(&mizan.id) && tolerant_match(mizan, external, date_tolerance_days)
        }) {
            used_mizan.insert(mizan.id.clone());
            used_external.insert(external.id.clone());
            matches.push(match_row(
                run_id,
                Some(mizan.id.clone()),
                Some(external.id.clone()),
                ReconciliationMatchStatus::PossibleMatch,
                "0.85",
                "Amount and currency match within the configured date tolerance.",
                created_at,
            ));
        }
    }

    for external in external_items {
        if used_external.contains(&external.id) {
            continue;
        }
        if let Some(mizan) = mizan_items.iter().find(|mizan| {
            !used_mizan.contains(&mizan.id) && same_currency_and_date(mizan, external)
        }) {
            used_mizan.insert(mizan.id.clone());
            used_external.insert(external.id.clone());
            matches.push(match_row(
                run_id,
                Some(mizan.id.clone()),
                Some(external.id.clone()),
                ReconciliationMatchStatus::Mismatch,
                "0.50",
                "Currency and date match, but amount differs.",
                created_at,
            ));
        }
    }

    for external in external_items {
        if !used_external.contains(&external.id) {
            matches.push(match_row(
                run_id,
                None,
                Some(external.id.clone()),
                ReconciliationMatchStatus::MissingInMizan,
                "0.00",
                "External item has no matching Mizan item.",
                created_at,
            ));
        }
    }
    for mizan in mizan_items {
        if !used_mizan.contains(&mizan.id) {
            matches.push(match_row(
                run_id,
                Some(mizan.id.clone()),
                None,
                ReconciliationMatchStatus::MissingInExternal,
                "0.00",
                "Mizan item has no matching external item.",
                created_at,
            ));
        }
    }

    matches
}

pub fn validate_date_tolerance(days: i64) -> Result<()> {
    if !(0..=31).contains(&days) {
        return Err(invalid("date_tolerance_days must be between 0 and 31"));
    }
    Ok(())
}

fn validate_item(item: &ReconciliationInputItem) -> Result<()> {
    if item.item_type.trim().is_empty() {
        return Err(invalid("item_type is required"));
    }
    if let Some(amount) = item.amount.as_deref() {
        parse_decimal(amount)?;
    }
    if let Some(currency) = item.currency.as_deref() {
        let trimmed = currency.trim();
        if trimmed.len() != 3 || !trimmed.chars().all(|value| value.is_ascii_alphabetic()) {
            return Err(invalid("currency must be an ISO 4217 code"));
        }
    }
    if let Some(date) = item.effective_date.as_deref() {
        parse_date(date)?;
    }
    Ok(())
}

fn exact_match(left: &ReconciliationItem, right: &ReconciliationItem) -> bool {
    same_amount(left, right)
        && same_currency(left, right)
        && left.effective_date == right.effective_date
}

fn tolerant_match(
    left: &ReconciliationItem,
    right: &ReconciliationItem,
    date_tolerance_days: i64,
) -> bool {
    same_amount(left, right)
        && same_currency(left, right)
        && date_distance(left, right)
            .map(|days| days <= date_tolerance_days)
            .unwrap_or(false)
}

fn same_currency_and_date(left: &ReconciliationItem, right: &ReconciliationItem) -> bool {
    same_currency(left, right) && left.effective_date == right.effective_date
}

fn same_amount(left: &ReconciliationItem, right: &ReconciliationItem) -> bool {
    match (left.amount.as_deref(), right.amount.as_deref()) {
        (Some(left), Some(right)) => parse_decimal(left).ok() == parse_decimal(right).ok(),
        (None, None) => true,
        _ => false,
    }
}

fn same_currency(left: &ReconciliationItem, right: &ReconciliationItem) -> bool {
    left.currency.as_deref().map(str::to_uppercase)
        == right.currency.as_deref().map(str::to_uppercase)
}

fn date_distance(left: &ReconciliationItem, right: &ReconciliationItem) -> Option<i64> {
    let left = parse_date(left.effective_date.as_deref()?).ok()?;
    let right = parse_date(right.effective_date.as_deref()?).ok()?;
    Some((left - right).num_days().abs())
}

fn duplicate_item_ids(
    mizan_items: &[ReconciliationItem],
    external_items: &[ReconciliationItem],
) -> BTreeSet<String> {
    let mut seen: HashMap<(ReconciliationSourceSide, String), String> = HashMap::new();
    let mut duplicate_ids = BTreeSet::new();
    for item in mizan_items.iter().chain(external_items.iter()) {
        let key = (item.source_side, item.normalized_hash.clone());
        if let Some(first_id) = seen.insert(key, item.id.clone()) {
            duplicate_ids.insert(duplicate_key(item.source_side, first_id));
            duplicate_ids.insert(duplicate_key(item.source_side, item.id.clone()));
        }
    }
    duplicate_ids
}

fn duplicate_key(side: ReconciliationSourceSide, id: String) -> String {
    format!("{}:{id}", side.as_str())
}

fn split_duplicate_id(value: &str) -> (Option<String>, Option<String>) {
    if let Some(id) = value.strip_prefix("mizan:") {
        (Some(id.to_string()), None)
    } else if let Some(id) = value.strip_prefix("external:") {
        (None, Some(id.to_string()))
    } else {
        (None, None)
    }
}

fn match_row(
    run_id: &str,
    mizan_item_id: Option<String>,
    external_item_id: Option<String>,
    status: ReconciliationMatchStatus,
    confidence: &str,
    reason: &str,
    created_at: &str,
) -> ReconciliationMatch {
    let mut seed = BTreeMap::new();
    seed.insert("run_id", run_id.to_string());
    seed.insert("mizan_item_id", mizan_item_id.clone().unwrap_or_default());
    seed.insert(
        "external_item_id",
        external_item_id.clone().unwrap_or_default(),
    );
    seed.insert("status", status.as_str().to_string());
    seed.insert("reason", reason.to_string());
    let id = deterministic_id(&serde_json::to_value(seed).expect("serializable match seed"));
    ReconciliationMatch {
        id,
        run_id: run_id.to_string(),
        mizan_item_id,
        external_item_id,
        match_status: status,
        confidence: confidence.to_string(),
        reason: reason.to_string(),
        created_at: created_at.to_string(),
    }
}

fn deterministic_id(seed: &Value) -> String {
    let mut hasher = Sha256::new();
    hasher.update(seed.to_string());
    format!("recon-{}", &hex::encode(hasher.finalize())[..24])
}

fn parse_decimal(value: &str) -> Result<Decimal> {
    Decimal::from_str(value.trim()).map_err(|_| invalid("amount must be a decimal string"))
}

fn parse_date(value: &str) -> Result<NaiveDate> {
    NaiveDate::parse_from_str(value.trim(), "%Y-%m-%d")
        .map_err(|_| invalid("effective_date must use YYYY-MM-DD"))
}

fn invalid(message: impl Into<String>) -> Error {
    Error::Validation(ValidationError::InvalidInput(message.into()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(
        id: &str,
        side: ReconciliationSourceSide,
        amount: &str,
        date: &str,
    ) -> ReconciliationItem {
        ReconciliationItem {
            id: id.to_string(),
            run_id: "run-1".into(),
            item_type: "activity".into(),
            source_side: side,
            raw_json: serde_json::json!({ "id": id }),
            normalized_hash: normalized_hash(&ReconciliationInputItem {
                id: Some(id.into()),
                item_type: "activity".into(),
                raw_json: serde_json::json!({ "id": id }),
                amount: Some(amount.into()),
                currency: Some("USD".into()),
                effective_date: Some(date.into()),
            })
            .expect("hash"),
            amount: Some(amount.into()),
            currency: Some("USD".into()),
            effective_date: Some(date.into()),
            status: ReconciliationItemStatus::Open,
        }
    }

    #[test]
    fn exact_match_pairs_amount_currency_and_date() {
        let matches = build_reconciliation_matches(
            "run-1",
            &[item(
                "m1",
                ReconciliationSourceSide::Mizan,
                "10.00",
                "2026-05-14",
            )],
            &[item(
                "e1",
                ReconciliationSourceSide::External,
                "10.0",
                "2026-05-14",
            )],
            0,
            "now",
        );
        assert_eq!(matches[0].match_status, ReconciliationMatchStatus::Matched);
    }

    #[test]
    fn date_tolerance_creates_possible_match() {
        let matches = build_reconciliation_matches(
            "run-1",
            &[item(
                "m1",
                ReconciliationSourceSide::Mizan,
                "10",
                "2026-05-15",
            )],
            &[item(
                "e1",
                ReconciliationSourceSide::External,
                "10",
                "2026-05-14",
            )],
            2,
            "now",
        );
        assert_eq!(
            matches[0].match_status,
            ReconciliationMatchStatus::PossibleMatch
        );
    }

    #[test]
    fn duplicate_detection_marks_repeated_hashes() {
        let left = item("m1", ReconciliationSourceSide::Mizan, "10", "2026-05-14");
        let mut right = item("m2", ReconciliationSourceSide::Mizan, "10.0", "2026-05-14");
        right.normalized_hash = left.normalized_hash.clone();
        let matches = build_reconciliation_matches("run-1", &[left, right], &[], 0, "now");
        assert!(matches
            .iter()
            .any(|row| row.match_status == ReconciliationMatchStatus::Duplicate));
    }

    #[test]
    fn missing_in_mizan_is_reported_for_unmatched_external_item() {
        let matches = build_reconciliation_matches(
            "run-1",
            &[],
            &[item(
                "e1",
                ReconciliationSourceSide::External,
                "10",
                "2026-05-14",
            )],
            0,
            "now",
        );
        assert_eq!(
            matches[0].match_status,
            ReconciliationMatchStatus::MissingInMizan
        );
    }

    #[test]
    fn mismatch_is_reported_for_same_date_currency_different_amount() {
        let matches = build_reconciliation_matches(
            "run-1",
            &[item(
                "m1",
                ReconciliationSourceSide::Mizan,
                "11",
                "2026-05-14",
            )],
            &[item(
                "e1",
                ReconciliationSourceSide::External,
                "10",
                "2026-05-14",
            )],
            0,
            "now",
        );
        assert_eq!(matches[0].match_status, ReconciliationMatchStatus::Mismatch);
    }
}
