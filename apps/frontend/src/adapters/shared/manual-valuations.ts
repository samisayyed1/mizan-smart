import { invoke } from "./platform";
import type { UniversalAssetClassification } from "./universal-assets";

export type ManualValuationStaleness = "current" | "warning" | "critical";

export interface ManualValuationAsset {
  assetId: string;
  name: string;
  classification: UniversalAssetClassification;
  currentValue: string | null;
  valuationDate: string | null;
  currency: string;
  notes: string | null;
  staleness: ManualValuationStaleness;
  historyCount: number;
}

export interface ManualValuationUpdateRow {
  assetId: string;
  currentValue: string;
  valuationDate: string;
  currency: string;
  notes?: string | null;
}

export interface BulkUpdateValuationsRequest {
  rows: ManualValuationUpdateRow[];
}

export interface RowValidationError {
  rowIndex: number;
  assetId?: string | null;
  field: string;
  message: string;
}

export interface BulkUpdateValuationsResult {
  updatedCount: number;
  errors: RowValidationError[];
}

export interface ManualValuationHistoryRow {
  id: string;
  assetId: string;
  valuationDate: string;
  valueNative: string;
  currency: string;
  notes: string | null;
  createdAt: string;
}

export async function listManualValuationAssets(): Promise<ManualValuationAsset[]> {
  return invoke<ManualValuationAsset[]>("list_manual_valuation_assets", {});
}

export async function bulkUpdateValuations(
  request: BulkUpdateValuationsRequest,
): Promise<BulkUpdateValuationsResult> {
  return invoke<BulkUpdateValuationsResult>("bulk_update_valuations", { request });
}

export async function getManualValuationHistory(
  assetId: string,
): Promise<ManualValuationHistoryRow[]> {
  return invoke<ManualValuationHistoryRow[]>("get_manual_valuation_history", { assetId });
}
