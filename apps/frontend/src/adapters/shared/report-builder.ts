import { invoke } from "./platform";

export type ReportType =
  | "net_worth"
  | "portfolio_summary"
  | "income"
  | "data_quality"
  | "tax_pack"
  | "monthly_wealth_letter"
  | "estate_binder"
  | "fee_report";
export type ReportRunStatus = "generated" | "exported";
export type FeeCategory =
  | "broker_fees"
  | "transaction_fees"
  | "platform_fees"
  | "advisory_fees"
  | "fund_expense_ratio_manual"
  | "insurance_ulip_charges"
  | "fx_fees"
  | "private_fund_fees"
  | "custody_admin_fees"
  | "other";
export type EstateBinderSection =
  | "accounts"
  | "assets"
  | "liabilities"
  | "property"
  | "insurance"
  | "pensions"
  | "private_investments"
  | "documents_manifest"
  | "entity_ownership"
  | "islamic_notes";

export interface GenerateReportRequest {
  reportType: ReportType;
  baseCurrency: string;
  periodMonth?: string | null;
  includedSections?: EstateBinderSection[] | null;
}

export interface ManualFeeEntryInput {
  feeDate: string;
  category: FeeCategory;
  amount: string;
  currency: string;
  accountId?: string | null;
  assetId?: string | null;
  sourceCitationId?: string | null;
  notes?: string | null;
}

export interface ManualFeeEntry extends ManualFeeEntryInput {
  id: string;
  createdAt: string;
  updatedAt: string;
}

export interface FeeCategoryTotal {
  category: FeeCategory;
  amount: string;
  currency: string;
}

export interface FeeCurrencyTotal {
  amount: string;
  currency: string;
}

export interface FeeSpikeAlert {
  currency: string;
  currentPeriodTotal: string;
  priorAverage: string;
  multiple: string;
}

export interface FeeIntelligenceSummary {
  periodMonth: string;
  totals: FeeCurrencyTotal[];
  categoryTotals: FeeCategoryTotal[];
  spike: FeeSpikeAlert | null;
  missingFeesState: boolean;
}

export type ConcentrationDimension =
  | "asset"
  | "account_custodian"
  | "currency"
  | "sector_taxonomy"
  | "country_taxonomy"
  | "asset_class"
  | "income_source"
  | "manual_stale"
  | "private_illiquid"
  | "shariah_unknown"
  | "document_uncited";

export interface ConcentrationExposure {
  dimension: ConcentrationDimension;
  label: string;
  amount: string;
  currency: string;
  percent: string;
  sourceCount: number;
}

export interface ConcentrationFinding {
  dimension: ConcentrationDimension;
  label: string;
  message: string;
  amount: string;
  currency: string;
  percent: string;
  thresholdPercent: string;
}

export interface ConcentrationFragilitySummary {
  asOfDate: string;
  baseCurrency: string;
  totalWealth: string;
  exposures: ConcentrationExposure[];
  findings: ConcentrationFinding[];
  emptyState: boolean;
  islamicModeEnabled: boolean;
  taxonomyState: string;
}

export interface ReportLine {
  id: string;
  sectionId: string;
  label: string;
  amount: string | null;
  currency: string | null;
  valueText: string | null;
  sourceCitationId: string | null;
  metadataJson: string | null;
}

export interface ReportSection {
  id: string;
  reportRunId: string;
  title: string;
  sectionOrder: number;
  metadataJson: string | null;
  lines: ReportLine[];
}

export interface ReportRun {
  id: string;
  reportType: ReportType;
  baseCurrency: string;
  status: ReportRunStatus;
  createdAt: string;
  completedAt: string | null;
  sections: ReportSection[];
  disclaimer: string;
}

export interface ReportExportBundle {
  fileName: string;
  mimeType: string;
  bytes: number[];
}

export function generateReport(request: GenerateReportRequest): Promise<ReportRun> {
  return invoke<ReportRun>("generate_report", { request });
}

export function getReportRun(reportRunId: string): Promise<ReportRun | null> {
  return invoke<ReportRun | null>("get_report_run", { reportRunId });
}

export function exportReport(reportRunId: string): Promise<ReportExportBundle> {
  return invoke<ReportExportBundle>("export_report", { reportRunId });
}

export function addManualFeeEntry(input: ManualFeeEntryInput): Promise<ManualFeeEntry> {
  return invoke<ManualFeeEntry>("add_manual_fee_entry", { input });
}

export function getFeeIntelligenceSummary(
  periodMonth?: string | null,
): Promise<FeeIntelligenceSummary> {
  return invoke<FeeIntelligenceSummary>("get_fee_intelligence_summary", { periodMonth });
}

export function getConcentrationFragilitySummary(
  baseCurrency: string,
): Promise<ConcentrationFragilitySummary> {
  return invoke<ConcentrationFragilitySummary>("get_concentration_fragility_summary", {
    baseCurrency,
  });
}
