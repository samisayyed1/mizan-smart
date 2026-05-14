// Universal Add Asset adapter (mizan-smart Phase 1 / Prompt 5).
//
// The wire shape matches Rust's `UniversalAssetCreateRequest`
// discriminated union (tag = "classification", camelCase fields) and
// the matching Axum endpoint at POST /api/v1/universal-assets.
//
// Frontend forms validate against the matching zod schema in
// `apps/frontend/src/features/universal-add-asset/schemas.ts` and call
// `createUniversalAsset` on submit.

import { invoke } from "./platform";

/** Snake_case classification values matching `assets.classification`. */
export type UniversalAssetClassification =
  | "public_equity"
  | "etf"
  | "mutual_fund"
  | "fixed_income"
  | "sukuk"
  | "fixed_deposit"
  | "cash"
  | "real_estate"
  | "private_equity"
  | "private_credit"
  | "hedge_fund"
  | "venture_capital"
  | "crypto"
  | "commodity"
  | "gold"
  | "silver"
  | "insurance"
  | "ulip"
  | "pension"
  | "business_ownership"
  | "collectible"
  | "liability"
  | "custom";

/** Common fields every classification request carries. */
export interface UniversalAssetCommon {
  name: string;
  currency: string; // 3-letter ISO 4217, uppercase
  notes?: string | null;
  initialValue: string; // Decimal serialised as a canonical string
  initialValueDate: string; // ISO 8601 yyyy-mm-dd
}

export type FixedIncomeSubtype =
  | "bond"
  | "sukuk"
  | "treasury_bill"
  | "fixed_deposit"
  | "cd"
  | "structured_note"
  | "other";

export type CommodityRequestType =
  | "gold"
  | "silver"
  | "platinum"
  | "palladium"
  | "other_commodity";

export type LiabilityRequestType =
  | "mortgage"
  | "loan"
  | "credit_card"
  | "line_of_credit"
  | "other_liability";

/**
 * Discriminated union mirroring Rust's
 * `mizan_core::universal_assets::create_request::UniversalAssetCreateRequest`.
 * The `classification` field is the serde tag; every other field is
 * the per-variant payload (camelCased by Rust's `rename_all_fields`).
 */
export type UniversalAssetCreateRequest =
  | ({ classification: "public_equity"; subClass?: "public_equity" | "etf" | "mutual_fund" | null; isin?: string | null } & UniversalAssetCommon)
  | ({ classification: "etf"; isin?: string | null } & UniversalAssetCommon)
  | ({ classification: "mutual_fund"; isin?: string | null } & UniversalAssetCommon)
  | ({ classification: "fixed_income"; instrumentSubtype: FixedIncomeSubtype; issuer?: string | null; maturityDate?: string | null } & UniversalAssetCommon)
  | ({ classification: "sukuk"; issuer?: string | null; maturityDate?: string | null } & UniversalAssetCommon)
  | ({ classification: "fixed_deposit"; issuer?: string | null; maturityDate?: string | null } & UniversalAssetCommon)
  | ({ classification: "cash" } & UniversalAssetCommon)
  | ({ classification: "real_estate"; propertyType?: string | null; addressApproximate?: string | null } & UniversalAssetCommon)
  | ({ classification: "private_equity"; manager?: string | null } & UniversalAssetCommon)
  | ({ classification: "private_credit"; manager?: string | null } & UniversalAssetCommon)
  | ({ classification: "hedge_fund"; manager?: string | null } & UniversalAssetCommon)
  | ({ classification: "venture_capital"; manager?: string | null } & UniversalAssetCommon)
  | ({ classification: "crypto"; symbol?: string | null } & UniversalAssetCommon)
  | ({ classification: "commodity"; commodityType: CommodityRequestType; weightValue?: string | null; weightUnit?: string | null; purity?: string | null } & UniversalAssetCommon)
  | ({ classification: "gold"; weightValue?: string | null; weightUnit?: string | null; purity?: string | null } & UniversalAssetCommon)
  | ({ classification: "silver"; weightValue?: string | null; weightUnit?: string | null; purity?: string | null } & UniversalAssetCommon)
  | ({ classification: "insurance"; provider?: string | null } & UniversalAssetCommon)
  | ({ classification: "ulip"; provider?: string | null } & UniversalAssetCommon)
  | ({ classification: "pension"; provider?: string | null } & UniversalAssetCommon)
  | ({ classification: "business_ownership"; businessName?: string | null; ownershipPercent?: string | null } & UniversalAssetCommon)
  | ({ classification: "collectible"; collectibleType?: string | null; maker?: string | null } & UniversalAssetCommon)
  | ({ classification: "liability"; liabilityType: LiabilityRequestType; lender?: string | null } & UniversalAssetCommon)
  | ({ classification: "custom" } & UniversalAssetCommon);

export interface CreateUniversalAssetResponse {
  assetId: string;
  classification: UniversalAssetClassification;
  valuationId: string;
}

/**
 * Calls the backend `create_universal_asset` command. On success
 * returns the new asset id + classification + initial valuation id
 * so the caller can route to the asset detail page.
 */
export async function createUniversalAsset(
  request: UniversalAssetCreateRequest,
): Promise<CreateUniversalAssetResponse> {
  return invoke<CreateUniversalAssetResponse>("create_universal_asset", { request });
}
