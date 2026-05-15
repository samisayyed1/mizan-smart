use async_trait::async_trait;
use chrono::{Datelike, NaiveDate, Utc};
use diesel::prelude::*;
use diesel::r2d2::{self, Pool};
use diesel::SqliteConnection;
use rust_decimal::Decimal;
use serde_json::json;
use std::collections::BTreeMap;
use std::str::FromStr;
use std::sync::Arc;
use uuid::Uuid;

use mizan_core::report_builder::{
    build_empty_report, build_report_export, EstateBinderSection, GenerateReportRequest,
    ReportBuilderRepositoryTrait, ReportExportBundle, ReportLine, ReportRun, ReportRunStatus,
    ReportSection, ReportType, ESTATE_BINDER_DISCLAIMER, REPORT_BUILDER_DISCLAIMER,
};
use mizan_core::{Error, Result};

use crate::db::{get_connection, WriteHandle};
use crate::errors::StorageError;
use crate::schema::{
    accounts, activities, app_settings, asset_insurance_details, asset_liability_details,
    asset_private_investment_details, asset_real_estate_details, assets, capital_calls, documents,
    extracted_facts, fixed_income_cashflows, report_lines, report_runs, report_sections,
    tax_pack_lines, tax_pack_missing_items, tax_packs, zakat_snapshots,
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

#[async_trait]
impl ReportBuilderRepositoryTrait for ReportBuilderRepository {
    async fn generate_report(&self, request: GenerateReportRequest) -> Result<ReportRun> {
        request.validate()?;
        let created_at = Utc::now().to_rfc3339();
        let report = match request.report_type {
            ReportType::TaxPack => self.build_tax_pack_report(request, created_at)?,
            ReportType::MonthlyWealthLetter => {
                self.build_monthly_wealth_letter(request, created_at)?
            }
            ReportType::EstateBinder => self.build_estate_binder(request, created_at)?,
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
}

impl ReportBuilderRepository {
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
        accounts, activities, assets, documents, source_citations, tax_pack_lines, tax_packs,
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
