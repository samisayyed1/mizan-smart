use async_trait::async_trait;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

use crate::errors::ValidationError;
use crate::{Error, Result};

pub const REPORT_BUILDER_DISCLAIMER: &str =
    "Deterministic report preview only. Mizan does not provide investment, tax, or legal advice.";
pub const ESTATE_BINDER_DISCLAIMER: &str =
    "Estate Binder is an organizational checklist only. It is not legal advice and does not generate wills, trusts, or estate documents.";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReportType {
    NetWorth,
    PortfolioSummary,
    Income,
    DataQuality,
    TaxPack,
    MonthlyWealthLetter,
    EstateBinder,
}

impl ReportType {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::NetWorth => "net_worth",
            Self::PortfolioSummary => "portfolio_summary",
            Self::Income => "income",
            Self::DataQuality => "data_quality",
            Self::TaxPack => "tax_pack",
            Self::MonthlyWealthLetter => "monthly_wealth_letter",
            Self::EstateBinder => "estate_binder",
        }
    }

    pub const fn title(self) -> &'static str {
        match self {
            Self::NetWorth => "Net Worth Report",
            Self::PortfolioSummary => "Portfolio Summary",
            Self::Income => "Income Report",
            Self::DataQuality => "Data Quality Report",
            Self::TaxPack => "Tax Pack Report",
            Self::MonthlyWealthLetter => "Monthly Wealth Letter",
            Self::EstateBinder => "Estate Binder",
        }
    }
}

impl FromStr for ReportType {
    type Err = Error;

    fn from_str(value: &str) -> Result<Self> {
        match value {
            "net_worth" => Ok(Self::NetWorth),
            "portfolio_summary" => Ok(Self::PortfolioSummary),
            "income" => Ok(Self::Income),
            "data_quality" => Ok(Self::DataQuality),
            "tax_pack" => Ok(Self::TaxPack),
            "monthly_wealth_letter" => Ok(Self::MonthlyWealthLetter),
            "estate_binder" => Ok(Self::EstateBinder),
            _ => Err(invalid(format!("Unsupported report type: {value}"))),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EstateBinderSection {
    Accounts,
    Assets,
    Liabilities,
    Property,
    Insurance,
    Pensions,
    PrivateInvestments,
    DocumentsManifest,
    EntityOwnership,
    IslamicNotes,
}

impl EstateBinderSection {
    pub const fn title(self) -> &'static str {
        match self {
            Self::Accounts => "Accounts",
            Self::Assets => "Assets",
            Self::Liabilities => "Liabilities",
            Self::Property => "Property",
            Self::Insurance => "Insurance / ULIP",
            Self::Pensions => "Pensions",
            Self::PrivateInvestments => "Private investments",
            Self::DocumentsManifest => "Documents manifest",
            Self::EntityOwnership => "Entity ownership summary",
            Self::IslamicNotes => "Zakat / waqf / charity notes",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReportRunStatus {
    Generated,
    Exported,
}

impl ReportRunStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Generated => "generated",
            Self::Exported => "exported",
        }
    }
}

impl FromStr for ReportRunStatus {
    type Err = Error;

    fn from_str(value: &str) -> Result<Self> {
        match value {
            "generated" => Ok(Self::Generated),
            "exported" => Ok(Self::Exported),
            _ => Err(invalid(format!("Unsupported report run status: {value}"))),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerateReportRequest {
    pub report_type: ReportType,
    pub base_currency: String,
    pub period_month: Option<String>,
    pub included_sections: Option<Vec<EstateBinderSection>>,
}

impl GenerateReportRequest {
    pub fn validate(&self) -> Result<()> {
        validate_currency(&self.base_currency)?;
        if let Some(period_month) = &self.period_month {
            validate_period_month(period_month)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportRun {
    pub id: String,
    pub report_type: ReportType,
    pub base_currency: String,
    pub status: ReportRunStatus,
    pub created_at: String,
    pub completed_at: Option<String>,
    pub sections: Vec<ReportSection>,
    pub disclaimer: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportSection {
    pub id: String,
    pub report_run_id: String,
    pub title: String,
    pub section_order: i32,
    pub metadata_json: Option<String>,
    pub lines: Vec<ReportLine>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportLine {
    pub id: String,
    pub section_id: String,
    pub label: String,
    pub amount: Option<Decimal>,
    pub currency: Option<String>,
    pub value_text: Option<String>,
    pub source_citation_id: Option<String>,
    pub metadata_json: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportExportBundle {
    pub file_name: String,
    pub mime_type: String,
    pub bytes: Vec<u8>,
}

#[async_trait]
pub trait ReportBuilderRepositoryTrait: Send + Sync {
    async fn generate_report(&self, request: GenerateReportRequest) -> Result<ReportRun>;
    fn get_report_run(&self, report_run_id: &str) -> Result<Option<ReportRun>>;
    fn export_report(&self, report_run_id: &str) -> Result<ReportExportBundle>;
}

pub fn build_empty_report(
    run_id: String,
    section_id: String,
    line_id: String,
    request: GenerateReportRequest,
    created_at: String,
) -> Result<ReportRun> {
    request.validate()?;
    let section = ReportSection {
        id: section_id.clone(),
        report_run_id: run_id.clone(),
        title: request.report_type.title().to_string(),
        section_order: 0,
        metadata_json: Some("{\"emptyState\":true}".to_string()),
        lines: vec![ReportLine {
            id: line_id,
            section_id,
            label: "No report data".to_string(),
            amount: None,
            currency: None,
            value_text: Some(
                "No deterministic source rows were available; no synthetic report lines were created."
                    .to_string(),
            ),
            source_citation_id: None,
            metadata_json: Some("{\"citationStatus\":\"missing\"}".to_string()),
        }],
    };

    Ok(ReportRun {
        id: run_id,
        report_type: request.report_type,
        base_currency: request.base_currency,
        status: ReportRunStatus::Generated,
        created_at: created_at.clone(),
        completed_at: Some(created_at),
        sections: vec![section],
        disclaimer: REPORT_BUILDER_DISCLAIMER.to_string(),
    })
}

pub fn build_report_export(report: &ReportRun) -> ReportExportBundle {
    let html = render_report_html(report);
    ReportExportBundle {
        file_name: format!(
            "report-{}-{}.html",
            report.report_type.as_str(),
            sanitize_file_token(&report.id)
        ),
        mime_type: "text/html".to_string(),
        bytes: html.into_bytes(),
    }
}

fn render_report_html(report: &ReportRun) -> String {
    let sections = report
        .sections
        .iter()
        .map(|section| {
            let rows = section
                .lines
                .iter()
                .map(|line| {
                    let amount = line
                        .amount
                        .map(|value| value.normalize().to_string())
                        .or_else(|| line.value_text.clone())
                        .unwrap_or_default();
                    let currency = line.currency.clone().unwrap_or_default();
                    let citation = line
                        .source_citation_id
                        .as_deref()
                        .unwrap_or("Missing citation");
                    format!(
                        "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
                        escape_html(&line.label),
                        escape_html(&amount),
                        escape_html(&currency),
                        escape_html(citation)
                    )
                })
                .collect::<String>();
            format!(
                "<h2>{}</h2><table><thead><tr><th>Line</th><th>Value</th><th>Currency</th><th>Citation</th></tr></thead><tbody>{}</tbody></table>",
                escape_html(&section.title),
                rows
            )
        })
        .collect::<String>();

    format!(
        "<!doctype html><html><head><meta charset=\"utf-8\"><title>{}</title></head><body><h1>{}</h1><p>{}</p><dl><dt>Run</dt><dd>{}</dd><dt>Status</dt><dd>{}</dd><dt>Base currency</dt><dd>{}</dd></dl>{}</body></html>",
        escape_html(report.report_type.title()),
        escape_html(report.report_type.title()),
        escape_html(&report.disclaimer),
        escape_html(&report.id),
        escape_html(report.status.as_str()),
        escape_html(&report.base_currency),
        sections
    )
}

fn sanitize_file_token(value: &str) -> String {
    value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch
            } else {
                '-'
            }
        })
        .collect()
}

fn validate_currency(value: &str) -> Result<()> {
    if value.len() != 3 || value.chars().any(|ch| !ch.is_ascii_uppercase()) {
        return Err(invalid(
            "base_currency must be a 3-letter uppercase ISO code",
        ));
    }
    Ok(())
}

fn validate_period_month(value: &str) -> Result<()> {
    if value.len() != 7 {
        return Err(invalid("period_month must use YYYY-MM format"));
    }
    let (year, suffix) = value.split_at(4);
    if suffix.as_bytes().first() != Some(&b'-')
        || !year.chars().all(|ch| ch.is_ascii_digit())
        || !suffix[1..].chars().all(|ch| ch.is_ascii_digit())
    {
        return Err(invalid("period_month must use YYYY-MM format"));
    }
    let month = suffix[1..]
        .parse::<u32>()
        .map_err(|_| invalid("period_month must use YYYY-MM format"))?;
    if !(1..=12).contains(&month) {
        return Err(invalid("period_month month must be 01 through 12"));
    }
    Ok(())
}

fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn invalid(message: impl Into<String>) -> Error {
    Error::Validation(ValidationError::InvalidInput(message.into()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn empty_report_has_honest_missing_citation_line() {
        let report = build_empty_report(
            "run-1".to_string(),
            "section-1".to_string(),
            "line-1".to_string(),
            GenerateReportRequest {
                report_type: ReportType::DataQuality,
                base_currency: "USD".to_string(),
                period_month: None,
                included_sections: None,
            },
            "2026-05-16T00:00:00Z".to_string(),
        )
        .expect("report");

        assert_eq!(report.sections[0].lines[0].source_citation_id, None);
        assert!(report.sections[0].lines[0]
            .value_text
            .as_deref()
            .expect("value")
            .contains("No deterministic source rows"));
    }

    #[test]
    fn export_html_preserves_decimal_precision_and_disclaimer() {
        let report = ReportRun {
            id: "run-1".to_string(),
            report_type: ReportType::Income,
            base_currency: "USD".to_string(),
            status: ReportRunStatus::Generated,
            created_at: "2026-05-16T00:00:00Z".to_string(),
            completed_at: Some("2026-05-16T00:00:00Z".to_string()),
            disclaimer: REPORT_BUILDER_DISCLAIMER.to_string(),
            sections: vec![ReportSection {
                id: "section-1".to_string(),
                report_run_id: "run-1".to_string(),
                title: "Income Report".to_string(),
                section_order: 0,
                metadata_json: None,
                lines: vec![ReportLine {
                    id: "line-1".to_string(),
                    section_id: "section-1".to_string(),
                    label: "Dividend".to_string(),
                    amount: Some(dec!(12.3400)),
                    currency: Some("USD".to_string()),
                    value_text: None,
                    source_citation_id: Some("citation-1".to_string()),
                    metadata_json: None,
                }],
            }],
        };

        let export = build_report_export(&report);
        let html = String::from_utf8(export.bytes).expect("html");

        assert_eq!(export.mime_type, "text/html");
        assert!(html.contains(REPORT_BUILDER_DISCLAIMER));
        assert!(html.contains("12.34"));
        assert!(html.contains("citation-1"));
    }

    #[test]
    fn monthly_letter_request_rejects_invalid_period_month() {
        let request = GenerateReportRequest {
            report_type: ReportType::MonthlyWealthLetter,
            base_currency: "USD".to_string(),
            period_month: Some("2026-13".to_string()),
            included_sections: None,
        };

        assert!(request.validate().is_err());
    }
}
