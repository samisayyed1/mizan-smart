import { invoke } from "./platform";

export type FixedIncomeInstrumentType =
  | "bond"
  | "sukuk"
  | "treasury_bill"
  | "fixed_deposit"
  | "cd"
  | "structured_note"
  | "other";

export type FixedIncomePaymentFrequency =
  | "monthly"
  | "quarterly"
  | "semi_annual"
  | "annual"
  | "at_maturity";

export type FixedIncomeDayCountConvention = "ACT_360" | "ACT_365" | "ACT_ACT" | "THIRTY_360";

export type FixedIncomeCashflowType = "coupon" | "profit" | "principal" | "maturity" | "interest";

export type FixedIncomeCashflowStatus = "expected" | "received" | "missed" | "cancelled";

export interface FixedIncomeDetails {
  assetId: string;
  instrumentType: FixedIncomeInstrumentType;
  issuer: string;
  isin?: string | null;
  faceValue: string;
  currency: string;
  purchaseDate?: string | null;
  maturityDate: string;
  couponOrProfitRate?: string | null;
  paymentFrequency?: FixedIncomePaymentFrequency | null;
  dayCountConvention: FixedIncomeDayCountConvention;
  isSukuk: boolean;
  sourceCitationId?: string | null;
}

export interface FixedIncomeCashflow {
  id: string;
  assetId: string;
  expectedDate: string;
  cashflowType: FixedIncomeCashflowType;
  expectedAmount: string;
  actualAmount?: string | null;
  currency: string;
  status: FixedIncomeCashflowStatus;
  sourceCitationId?: string | null;
}

export interface FixedIncomeProjection {
  details: FixedIncomeDetails;
  accruedAmount: string;
  cashflows: FixedIncomeCashflow[];
  warnings: string[];
}

export interface UpsertFixedIncomeDetailsRequest {
  assetId: string;
  instrumentType: FixedIncomeInstrumentType;
  issuer: string;
  isin?: string | null;
  faceValue: string;
  currency: string;
  purchaseDate?: string | null;
  maturityDate: string;
  couponOrProfitRate?: string | null;
  paymentFrequency?: FixedIncomePaymentFrequency | null;
  dayCountConvention: FixedIncomeDayCountConvention;
  isSukuk: boolean;
  sourceCitationId?: string | null;
}

export function upsertFixedIncomeDetails(
  request: UpsertFixedIncomeDetailsRequest,
): Promise<FixedIncomeProjection> {
  return invoke<FixedIncomeProjection>("upsert_fixed_income_details", { request });
}

export function getFixedIncomeProjection(assetId: string): Promise<FixedIncomeProjection | null> {
  return invoke<FixedIncomeProjection | null>("get_fixed_income_projection", { assetId });
}
