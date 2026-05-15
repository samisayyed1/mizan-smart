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

export function listShariahScreeningProfiles(): Promise<ShariahScreeningProfile[]> {
  return invoke<ShariahScreeningProfile[]>("list_shariah_screening_profiles");
}

export function evaluateShariahScreeningRatios(
  ratios: ShariahScreeningRatios,
): Promise<ShariahScreeningEvaluation> {
  return invoke<ShariahScreeningEvaluation>("evaluate_shariah_screening_ratios", { ratios });
}
