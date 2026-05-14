use serde::{Deserialize, Serialize};

use crate::Result;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DataLineageEntityType {
    Portfolio,
    Account,
    Asset,
    Valuation,
    Alert,
}

impl DataLineageEntityType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Portfolio => "portfolio",
            Self::Account => "account",
            Self::Asset => "asset",
            Self::Valuation => "valuation",
            Self::Alert => "alert",
        }
    }
}

impl TryFrom<&str> for DataLineageEntityType {
    type Error = String;

    fn try_from(value: &str) -> std::result::Result<Self, Self::Error> {
        match value.trim() {
            "portfolio" => Ok(Self::Portfolio),
            "account" => Ok(Self::Account),
            "asset" => Ok(Self::Asset),
            "valuation" => Ok(Self::Valuation),
            "alert" => Ok(Self::Alert),
            other => Err(format!("Unsupported lineage entity type '{other}'")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DataLineageMetricType {
    NetWorth,
    AssetValue,
    Valuation,
    IncomeThisMonth,
    DataQualityScore,
    AlertReason,
    PrivateInvestmentMetric,
    TaxPackLine,
    ZakatLine,
}

impl DataLineageMetricType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::NetWorth => "net_worth",
            Self::AssetValue => "asset_value",
            Self::Valuation => "valuation",
            Self::IncomeThisMonth => "income_this_month",
            Self::DataQualityScore => "data_quality_score",
            Self::AlertReason => "alert_reason",
            Self::PrivateInvestmentMetric => "private_investment_metric",
            Self::TaxPackLine => "tax_pack_line",
            Self::ZakatLine => "zakat_line",
        }
    }
}

impl TryFrom<&str> for DataLineageMetricType {
    type Error = String;

    fn try_from(value: &str) -> std::result::Result<Self, Self::Error> {
        match value.trim() {
            "net_worth" => Ok(Self::NetWorth),
            "asset_value" => Ok(Self::AssetValue),
            "valuation" => Ok(Self::Valuation),
            "income_this_month" => Ok(Self::IncomeThisMonth),
            "data_quality_score" => Ok(Self::DataQualityScore),
            "alert_reason" => Ok(Self::AlertReason),
            "private_investment_metric" => Ok(Self::PrivateInvestmentMetric),
            "tax_pack_line" => Ok(Self::TaxPackLine),
            "zakat_line" => Ok(Self::ZakatLine),
            other => Err(format!("Unsupported lineage metric type '{other}'")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DataLineageRequest {
    pub entity_type: DataLineageEntityType,
    pub entity_id: String,
    pub metric_type: DataLineageMetricType,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DataLineageResponse {
    pub entity_type: DataLineageEntityType,
    pub entity_id: String,
    pub metric_type: DataLineageMetricType,
    pub displayed_value: String,
    pub currency: Option<String>,
    pub formula_name: String,
    pub formula_description: String,
    pub input_rows: Vec<DataLineageInputRow>,
    pub source_citations: Vec<DataLineageSourceCitation>,
    pub source_documents: Vec<DataLineageSourceDocument>,
    pub fx_rates_used: Vec<DataLineageFxRate>,
    pub valuation_dates: Vec<String>,
    pub rounding_policy: String,
    pub warnings: Vec<String>,
    pub confidence: Option<String>,
    pub freshness: Option<String>,
    pub last_updated: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DataLineageInputRow {
    pub source_table: String,
    pub source_id: String,
    pub label: String,
    pub value: String,
    pub currency: Option<String>,
    pub as_of_date: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DataLineageSourceCitation {
    pub id: String,
    pub label: String,
    pub source_type: String,
    pub source_id: Option<String>,
    pub document_id: Option<String>,
    pub extracted_fact_id: Option<String>,
    pub page_number: Option<i32>,
    pub bounding_box_json: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DataLineageSourceDocument {
    pub id: String,
    pub name: String,
    pub page_number: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DataLineageFxRate {
    pub from_currency: String,
    pub to_currency: String,
    pub rate: String,
    pub as_of_date: Option<String>,
}

pub trait DataLineageRepositoryTrait: Send + Sync {
    fn get_data_lineage(&self, request: DataLineageRequest) -> Result<DataLineageResponse>;
}
