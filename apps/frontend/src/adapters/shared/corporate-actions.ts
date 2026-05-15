import { invoke } from "./platform";

export type CorporateActionType =
  | "split"
  | "reverse_split"
  | "merger"
  | "spinoff"
  | "symbol_change"
  | "return_of_capital"
  | "stock_dividend";

export type CorporateActionJsonValue =
  | string
  | number
  | boolean
  | null
  | { [key: string]: CorporateActionJsonValue }
  | CorporateActionJsonValue[];

export interface CorporateAction {
  id: string;
  assetId: string;
  actionType: CorporateActionType;
  effectiveDate: string;
  ratioNumerator?: string | null;
  ratioDenominator?: string | null;
  newSymbol?: string | null;
  metadataJson?: CorporateActionJsonValue;
  sourceCitationId?: string | null;
  createdAt: string;
}

export interface CorporateActionPositionPreview {
  accountId: string;
  quantityBefore: string;
  quantityAfter: string;
  averageCostBefore: string;
  averageCostAfter: string;
  totalCostBasis: string;
  currency: string;
}

export interface CorporateActionPreview {
  assetId: string;
  actionType: CorporateActionType;
  effectiveDate: string;
  ratio?: string | null;
  newSymbol?: string | null;
  positions: CorporateActionPositionPreview[];
  warnings: string[];
}

export interface ApplyCorporateActionRequest {
  assetId: string;
  actionType: CorporateActionType;
  effectiveDate: string;
  ratioNumerator?: string | null;
  ratioDenominator?: string | null;
  newSymbol?: string | null;
  sourceCitationId?: string | null;
}

export interface AppliedCorporateAction {
  action: CorporateAction;
  preview: CorporateActionPreview;
}

export function previewCorporateAction(
  request: ApplyCorporateActionRequest,
): Promise<CorporateActionPreview> {
  return invoke<CorporateActionPreview>("preview_corporate_action", { request });
}

export function applyCorporateAction(
  request: ApplyCorporateActionRequest,
): Promise<AppliedCorporateAction> {
  return invoke<AppliedCorporateAction>("apply_corporate_action", { request });
}

export function listCorporateActions(assetId: string): Promise<CorporateAction[]> {
  return invoke<CorporateAction[]>("list_corporate_actions", { assetId });
}
