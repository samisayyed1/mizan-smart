import { invoke } from "./platform";

export type LiquidityLadderWindow = "next_30_days" | "next_90_days" | "next_12_months";
export type LiquidityDirection = "incoming" | "outgoing" | "balance";
export type LiquidityConfidence = "confirmed" | "expected";
export type LiquidityItemType =
  | "cash_balance"
  | "fixed_income_cashflow"
  | "sukuk_profit"
  | "fixed_deposit_maturity"
  | "private_capital_call"
  | "private_distribution"
  | "scheduled_dividend"
  | "scheduled_interest";

export interface LiquidityLadderItem {
  id: string;
  date: string;
  currency: string;
  amount: string;
  direction: LiquidityDirection;
  confidence: LiquidityConfidence;
  itemType: LiquidityItemType;
  label: string;
  sourceId?: string | null;
  notes?: string | null;
}

export interface LiquidityCurrencyGroup {
  currency: string;
  availableCash: string;
  confirmedIncoming: string;
  expectedIncoming: string;
  confirmedOutgoing: string;
  expectedOutgoing: string;
  netConfirmed: string;
  netExpected: string;
  items: LiquidityLadderItem[];
}

export interface LiquidityLadderView {
  window: LiquidityLadderWindow;
  startDate: string;
  endDate: string;
  currencyGroups: LiquidityCurrencyGroup[];
  warnings: string[];
}

export interface LiquidityLadderReport {
  asOf: string;
  views: LiquidityLadderView[];
}

export function getLiquidityLadder(asOf?: string): Promise<LiquidityLadderReport> {
  return invoke<LiquidityLadderReport>("get_liquidity_ladder", asOf ? { asOf } : undefined);
}
