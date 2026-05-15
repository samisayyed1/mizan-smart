use async_trait::async_trait;
use chrono::Utc;
use diesel::prelude::*;
use diesel::r2d2::{self, Pool};
use diesel::SqliteConnection;
use rust_decimal::Decimal;
use serde_json::json;
use std::str::FromStr;
use std::sync::Arc;
use uuid::Uuid;

use mizan_core::report_builder::{
    build_empty_report, build_report_export, GenerateReportRequest, ReportBuilderRepositoryTrait,
    ReportExportBundle, ReportLine, ReportRun, ReportRunStatus, ReportSection, ReportType,
    REPORT_BUILDER_DISCLAIMER,
};
use mizan_core::{Error, Result};

use crate::db::{get_connection, WriteHandle};
use crate::errors::StorageError;
use crate::schema::{report_lines, report_runs, report_sections, tax_pack_lines, tax_packs};

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

#[async_trait]
impl ReportBuilderRepositoryTrait for ReportBuilderRepository {
    async fn generate_report(&self, request: GenerateReportRequest) -> Result<ReportRun> {
        request.validate()?;
        let created_at = Utc::now().to_rfc3339();
        let report = match request.report_type {
            ReportType::TaxPack => self.build_tax_pack_report(request, created_at)?,
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
    Ok(ReportRun {
        id: row.id,
        report_type: ReportType::from_str(&row.report_type)?,
        base_currency: row.base_currency,
        status: ReportRunStatus::from_str(&row.status)?,
        created_at: row.created_at,
        completed_at: row.completed_at,
        sections,
        disclaimer: REPORT_BUILDER_DISCLAIMER.to_string(),
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
    use crate::schema::{source_citations, tax_pack_lines, tax_packs};
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

    fn request(report_type: ReportType) -> GenerateReportRequest {
        GenerateReportRequest {
            report_type,
            base_currency: "USD".to_string(),
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
}
