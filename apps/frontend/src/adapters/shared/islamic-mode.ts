import { invoke } from "./platform";

export type ShariahScreeningStatus =
  | "compliant"
  | "non_compliant"
  | "questionable"
  | "unknown"
  | "needs_review";

export interface ShariahScreeningProfile {
  id: string;
  name: string;
  debtThreshold: string;
  liquidAssetsThreshold: string;
  impureIncomeThreshold: string;
  isDefault: boolean;
  createdAt: string;
  updatedAt: string;
}

export interface ShariahScreeningRatios {
  debtRatio?: string | null;
  liquidAssetsRatio?: string | null;
  impureIncomeRatio?: string | null;
}

export interface ShariahScreeningEvaluation {
  status: ShariahScreeningStatus;
  missingFields: string[];
}

export interface AssetShariahScreening {
  id: string;
  assetId: string;
  profileId: string;
  status: ShariahScreeningStatus;
  debtRatio?: string | null;
  liquidAssetsRatio?: string | null;
  impureIncomeRatio?: string | null;
  sourceCitationId?: string | null;
  manualOverrideReason?: string | null;
  reviewedAt?: string | null;
  notes?: string | null;
  createdAt: string;
  updatedAt: string;
}

export interface UpsertAssetShariahScreeningRequest {
  assetId: string;
  profileId: string;
  ratios: ShariahScreeningRatios;
  sourceCitationId?: string | null;
  notes?: string | null;
  manualOverrideStatus?: ShariahScreeningStatus | null;
  manualOverrideReason?: string | null;
}

export interface ShariahScreeningAuditEntry {
  id: string;
  screeningId: string;
  assetId: string;
  profileId: string;
  previousStatus?: ShariahScreeningStatus | null;
  newStatus: ShariahScreeningStatus;
  notes?: string | null;
  createdAt: string;
}

export function listShariahScreeningProfiles(): Promise<ShariahScreeningProfile[]> {
  return invoke<ShariahScreeningProfile[]>("list_shariah_screening_profiles");
}

export function evaluateShariahScreeningRatios(
  ratios: ShariahScreeningRatios,
): Promise<ShariahScreeningEvaluation> {
  return invoke<ShariahScreeningEvaluation>("evaluate_shariah_screening_ratios", { ratios });
}

export function evaluateShariahCompliance(
  assetId: string,
  profileId: string,
): Promise<ShariahScreeningEvaluation> {
  return invoke<ShariahScreeningEvaluation>("evaluate_shariah_compliance", {
    assetId,
    profileId,
  });
}

export function upsertAssetShariahScreening(
  request: UpsertAssetShariahScreeningRequest,
): Promise<AssetShariahScreening> {
  return invoke<AssetShariahScreening>("upsert_asset_shariah_screening", { request });
}

export function getAssetShariahScreening(
  assetId: string,
  profileId: string,
): Promise<AssetShariahScreening | null> {
  return invoke<AssetShariahScreening | null>("get_asset_shariah_screening", {
    assetId,
    profileId,
  });
}

export function listShariahScreeningAudit(
  assetId: string,
  profileId: string,
): Promise<ShariahScreeningAuditEntry[]> {
  return invoke<ShariahScreeningAuditEntry[]>("list_shariah_screening_audit", {
    assetId,
    profileId,
  });
}
