import { invoke } from "./platform";

export type DataLineageEntityType = "portfolio" | "account" | "asset" | "valuation" | "alert";

export type DataLineageMetricType =
  | "net_worth"
  | "asset_value"
  | "valuation"
  | "income_this_month"
  | "data_quality_score"
  | "alert_reason"
  | "private_investment_metric"
  | "tax_pack_line"
  | "zakat_line";

export interface DataLineageInputRow {
  sourceTable: string;
  sourceId: string;
  label: string;
  value: string;
  currency: string | null;
  asOfDate: string | null;
  notes: string | null;
}

export interface DataLineageSourceCitation {
  id: string;
  label: string;
  sourceType: string;
  sourceId: string | null;
  documentId: string | null;
  extractedFactId: string | null;
  pageNumber: number | null;
  boundingBoxJson: string | null;
}

export interface DataLineageSourceDocument {
  id: string;
  name: string;
  pageNumber: number | null;
}

export interface DataLineageFxRate {
  fromCurrency: string;
  toCurrency: string;
  rate: string;
  asOfDate: string | null;
}

export interface DataLineageResponse {
  entityType: DataLineageEntityType;
  entityId: string;
  metricType: DataLineageMetricType;
  displayedValue: string;
  currency: string | null;
  formulaName: string;
  formulaDescription: string;
  inputRows: DataLineageInputRow[];
  sourceCitations: DataLineageSourceCitation[];
  sourceDocuments: DataLineageSourceDocument[];
  fxRatesUsed: DataLineageFxRate[];
  valuationDates: string[];
  roundingPolicy: string;
  warnings: string[];
  confidence: string | null;
  freshness: string | null;
  lastUpdated: string | null;
}

export interface GetDataLineageRequest {
  entityType: DataLineageEntityType;
  entityId: string;
  metricType: DataLineageMetricType;
}

export async function getDataLineage(
  request: GetDataLineageRequest,
): Promise<DataLineageResponse> {
  const args: Record<string, unknown> = {
    entityType: request.entityType,
    entityId: request.entityId,
    metricType: request.metricType,
  };
  return invoke<DataLineageResponse>("get_data_lineage", args);
}
