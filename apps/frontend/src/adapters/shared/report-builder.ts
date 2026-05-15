import { invoke } from "./platform";

export type ReportType =
  | "net_worth"
  | "portfolio_summary"
  | "income"
  | "data_quality"
  | "tax_pack";
export type ReportRunStatus = "generated" | "exported";

export interface GenerateReportRequest {
  reportType: ReportType;
  baseCurrency: string;
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
