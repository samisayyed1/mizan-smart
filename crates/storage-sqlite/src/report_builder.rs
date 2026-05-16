use async_trait::async_trait;
use chrono::{Datelike, NaiveDate, Utc};
use diesel::prelude::*;
use diesel::r2d2::{self, Pool};
use diesel::sql_query;
use diesel::sql_types::{Integer, Nullable, Text};
use diesel::SqliteConnection;
use rust_decimal::Decimal;
use serde_json::json;
use std::collections::BTreeMap;
use std::str::FromStr;
use std::sync::Arc;
use uuid::Uuid;

use mizan_core::report_builder::{
    build_empty_report, build_report_export, ConcentrationDimension, ConcentrationExposure,
    ConcentrationFinding, ConcentrationFragilitySummary, EstateBinderSection, FeeCategory,
    FeeCategoryTotal, FeeCurrencyTotal, FeeIntelligenceSummary, FeeSpikeAlert,
    GenerateReportRequest, ManualFeeEntry, ManualFeeEntryInput, ReportBuilderRepositoryTrait,
    ReportExportBundle, ReportLine, ReportRun, ReportRunStatus, ReportSection, ReportType,
    ESTATE_BINDER_DISCLAIMER, REPORT_BUILDER_DISCLAIMER,
};
use mizan_core::{Error, Result};

use crate::db::{get_connection, WriteHandle};
use crate::errors::StorageError;
use crate::schema::{
    accounts, activities, app_settings, asset_insurance_details, asset_liability_details,
    asset_private_investment_details, asset_real_estate_details, assets, capital_calls, documents,
    extracted_facts, fixed_income_cashflows, manual_fee_entries, report_lines, report_runs,
    report_sections, tax_pack_lines, tax_pack_missing_items, tax_packs, zakat_snapshots,
};

pub struct ReportBuilderRepository {
    pool: Arc<Pool<r2d2::ConnectionManager<SqliteConnection>>>,
    writer: WriteHandle,
}

impl ReportBuilderRepository {
    pub fn new(
        pool: Arc<Pool<r2d2::ConnectionManager<SqliteConnection>>>,
        writer: WriteHandle,
    ) -> Self {
        Self { pool, writer }
    }
}

#[derive(Debug, Clone, Queryable, Insertable, Selectable)]
#[diesel(table_name = report_runs)]
struct ReportRunRow {
    id: String,
    report_type: String,
    base_currency: String,
    status: String,
    created_at: String,
    completed_at: Option<String>,
}

#[derive(Debug, Clone, Queryable, Insertable, Selectable)]
#[diesel(table_name = report_sections)]
struct ReportSectionRow {
    id: String,
    report_run_id: String,
    title: String,
    section_order: i32,
    metadata_json: Option<String>,
}

#[derive(Debug, Clone, Queryable, Insertable, Selectable)]
#[diesel(table_name = report_lines)]
struct ReportLineRow {
    id: String,
    section_id: String,
    label: String,
    amount: Option<String>,
    currency: Option<String>,
    value_text: Option<String>,
    source_citation_id: Option<String>,
    metadata_json: Option<String>,
}

#[derive(Debug, Clone, Queryable)]
struct LatestTaxPackRow {
    id: String,
    tax_year: i32,
    jurisdiction: String,
}

#[derive(Debug, Clone, Queryable)]
struct TaxPackReportLineRow {
    id: String,
    category: String,
    amount: String,
    currency: String,
    taxable_date: String,
    source_citation_id: Option<String>,
}

#[derive(Debug, Clone, Queryable)]
struct MonthlyActivityRow {
    id: String,
    activity_type: String,
    amount: Option<String>,
    fee: Option<String>,
    currency: String,
}

#[derive(Debug, Clone, Queryable)]
struct UpcomingCapitalCallRow {
    id: String,
    due_date: String,
    amount: String,
    currency: String,
    source_citation_id: Option<String>,
}

#[derive(Debug, Clone, Queryable)]
struct UpcomingFixedIncomeCashflowRow {
    id: String,
    expected_date: String,
    cashflow_type: String,
    expected_amount: String,
    currency: String,
    source_citation_id: Option<String>,
}

#[derive(Debug, Clone, Queryable)]
struct LatestTaxReadinessRow {
    id: String,
    tax_year: i32,
    status: String,
}

#[derive(Debug, Clone, Queryable)]
struct LatestZakatSnapshotRow {
    id: String,
    snapshot_date: String,
    zakat_due: String,
    base_currency: String,
}

#[derive(Debug, Clone, Queryable)]
struct EstateAccountRow {
    id: String,
    name: String,
    account_type: String,
    currency: String,
}

#[derive(Debug, Clone, Queryable)]
struct EstateAssetRow {
    id: String,
    kind: String,
    name: Option<String>,
    display_code: Option<String>,
    classification: Option<String>,
}

#[derive(Debug, Clone, Queryable)]
struct EstatePropertyRow {
    asset_id: String,
    property_type: Option<String>,
    address_approximate: Option<String>,
    source_citation_id: Option<String>,
}

#[derive(Debug, Clone, Queryable)]
struct EstateLiabilityRow {
    asset_id: String,
    liability_type: String,
    lender: Option<String>,
    source_citation_id: Option<String>,
}

#[derive(Debug, Clone, Queryable)]
struct EstateInsuranceRow {
    asset_id: String,
    policy_type: String,
    provider: Option<String>,
    source_citation_id: Option<String>,
}

#[derive(Debug, Clone, Queryable)]
struct EstatePrivateInvestmentRow {
    asset_id: String,
    instrument_subtype: String,
    manager: Option<String>,
    source_citation_id: Option<String>,
}

#[derive(Debug, Clone, Queryable)]
struct EstateDocumentRow {
    id: String,
    original_name: String,
    mime_type: String,
    status: String,
}

#[derive(Debug, Clone, Queryable, Insertable, Selectable)]
#[diesel(table_name = manual_fee_entries)]
struct ManualFeeEntryRow {
    id: String,
    fee_date: String,
    category: String,
    amount: String,
    currency: String,
    account_id: Option<String>,
    asset_id: Option<String>,
    source_citation_id: Option<String>,
    notes: Option<String>,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, Clone)]
struct FeeSourceRow {
    id: String,
    fee_date: String,
    category: FeeCategory,
    amount: Decimal,
    currency: String,
    account_id: Option<String>,
    asset_id: Option<String>,
    source_citation_id: Option<String>,
    source_table: &'static str,
}

#[derive(Debug, Clone, Queryable)]
struct ActivityFeeRow {
    id: String,
    account_id: String,
    asset_id: Option<String>,
    activity_type: String,
    subtype: Option<String>,
    activity_date: String,
    amount: Option<String>,
    fee: Option<String>,
    currency: String,
}

#[derive(Debug, Clone, diesel::QueryableByName)]
struct ConcentrationAssetRow {
    #[diesel(sql_type = Text)]
    asset_id: String,
    #[diesel(sql_type = Nullable<Text>)]
    asset_name: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    display_code: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    classification: Option<String>,
    #[diesel(sql_type = Text)]
    quote_mode: String,
    #[diesel(sql_type = Text)]
    valuation_date: String,
    #[diesel(sql_type = Text)]
    value_native: String,
    #[diesel(sql_type = Text)]
    currency: String,
    #[diesel(sql_type = Text)]
    source_type: String,
    #[diesel(sql_type = Nullable<Text>)]
    source_citation_id: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    private_asset_id: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    shariah_status: Option<String>,
}

#[derive(Debug, Clone, diesel::QueryableByName)]
struct ConcentrationAccountRow {
    #[diesel(sql_type = Text)]
    account_id: String,
    #[diesel(sql_type = Text)]
    account_name: String,
    #[diesel(sql_type = Text)]
    total_value: String,
    #[diesel(sql_type = Text)]
    base_currency: String,
}

#[derive(Debug, Clone, diesel::QueryableByName)]
struct ConcentrationTaxonomyRow {
    #[diesel(sql_type = Text)]
    asset_id: String,
    #[diesel(sql_type = Text)]
    taxonomy_name: String,
    #[diesel(sql_type = Text)]
    category_name: String,
    #[diesel(sql_type = Integer)]
    weight: i32,
}

#[derive(Debug, Clone, diesel::QueryableByName)]
struct ConcentrationIncomeRow {
    #[diesel(sql_type = Text)]
    source_label: String,
    #[diesel(sql_type = Text)]
    amount: String,
    #[diesel(sql_type = Text)]
    currency: String,
}

#[derive(Debug, Clone, Copy)]
struct ConcentrationThresholds {
    single_exposure_percent: Decimal,
    combined_income_percent: Decimal,
    fragility_percent: Decimal,
}

#[async_trait]
impl ReportBuilderRepositoryTrait for ReportBuilderRepository {
    async fn generate_report(&self, request: GenerateReportRequest) -> Result<ReportRun> {
        request.validate()?;
        let created_at = Utc::now().to_rfc3339();
        let report = match request.report_type {
            ReportType::TaxPack => self.build_tax_pack_report(request, created_at)?,
            ReportType::PortfolioSummary => {
                self.build_portfolio_summary_report(request, created_at)?
            }
            ReportType::MonthlyWealthLetter => {
                self.build_monthly_wealth_letter(request, created_at)?
            }
            ReportType::EstateBinder => self.build_estate_binder(request, created_at)?,
            ReportType::FeeReport => self.build_fee_report(request, created_at)?,
            _ => build_empty_report(
                Uuid::new_v4().to_string(),
                Uuid::new_v4().to_string(),
                Uuid::new_v4().to_string(),
                request,
                created_at,
            )?,
        };
        self.persist_report(report.clone()).await?;
        Ok(report)
    }

    fn get_report_run(&self, report_run_id: &str) -> Result<Option<ReportRun>> {
        let mut conn = get_connection(&self.pool)?;
        let run_row = report_runs::table
            .find(report_run_id)
            .select(ReportRunRow::as_select())
            .first::<ReportRunRow>(&mut conn)
            .optional()
            .map_err(StorageError::from)?;
        let Some(run_row) = run_row else {
            return Ok(None);
        };

        let section_rows = report_sections::table
            .filter(report_sections::report_run_id.eq(report_run_id))
            .order((
                report_sections::section_order.asc(),
                report_sections::id.asc(),
            ))
            .select(ReportSectionRow::as_select())
            .load::<ReportSectionRow>(&mut conn)
            .map_err(StorageError::from)?;

        let mut sections = Vec::with_capacity(section_rows.len());
        for section_row in section_rows {
            let line_rows = report_lines::table
                .filter(report_lines::section_id.eq(&section_row.id))
                .order(report_lines::id.asc())
                .select(ReportLineRow::as_select())
                .load::<ReportLineRow>(&mut conn)
                .map_err(StorageError::from)?;
            sections.push(section_row_to_domain(section_row, line_rows)?);
        }

        Ok(Some(run_row_to_domain(run_row, sections)?))
    }

    fn export_report(&self, report_run_id: &str) -> Result<ReportExportBundle> {
        let report = self.get_report_run(report_run_id)?.ok_or_else(|| {
            Error::Database(mizan_core::errors::DatabaseError::NotFound(format!(
                "Report run {report_run_id} not found"
            )))
        })?;
        Ok(build_report_export(&report))
    }

    async fn add_manual_fee_entry(&self, input: ManualFeeEntryInput) -> Result<ManualFeeEntry> {
        input.validate()?;
        let now = Utc::now().to_rfc3339();
        let row = ManualFeeEntryRow {
            id: Uuid::new_v4().to_string(),
            fee_date: input.fee_date,
            category: input.category.as_str().to_string(),
            amount: input.amount.normalize().to_string(),
            currency: input.currency.to_ascii_uppercase(),
            account_id: input.account_id,
            asset_id: input.asset_id,
            source_citation_id: input.source_citation_id,
            notes: input.notes,
            created_at: now.clone(),
            updated_at: now,
        };
        self.writer
            .exec_tx(move |tx| {
                let conn = tx.conn();
                diesel::insert_into(manual_fee_entries::table)
                    .values(&row)
                    .execute(conn)
                    .map_err(StorageError::from)?;
                manual_fee_entry_row_to_domain(row)
            })
            .await
    }

    fn get_fee_intelligence_summary(
        &self,
        period_month: Option<String>,
    ) -> Result<FeeIntelligenceSummary> {
        let created_at = Utc::now().to_rfc3339();
        let period_month = period_month.unwrap_or_else(|| created_at.chars().take(7).collect());
        let mut conn = get_connection(&self.pool)?;
        build_fee_summary(&mut conn, &period_month)
    }

    fn get_concentration_fragility_summary(
        &self,
        base_currency: String,
    ) -> Result<ConcentrationFragilitySummary> {
        validate_currency_code(&base_currency)?;
        let mut conn = get_connection(&self.pool)?;
        build_concentration_summary(&mut conn, base_currency.to_ascii_uppercase())
    }
}

impl ReportBuilderRepository {
    fn build_portfolio_summary_report(
        &self,
        request: GenerateReportRequest,
        created_at: String,
    ) -> Result<ReportRun> {
        let mut conn = get_connection(&self.pool)?;
        let summary = build_concentration_summary(&mut conn, request.base_currency.clone())?;
        if summary.empty_state {
            return build_empty_report(
                Uuid::new_v4().to_string(),
                Uuid::new_v4().to_string(),
                Uuid::new_v4().to_string(),
                request,
                created_at,
            );
        }

        let run_id = Uuid::new_v4().to_string();
        let section_id = Uuid::new_v4().to_string();
        let lines = if summary.findings.is_empty() {
            vec![text_line(
                &section_id,
                "No concentration thresholds crossed".to_string(),
                "Current deterministic data did not cross the configured concentration thresholds."
                    .to_string(),
                json!({"sourceTable":"valuations","citationStatus":"missing"}),
            )]
        } else {
            summary
                .findings
                .iter()
                .map(|finding| ReportLine {
                    id: Uuid::new_v4().to_string(),
                    section_id: section_id.clone(),
                    label: finding.label.clone(),
                    amount: Some(finding.amount),
                    currency: Some(finding.currency.clone()),
                    value_text: Some(finding.message.clone()),
                    source_citation_id: None,
                    metadata_json: Some(
                        json!({
                            "dimension": finding.dimension,
                            "percent": finding.percent,
                            "thresholdPercent": finding.threshold_percent,
                            "citationStatus":"missing"
                        })
                        .to_string(),
                    ),
                })
                .collect()
        };

        Ok(ReportRun {
            id: run_id.clone(),
            report_type: request.report_type,
            base_currency: request.base_currency,
            status: ReportRunStatus::Generated,
            created_at: created_at.clone(),
            completed_at: Some(created_at),
            disclaimer: REPORT_BUILDER_DISCLAIMER.to_string(),
            sections: vec![section(
                &run_id,
                section_id,
                "Concentration and fragility",
                1,
                json!({
                    "asOfDate": summary.as_of_date,
                    "taxonomyState": summary.taxonomy_state,
                    "islamicModeEnabled": summary.islamic_mode_enabled
                }),
                lines,
            )],
        })
    }

    fn build_tax_pack_report(
        &self,
        request: GenerateReportRequest,
        created_at: String,
    ) -> Result<ReportRun> {
        let mut conn = get_connection(&self.pool)?;
        let latest = tax_packs::table
            .select((tax_packs::id, tax_packs::tax_year, tax_packs::jurisdiction))
            .order((tax_packs::created_at.desc(), tax_packs::id.desc()))
            .first::<LatestTaxPackRow>(&mut conn)
            .optional()
            .map_err(StorageError::from)?;

        let Some(latest) = latest else {
            return build_empty_report(
                Uuid::new_v4().to_string(),
                Uuid::new_v4().to_string(),
                Uuid::new_v4().to_string(),
                request,
                created_at,
            );
        };

        let line_rows = tax_pack_lines::table
            .filter(tax_pack_lines::tax_pack_id.eq(&latest.id))
            .select((
                tax_pack_lines::id,
                tax_pack_lines::category,
                tax_pack_lines::amount,
                tax_pack_lines::currency,
                tax_pack_lines::taxable_date,
                tax_pack_lines::source_citation_id,
            ))
            .order((tax_pack_lines::taxable_date.asc(), tax_pack_lines::id.asc()))
            .load::<TaxPackReportLineRow>(&mut conn)
            .map_err(StorageError::from)?;

        let run_id = Uuid::new_v4().to_string();
        let section_id = Uuid::new_v4().to_string();
        let lines = line_rows
            .into_iter()
            .map(|line| {
                let amount = Decimal::from_str(&line.amount).map_err(|err| {
                    Error::Validation(mizan_core::errors::ValidationError::InvalidInput(format!(
                        "tax pack line amount {:?} is not a valid decimal: {err}",
                        line.amount
                    )))
                })?;
                let missing_citation = line.source_citation_id.is_none();
                Ok(ReportLine {
                    id: Uuid::new_v4().to_string(),
                    section_id: section_id.clone(),
                    label: format!(
                        "{} on {}",
                        line.category.replace('_', " "),
                        line.taxable_date
                    ),
                    amount: Some(amount),
                    currency: Some(line.currency),
                    value_text: missing_citation.then(|| "Missing source citation".to_string()),
                    source_citation_id: line.source_citation_id,
                    metadata_json: Some(
                        json!({
                            "taxPackId": latest.id,
                            "taxPackLineId": line.id,
                            "citationStatus": if missing_citation { "missing" } else { "included" }
                        })
                        .to_string(),
                    ),
                })
            })
            .collect::<Result<Vec<_>>>()?;

        if lines.is_empty() {
            return build_empty_report(
                run_id,
                section_id,
                Uuid::new_v4().to_string(),
                request,
                created_at,
            );
        }

        Ok(ReportRun {
            id: run_id.clone(),
            report_type: request.report_type,
            base_currency: request.base_currency,
            status: ReportRunStatus::Generated,
            created_at: created_at.clone(),
            completed_at: Some(created_at),
            disclaimer: REPORT_BUILDER_DISCLAIMER.to_string(),
            sections: vec![ReportSection {
                id: section_id,
                report_run_id: run_id,
                title: "Tax Pack Report".to_string(),
                section_order: 0,
                metadata_json: Some(
                    json!({
                        "taxPackId": latest.id,
                        "taxYear": latest.tax_year,
                        "jurisdiction": latest.jurisdiction
                    })
                    .to_string(),
                ),
                lines,
            }],
        })
    }

    fn build_monthly_wealth_letter(
        &self,
        request: GenerateReportRequest,
        created_at: String,
    ) -> Result<ReportRun> {
        let period_month = request
            .period_month
            .clone()
            .unwrap_or_else(|| created_at.chars().take(7).collect::<String>());
        let next_month = next_month_start(&period_month)?;
        let start_date = format!("{period_month}-01");
        let year = period_month[0..4].parse::<i32>().map_err(|err| {
            Error::Validation(mizan_core::errors::ValidationError::InvalidInput(format!(
                "period_month year is invalid: {err}"
            )))
        })?;

        let mut conn = get_connection(&self.pool)?;
        let run_id = Uuid::new_v4().to_string();
        let mut sections = Vec::new();
        let mut missing_lines = Vec::new();

        let activity_rows = activities::table
            .filter(activities::activity_date.ge(&start_date))
            .filter(activities::activity_date.lt(&next_month))
            .select((
                activities::id,
                activities::activity_type,
                activities::amount,
                activities::fee,
                activities::currency,
            ))
            .load::<MonthlyActivityRow>(&mut conn)
            .map_err(StorageError::from)?;
        let income_rows = activity_rows
            .iter()
            .filter(|row| is_income_activity(&row.activity_type))
            .collect::<Vec<_>>();

        let income_section_id = Uuid::new_v4().to_string();
        let mut income_by_currency = BTreeMap::<String, Decimal>::new();
        let mut largest_income: Option<(&MonthlyActivityRow, Decimal)> = None;
        for row in &income_rows {
            let Some(amount_text) = row.amount.as_deref() else {
                missing_lines.push(text_line(
                    &income_section_id,
                    format!("Income activity {} missing amount", row.id),
                    "Activity has no amount, so it was omitted from income totals.".to_string(),
                    json!({"sourceTable":"activities","sourceId":row.id,"citationStatus":"missing"}),
                ));
                continue;
            };
            let amount = parse_decimal("activity amount", amount_text)?;
            *income_by_currency.entry(row.currency.clone()).or_default() += amount;
            if largest_income
                .as_ref()
                .is_none_or(|(_, current)| amount.abs() > current.abs())
            {
                largest_income = Some((row, amount));
            }
        }
        if !income_by_currency.is_empty() {
            let mut lines = income_by_currency
                .into_iter()
                .map(|(currency, amount)| {
                    amount_line(
                        &income_section_id,
                        "Dividend and interest received".to_string(),
                        amount,
                        currency,
                        None,
                        json!({"sourceTable":"activities","periodMonth":period_month,"citationStatus":"missing"}),
                    )
                })
                .collect::<Vec<_>>();
            sections.push(section(
                &run_id,
                income_section_id,
                "Income received",
                sections.len() as i32 + 1,
                json!({"periodMonth":period_month}),
                lines.split_off(0),
            ));
        }

        if let Some((row, amount)) = largest_income {
            let section_id = Uuid::new_v4().to_string();
            sections.push(section(
                &run_id,
                section_id.clone(),
                "Largest contributors",
                sections.len() as i32 + 1,
                json!({"periodMonth":period_month}),
                vec![amount_line(
                    &section_id,
                    format!("Largest income contributor: {}", row.activity_type),
                    amount,
                    row.currency.clone(),
                    None,
                    json!({"sourceTable":"activities","sourceId":row.id,"citationStatus":"missing"}),
                )],
            ));
        }

        let fees_section_id = Uuid::new_v4().to_string();
        let mut fees_by_currency = BTreeMap::<String, Decimal>::new();
        for row in &activity_rows {
            if let Some(fee_text) = row.fee.as_deref() {
                let fee = parse_decimal("activity fee", fee_text)?;
                if fee != Decimal::ZERO {
                    *fees_by_currency.entry(row.currency.clone()).or_default() += fee;
                }
            }
        }
        if !fees_by_currency.is_empty() {
            let lines = fees_by_currency
                .into_iter()
                .map(|(currency, fee)| {
                    amount_line(
                        &fees_section_id,
                        "Fees recorded".to_string(),
                        fee,
                        currency,
                        None,
                        json!({"sourceTable":"activities","periodMonth":period_month,"citationStatus":"missing"}),
                    )
                })
                .collect::<Vec<_>>();
            sections.push(section(
                &run_id,
                fees_section_id,
                "Fees",
                sections.len() as i32 + 1,
                json!({"periodMonth":period_month}),
                lines,
            ));
        }

        let pending_fact_count = extracted_facts::table
            .filter(extracted_facts::status.eq("pending"))
            .count()
            .get_result::<i64>(&mut conn)
            .map_err(StorageError::from)?;
        if pending_fact_count > 0 {
            let section_id = Uuid::new_v4().to_string();
            sections.push(section(
                &run_id,
                section_id.clone(),
                "Pending document reviews",
                sections.len() as i32 + 1,
                json!({"sourceTable":"extracted_facts"}),
                vec![text_line(
                    &section_id,
                    "Facts awaiting review".to_string(),
                    pending_fact_count.to_string(),
                    json!({"sourceTable":"extracted_facts","status":"pending","citationStatus":"missing"}),
                )],
            ));
        }

        let capital_call_rows = capital_calls::table
            .filter(capital_calls::status.eq("due"))
            .filter(capital_calls::due_date.ge(&start_date))
            .filter(capital_calls::due_date.lt(&next_month))
            .select((
                capital_calls::id,
                capital_calls::due_date,
                capital_calls::amount,
                capital_calls::currency,
                capital_calls::source_citation_id,
            ))
            .order((capital_calls::due_date.asc(), capital_calls::id.asc()))
            .load::<UpcomingCapitalCallRow>(&mut conn)
            .map_err(StorageError::from)?;
        if !capital_call_rows.is_empty() {
            let section_id = Uuid::new_v4().to_string();
            let lines = capital_call_rows
                .into_iter()
                .map(|row| {
                    let amount = parse_decimal("capital call amount", &row.amount)?;
                    Ok(amount_line(
                        &section_id,
                        format!("Capital call due {}", row.due_date),
                        amount,
                        row.currency,
                        row.source_citation_id,
                        json!({"sourceTable":"capital_calls","sourceId":row.id}),
                    ))
                })
                .collect::<Result<Vec<_>>>()?;
            sections.push(section(
                &run_id,
                section_id,
                "Upcoming capital calls",
                sections.len() as i32 + 1,
                json!({"periodMonth":period_month}),
                lines,
            ));
        }

        let fixed_income_rows = fixed_income_cashflows::table
            .filter(fixed_income_cashflows::status.eq("expected"))
            .filter(fixed_income_cashflows::expected_date.ge(&start_date))
            .filter(fixed_income_cashflows::expected_date.lt(&next_month))
            .select((
                fixed_income_cashflows::id,
                fixed_income_cashflows::expected_date,
                fixed_income_cashflows::cashflow_type,
                fixed_income_cashflows::expected_amount,
                fixed_income_cashflows::currency,
                fixed_income_cashflows::source_citation_id,
            ))
            .order((
                fixed_income_cashflows::expected_date.asc(),
                fixed_income_cashflows::id.asc(),
            ))
            .load::<UpcomingFixedIncomeCashflowRow>(&mut conn)
            .map_err(StorageError::from)?;
        if !fixed_income_rows.is_empty() {
            let section_id = Uuid::new_v4().to_string();
            let lines = fixed_income_rows
                .into_iter()
                .map(|row| {
                    let amount =
                        parse_decimal("fixed income cashflow amount", &row.expected_amount)?;
                    Ok(amount_line(
                        &section_id,
                        format!("{} expected {}", row.cashflow_type, row.expected_date),
                        amount,
                        row.currency,
                        row.source_citation_id,
                        json!({"sourceTable":"fixed_income_cashflows","sourceId":row.id}),
                    ))
                })
                .collect::<Result<Vec<_>>>()?;
            sections.push(section(
                &run_id,
                section_id,
                "Upcoming coupons and maturities",
                sections.len() as i32 + 1,
                json!({"periodMonth":period_month}),
                lines,
            ));
        }

        if let Some(tax_pack) = tax_packs::table
            .filter(tax_packs::tax_year.eq(year))
            .select((tax_packs::id, tax_packs::tax_year, tax_packs::status))
            .order((tax_packs::created_at.desc(), tax_packs::id.desc()))
            .first::<LatestTaxReadinessRow>(&mut conn)
            .optional()
            .map_err(StorageError::from)?
        {
            let line_count = tax_pack_lines::table
                .filter(tax_pack_lines::tax_pack_id.eq(&tax_pack.id))
                .count()
                .get_result::<i64>(&mut conn)
                .map_err(StorageError::from)?;
            let missing_count = tax_pack_missing_items::table
                .filter(tax_pack_missing_items::tax_pack_id.eq(&tax_pack.id))
                .count()
                .get_result::<i64>(&mut conn)
                .map_err(StorageError::from)?;
            let section_id = Uuid::new_v4().to_string();
            sections.push(section(
                &run_id,
                section_id.clone(),
                "Tax readiness",
                sections.len() as i32 + 1,
                json!({"taxPackId":tax_pack.id,"taxYear":tax_pack.tax_year}),
                vec![text_line(
                    &section_id,
                    "Latest tax pack status".to_string(),
                    format!(
                        "{} with {} lines and {} missing source items",
                        tax_pack.status, line_count, missing_count
                    ),
                    json!({"sourceTable":"tax_packs","sourceId":tax_pack.id,"citationStatus":"missing"}),
                )],
            ));
            if missing_count > 0 {
                missing_lines.push(text_line(
                    &section_id,
                    "Tax pack missing source items".to_string(),
                    missing_count.to_string(),
                    json!({"sourceTable":"tax_pack_missing_items","taxPackId":tax_pack.id,"citationStatus":"missing"}),
                ));
            }
        }

        if shariah_mode_enabled(&mut conn)? {
            if let Some(snapshot) = zakat_snapshots::table
                .filter(zakat_snapshots::snapshot_date.lt(&next_month))
                .select((
                    zakat_snapshots::id,
                    zakat_snapshots::snapshot_date,
                    zakat_snapshots::zakat_due,
                    zakat_snapshots::base_currency,
                ))
                .order((
                    zakat_snapshots::snapshot_date.desc(),
                    zakat_snapshots::id.desc(),
                ))
                .first::<LatestZakatSnapshotRow>(&mut conn)
                .optional()
                .map_err(StorageError::from)?
            {
                let section_id = Uuid::new_v4().to_string();
                let zakat_due = parse_decimal("zakat due", &snapshot.zakat_due)?;
                sections.push(section(
                    &run_id,
                    section_id.clone(),
                    "Zakat readiness",
                    sections.len() as i32 + 1,
                    json!({"zakatSnapshotId":snapshot.id,"snapshotDate":snapshot.snapshot_date}),
                    vec![amount_line(
                        &section_id,
                        format!("Zakat snapshot {}", snapshot.snapshot_date),
                        zakat_due,
                        snapshot.base_currency,
                        None,
                        json!({"sourceTable":"zakat_snapshots","sourceId":snapshot.id,"citationStatus":"missing"}),
                    )],
                ));
            }
        }

        if !missing_lines.is_empty() {
            let section_id = Uuid::new_v4().to_string();
            for line in &mut missing_lines {
                line.section_id = section_id.clone();
            }
            sections.push(section(
                &run_id,
                section_id,
                "Stale or missing data",
                sections.len() as i32 + 1,
                json!({"periodMonth":period_month}),
                missing_lines,
            ));
        }

        if sections.is_empty() {
            return build_empty_report(
                run_id,
                Uuid::new_v4().to_string(),
                Uuid::new_v4().to_string(),
                request,
                created_at,
            );
        }

        let opening_section_id = Uuid::new_v4().to_string();
        sections.insert(
            0,
            section(
                &run_id,
                opening_section_id.clone(),
                "Opening summary",
                0,
                json!({"periodMonth":period_month}),
                vec![text_line(
                    &opening_section_id,
                    "Monthly summary period".to_string(),
                    format!(
                        "Deterministic monthly wealth letter for {period_month}, generated only from supported local source rows."
                    ),
                    json!({"periodMonth":period_month,"citationStatus":"missing"}),
                )],
            ),
        );
        for (index, section) in sections.iter_mut().enumerate() {
            section.section_order = index as i32;
        }

        Ok(ReportRun {
            id: run_id,
            report_type: request.report_type,
            base_currency: request.base_currency,
            status: ReportRunStatus::Generated,
            created_at: created_at.clone(),
            completed_at: Some(created_at),
            disclaimer: REPORT_BUILDER_DISCLAIMER.to_string(),
            sections,
        })
    }

    fn build_estate_binder(
        &self,
        request: GenerateReportRequest,
        created_at: String,
    ) -> Result<ReportRun> {
        let selected = request
            .included_sections
            .clone()
            .filter(|sections| !sections.is_empty())
            .unwrap_or_else(|| {
                vec![
                    EstateBinderSection::Accounts,
                    EstateBinderSection::Assets,
                    EstateBinderSection::Liabilities,
                    EstateBinderSection::Property,
                    EstateBinderSection::Insurance,
                    EstateBinderSection::Pensions,
                    EstateBinderSection::PrivateInvestments,
                    EstateBinderSection::DocumentsManifest,
                    EstateBinderSection::EntityOwnership,
                    EstateBinderSection::IslamicNotes,
                ]
            });
        let mut conn = get_connection(&self.pool)?;
        let run_id = Uuid::new_v4().to_string();
        let mut sections = Vec::new();

        for selected_section in selected {
            let lines = match selected_section {
                EstateBinderSection::Accounts => estate_account_lines(&mut conn)?,
                EstateBinderSection::Assets => estate_asset_lines(&mut conn, false)?,
                EstateBinderSection::Liabilities => estate_liability_lines(&mut conn)?,
                EstateBinderSection::Property => estate_property_lines(&mut conn)?,
                EstateBinderSection::Insurance => estate_insurance_lines(&mut conn)?,
                EstateBinderSection::Pensions => estate_pension_lines(&mut conn)?,
                EstateBinderSection::PrivateInvestments => {
                    estate_private_investment_lines(&mut conn)?
                }
                EstateBinderSection::DocumentsManifest => estate_document_lines(&mut conn)?,
                EstateBinderSection::EntityOwnership => Vec::new(),
                EstateBinderSection::IslamicNotes => {
                    if shariah_mode_enabled(&mut conn)? {
                        estate_islamic_lines(&mut conn)?
                    } else {
                        Vec::new()
                    }
                }
            };
            if lines.is_empty() {
                continue;
            }
            let section_id = Uuid::new_v4().to_string();
            let mut section_lines = lines;
            for line in &mut section_lines {
                line.section_id = section_id.clone();
            }
            sections.push(section(
                &run_id,
                section_id,
                selected_section.title(),
                sections.len() as i32,
                json!({"estateBinderSection":selected_section.title()}),
                section_lines,
            ));
        }

        if sections.is_empty() {
            return build_empty_report(
                run_id,
                Uuid::new_v4().to_string(),
                Uuid::new_v4().to_string(),
                request,
                created_at,
            );
        }

        Ok(ReportRun {
            id: run_id,
            report_type: request.report_type,
            base_currency: request.base_currency,
            status: ReportRunStatus::Generated,
            created_at: created_at.clone(),
            completed_at: Some(created_at),
            disclaimer: ESTATE_BINDER_DISCLAIMER.to_string(),
            sections,
        })
    }

    fn build_fee_report(
        &self,
        request: GenerateReportRequest,
        created_at: String,
    ) -> Result<ReportRun> {
        let period_month = request
            .period_month
            .clone()
            .unwrap_or_else(|| created_at.chars().take(7).collect::<String>());
        let mut conn = get_connection(&self.pool)?;
        let summary = build_fee_summary(&mut conn, &period_month)?;
        let source_rows = fee_source_rows(&mut conn)?
            .into_iter()
            .filter(|row| row.fee_date.starts_with(&period_month))
            .collect::<Vec<_>>();
        let run_id = Uuid::new_v4().to_string();

        if source_rows.is_empty() {
            return build_empty_report(
                run_id,
                Uuid::new_v4().to_string(),
                Uuid::new_v4().to_string(),
                request,
                created_at,
            );
        }

        let totals_section_id = Uuid::new_v4().to_string();
        let totals_lines = summary
            .totals
            .iter()
            .map(|total| {
                amount_line(
                    &totals_section_id,
                    format!("Total fees for {}", summary.period_month),
                    total.amount,
                    total.currency.clone(),
                    None,
                    json!({"periodMonth":summary.period_month,"sourceTables":["activities","manual_fee_entries"],"citationStatus":"mixed"}),
                )
            })
            .collect::<Vec<_>>();

        let category_section_id = Uuid::new_v4().to_string();
        let category_lines = summary
            .category_totals
            .iter()
            .map(|total| {
                amount_line(
                    &category_section_id,
                    total.category.title().to_string(),
                    total.amount,
                    total.currency.clone(),
                    None,
                    json!({"periodMonth":summary.period_month,"feeCategory":total.category.as_str(),"citationStatus":"mixed"}),
                )
            })
            .collect::<Vec<_>>();

        let account_section_id = Uuid::new_v4().to_string();
        let account_lines = aggregate_fee_lines(
            &account_section_id,
            source_rows.iter().filter_map(|row| {
                row.account_id
                    .as_ref()
                    .map(|account_id| (account_id.as_str(), row.currency.as_str(), row.amount))
            }),
            "Account",
        );

        let asset_section_id = Uuid::new_v4().to_string();
        let asset_lines = aggregate_fee_lines(
            &asset_section_id,
            source_rows.iter().filter_map(|row| {
                row.asset_id
                    .as_ref()
                    .map(|asset_id| (asset_id.as_str(), row.currency.as_str(), row.amount))
            }),
            "Asset",
        );

        let source_section_id = Uuid::new_v4().to_string();
        let source_lines = source_rows
            .into_iter()
            .map(|row| {
                amount_line(
                    &source_section_id,
                    format!("{} on {}", row.category.title(), row.fee_date),
                    row.amount,
                    row.currency,
                    row.source_citation_id,
                    json!({"sourceTable":row.source_table,"sourceId":row.id,"feeCategory":row.category.as_str()}),
                )
            })
            .collect::<Vec<_>>();

        let mut sections = vec![
            section(
                &run_id,
                totals_section_id,
                "Period totals",
                0,
                json!({"periodMonth":summary.period_month}),
                totals_lines,
            ),
            section(
                &run_id,
                category_section_id,
                "Fee categories",
                1,
                json!({"periodMonth":summary.period_month}),
                category_lines,
            ),
        ];
        if !account_lines.is_empty() {
            sections.push(section(
                &run_id,
                account_section_id,
                "Account totals",
                sections.len() as i32,
                json!({"periodMonth":summary.period_month}),
                account_lines,
            ));
        }
        if !asset_lines.is_empty() {
            sections.push(section(
                &run_id,
                asset_section_id,
                "Asset totals",
                sections.len() as i32,
                json!({"periodMonth":summary.period_month}),
                asset_lines,
            ));
        }
        if let Some(spike) = summary.spike {
            let spike_section_id = Uuid::new_v4().to_string();
            sections.push(section(
                &run_id,
                spike_section_id.clone(),
                "Fee spike warning",
                sections.len() as i32,
                json!({"periodMonth":summary.period_month,"rule":"current_period_gt_2x_prior_3_month_average"}),
                vec![text_line(
                    &spike_section_id,
                    format!("{} fee spike", spike.currency),
                    format!(
                        "{} versus prior average {} ({}x)",
                        spike.current_period_total, spike.prior_average, spike.multiple
                    ),
                    json!({"currency":spike.currency,"citationStatus":"missing"}),
                )],
            ));
        }
        sections.push(section(
            &run_id,
            source_section_id,
            "Source rows",
            sections.len() as i32,
            json!({"periodMonth":summary.period_month}),
            source_lines,
        ));

        Ok(ReportRun {
            id: run_id,
            report_type: request.report_type,
            base_currency: request.base_currency,
            status: ReportRunStatus::Generated,
            created_at: created_at.clone(),
            completed_at: Some(created_at),
            disclaimer: REPORT_BUILDER_DISCLAIMER.to_string(),
            sections,
        })
    }

    async fn persist_report(&self, report: ReportRun) -> Result<()> {
        let run_row = ReportRunRow::from(&report);
        let section_rows = report
            .sections
            .iter()
            .map(ReportSectionRow::from)
            .collect::<Vec<_>>();
        let line_rows = report
            .sections
            .iter()
            .flat_map(|section| section.lines.iter().map(ReportLineRow::from))
            .collect::<Vec<_>>();

        self.writer
            .exec_tx(move |tx| -> Result<()> {
                let conn = tx.conn();
                diesel::insert_into(report_runs::table)
                    .values(&run_row)
                    .execute(conn)
                    .map_err(StorageError::from)?;
                diesel::insert_into(report_sections::table)
                    .values(&section_rows)
                    .execute(conn)
                    .map_err(StorageError::from)?;
                diesel::insert_into(report_lines::table)
                    .values(&line_rows)
                    .execute(conn)
                    .map_err(StorageError::from)?;
                Ok(())
            })
            .await
    }
}

fn section(
    run_id: &str,
    section_id: String,
    title: &str,
    section_order: i32,
    metadata: serde_json::Value,
    lines: Vec<ReportLine>,
) -> ReportSection {
    ReportSection {
        id: section_id,
        report_run_id: run_id.to_string(),
        title: title.to_string(),
        section_order,
        metadata_json: Some(metadata.to_string()),
        lines,
    }
}

fn amount_line(
    section_id: &str,
    label: String,
    amount: Decimal,
    currency: String,
    source_citation_id: Option<String>,
    metadata: serde_json::Value,
) -> ReportLine {
    ReportLine {
        id: Uuid::new_v4().to_string(),
        section_id: section_id.to_string(),
        label,
        amount: Some(amount),
        currency: Some(currency),
        value_text: source_citation_id
            .is_none()
            .then(|| "Missing source citation".to_string()),
        source_citation_id,
        metadata_json: Some(metadata.to_string()),
    }
}

fn text_line(
    section_id: &str,
    label: String,
    value_text: String,
    metadata: serde_json::Value,
) -> ReportLine {
    ReportLine {
        id: Uuid::new_v4().to_string(),
        section_id: section_id.to_string(),
        label,
        amount: None,
        currency: None,
        value_text: Some(value_text),
        source_citation_id: None,
        metadata_json: Some(metadata.to_string()),
    }
}

fn is_income_activity(activity_type: &str) -> bool {
    matches!(
        activity_type.to_ascii_lowercase().as_str(),
        "dividend" | "interest"
    )
}

fn parse_decimal(label: &str, value: &str) -> Result<Decimal> {
    Decimal::from_str(value).map_err(|err| {
        Error::Validation(mizan_core::errors::ValidationError::InvalidInput(format!(
            "{label} {value:?} is not a valid decimal: {err}"
        )))
    })
}

fn next_month_start(period_month: &str) -> Result<String> {
    let date =
        NaiveDate::parse_from_str(&format!("{period_month}-01"), "%Y-%m-%d").map_err(|err| {
            Error::Validation(mizan_core::errors::ValidationError::InvalidInput(format!(
                "period_month {period_month:?} is not valid: {err}"
            )))
        })?;
    let (year, month) = if date.month() == 12 {
        (date.year() + 1, 1)
    } else {
        (date.year(), date.month() + 1)
    };
    Ok(format!("{year:04}-{month:02}-01"))
}

fn shariah_mode_enabled(conn: &mut SqliteConnection) -> Result<bool> {
    let value = app_settings::table
        .find("shariah_mode_enabled")
        .select(app_settings::setting_value)
        .first::<String>(conn)
        .optional()
        .map_err(StorageError::from)?;
    Ok(value.as_deref() == Some("true"))
}

fn build_concentration_summary(
    conn: &mut SqliteConnection,
    base_currency: String,
) -> Result<ConcentrationFragilitySummary> {
    let as_of_date = Utc::now().date_naive();
    let thresholds = concentration_thresholds(conn)?;
    let asset_rows = concentration_asset_rows(conn)?;
    let account_rows = concentration_account_rows(conn)?;
    let taxonomy_rows = concentration_taxonomy_rows(conn)?;
    let income_rows = concentration_income_rows(conn)?;
    let islamic_enabled = shariah_mode_enabled(conn)?;

    let mut exposures = Vec::new();
    let mut findings = Vec::new();
    let mut wealth_total = Decimal::ZERO;
    let mut asset_groups = BTreeMap::<String, (Decimal, i64)>::new();
    let mut currency_groups = BTreeMap::<String, (Decimal, i64)>::new();
    let mut class_groups = BTreeMap::<String, (Decimal, i64)>::new();
    let mut stale_manual = (Decimal::ZERO, 0_i64);
    let mut private_illiquid = (Decimal::ZERO, 0_i64);
    let mut shariah_unknown = (Decimal::ZERO, 0_i64);
    let mut uncited = (Decimal::ZERO, 0_i64);
    let has_any_citation = asset_rows
        .iter()
        .any(|row| row.source_citation_id.is_some());

    for row in &asset_rows {
        let value = parse_decimal("valuation amount", &row.value_native)?.abs();
        if value == Decimal::ZERO {
            continue;
        }
        if row.currency == base_currency {
            wealth_total += value;
            add_group(&mut asset_groups, asset_label(row), value);
            add_group(&mut class_groups, asset_class_label(row), value);

            if is_stale_manual(row, as_of_date) {
                stale_manual.0 += value;
                stale_manual.1 += 1;
            }
            if is_private_or_illiquid(row) {
                private_illiquid.0 += value;
                private_illiquid.1 += 1;
            }
            if islamic_enabled && shariah_unknown_status(row.shariah_status.as_deref()) {
                shariah_unknown.0 += value;
                shariah_unknown.1 += 1;
            }
            if has_any_citation && row.source_citation_id.is_none() {
                uncited.0 += value;
                uncited.1 += 1;
            }
        }
        add_group(&mut currency_groups, row.currency.clone(), value);
    }

    let taxonomy_asset_ids = taxonomy_rows
        .iter()
        .map(|row| row.asset_id.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    let taxonomy_state = if taxonomy_rows.is_empty() {
        "missing"
    } else if taxonomy_asset_ids.len() < asset_rows.len() {
        "partial"
    } else {
        "available"
    }
    .to_string();

    add_group_exposures(
        (&mut exposures, &mut findings),
        ConcentrationDimension::Asset,
        asset_groups,
        &base_currency,
        wealth_total,
        thresholds.single_exposure_percent,
        |label, percent| {
            format!(
                "{}% of recorded wealth is in {label}.",
                display_decimal(percent)
            )
        },
    );
    add_currency_exposures(
        &mut exposures,
        &mut findings,
        currency_groups,
        asset_rows
            .iter()
            .map(|row| {
                parse_decimal("valuation amount", &row.value_native).map(|value| value.abs())
            })
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .sum(),
        thresholds.single_exposure_percent,
        |label, percent| {
            format!(
                "{}% of recorded valuation exposure is in {label}.",
                display_decimal(percent)
            )
        },
    );
    add_group_exposures(
        (&mut exposures, &mut findings),
        ConcentrationDimension::AssetClass,
        class_groups,
        &base_currency,
        wealth_total,
        thresholds.single_exposure_percent,
        |label, percent| format!("{}% of wealth is in {label}.", display_decimal(percent)),
    );

    let account_total = account_rows
        .iter()
        .filter(|row| row.base_currency == base_currency)
        .map(|row| parse_decimal("account valuation", &row.total_value).map(|value| value.abs()))
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .sum();
    let mut account_groups = BTreeMap::<String, (Decimal, i64)>::new();
    for row in &account_rows {
        if row.base_currency == base_currency {
            add_group(
                &mut account_groups,
                format!("{} ({})", row.account_name, row.account_id),
                parse_decimal("account valuation", &row.total_value)?.abs(),
            );
        }
    }
    add_group_exposures(
        (&mut exposures, &mut findings),
        ConcentrationDimension::AccountCustodian,
        account_groups,
        &base_currency,
        account_total,
        thresholds.single_exposure_percent,
        |label, percent| {
            format!(
                "{}% of account-level wealth is held at {label}.",
                display_decimal(percent)
            )
        },
    );

    add_taxonomy_exposures(
        &mut exposures,
        &mut findings,
        &asset_rows,
        &taxonomy_rows,
        &base_currency,
        wealth_total,
        thresholds.single_exposure_percent,
    )?;
    add_income_exposures(
        &mut exposures,
        &mut findings,
        income_rows,
        thresholds.combined_income_percent,
    )?;
    add_fragility_exposure(
        (&mut exposures, &mut findings),
        (
            ConcentrationDimension::ManualStale,
            "Manual valuations older than 90 days",
        ),
        stale_manual,
        &base_currency,
        wealth_total,
        thresholds.fragility_percent,
        |percent| {
            format!(
                "{}% of wealth is valued manually and older than 90 days.",
                display_decimal(percent)
            )
        },
    );
    add_fragility_exposure(
        (&mut exposures, &mut findings),
        (
            ConcentrationDimension::PrivateIlliquid,
            "Private or illiquid assets",
        ),
        private_illiquid,
        &base_currency,
        wealth_total,
        thresholds.fragility_percent,
        |percent| {
            format!(
                "{}% of wealth is in private or illiquid assets.",
                display_decimal(percent)
            )
        },
    );
    if islamic_enabled {
        add_fragility_exposure(
            (&mut exposures, &mut findings),
            (
                ConcentrationDimension::ShariahUnknown,
                "Unknown Shariah screening",
            ),
            shariah_unknown,
            &base_currency,
            wealth_total,
            Decimal::ZERO,
            |percent| {
                format!(
                    "{}% of wealth has unknown Shariah screening status.",
                    display_decimal(percent)
                )
            },
        );
    }
    if has_any_citation {
        add_fragility_exposure(
            (&mut exposures, &mut findings),
            (
                ConcentrationDimension::DocumentUncited,
                "Valuations without source citations",
            ),
            uncited,
            &base_currency,
            wealth_total,
            thresholds.fragility_percent,
            |percent| {
                format!(
                    "{}% of cited valuation coverage is missing source citations.",
                    display_decimal(percent)
                )
            },
        );
    }

    exposures.sort_by(|left, right| {
        left.dimension
            .title()
            .cmp(right.dimension.title())
            .then_with(|| right.amount.cmp(&left.amount))
            .then_with(|| left.label.cmp(&right.label))
    });
    findings.sort_by(|left, right| {
        right
            .percent
            .cmp(&left.percent)
            .then_with(|| left.label.cmp(&right.label))
    });

    Ok(ConcentrationFragilitySummary {
        as_of_date: as_of_date.to_string(),
        base_currency,
        total_wealth: wealth_total,
        exposures,
        findings,
        empty_state: asset_rows.is_empty() && account_rows.is_empty(),
        islamic_mode_enabled: islamic_enabled,
        taxonomy_state,
    })
}

fn concentration_asset_rows(conn: &mut SqliteConnection) -> Result<Vec<ConcentrationAssetRow>> {
    sql_query(
        "
        WITH ranked AS (
            SELECT
                v.asset_id,
                a.name AS asset_name,
                a.display_code,
                a.classification,
                a.quote_mode,
                v.valuation_date,
                v.value_native,
                v.currency,
                v.source_type,
                v.source_citation_id,
                pid.asset_id AS private_asset_id,
                ass.status AS shariah_status,
                ROW_NUMBER() OVER (
                    PARTITION BY v.asset_id
                    ORDER BY v.valuation_date DESC, v.created_at DESC, v.id DESC
                ) AS rn
            FROM valuations v
            JOIN assets a ON a.id = v.asset_id
            LEFT JOIN asset_private_investment_details pid ON pid.asset_id = a.id
            LEFT JOIN asset_shariah_screening ass ON ass.asset_id = a.id
            WHERE a.is_active = 1
        )
        SELECT
            asset_id,
            asset_name,
            display_code,
            classification,
            quote_mode,
            valuation_date,
            value_native,
            currency,
            source_type,
            source_citation_id,
            private_asset_id,
            shariah_status
        FROM ranked
        WHERE rn = 1
        ORDER BY asset_id ASC
        ",
    )
    .load(conn)
    .map_err(StorageError::from)
    .map_err(Error::from)
}

fn concentration_account_rows(conn: &mut SqliteConnection) -> Result<Vec<ConcentrationAccountRow>> {
    sql_query(
        "
        WITH ranked AS (
            SELECT
                dav.account_id,
                accounts.name AS account_name,
                dav.total_value,
                dav.base_currency,
                ROW_NUMBER() OVER (
                    PARTITION BY dav.account_id
                    ORDER BY dav.valuation_date DESC, dav.calculated_at DESC, dav.id DESC
                ) AS rn
            FROM daily_account_valuation dav
            JOIN accounts ON accounts.id = dav.account_id
            WHERE accounts.is_active = 1 AND accounts.is_archived = 0
        )
        SELECT account_id, account_name, total_value, base_currency
        FROM ranked
        WHERE rn = 1
        ORDER BY account_name ASC
        ",
    )
    .load(conn)
    .map_err(StorageError::from)
    .map_err(Error::from)
}

fn concentration_taxonomy_rows(
    conn: &mut SqliteConnection,
) -> Result<Vec<ConcentrationTaxonomyRow>> {
    sql_query(
        "
        SELECT
            ata.asset_id,
            taxonomies.name AS taxonomy_name,
            taxonomy_categories.name AS category_name,
            ata.weight
        FROM asset_taxonomy_assignments ata
        JOIN taxonomies ON taxonomies.id = ata.taxonomy_id
        JOIN taxonomy_categories
            ON taxonomy_categories.id = ata.category_id
           AND taxonomy_categories.taxonomy_id = ata.taxonomy_id
        ORDER BY ata.asset_id ASC, taxonomies.name ASC, taxonomy_categories.name ASC
        ",
    )
    .load(conn)
    .map_err(StorageError::from)
    .map_err(Error::from)
}

fn concentration_income_rows(conn: &mut SqliteConnection) -> Result<Vec<ConcentrationIncomeRow>> {
    sql_query(
        "
        SELECT
            COALESCE(assets.name, assets.display_code, activities.asset_id, activities.activity_type) AS source_label,
            activities.amount,
            activities.currency
        FROM activities
        LEFT JOIN assets ON assets.id = activities.asset_id
        WHERE LOWER(activities.activity_type) IN ('dividend', 'interest', 'coupon', 'distribution')
          AND activities.amount IS NOT NULL
        ",
    )
    .load(conn)
    .map_err(StorageError::from)
    .map_err(Error::from)
}

fn concentration_thresholds(conn: &mut SqliteConnection) -> Result<ConcentrationThresholds> {
    let rows = app_settings::table
        .filter(app_settings::setting_key.like("concentration_%_threshold_percent"))
        .select((app_settings::setting_key, app_settings::setting_value))
        .load::<(String, String)>(conn)
        .map_err(StorageError::from)?;
    let mut thresholds = ConcentrationThresholds {
        single_exposure_percent: Decimal::new(25, 0),
        combined_income_percent: Decimal::new(40, 0),
        fragility_percent: Decimal::new(20, 0),
    };
    for (key, value) in rows {
        let parsed = parse_decimal("concentration threshold", &value)?;
        match key.as_str() {
            "concentration_single_exposure_threshold_percent" => {
                thresholds.single_exposure_percent = parsed;
            }
            "concentration_combined_income_threshold_percent" => {
                thresholds.combined_income_percent = parsed;
            }
            "concentration_fragility_threshold_percent" => {
                thresholds.fragility_percent = parsed;
            }
            _ => {}
        }
    }
    Ok(thresholds)
}

fn add_group(groups: &mut BTreeMap<String, (Decimal, i64)>, label: String, amount: Decimal) {
    let entry = groups.entry(label).or_default();
    entry.0 += amount;
    entry.1 += 1;
}

fn add_group_exposures(
    output: (
        &mut Vec<ConcentrationExposure>,
        &mut Vec<ConcentrationFinding>,
    ),
    dimension: ConcentrationDimension,
    groups: BTreeMap<String, (Decimal, i64)>,
    currency: &str,
    total: Decimal,
    threshold_percent: Decimal,
    message: impl Fn(&str, Decimal) -> String,
) {
    let (exposures, findings) = output;
    if total <= Decimal::ZERO {
        return;
    }
    for (label, (amount, source_count)) in groups {
        if amount <= Decimal::ZERO {
            continue;
        }
        let percent = percent_of(amount, total);
        exposures.push(ConcentrationExposure {
            dimension,
            label: label.clone(),
            amount,
            currency: currency.to_string(),
            percent,
            source_count,
        });
        if percent >= threshold_percent {
            findings.push(ConcentrationFinding {
                dimension,
                label: dimension.title().to_string(),
                message: message(&label, percent),
                amount,
                currency: currency.to_string(),
                percent,
                threshold_percent,
            });
        }
    }
}

fn add_currency_exposures(
    exposures: &mut Vec<ConcentrationExposure>,
    findings: &mut Vec<ConcentrationFinding>,
    groups: BTreeMap<String, (Decimal, i64)>,
    total: Decimal,
    threshold_percent: Decimal,
    message: impl Fn(&str, Decimal) -> String,
) {
    if total <= Decimal::ZERO {
        return;
    }
    for (currency, (amount, source_count)) in groups {
        if amount <= Decimal::ZERO {
            continue;
        }
        let percent = percent_of(amount, total);
        exposures.push(ConcentrationExposure {
            dimension: ConcentrationDimension::Currency,
            label: currency.clone(),
            amount,
            currency: currency.clone(),
            percent,
            source_count,
        });
        if percent >= threshold_percent {
            findings.push(ConcentrationFinding {
                dimension: ConcentrationDimension::Currency,
                label: ConcentrationDimension::Currency.title().to_string(),
                message: message(&currency, percent),
                amount,
                currency,
                percent,
                threshold_percent,
            });
        }
    }
}

fn add_taxonomy_exposures(
    exposures: &mut Vec<ConcentrationExposure>,
    findings: &mut Vec<ConcentrationFinding>,
    asset_rows: &[ConcentrationAssetRow],
    taxonomy_rows: &[ConcentrationTaxonomyRow],
    base_currency: &str,
    wealth_total: Decimal,
    threshold_percent: Decimal,
) -> Result<()> {
    if wealth_total <= Decimal::ZERO || taxonomy_rows.is_empty() {
        return Ok(());
    }
    let values_by_asset = asset_rows
        .iter()
        .filter(|row| row.currency == base_currency)
        .map(|row| {
            Ok((
                row.asset_id.as_str(),
                parse_decimal("valuation amount", &row.value_native)?.abs(),
            ))
        })
        .collect::<Result<BTreeMap<_, _>>>()?;
    let mut sector_groups = BTreeMap::<String, (Decimal, i64)>::new();
    let mut country_groups = BTreeMap::<String, (Decimal, i64)>::new();
    for row in taxonomy_rows {
        let Some(value) = values_by_asset.get(row.asset_id.as_str()) else {
            continue;
        };
        let weighted = *value * Decimal::from(row.weight) / Decimal::new(10000, 0);
        let taxonomy_name = row.taxonomy_name.to_ascii_lowercase();
        if taxonomy_name.contains("sector") {
            add_group(&mut sector_groups, row.category_name.clone(), weighted);
        } else if taxonomy_name.contains("country") || taxonomy_name.contains("region") {
            add_group(&mut country_groups, row.category_name.clone(), weighted);
        }
    }
    add_group_exposures(
        (exposures, findings),
        ConcentrationDimension::SectorTaxonomy,
        sector_groups,
        base_currency,
        wealth_total,
        threshold_percent,
        |label, percent| {
            format!(
                "{}% of wealth is mapped to sector {label}.",
                display_decimal(percent)
            )
        },
    );
    add_group_exposures(
        (exposures, findings),
        ConcentrationDimension::CountryTaxonomy,
        country_groups,
        base_currency,
        wealth_total,
        threshold_percent,
        |label, percent| {
            format!(
                "{}% of wealth is mapped to country/region {label}.",
                display_decimal(percent)
            )
        },
    );
    Ok(())
}

fn add_income_exposures(
    exposures: &mut Vec<ConcentrationExposure>,
    findings: &mut Vec<ConcentrationFinding>,
    income_rows: Vec<ConcentrationIncomeRow>,
    threshold_percent: Decimal,
) -> Result<()> {
    let mut groups_by_currency = BTreeMap::<String, BTreeMap<String, (Decimal, i64)>>::new();
    for row in income_rows {
        let amount = parse_decimal("income amount", &row.amount)?.abs();
        if amount == Decimal::ZERO {
            continue;
        }
        add_group(
            groups_by_currency.entry(row.currency).or_default(),
            row.source_label,
            amount,
        );
    }
    for (currency, groups) in groups_by_currency {
        let total = groups.values().map(|(amount, _)| *amount).sum();
        if total <= Decimal::ZERO {
            continue;
        }
        let mut ranked = groups.into_iter().collect::<Vec<_>>();
        ranked.sort_by_key(|row| std::cmp::Reverse(row.1 .0));
        for (label, (amount, source_count)) in &ranked {
            let percent = percent_of(*amount, total);
            exposures.push(ConcentrationExposure {
                dimension: ConcentrationDimension::IncomeSource,
                label: label.clone(),
                amount: *amount,
                currency: currency.clone(),
                percent,
                source_count: *source_count,
            });
        }
        let top_count = ranked.len().min(2);
        if top_count > 0 {
            let combined = ranked
                .iter()
                .take(top_count)
                .map(|(_, (amount, _))| *amount)
                .sum();
            let percent = percent_of(combined, total);
            if percent >= threshold_percent {
                findings.push(ConcentrationFinding {
                    dimension: ConcentrationDimension::IncomeSource,
                    label: if top_count == 1 {
                        "Top income source".to_string()
                    } else {
                        "Top two income sources".to_string()
                    },
                    message: if top_count == 1 {
                        format!(
                            "{}% of income comes from one asset.",
                            display_decimal(percent)
                        )
                    } else {
                        format!(
                            "{}% of income comes from two assets.",
                            display_decimal(percent)
                        )
                    },
                    amount: combined,
                    currency: currency.clone(),
                    percent,
                    threshold_percent,
                });
            }
        }
    }
    Ok(())
}

fn add_fragility_exposure(
    output: (
        &mut Vec<ConcentrationExposure>,
        &mut Vec<ConcentrationFinding>,
    ),
    target: (ConcentrationDimension, &str),
    exposure: (Decimal, i64),
    currency: &str,
    wealth_total: Decimal,
    threshold_percent: Decimal,
    message: impl Fn(Decimal) -> String,
) {
    let (exposures, findings) = output;
    let (dimension, label) = target;
    if wealth_total <= Decimal::ZERO || exposure.0 <= Decimal::ZERO {
        return;
    }
    let percent = percent_of(exposure.0, wealth_total);
    exposures.push(ConcentrationExposure {
        dimension,
        label: label.to_string(),
        amount: exposure.0,
        currency: currency.to_string(),
        percent,
        source_count: exposure.1,
    });
    if percent >= threshold_percent {
        findings.push(ConcentrationFinding {
            dimension,
            label: label.to_string(),
            message: message(percent),
            amount: exposure.0,
            currency: currency.to_string(),
            percent,
            threshold_percent,
        });
    }
}

fn percent_of(amount: Decimal, total: Decimal) -> Decimal {
    if total <= Decimal::ZERO {
        Decimal::ZERO
    } else {
        (amount / total * Decimal::new(100, 0)).round_dp(2)
    }
}

fn display_decimal(value: Decimal) -> String {
    value.normalize().to_string()
}

fn asset_label(row: &ConcentrationAssetRow) -> String {
    row.asset_name
        .clone()
        .or_else(|| row.display_code.clone())
        .unwrap_or_else(|| row.asset_id.clone())
}

fn asset_class_label(row: &ConcentrationAssetRow) -> String {
    row.classification
        .clone()
        .unwrap_or_else(|| row.quote_mode.clone())
        .replace('_', " ")
}

fn is_stale_manual(row: &ConcentrationAssetRow, as_of_date: NaiveDate) -> bool {
    if !row.source_type.eq_ignore_ascii_case("manual")
        && !row.quote_mode.eq_ignore_ascii_case("manual")
    {
        return false;
    }
    let Some(date_text) = row.valuation_date.get(0..10) else {
        return false;
    };
    let Ok(date) = NaiveDate::parse_from_str(date_text, "%Y-%m-%d") else {
        return false;
    };
    (as_of_date - date).num_days() > 90
}

fn is_private_or_illiquid(row: &ConcentrationAssetRow) -> bool {
    row.private_asset_id.is_some()
        || row
            .classification
            .as_deref()
            .is_some_and(|classification| classification.contains("private"))
}

fn shariah_unknown_status(status: Option<&str>) -> bool {
    matches!(status, None | Some("unknown") | Some("needs_review"))
}

fn validate_currency_code(currency: &str) -> Result<()> {
    if currency.len() != 3 || !currency.chars().all(|ch| ch.is_ascii_uppercase()) {
        return Err(Error::Validation(
            mizan_core::errors::ValidationError::InvalidInput(format!(
                "currency {currency:?} must be a 3-letter uppercase ISO code"
            )),
        ));
    }
    Ok(())
}

fn estate_account_lines(conn: &mut SqliteConnection) -> Result<Vec<ReportLine>> {
    let rows = accounts::table
        .filter(accounts::is_archived.eq(false))
        .select((
            accounts::id,
            accounts::name,
            accounts::account_type,
            accounts::currency,
        ))
        .order(accounts::name.asc())
        .load::<EstateAccountRow>(conn)
        .map_err(StorageError::from)?;
    Ok(rows
        .into_iter()
        .map(|row| {
            text_line(
                "",
                row.name,
                format!("{} account in {}", row.account_type, row.currency),
                json!({"sourceTable":"accounts","sourceId":row.id,"citationStatus":"missing"}),
            )
        })
        .collect())
}

fn estate_asset_lines(
    conn: &mut SqliteConnection,
    liabilities_only: bool,
) -> Result<Vec<ReportLine>> {
    let rows = assets::table
        .filter(assets::is_active.eq(1))
        .select((
            assets::id,
            assets::kind,
            assets::name,
            assets::display_code,
            assets::classification,
        ))
        .order((assets::kind.asc(), assets::id.asc()))
        .load::<EstateAssetRow>(conn)
        .map_err(StorageError::from)?;
    Ok(rows
        .into_iter()
        .filter(|row| {
            let is_liability = row.kind.eq_ignore_ascii_case("LIABILITY")
                || row
                    .classification
                    .as_deref()
                    .is_some_and(|value| value.eq_ignore_ascii_case("liability"));
            is_liability == liabilities_only
        })
        .map(|row| {
            let label = row
                .name
                .or(row.display_code)
                .unwrap_or_else(|| row.id.clone());
            text_line(
                "",
                label,
                format!(
                    "{}{}",
                    row.kind,
                    row.classification
                        .map(|classification| format!("; {classification}"))
                        .unwrap_or_default()
                ),
                json!({"sourceTable":"assets","sourceId":row.id,"citationStatus":"missing"}),
            )
        })
        .collect())
}

fn estate_liability_lines(conn: &mut SqliteConnection) -> Result<Vec<ReportLine>> {
    let detail_rows = asset_liability_details::table
        .select((
            asset_liability_details::asset_id,
            asset_liability_details::liability_type,
            asset_liability_details::lender,
            asset_liability_details::source_citation_id,
        ))
        .order(asset_liability_details::asset_id.asc())
        .load::<EstateLiabilityRow>(conn)
        .map_err(StorageError::from)?;
    if !detail_rows.is_empty() {
        return Ok(detail_rows
            .into_iter()
            .map(|row| ReportLine {
                id: Uuid::new_v4().to_string(),
                section_id: String::new(),
                label: row.lender.unwrap_or_else(|| row.asset_id.clone()),
                amount: None,
                currency: None,
                value_text: Some(row.liability_type),
                source_citation_id: row.source_citation_id,
                metadata_json: Some(
                    json!({"sourceTable":"asset_liability_details","sourceId":row.asset_id})
                        .to_string(),
                ),
            })
            .collect());
    }
    estate_asset_lines(conn, true)
}

fn estate_property_lines(conn: &mut SqliteConnection) -> Result<Vec<ReportLine>> {
    let rows = asset_real_estate_details::table
        .select((
            asset_real_estate_details::asset_id,
            asset_real_estate_details::property_type,
            asset_real_estate_details::address_approximate,
            asset_real_estate_details::source_citation_id,
        ))
        .order(asset_real_estate_details::asset_id.asc())
        .load::<EstatePropertyRow>(conn)
        .map_err(StorageError::from)?;
    Ok(rows
        .into_iter()
        .map(|row| ReportLine {
            id: Uuid::new_v4().to_string(),
            section_id: String::new(),
            label: row
                .address_approximate
                .clone()
                .unwrap_or_else(|| row.asset_id.clone()),
            amount: None,
            currency: None,
            value_text: Some(
                row.property_type
                    .unwrap_or_else(|| "Real estate".to_string()),
            ),
            source_citation_id: row.source_citation_id,
            metadata_json: Some(
                json!({"sourceTable":"asset_real_estate_details","sourceId":row.asset_id})
                    .to_string(),
            ),
        })
        .collect())
}

fn estate_insurance_lines(conn: &mut SqliteConnection) -> Result<Vec<ReportLine>> {
    let rows = asset_insurance_details::table
        .select((
            asset_insurance_details::asset_id,
            asset_insurance_details::policy_type,
            asset_insurance_details::provider,
            asset_insurance_details::source_citation_id,
        ))
        .order(asset_insurance_details::asset_id.asc())
        .load::<EstateInsuranceRow>(conn)
        .map_err(StorageError::from)?;
    Ok(rows
        .into_iter()
        .map(|row| ReportLine {
            id: Uuid::new_v4().to_string(),
            section_id: String::new(),
            label: row.provider.unwrap_or_else(|| row.asset_id.clone()),
            amount: None,
            currency: None,
            value_text: Some(row.policy_type),
            source_citation_id: row.source_citation_id,
            metadata_json: Some(
                json!({"sourceTable":"asset_insurance_details","sourceId":row.asset_id})
                    .to_string(),
            ),
        })
        .collect())
}

fn estate_pension_lines(conn: &mut SqliteConnection) -> Result<Vec<ReportLine>> {
    Ok(estate_account_lines(conn)?
        .into_iter()
        .filter(|line| {
            line.value_text.as_deref().is_some_and(|value| {
                let lower = value.to_ascii_lowercase();
                lower.contains("pension") || lower.contains("retirement")
            })
        })
        .collect())
}

fn estate_private_investment_lines(conn: &mut SqliteConnection) -> Result<Vec<ReportLine>> {
    let rows = asset_private_investment_details::table
        .select((
            asset_private_investment_details::asset_id,
            asset_private_investment_details::instrument_subtype,
            asset_private_investment_details::manager,
            asset_private_investment_details::source_citation_id,
        ))
        .order(asset_private_investment_details::asset_id.asc())
        .load::<EstatePrivateInvestmentRow>(conn)
        .map_err(StorageError::from)?;
    Ok(rows
        .into_iter()
        .map(|row| ReportLine {
            id: Uuid::new_v4().to_string(),
            section_id: String::new(),
            label: row.manager.unwrap_or_else(|| row.asset_id.clone()),
            amount: None,
            currency: None,
            value_text: Some(row.instrument_subtype),
            source_citation_id: row.source_citation_id,
            metadata_json: Some(
                json!({"sourceTable":"asset_private_investment_details","sourceId":row.asset_id})
                    .to_string(),
            ),
        })
        .collect())
}

fn estate_document_lines(conn: &mut SqliteConnection) -> Result<Vec<ReportLine>> {
    let rows = documents::table
        .select((
            documents::id,
            documents::original_name,
            documents::mime_type,
            documents::status,
        ))
        .order(documents::original_name.asc())
        .load::<EstateDocumentRow>(conn)
        .map_err(StorageError::from)?;
    Ok(rows
        .into_iter()
        .map(|row| {
            text_line(
                "",
                row.original_name,
                format!("{}; {}", row.mime_type, row.status),
                json!({"sourceTable":"documents","sourceId":row.id,"citationStatus":"missing"}),
            )
        })
        .collect())
}

fn estate_islamic_lines(conn: &mut SqliteConnection) -> Result<Vec<ReportLine>> {
    let latest = zakat_snapshots::table
        .select((
            zakat_snapshots::id,
            zakat_snapshots::snapshot_date,
            zakat_snapshots::zakat_due,
            zakat_snapshots::base_currency,
        ))
        .order((
            zakat_snapshots::snapshot_date.desc(),
            zakat_snapshots::id.desc(),
        ))
        .first::<LatestZakatSnapshotRow>(conn)
        .optional()
        .map_err(StorageError::from)?;
    let Some(snapshot) = latest else {
        return Ok(Vec::new());
    };
    let zakat_due = parse_decimal("zakat due", &snapshot.zakat_due)?;
    Ok(vec![amount_line(
        "",
        format!("Zakat snapshot {}", snapshot.snapshot_date),
        zakat_due,
        snapshot.base_currency,
        None,
        json!({"sourceTable":"zakat_snapshots","sourceId":snapshot.id,"citationStatus":"missing"}),
    )])
}

fn build_fee_summary(
    conn: &mut SqliteConnection,
    period_month: &str,
) -> Result<FeeIntelligenceSummary> {
    let current_rows = fee_source_rows(conn)?
        .into_iter()
        .filter(|row| row.fee_date.starts_with(period_month))
        .collect::<Vec<_>>();
    let mut totals_by_currency = BTreeMap::<String, Decimal>::new();
    let mut totals_by_category = BTreeMap::<(String, String), Decimal>::new();
    for row in &current_rows {
        *totals_by_currency.entry(row.currency.clone()).or_default() += row.amount;
        *totals_by_category
            .entry((row.category.as_str().to_string(), row.currency.clone()))
            .or_default() += row.amount;
    }

    let totals = totals_by_currency
        .iter()
        .map(|(currency, amount)| FeeCurrencyTotal {
            amount: *amount,
            currency: currency.clone(),
        })
        .collect::<Vec<_>>();
    let category_totals = totals_by_category
        .into_iter()
        .map(|((category, currency), amount)| {
            Ok(FeeCategoryTotal {
                category: FeeCategory::from_str(&category)?,
                amount,
                currency,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    let spike = detect_fee_spike(conn, period_month, &totals)?;

    Ok(FeeIntelligenceSummary {
        period_month: period_month.to_string(),
        totals,
        category_totals,
        spike,
        missing_fees_state: current_rows.is_empty(),
    })
}

fn fee_source_rows(conn: &mut SqliteConnection) -> Result<Vec<FeeSourceRow>> {
    let manual_rows = manual_fee_entries::table
        .select(ManualFeeEntryRow::as_select())
        .load::<ManualFeeEntryRow>(conn)
        .map_err(StorageError::from)?;
    let mut rows = manual_rows
        .into_iter()
        .map(|row| {
            Ok(FeeSourceRow {
                id: row.id,
                fee_date: row.fee_date,
                category: FeeCategory::from_str(&row.category)?,
                amount: parse_decimal("manual fee amount", &row.amount)?,
                currency: row.currency,
                account_id: row.account_id,
                asset_id: row.asset_id,
                source_citation_id: row.source_citation_id,
                source_table: "manual_fee_entries",
            })
        })
        .collect::<Result<Vec<_>>>()?;

    let activity_rows = activities::table
        .select((
            activities::id,
            activities::account_id,
            activities::asset_id,
            activities::activity_type,
            activities::subtype,
            activities::activity_date,
            activities::amount,
            activities::fee,
            activities::currency,
        ))
        .load::<ActivityFeeRow>(conn)
        .map_err(StorageError::from)?;
    for row in activity_rows {
        if let Some(fee_text) = row.fee.as_deref() {
            let fee = parse_decimal("activity fee", fee_text)?;
            if fee != Decimal::ZERO {
                rows.push(FeeSourceRow {
                    id: row.id.clone(),
                    fee_date: row.activity_date.chars().take(10).collect(),
                    category: classify_activity_fee(&row, false),
                    amount: fee.abs(),
                    currency: row.currency.clone(),
                    account_id: Some(row.account_id.clone()),
                    asset_id: row.asset_id.clone(),
                    source_citation_id: None,
                    source_table: "activities",
                });
            }
        }
        if row.activity_type.eq_ignore_ascii_case("FEE") {
            if let Some(amount_text) = row.amount.as_deref() {
                let amount = parse_decimal("fee activity amount", amount_text)?;
                if amount != Decimal::ZERO {
                    let category = classify_activity_fee(&row, true);
                    rows.push(FeeSourceRow {
                        id: row.id,
                        fee_date: row.activity_date.chars().take(10).collect(),
                        category,
                        amount: amount.abs(),
                        currency: row.currency,
                        account_id: Some(row.account_id),
                        asset_id: row.asset_id,
                        source_citation_id: None,
                        source_table: "activities",
                    });
                }
            }
        }
    }
    Ok(rows)
}

fn classify_activity_fee(row: &ActivityFeeRow, fee_activity_amount: bool) -> FeeCategory {
    let subtype = row.subtype.as_deref().unwrap_or("").to_ascii_lowercase();
    if subtype.contains("fx") || subtype.contains("foreign") || subtype.contains("currency") {
        FeeCategory::FxFees
    } else if subtype.contains("platform") {
        FeeCategory::PlatformFees
    } else if subtype.contains("advis") {
        FeeCategory::AdvisoryFees
    } else if subtype.contains("custody") || subtype.contains("admin") {
        FeeCategory::CustodyAdminFees
    } else if fee_activity_amount {
        FeeCategory::BrokerFees
    } else {
        FeeCategory::TransactionFees
    }
}

fn detect_fee_spike(
    conn: &mut SqliteConnection,
    period_month: &str,
    current_totals: &[FeeCurrencyTotal],
) -> Result<Option<FeeSpikeAlert>> {
    let previous_months = previous_three_months(period_month)?;
    if previous_months.is_empty() {
        return Ok(None);
    }
    let all_rows = fee_source_rows(conn)?;
    let mut prior_totals = BTreeMap::<String, Decimal>::new();
    for row in all_rows {
        if previous_months
            .iter()
            .any(|month| row.fee_date.starts_with(month))
        {
            *prior_totals.entry(row.currency).or_default() += row.amount;
        }
    }
    for total in current_totals {
        let prior_average = prior_totals
            .get(&total.currency)
            .copied()
            .unwrap_or_default()
            / Decimal::from(3);
        if prior_average > Decimal::ZERO && total.amount > prior_average * Decimal::from(2) {
            return Ok(Some(FeeSpikeAlert {
                currency: total.currency.clone(),
                current_period_total: total.amount,
                prior_average,
                multiple: total.amount / prior_average,
            }));
        }
    }
    Ok(None)
}

fn previous_three_months(period_month: &str) -> Result<Vec<String>> {
    let start =
        NaiveDate::parse_from_str(&format!("{period_month}-01"), "%Y-%m-%d").map_err(|err| {
            Error::Validation(mizan_core::errors::ValidationError::InvalidInput(format!(
                "period_month is invalid: {err}"
            )))
        })?;
    let mut months = Vec::new();
    let mut year = start.year();
    let mut month = start.month() as i32;
    for _ in 0..3 {
        month -= 1;
        if month == 0 {
            month = 12;
            year -= 1;
        }
        months.push(format!("{year:04}-{month:02}"));
    }
    Ok(months)
}

fn aggregate_fee_lines<'a>(
    section_id: &str,
    rows: impl Iterator<Item = (&'a str, &'a str, Decimal)>,
    prefix: &str,
) -> Vec<ReportLine> {
    let mut totals = BTreeMap::<(String, String), Decimal>::new();
    for (entity_id, currency, amount) in rows {
        *totals
            .entry((entity_id.to_string(), currency.to_string()))
            .or_default() += amount;
    }
    totals
        .into_iter()
        .map(|((entity_id, currency), amount)| {
            amount_line(
                section_id,
                format!("{prefix} {entity_id}"),
                amount,
                currency,
                None,
                json!({"entityId":entity_id,"citationStatus":"mixed"}),
            )
        })
        .collect()
}

impl From<&ReportRun> for ReportRunRow {
    fn from(report: &ReportRun) -> Self {
        Self {
            id: report.id.clone(),
            report_type: report.report_type.as_str().to_string(),
            base_currency: report.base_currency.clone(),
            status: report.status.as_str().to_string(),
            created_at: report.created_at.clone(),
            completed_at: report.completed_at.clone(),
        }
    }
}

impl From<&ReportSection> for ReportSectionRow {
    fn from(section: &ReportSection) -> Self {
        Self {
            id: section.id.clone(),
            report_run_id: section.report_run_id.clone(),
            title: section.title.clone(),
            section_order: section.section_order,
            metadata_json: section.metadata_json.clone(),
        }
    }
}

impl From<&ReportLine> for ReportLineRow {
    fn from(line: &ReportLine) -> Self {
        Self {
            id: line.id.clone(),
            section_id: line.section_id.clone(),
            label: line.label.clone(),
            amount: line.amount.map(|value| value.normalize().to_string()),
            currency: line.currency.clone(),
            value_text: line.value_text.clone(),
            source_citation_id: line.source_citation_id.clone(),
            metadata_json: line.metadata_json.clone(),
        }
    }
}

fn manual_fee_entry_row_to_domain(row: ManualFeeEntryRow) -> Result<ManualFeeEntry> {
    Ok(ManualFeeEntry {
        id: row.id,
        fee_date: row.fee_date,
        category: FeeCategory::from_str(&row.category)?,
        amount: parse_decimal("manual fee amount", &row.amount)?,
        currency: row.currency,
        account_id: row.account_id,
        asset_id: row.asset_id,
        source_citation_id: row.source_citation_id,
        notes: row.notes,
        created_at: row.created_at,
        updated_at: row.updated_at,
    })
}

fn run_row_to_domain(row: ReportRunRow, sections: Vec<ReportSection>) -> Result<ReportRun> {
    let report_type = ReportType::from_str(&row.report_type)?;
    let disclaimer = match report_type {
        ReportType::EstateBinder => ESTATE_BINDER_DISCLAIMER,
        _ => REPORT_BUILDER_DISCLAIMER,
    };
    Ok(ReportRun {
        id: row.id,
        report_type,
        base_currency: row.base_currency,
        status: ReportRunStatus::from_str(&row.status)?,
        created_at: row.created_at,
        completed_at: row.completed_at,
        sections,
        disclaimer: disclaimer.to_string(),
    })
}

fn section_row_to_domain(
    row: ReportSectionRow,
    line_rows: Vec<ReportLineRow>,
) -> Result<ReportSection> {
    Ok(ReportSection {
        id: row.id,
        report_run_id: row.report_run_id,
        title: row.title,
        section_order: row.section_order,
        metadata_json: row.metadata_json,
        lines: line_rows
            .into_iter()
            .map(line_row_to_domain)
            .collect::<Result<Vec<_>>>()?,
    })
}

fn line_row_to_domain(row: ReportLineRow) -> Result<ReportLine> {
    Ok(ReportLine {
        id: row.id,
        section_id: row.section_id,
        label: row.label,
        amount: row
            .amount
            .map(|value| {
                Decimal::from_str(&value).map_err(|err| {
                    Error::Validation(mizan_core::errors::ValidationError::InvalidInput(format!(
                        "report line amount {value:?} is not a valid decimal: {err}"
                    )))
                })
            })
            .transpose()?,
        currency: row.currency,
        value_text: row.value_text,
        source_citation_id: row.source_citation_id,
        metadata_json: row.metadata_json,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::write_actor::spawn_writer;
    use crate::db::{create_pool, init, run_migrations};
    use crate::schema::{
        accounts, activities, app_settings, asset_private_investment_details,
        asset_shariah_screening, assets, daily_account_valuation, documents, source_citations,
        tax_pack_lines, tax_packs, valuations,
    };
    use tempfile::tempdir;

    fn setup() -> (
        Arc<Pool<r2d2::ConnectionManager<SqliteConnection>>>,
        WriteHandle,
    ) {
        std::env::set_var("CONNECT_API_URL", "http://test.local");
        let app_data = tempdir()
            .expect("tempdir")
            .keep()
            .to_string_lossy()
            .to_string();
        let db_path = init(&app_data).expect("init db");
        run_migrations(&db_path).expect("run migrations");
        let pool = create_pool(&db_path).expect("create pool");
        let writer = spawn_writer(pool.as_ref().clone()).expect("spawn writer");
        (pool, writer)
    }

    #[tokio::test]
    async fn report_run_created_with_tax_pack_lines_and_citations() {
        let (pool, writer) = setup();
        seed_tax_pack(&pool, true);
        let repo = ReportBuilderRepository::new(pool, writer);

        let report = repo
            .generate_report(request(ReportType::TaxPack))
            .await
            .expect("report");
        let reloaded = repo
            .get_report_run(&report.id)
            .expect("lookup")
            .expect("report");

        assert_eq!(reloaded.sections.len(), 1);
        assert_eq!(reloaded.sections[0].lines.len(), 2);
        assert!(reloaded.sections[0]
            .lines
            .iter()
            .any(|line| line.source_citation_id.as_deref() == Some("citation-1")));
        assert!(reloaded.sections[0]
            .lines
            .iter()
            .any(|line| line.value_text.as_deref() == Some("Missing source citation")));
    }

    #[tokio::test]
    async fn empty_report_state_is_persisted_without_fake_lines() {
        let (pool, writer) = setup();
        let repo = ReportBuilderRepository::new(pool, writer);

        let report = repo
            .generate_report(request(ReportType::NetWorth))
            .await
            .expect("report");

        assert_eq!(report.sections[0].lines[0].label, "No report data");
        assert!(report.sections[0].lines[0]
            .value_text
            .as_deref()
            .expect("empty")
            .contains("No deterministic source rows"));
    }

    #[tokio::test]
    async fn export_bytes_are_generated_for_report_run() {
        let (pool, writer) = setup();
        seed_tax_pack(&pool, true);
        let repo = ReportBuilderRepository::new(pool, writer);
        let report = repo
            .generate_report(request(ReportType::TaxPack))
            .await
            .expect("report");

        let export = repo.export_report(&report.id).expect("export");
        let html = String::from_utf8(export.bytes).expect("html");

        assert_eq!(export.mime_type, "text/html");
        assert!(export.file_name.starts_with("report-tax_pack-"));
        assert!(html.contains("citation-1"));
        assert!(html.contains(REPORT_BUILDER_DISCLAIMER));
    }

    #[tokio::test]
    async fn monthly_wealth_letter_generates_month_with_data() {
        let (pool, writer) = setup();
        seed_account(&pool);
        seed_activity(
            &pool,
            "income-1",
            "DIVIDEND",
            "2026-05-10",
            Some("12.3400"),
            Some("0.10"),
        );
        seed_activity(&pool, "income-2", "INTEREST", "2026-05-11", Some("2"), None);
        seed_activity(
            &pool,
            "old-income",
            "DIVIDEND",
            "2026-04-10",
            Some("99"),
            None,
        );
        let repo = ReportBuilderRepository::new(pool, writer);

        let report = repo
            .generate_report(monthly_request("2026-05"))
            .await
            .expect("monthly report");

        assert_eq!(report.report_type, ReportType::MonthlyWealthLetter);
        assert!(report
            .sections
            .iter()
            .any(|section| section.title == "Opening summary"));
        let income_line = report
            .sections
            .iter()
            .find(|section| section.title == "Income received")
            .expect("income section")
            .lines
            .first()
            .expect("income line");
        assert_eq!(
            income_line.amount,
            Some(Decimal::from_str("14.3400").expect("decimal"))
        );
        assert_eq!(income_line.currency.as_deref(), Some("USD"));
        assert!(report
            .sections
            .iter()
            .any(|section| section.title == "Fees"));
    }

    #[tokio::test]
    async fn monthly_wealth_letter_empty_month_is_honest() {
        let (pool, writer) = setup();
        let repo = ReportBuilderRepository::new(pool, writer);

        let report = repo
            .generate_report(monthly_request("2026-05"))
            .await
            .expect("monthly report");

        assert_eq!(report.sections.len(), 1);
        assert_eq!(report.sections[0].lines[0].label, "No report data");
        assert!(!report
            .sections
            .iter()
            .any(|section| section.title == "Income received"));
    }

    #[tokio::test]
    async fn monthly_wealth_letter_omits_unsupported_sections() {
        let (pool, writer) = setup();
        seed_account(&pool);
        seed_activity(&pool, "income-1", "DIVIDEND", "2026-05-10", Some("1"), None);
        let repo = ReportBuilderRepository::new(pool, writer);

        let report = repo
            .generate_report(monthly_request("2026-05"))
            .await
            .expect("monthly report");
        let titles = report
            .sections
            .iter()
            .map(|section| section.title.as_str())
            .collect::<Vec<_>>();

        assert!(titles.contains(&"Income received"));
        assert!(!titles.contains(&"Tax readiness"));
        assert!(!titles.contains(&"Upcoming capital calls"));
        assert!(!titles.contains(&"Upcoming coupons and maturities"));
    }

    #[tokio::test]
    async fn monthly_wealth_letter_preserves_high_precision_values() {
        let (pool, writer) = setup();
        seed_account(&pool);
        seed_activity(
            &pool,
            "income-1",
            "DIVIDEND",
            "2026-05-10",
            Some("1234.567890123456789"),
            None,
        );
        let repo = ReportBuilderRepository::new(pool, writer);

        let report = repo
            .generate_report(monthly_request("2026-05"))
            .await
            .expect("monthly report");
        let export = repo.export_report(&report.id).expect("export");
        let html = String::from_utf8(export.bytes).expect("html");

        assert!(html.contains("1234.567890123456789"));
    }

    #[tokio::test]
    async fn estate_binder_respects_section_selection() {
        let (pool, writer) = setup();
        seed_account(&pool);
        seed_asset(&pool, "asset-1", "INVESTMENT");
        seed_document_manifest_row(&pool, "doc-1", "statement.pdf");
        let repo = ReportBuilderRepository::new(pool, writer);

        let report = repo
            .generate_report(estate_request(vec![
                EstateBinderSection::Accounts,
                EstateBinderSection::DocumentsManifest,
            ]))
            .await
            .expect("estate binder");
        let titles = report
            .sections
            .iter()
            .map(|section| section.title.as_str())
            .collect::<Vec<_>>();

        assert_eq!(report.report_type, ReportType::EstateBinder);
        assert!(titles.contains(&"Accounts"));
        assert!(titles.contains(&"Documents manifest"));
        assert!(!titles.contains(&"Assets"));
    }

    #[tokio::test]
    async fn estate_binder_export_contains_selected_sections_only_and_disclaimer() {
        let (pool, writer) = setup();
        seed_account(&pool);
        seed_asset(&pool, "asset-1", "INVESTMENT");
        let repo = ReportBuilderRepository::new(pool, writer);

        let report = repo
            .generate_report(estate_request(vec![EstateBinderSection::Accounts]))
            .await
            .expect("estate binder");
        let export = repo.export_report(&report.id).expect("export");
        let html = String::from_utf8(export.bytes).expect("html");

        assert!(html.contains(ESTATE_BINDER_DISCLAIMER));
        assert!(html.contains("Accounts"));
        assert!(!html.contains("Assets"));
    }

    #[tokio::test]
    async fn fee_report_aggregates_manual_and_activity_fees_by_period() {
        let (pool, writer) = setup();
        seed_account(&pool);
        seed_activity(
            &pool,
            "fee-may-1",
            "BUY",
            "2026-05-10",
            Some("100"),
            Some("2"),
        );
        seed_activity(
            &pool,
            "fee-old",
            "BUY",
            "2026-04-10",
            Some("100"),
            Some("99"),
        );
        let repo = ReportBuilderRepository::new(pool, writer);
        repo.add_manual_fee_entry(ManualFeeEntryInput {
            fee_date: "2026-05-11".to_string(),
            category: FeeCategory::AdvisoryFees,
            amount: Decimal::from_str("7").expect("decimal"),
            currency: "USD".to_string(),
            account_id: Some("acc-1".to_string()),
            asset_id: None,
            source_citation_id: None,
            notes: Some("Advisory invoice".to_string()),
        })
        .await
        .expect("manual fee");

        let report = repo
            .generate_report(fee_request("2026-05"))
            .await
            .expect("fee report");
        let summary = repo
            .get_fee_intelligence_summary(Some("2026-05".to_string()))
            .expect("summary");

        assert_eq!(report.report_type, ReportType::FeeReport);
        assert!(report
            .sections
            .iter()
            .any(|section| section.title == "Fee categories"));
        assert_eq!(
            summary.totals[0].amount,
            Decimal::from_str("9").expect("decimal")
        );
        assert!(summary
            .category_totals
            .iter()
            .any(|total| total.category == FeeCategory::AdvisoryFees));
        assert!(!summary
            .category_totals
            .iter()
            .any(|total| total.amount == Decimal::from_str("99").expect("decimal")));
    }

    #[tokio::test]
    async fn fee_intelligence_detects_spike_deterministically() {
        let (pool, writer) = setup();
        seed_account(&pool);
        seed_activity(
            &pool,
            "fee-feb",
            "BUY",
            "2026-02-10",
            Some("100"),
            Some("10"),
        );
        seed_activity(
            &pool,
            "fee-mar",
            "BUY",
            "2026-03-10",
            Some("100"),
            Some("10"),
        );
        seed_activity(
            &pool,
            "fee-apr",
            "BUY",
            "2026-04-10",
            Some("100"),
            Some("10"),
        );
        seed_activity(
            &pool,
            "fee-may",
            "BUY",
            "2026-05-10",
            Some("100"),
            Some("50"),
        );
        let repo = ReportBuilderRepository::new(pool, writer);

        let summary = repo
            .get_fee_intelligence_summary(Some("2026-05".to_string()))
            .expect("summary");

        let spike = summary.spike.expect("spike");
        assert_eq!(spike.currency, "USD");
        assert_eq!(
            spike.current_period_total,
            Decimal::from_str("50").expect("decimal")
        );
    }

    #[tokio::test]
    async fn fee_report_empty_state_is_honest_when_no_fees_exist() {
        let (pool, writer) = setup();
        let repo = ReportBuilderRepository::new(pool, writer);

        let report = repo
            .generate_report(fee_request("2026-05"))
            .await
            .expect("fee report");
        let summary = repo
            .get_fee_intelligence_summary(Some("2026-05".to_string()))
            .expect("summary");

        assert_eq!(report.sections[0].lines[0].label, "No report data");
        assert!(summary.missing_fees_state);
    }

    #[tokio::test]
    async fn concentration_radar_flags_single_asset_and_report_section() {
        let (pool, writer) = setup();
        seed_concentration_asset(
            &pool,
            ConcentrationAssetSeed {
                id: "asset-large",
                name: "ACME",
                classification: "public_equity",
                value: "700",
                currency: "USD",
                valuation_date: "2026-05-01",
                source_type: "market",
                quote_mode: "MARKET",
                private_detail: false,
                citation: false,
                shariah_status: None,
            },
        );
        seed_concentration_asset(
            &pool,
            ConcentrationAssetSeed {
                id: "asset-small",
                name: "Cash fund",
                classification: "cash",
                value: "300",
                currency: "USD",
                valuation_date: "2026-05-01",
                source_type: "market",
                quote_mode: "MARKET",
                private_detail: false,
                citation: false,
                shariah_status: None,
            },
        );
        seed_daily_account_valuation(&pool, "acc-risk", "Main custodian", "1000", "USD");
        let repo = ReportBuilderRepository::new(pool, writer);

        let summary = repo
            .get_concentration_fragility_summary("USD".to_string())
            .expect("summary");
        let report = repo
            .generate_report(request(ReportType::PortfolioSummary))
            .await
            .expect("portfolio summary");

        assert!(summary
            .findings
            .iter()
            .any(|finding| finding.message == "70% of recorded wealth is in ACME."));
        assert!(report
            .sections
            .iter()
            .any(|section| section.title == "Concentration and fragility"));
        assert!(!summary
            .findings
            .iter()
            .any(|finding| finding.message.to_ascii_lowercase().contains("buy")));
    }

    #[tokio::test]
    async fn concentration_radar_flags_currency_concentration() {
        let (pool, writer) = setup();
        seed_concentration_asset(
            &pool,
            ConcentrationAssetSeed {
                id: "asset-usd",
                name: "USD asset",
                classification: "public_equity",
                value: "800",
                currency: "USD",
                valuation_date: "2026-05-01",
                source_type: "market",
                quote_mode: "MARKET",
                private_detail: false,
                citation: false,
                shariah_status: None,
            },
        );
        seed_concentration_asset(
            &pool,
            ConcentrationAssetSeed {
                id: "asset-eur",
                name: "EUR asset",
                classification: "public_equity",
                value: "200",
                currency: "EUR",
                valuation_date: "2026-05-01",
                source_type: "market",
                quote_mode: "MARKET",
                private_detail: false,
                citation: false,
                shariah_status: None,
            },
        );
        let repo = ReportBuilderRepository::new(pool, writer);

        let summary = repo
            .get_concentration_fragility_summary("USD".to_string())
            .expect("summary");

        assert!(summary.findings.iter().any(|finding| {
            finding.dimension == ConcentrationDimension::Currency
                && finding.message == "80% of recorded valuation exposure is in USD."
        }));
    }

    #[tokio::test]
    async fn concentration_radar_flags_stale_manual_exposure() {
        let (pool, writer) = setup();
        seed_concentration_asset(
            &pool,
            ConcentrationAssetSeed {
                id: "manual-old",
                name: "Apartment",
                classification: "real_estate_residential",
                value: "280",
                currency: "USD",
                valuation_date: "2026-01-01",
                source_type: "manual",
                quote_mode: "MANUAL",
                private_detail: false,
                citation: false,
                shariah_status: None,
            },
        );
        seed_concentration_asset(
            &pool,
            ConcentrationAssetSeed {
                id: "fresh",
                name: "Fresh",
                classification: "cash",
                value: "720",
                currency: "USD",
                valuation_date: "2026-05-01",
                source_type: "market",
                quote_mode: "MARKET",
                private_detail: false,
                citation: false,
                shariah_status: None,
            },
        );
        let repo = ReportBuilderRepository::new(pool, writer);

        let summary = repo
            .get_concentration_fragility_summary("USD".to_string())
            .expect("summary");

        assert!(summary.findings.iter().any(|finding| {
            finding.dimension == ConcentrationDimension::ManualStale
                && finding
                    .message
                    .contains("wealth is valued manually and older than 90 days")
        }));
    }

    #[tokio::test]
    async fn concentration_radar_handles_missing_taxonomy_without_fake_rows() {
        let (pool, writer) = setup();
        seed_concentration_asset(
            &pool,
            ConcentrationAssetSeed {
                id: "asset-no-taxonomy",
                name: "No taxonomy",
                classification: "public_equity",
                value: "100",
                currency: "USD",
                valuation_date: "2026-05-01",
                source_type: "market",
                quote_mode: "MARKET",
                private_detail: false,
                citation: false,
                shariah_status: None,
            },
        );
        let repo = ReportBuilderRepository::new(pool, writer);

        let summary = repo
            .get_concentration_fragility_summary("USD".to_string())
            .expect("summary");

        assert_eq!(summary.taxonomy_state, "missing");
        assert!(!summary.exposures.iter().any(|exposure| matches!(
            exposure.dimension,
            ConcentrationDimension::SectorTaxonomy | ConcentrationDimension::CountryTaxonomy
        )));
    }

    #[tokio::test]
    async fn concentration_radar_respects_islamic_mode_off_and_on() {
        let (pool, writer) = setup();
        seed_concentration_asset(
            &pool,
            ConcentrationAssetSeed {
                id: "unknown-shariah",
                name: "Unknown Shariah asset",
                classification: "public_equity",
                value: "100",
                currency: "USD",
                valuation_date: "2026-05-01",
                source_type: "market",
                quote_mode: "MARKET",
                private_detail: false,
                citation: false,
                shariah_status: None,
            },
        );
        let repo = ReportBuilderRepository::new(pool.clone(), writer);

        let disabled = repo
            .get_concentration_fragility_summary("USD".to_string())
            .expect("disabled summary");
        set_shariah_mode(&pool, true);
        let enabled = repo
            .get_concentration_fragility_summary("USD".to_string())
            .expect("enabled summary");

        assert!(!disabled
            .findings
            .iter()
            .any(|finding| finding.dimension == ConcentrationDimension::ShariahUnknown));
        assert!(enabled
            .findings
            .iter()
            .any(|finding| finding.dimension == ConcentrationDimension::ShariahUnknown));
    }

    fn request(report_type: ReportType) -> GenerateReportRequest {
        GenerateReportRequest {
            report_type,
            base_currency: "USD".to_string(),
            period_month: None,
            included_sections: None,
        }
    }

    fn monthly_request(period_month: &str) -> GenerateReportRequest {
        GenerateReportRequest {
            report_type: ReportType::MonthlyWealthLetter,
            base_currency: "USD".to_string(),
            period_month: Some(period_month.to_string()),
            included_sections: None,
        }
    }

    fn estate_request(included_sections: Vec<EstateBinderSection>) -> GenerateReportRequest {
        GenerateReportRequest {
            report_type: ReportType::EstateBinder,
            base_currency: "USD".to_string(),
            period_month: None,
            included_sections: Some(included_sections),
        }
    }

    fn fee_request(period_month: &str) -> GenerateReportRequest {
        GenerateReportRequest {
            report_type: ReportType::FeeReport,
            base_currency: "USD".to_string(),
            period_month: Some(period_month.to_string()),
            included_sections: None,
        }
    }

    struct ConcentrationAssetSeed<'a> {
        id: &'a str,
        name: &'a str,
        classification: &'a str,
        value: &'a str,
        currency: &'a str,
        valuation_date: &'a str,
        source_type: &'a str,
        quote_mode: &'a str,
        private_detail: bool,
        citation: bool,
        shariah_status: Option<&'a str>,
    }

    fn seed_concentration_asset(
        pool: &Arc<Pool<r2d2::ConnectionManager<SqliteConnection>>>,
        seed: ConcentrationAssetSeed<'_>,
    ) {
        let mut conn = get_connection(pool).expect("conn");
        diesel::insert_into(assets::table)
            .values((
                assets::id.eq(seed.id),
                assets::kind.eq("INVESTMENT"),
                assets::name.eq(Some(seed.name)),
                assets::display_code.eq(Some(seed.id)),
                assets::is_active.eq(1),
                assets::quote_mode.eq(seed.quote_mode),
                assets::quote_ccy.eq(seed.currency),
                assets::classification.eq(Some(seed.classification)),
                assets::created_at.eq("2026-05-01T00:00:00Z"),
                assets::updated_at.eq("2026-05-01T00:00:00Z"),
            ))
            .execute(&mut conn)
            .expect("seed concentration asset");
        let citation_id = if seed.citation {
            let citation_id = format!("citation-{}", seed.id);
            diesel::insert_into(source_citations::table)
                .values((
                    source_citations::id.eq(&citation_id),
                    source_citations::source_type.eq("manual"),
                    source_citations::citation_label.eq(format!("Citation {}", seed.id)),
                    source_citations::created_at.eq("2026-05-01T00:00:00Z"),
                ))
                .execute(&mut conn)
                .expect("seed concentration citation");
            Some(citation_id)
        } else {
            None
        };
        diesel::insert_into(valuations::table)
            .values((
                valuations::id.eq(format!("valuation-{}", seed.id)),
                valuations::asset_id.eq(seed.id),
                valuations::valuation_date.eq(seed.valuation_date),
                valuations::value_native.eq(seed.value),
                valuations::currency.eq(seed.currency),
                valuations::source_type.eq(seed.source_type),
                valuations::source_id.eq::<Option<String>>(None),
                valuations::confidence.eq(Some("1")),
                valuations::notes.eq::<Option<String>>(None),
                valuations::created_at.eq("2026-05-01T00:00:00Z"),
                valuations::updated_at.eq("2026-05-01T00:00:00Z"),
                valuations::source_citation_id.eq(citation_id.as_deref()),
            ))
            .execute(&mut conn)
            .expect("seed concentration valuation");
        if seed.private_detail {
            diesel::insert_into(asset_private_investment_details::table)
                .values((
                    asset_private_investment_details::asset_id.eq(seed.id),
                    asset_private_investment_details::instrument_subtype.eq("private_equity"),
                    asset_private_investment_details::created_at.eq("2026-05-01T00:00:00Z"),
                    asset_private_investment_details::updated_at.eq("2026-05-01T00:00:00Z"),
                ))
                .execute(&mut conn)
                .expect("seed private detail");
        }
        if let Some(status) = seed.shariah_status {
            diesel::insert_into(asset_shariah_screening::table)
                .values((
                    asset_shariah_screening::id.eq(format!("screening-{}", seed.id)),
                    asset_shariah_screening::asset_id.eq(seed.id),
                    asset_shariah_screening::profile_id
                        .eq(mizan_core::islamic_mode::DEFAULT_SHARIAH_PROFILE_ID),
                    asset_shariah_screening::status.eq(status),
                    asset_shariah_screening::created_at.eq("2026-05-01T00:00:00Z"),
                    asset_shariah_screening::updated_at.eq("2026-05-01T00:00:00Z"),
                ))
                .execute(&mut conn)
                .expect("seed shariah screening");
        }
    }

    fn seed_daily_account_valuation(
        pool: &Arc<Pool<r2d2::ConnectionManager<SqliteConnection>>>,
        account_id: &str,
        account_name: &str,
        total_value: &str,
        base_currency: &str,
    ) {
        let mut conn = get_connection(pool).expect("conn");
        diesel::insert_into(accounts::table)
            .values((
                accounts::id.eq(account_id),
                accounts::name.eq(account_name),
                accounts::account_type.eq("brokerage"),
                accounts::currency.eq(base_currency),
                accounts::is_default.eq(false),
                accounts::is_active.eq(true),
                accounts::created_at.eq("2026-05-01T00:00:00Z"),
                accounts::updated_at.eq("2026-05-01T00:00:00Z"),
                accounts::is_archived.eq(false),
                accounts::tracking_mode.eq("portfolio"),
            ))
            .execute(&mut conn)
            .expect("seed concentration account");
        diesel::insert_into(daily_account_valuation::table)
            .values((
                daily_account_valuation::id.eq(format!("daily-{account_id}")),
                daily_account_valuation::account_id.eq(account_id),
                daily_account_valuation::valuation_date
                    .eq(NaiveDate::from_ymd_opt(2026, 5, 1).expect("date")),
                daily_account_valuation::account_currency.eq(base_currency),
                daily_account_valuation::base_currency.eq(base_currency),
                daily_account_valuation::fx_rate_to_base.eq("1"),
                daily_account_valuation::cash_balance.eq("0"),
                daily_account_valuation::investment_market_value.eq(total_value),
                daily_account_valuation::total_value.eq(total_value),
                daily_account_valuation::cost_basis.eq("0"),
                daily_account_valuation::net_contribution.eq("0"),
                daily_account_valuation::calculated_at.eq("2026-05-01T00:00:00Z"),
            ))
            .execute(&mut conn)
            .expect("seed daily account valuation");
    }

    fn set_shariah_mode(
        pool: &Arc<Pool<r2d2::ConnectionManager<SqliteConnection>>>,
        enabled: bool,
    ) {
        let mut conn = get_connection(pool).expect("conn");
        diesel::update(app_settings::table.find("shariah_mode_enabled"))
            .set(app_settings::setting_value.eq(if enabled { "true" } else { "false" }))
            .execute(&mut conn)
            .expect("set shariah mode");
    }

    fn seed_tax_pack(
        pool: &Arc<Pool<r2d2::ConnectionManager<SqliteConnection>>>,
        include_citation: bool,
    ) {
        let mut conn = get_connection(pool).expect("conn");
        if include_citation {
            diesel::insert_into(source_citations::table)
                .values((
                    source_citations::id.eq("citation-1"),
                    source_citations::source_type.eq("manual"),
                    source_citations::citation_label.eq("Manual CPA review"),
                    source_citations::created_at.eq("2026-05-16T00:00:00Z"),
                ))
                .execute(&mut conn)
                .expect("seed citation");
        }
        diesel::insert_into(tax_packs::table)
            .values((
                tax_packs::id.eq("tax-pack-1"),
                tax_packs::tax_year.eq(2026),
                tax_packs::jurisdiction.eq("General"),
                tax_packs::base_currency.eq("USD"),
                tax_packs::status.eq("draft"),
                tax_packs::created_at.eq("2026-05-16T00:00:00Z"),
            ))
            .execute(&mut conn)
            .expect("seed tax pack");
        diesel::insert_into(tax_pack_lines::table)
            .values(vec![
                (
                    tax_pack_lines::id.eq("line-1"),
                    tax_pack_lines::tax_pack_id.eq("tax-pack-1"),
                    tax_pack_lines::category.eq("dividend"),
                    tax_pack_lines::amount.eq("12.3400"),
                    tax_pack_lines::currency.eq("USD"),
                    tax_pack_lines::taxable_date.eq("2026-01-01"),
                    tax_pack_lines::source_citation_id.eq(Some("citation-1")),
                ),
                (
                    tax_pack_lines::id.eq("line-2"),
                    tax_pack_lines::tax_pack_id.eq("tax-pack-1"),
                    tax_pack_lines::category.eq("interest"),
                    tax_pack_lines::amount.eq("2.50"),
                    tax_pack_lines::currency.eq("USD"),
                    tax_pack_lines::taxable_date.eq("2026-01-02"),
                    tax_pack_lines::source_citation_id.eq(None::<&str>),
                ),
            ])
            .execute(&mut conn)
            .expect("seed tax lines");
    }

    fn seed_account(pool: &Arc<Pool<r2d2::ConnectionManager<SqliteConnection>>>) {
        let mut conn = get_connection(pool).expect("conn");
        diesel::insert_into(accounts::table)
            .values((
                accounts::id.eq("acc-1"),
                accounts::name.eq("Taxable"),
                accounts::account_type.eq("brokerage"),
                accounts::currency.eq("USD"),
                accounts::is_default.eq(false),
                accounts::is_active.eq(true),
                accounts::created_at.eq("2026-05-01T00:00:00Z"),
                accounts::updated_at.eq("2026-05-01T00:00:00Z"),
                accounts::is_archived.eq(false),
                accounts::tracking_mode.eq("portfolio"),
            ))
            .execute(&mut conn)
            .expect("seed account");
    }

    fn seed_activity(
        pool: &Arc<Pool<r2d2::ConnectionManager<SqliteConnection>>>,
        id: &str,
        activity_type: &str,
        date: &str,
        amount: Option<&str>,
        fee: Option<&str>,
    ) {
        let mut conn = get_connection(pool).expect("conn");
        diesel::insert_into(activities::table)
            .values((
                activities::id.eq(id),
                activities::account_id.eq("acc-1"),
                activities::activity_type.eq(activity_type),
                activities::status.eq("POSTED"),
                activities::activity_date.eq(format!("{date}T00:00:00Z")),
                activities::amount.eq(amount),
                activities::fee.eq(fee),
                activities::currency.eq("USD"),
                activities::is_user_modified.eq(0),
                activities::needs_review.eq(0),
                activities::created_at.eq(format!("{date}T00:00:00Z")),
                activities::updated_at.eq(format!("{date}T00:00:00Z")),
            ))
            .execute(&mut conn)
            .expect("seed activity");
    }

    fn seed_asset(
        pool: &Arc<Pool<r2d2::ConnectionManager<SqliteConnection>>>,
        asset_id: &str,
        kind: &str,
    ) {
        let mut conn = get_connection(pool).expect("conn");
        diesel::insert_into(assets::table)
            .values((
                assets::id.eq(asset_id),
                assets::kind.eq(kind),
                assets::name.eq(Some("Estate asset")),
                assets::display_code.eq(Some("EST")),
                assets::is_active.eq(1),
                assets::quote_mode.eq("MANUAL"),
                assets::quote_ccy.eq("USD"),
                assets::classification.eq(Some("public_equity")),
                assets::created_at.eq("2026-05-01T00:00:00Z"),
                assets::updated_at.eq("2026-05-01T00:00:00Z"),
            ))
            .execute(&mut conn)
            .expect("seed asset");
    }

    fn seed_document_manifest_row(
        pool: &Arc<Pool<r2d2::ConnectionManager<SqliteConnection>>>,
        document_id: &str,
        original_name: &str,
    ) {
        let mut conn = get_connection(pool).expect("conn");
        diesel::insert_into(documents::table)
            .values((
                documents::id.eq(document_id),
                documents::file_hash.eq(format!("hash-{document_id}")),
                documents::original_name.eq(original_name),
                documents::mime_type.eq("application/pdf"),
                documents::file_size_bytes.eq(128_i64),
                documents::encrypted_storage_path.eq(format!("{document_id}.mizdoc")),
                documents::status.eq("processed"),
                documents::created_at.eq("2026-05-01T00:00:00Z"),
                documents::updated_at.eq("2026-05-01T00:00:00Z"),
            ))
            .execute(&mut conn)
            .expect("seed document");
    }
}
