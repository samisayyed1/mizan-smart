import type {
  LiquidityConfidence,
  LiquidityCurrencyGroup,
  LiquidityDirection,
  LiquidityLadderView,
  LiquidityLadderWindow,
} from "@/adapters";

export const WINDOW_LABELS: Record<LiquidityLadderWindow, string> = {
  next_30_days: "Next 30 days",
  next_90_days: "Next 90 days",
  next_12_months: "Next 12 months",
};

export function formatDecimalMoney(value: string, currency: string): string {
  const amount = Number(value);
  if (!Number.isFinite(amount)) return `${value} ${currency}`;
  return `${amount.toLocaleString(undefined, {
    minimumFractionDigits: 0,
    maximumFractionDigits: 2,
  })} ${currency}`;
}

export function confidenceLabel(value: LiquidityConfidence): string {
  return value === "confirmed" ? "Confirmed" : "Expected";
}

export function directionLabel(value: LiquidityDirection): string {
  if (value === "incoming") return "Incoming";
  if (value === "outgoing") return "Outgoing";
  return "Cash balance";
}

export function findView(
  views: LiquidityLadderView[] | undefined,
  window: LiquidityLadderWindow,
): LiquidityLadderView | null {
  return views?.find((view) => view.window === window) ?? null;
}

export function datedItemCount(group: LiquidityCurrencyGroup): number {
  return group.items.filter((item) => item.itemType !== "cash_balance").length;
}
