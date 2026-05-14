import { invoke } from "./platform";

export type ReconciliationScopeType = "account" | "asset" | "document" | "import";
export type ReconciliationSourceSide = "mizan" | "external";
export type ReconciliationRunStatus = "open" | "completed" | "failed";
export type ReconciliationItemStatus = "open" | "ignored" | "accepted_adjustment";
export type ReconciliationMatchStatus =
  | "matched"
  | "possible_match"
  | "missing_in_mizan"
  | "missing_in_external"
  | "duplicate"
  | "mismatch";

export type JsonValue =
  | string
  | number
  | boolean
  | null
  | { [key: string]: JsonValue }
  | JsonValue[];

export interface ReconciliationInputItem {
  id?: string;
  itemType: string;
  rawJson: JsonValue;
  amount?: string | null;
  currency?: string | null;
  effectiveDate?: string | null;
}

export interface ReconciliationItem {
  id: string;
  runId: string;
  itemType: string;
  sourceSide: ReconciliationSourceSide;
  rawJson: JsonValue;
  normalizedHash: string;
  amount: string | null;
  currency: string | null;
  effectiveDate: string | null;
  status: ReconciliationItemStatus;
}

export interface ReconciliationMatch {
  id: string;
  runId: string;
  mizanItemId: string | null;
  externalItemId: string | null;
  matchStatus: ReconciliationMatchStatus;
  confidence: string;
  reason: string;
  createdAt: string;
}

export interface ReconciliationRun {
  id: string;
  scopeType: ReconciliationScopeType;
  scopeId: string;
  status: ReconciliationRunStatus;
  dateToleranceDays: number;
  createdAt: string;
  completedAt: string | null;
}

export interface ReconciliationRunDetail {
  run: ReconciliationRun;
  items: ReconciliationItem[];
  matches: ReconciliationMatch[];
}

export interface ReconcileImportPreviewRequest {
  scopeType: ReconciliationScopeType;
  scopeId: string;
  mizanItems: ReconciliationInputItem[];
  externalItems: ReconciliationInputItem[];
  dateToleranceDays: number;
}

export interface ReconcileAccountRequest {
  accountId: string;
  externalItems: ReconciliationInputItem[];
  dateToleranceDays: number;
}

export interface ReconcileDocumentFactsRequest {
  documentId: string;
  accountId: string | null;
  dateToleranceDays: number;
}

export interface AcceptReconciliationAdjustmentRequest {
  matchId: string;
  accountId: string;
  activityType: string;
  reason: string;
}

export interface AcceptReconciliationAdjustmentResult {
  activityId: string;
}

export interface IgnoreReconciliationMatchRequest {
  matchId: string;
  reason: string;
}

export interface ManualReconciliationMatchRequest {
  runId: string;
  mizanItemId: string;
  externalItemId: string;
  reason: string;
}

export async function reconcileImportPreview(
  request: ReconcileImportPreviewRequest,
): Promise<ReconciliationRunDetail> {
  return invoke<ReconciliationRunDetail>("reconcile_import_preview", toArgs(request));
}

export async function reconcileAccount(
  request: ReconcileAccountRequest,
): Promise<ReconciliationRunDetail> {
  return invoke<ReconciliationRunDetail>("reconcile_account", toArgs(request));
}

export async function reconcileDocumentFacts(
  request: ReconcileDocumentFactsRequest,
): Promise<ReconciliationRunDetail> {
  return invoke<ReconciliationRunDetail>("reconcile_document_facts", toArgs(request));
}

export async function getReconciliationRun(runId: string): Promise<ReconciliationRunDetail> {
  return invoke<ReconciliationRunDetail>("get_reconciliation_run", { runId });
}

export async function acceptReconciliationAdjustment(
  request: AcceptReconciliationAdjustmentRequest,
): Promise<AcceptReconciliationAdjustmentResult> {
  return invoke<AcceptReconciliationAdjustmentResult>(
    "accept_reconciliation_adjustment",
    toArgs(request),
  );
}

export async function ignoreReconciliationMatch(
  request: IgnoreReconciliationMatchRequest,
): Promise<void> {
  return invoke<void>("ignore_reconciliation_match", toArgs(request));
}

export async function manualReconciliationMatch(
  request: ManualReconciliationMatchRequest,
): Promise<ReconciliationMatch> {
  return invoke<ReconciliationMatch>("manual_reconciliation_match", toArgs(request));
}

function toArgs<T extends object>(request: T): Record<string, unknown> {
  return { ...request } as Record<string, unknown>;
}
