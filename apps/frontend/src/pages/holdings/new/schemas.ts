// Zod schemas + payload builder for the universal Add Asset flow.
// Phase 1 / Prompt 5.
//
// The form keeps the shape simple: a common block (name, currency,
// initial value, valuation date) plus a handful of class-specific
// fields. The zod schema validates the form state; `toRequest` shapes
// the discriminated-union request the backend expects.

import * as z from "zod";

import type {
  CommodityRequestType,
  FixedIncomeSubtype,
  LiabilityRequestType,
  UniversalAssetClassification,
  UniversalAssetCreateRequest,
} from "@/adapters";

// ─────────────────────────────────────────────────────────────────────
// Common form fields
// ─────────────────────────────────────────────────────────────────────

const isoDate = z
  .string()
  .regex(/^\d{4}-\d{2}-\d{2}$/, "Use the format yyyy-mm-dd");

const decimalString = z
  .string()
  .min(1, "Required")
  .refine((s) => /^-?\d+(?:\.\d+)?$/.test(s.trim()), {
    message: "Enter a number (e.g. 1000 or 1234.56)",
  });

const optionalDecimalString = z
  .string()
  .optional()
  .transform((v) => (v && v.trim() !== "" ? v.trim() : undefined))
  .refine((v) => v === undefined || /^-?\d+(?:\.\d+)?$/.test(v), {
    message: "Enter a number (e.g. 1000 or 1234.56)",
  });

const optionalIsoDate = z
  .string()
  .optional()
  .transform((v) => (v && v.trim() !== "" ? v.trim() : undefined))
  .refine((v) => v === undefined || /^\d{4}-\d{2}-\d{2}$/.test(v), {
    message: "Use the format yyyy-mm-dd",
  });

const commonFields = {
  name: z.string().trim().min(1, "Required"),
  currency: z
    .string()
    .trim()
    .length(3, "Currency must be a 3-letter ISO code")
    .regex(/^[A-Z]{3}$/, "Use uppercase ISO 4217 (e.g. USD, GBP, AED)"),
  initialValue: decimalString,
  initialValueDate: isoDate,
  notes: z.string().trim().optional(),
};

// ─────────────────────────────────────────────────────────────────────
// Per-classification form schemas
// ─────────────────────────────────────────────────────────────────────

export const formSchema = z.object({
  ...commonFields,
  /**
   * The wire classification — the same string that becomes the
   * `classification` tag on the request. The card chooser writes this
   * via setValue when the user picks a card or changes the subtype
   * dropdown.
   */
  classification: z.enum([
    "public_equity",
    "etf",
    "mutual_fund",
    "fixed_income",
    "sukuk",
    "fixed_deposit",
    "cash",
    "real_estate",
    "private_equity",
    "private_credit",
    "hedge_fund",
    "venture_capital",
    "crypto",
    "commodity",
    "gold",
    "silver",
    "insurance",
    "ulip",
    "pension",
    "business_ownership",
    "collectible",
    "liability",
    "custom",
  ]),

  // Optional class-specific extras. None of these are required by
  // the backend; the form only renders the ones that match the
  // chosen classification.
  isin: z.string().trim().optional(),
  fixedIncomeSubtype: z
    .enum(["bond", "sukuk", "treasury_bill", "fixed_deposit", "cd", "structured_note", "other"])
    .optional(),
  issuer: z.string().trim().optional(),
  maturityDate: optionalIsoDate,
  propertyType: z.string().trim().optional(),
  addressApproximate: z.string().trim().optional(),
  manager: z.string().trim().optional(),
  symbol: z.string().trim().optional(),
  commodityRequestType: z
    .enum(["gold", "silver", "platinum", "palladium", "other_commodity"])
    .optional(),
  weightValue: optionalDecimalString,
  weightUnit: z.string().trim().optional(),
  purity: z.string().trim().optional(),
  provider: z.string().trim().optional(),
  businessName: z.string().trim().optional(),
  ownershipPercent: optionalDecimalString,
  collectibleType: z.string().trim().optional(),
  maker: z.string().trim().optional(),
  liabilityType: z
    .enum(["mortgage", "loan", "credit_card", "line_of_credit", "other_liability"])
    .optional(),
  lender: z.string().trim().optional(),
});

export type UniversalAssetFormValues = z.infer<typeof formSchema>;

// ─────────────────────────────────────────────────────────────────────
// Form → request transformer
// ─────────────────────────────────────────────────────────────────────

/**
 * Build the backend request from validated form state. Only the
 * fields relevant to the chosen classification flow through;
 * everything else is dropped so the wire payload stays small.
 *
 * Currency is uppercased and value strings are passed through as the
 * canonical decimal representation — the backend parses them via
 * `rust_decimal` so no precision is lost.
 */
export function toRequest(values: UniversalAssetFormValues): UniversalAssetCreateRequest {
  const common = {
    name: values.name.trim(),
    currency: values.currency.toUpperCase().trim(),
    initialValue: values.initialValue.trim(),
    initialValueDate: values.initialValueDate,
    notes: values.notes?.trim() ? values.notes.trim() : null,
  };

  const c = values.classification as UniversalAssetClassification;
  switch (c) {
    case "public_equity":
      return { classification: c, ...common, isin: values.isin?.trim() || null };
    case "etf":
      return { classification: c, ...common, isin: values.isin?.trim() || null };
    case "mutual_fund":
      return { classification: c, ...common, isin: values.isin?.trim() || null };
    case "fixed_income":
      return {
        classification: c,
        ...common,
        instrumentSubtype: (values.fixedIncomeSubtype ?? "bond") as FixedIncomeSubtype,
        issuer: values.issuer?.trim() || null,
        maturityDate: values.maturityDate ?? null,
      };
    case "sukuk":
      return {
        classification: c,
        ...common,
        issuer: values.issuer?.trim() || null,
        maturityDate: values.maturityDate ?? null,
      };
    case "fixed_deposit":
      return {
        classification: c,
        ...common,
        issuer: values.issuer?.trim() || null,
        maturityDate: values.maturityDate ?? null,
      };
    case "cash":
      return { classification: c, ...common };
    case "real_estate":
      return {
        classification: c,
        ...common,
        propertyType: values.propertyType?.trim() || null,
        addressApproximate: values.addressApproximate?.trim() || null,
      };
    case "private_equity":
    case "private_credit":
    case "hedge_fund":
    case "venture_capital":
      return { classification: c, ...common, manager: values.manager?.trim() || null };
    case "crypto":
      return { classification: c, ...common, symbol: values.symbol?.trim() || null };
    case "commodity":
      return {
        classification: c,
        ...common,
        commodityType: (values.commodityRequestType ?? "other_commodity") as CommodityRequestType,
        weightValue: values.weightValue ?? null,
        weightUnit: values.weightUnit?.trim() || null,
        purity: values.purity?.trim() || null,
      };
    case "gold":
      return {
        classification: c,
        ...common,
        weightValue: values.weightValue ?? null,
        weightUnit: values.weightUnit?.trim() || null,
        purity: values.purity?.trim() || null,
      };
    case "silver":
      return {
        classification: c,
        ...common,
        weightValue: values.weightValue ?? null,
        weightUnit: values.weightUnit?.trim() || null,
        purity: values.purity?.trim() || null,
      };
    case "insurance":
    case "ulip":
    case "pension":
      return { classification: c, ...common, provider: values.provider?.trim() || null };
    case "business_ownership":
      return {
        classification: c,
        ...common,
        businessName: values.businessName?.trim() || null,
        ownershipPercent: values.ownershipPercent ?? null,
      };
    case "collectible":
      return {
        classification: c,
        ...common,
        collectibleType: values.collectibleType?.trim() || null,
        maker: values.maker?.trim() || null,
      };
    case "liability":
      return {
        classification: c,
        ...common,
        liabilityType: (values.liabilityType ?? "other_liability") as LiabilityRequestType,
        lender: values.lender?.trim() || null,
      };
    case "custom":
      return { classification: c, ...common };
  }
}

/**
 * Helper used by the form: returns today's date as a yyyy-mm-dd
 * string so the initialValueDate field has a sensible default.
 */
export function todayIso(): string {
  const today = new Date();
  return today.toISOString().slice(0, 10);
}
