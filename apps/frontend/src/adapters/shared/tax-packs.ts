import { invoke } from "./platform";

export type TaxJurisdiction = "US" | "UK" | "Singapore" | "GCC" | "General";
export type TaxPackStatus = "draft" | "finalized" | "exported";
export type TaxPackLineCategory =
  | "realized_gain"
  | "dividend"
  | "interest"
  | "coupon"
  | "fx"
  | "private_distribution"
  | "fee"
  | "other";

export interface GenerateTaxPackRequest {
  taxYear: number;
  jurisdiction: TaxJurisdiction;
  baseCurrency: string;
}

export interface TaxPackLine {
  id: string;
  taxPackId: string;
  category: TaxPackLineCategory;
  assetId: string | null;
  activityId: string | null;
  amount: string;
  currency: string;
  taxableDate: string;
  sourceCitationId: string | null;
  notes: string | null;
}

export interface TaxPackMissingItem {
  id: string;
  taxPackId: string;
  severity: "info" | "warning";
  message: string;
  relatedActivityId: string | null;
  relatedAssetId: string | null;
}

export interface TaxPack {
  id: string;
  taxYear: number;
  jurisdiction: TaxJurisdiction;
  baseCurrency: string;
  status: TaxPackStatus;
  createdAt: string;
  finalizedAt: string | null;
  lines: TaxPackLine[];
  missingDataChecklist: TaxPackMissingItem[];
  disclaimer: string;
}

export interface TaxPackSourceDocumentManifestEntry {
  lineId: string;
  sourceCitationId: string | null;
  included: boolean;
  status: string;
}

export interface TaxPackExportManifest {
  taxPackId: string;
  files: string[];
  sourceDocuments: TaxPackSourceDocumentManifestEntry[];
  missingSources: string[];
  disclaimer: string;
}

export interface TaxPackExportBundle {
  fileName: string;
  mimeType: string;
  bytes: number[];
  manifest: TaxPackExportManifest;
}

export function generateTaxPack(request: GenerateTaxPackRequest): Promise<TaxPack> {
  return invoke<TaxPack>("generate_tax_pack", { request });
}

export function getTaxPack(taxPackId: string): Promise<TaxPack | null> {
  return invoke<TaxPack | null>("get_tax_pack", { taxPackId });
}

export function generateTaxPackExport(taxPackId: string): Promise<TaxPackExportBundle> {
  return invoke<TaxPackExportBundle>("generate_tax_pack_export", { taxPackId });
}
