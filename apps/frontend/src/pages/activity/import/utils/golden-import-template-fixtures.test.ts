import { describe, expect, it } from "vitest";
import type { GoldenImportTemplateConfig, ImportMappingData } from "@/lib/types";
import { buildGoldenRowIssues, validateGoldenHeaders } from "./golden-import-templates";

interface GoldenFixture {
  id: string;
  kind: "CSV_ACTIVITY" | "CSV_HOLDINGS";
  headers: string[];
  row: string[];
  fieldMappings: Record<string, string>;
  requiredFields: string[];
}

const fixtures: GoldenFixture[] = [
  {
    id: "yahoo_finance_holdings",
    kind: "CSV_HOLDINGS",
    headers: ["Date", "Symbol", "Name", "Quantity", "Average Cost", "Currency", "Market Value"],
    row: ["2024-01-31", "AAPL", "Apple Inc.", "10", "180.25", "USD", "1802.50"],
    fieldMappings: { date: "Date", symbol: "Symbol", quantity: "Quantity", avgCost: "Average Cost", currency: "Currency" },
    requiredFields: ["date", "symbol", "quantity"],
  },
  {
    id: "yahoo_finance_transactions",
    kind: "CSV_ACTIVITY",
    headers: ["Date", "Type", "Symbol", "Quantity", "Price", "Fees", "Amount", "Currency", "Description"],
    row: ["2024-01-15", "BUY", "AAPL", "2", "180.00", "1.00", "361.00", "USD", "Buy AAPL"],
    fieldMappings: { date: "Date", activityType: "Type", symbol: "Symbol", quantity: "Quantity", unitPrice: "Price", fee: "Fees", amount: "Amount", currency: "Currency" },
    requiredFields: ["date", "activityType", "symbol", "amount"],
  },
  {
    id: "ibkr_activity",
    kind: "CSV_ACTIVITY",
    headers: ["TradeDate", "ActivityType", "Symbol", "Quantity", "TradePrice", "IBCommission", "NetCash", "Currency", "Description"],
    row: ["2024-01-15", "Buy", "MSFT", "3", "400", "1", "-1201", "USD", "Bought shares"],
    fieldMappings: { date: "TradeDate", activityType: "ActivityType", symbol: "Symbol", quantity: "Quantity", unitPrice: "TradePrice", fee: "IBCommission", amount: "NetCash", currency: "Currency" },
    requiredFields: ["date", "activityType", "amount", "currency"],
  },
  {
    id: "fidelity_activity",
    kind: "CSV_ACTIVITY",
    headers: ["Run Date", "Action", "Symbol", "Quantity", "Price ($)", "Commission", "Amount ($)", "Currency", "Description"],
    row: ["01/15/2024", "YOU BOUGHT", "VOO", "1", "430", "0", "-430", "USD", "Trade"],
    fieldMappings: { date: "Run Date", activityType: "Action", symbol: "Symbol", quantity: "Quantity", unitPrice: "Price ($)", fee: "Commission", amount: "Amount ($)", currency: "Currency" },
    requiredFields: ["date", "activityType", "amount"],
  },
  {
    id: "schwab_activity",
    kind: "CSV_ACTIVITY",
    headers: ["Date", "Action", "Symbol", "Description", "Quantity", "Price", "Fees & Comm", "Amount"],
    row: ["01/15/2024", "Buy", "SCHB", "Trade", "5", "50", "0", "-250"],
    fieldMappings: { date: "Date", activityType: "Action", symbol: "Symbol", quantity: "Quantity", unitPrice: "Price", fee: "Fees & Comm", amount: "Amount" },
    requiredFields: ["date", "activityType", "amount"],
  },
  {
    id: "generic_bank",
    kind: "CSV_ACTIVITY",
    headers: ["Date", "Type", "Description", "Amount", "Currency", "Account"],
    row: ["2024-01-15", "DEPOSIT", "Payroll", "1000", "USD", "Checking"],
    fieldMappings: { date: "Date", activityType: "Type", amount: "Amount", currency: "Currency", account: "Account" },
    requiredFields: ["date", "activityType", "amount", "currency"],
  },
  {
    id: "fixed_deposit",
    kind: "CSV_ACTIVITY",
    headers: ["Date", "Activity Type", "Instrument", "Principal", "Rate", "Currency", "Maturity Date", "Notes"],
    row: ["2024-01-15", "OPEN", "FD-001", "10000", "5.00", "USD", "2025-01-15", "Opened"],
    fieldMappings: { date: "Date", activityType: "Activity Type", symbol: "Instrument", amount: "Principal", currency: "Currency" },
    requiredFields: ["date", "activityType", "symbol", "amount", "currency"],
  },
  {
    id: "private_capital_call",
    kind: "CSV_ACTIVITY",
    headers: ["Date", "Fund", "Call Type", "Amount", "Currency", "Notice ID", "Due Date", "Notes"],
    row: ["2024-01-15", "Fund I", "CAPITAL_CALL", "25000", "USD", "N-1", "2024-01-30", "Capital call"],
    fieldMappings: { date: "Date", activityType: "Call Type", symbol: "Fund", amount: "Amount", currency: "Currency" },
    requiredFields: ["date", "activityType", "symbol", "amount", "currency"],
  },
  {
    id: "fixed_income_cashflow",
    kind: "CSV_ACTIVITY",
    headers: ["Date", "Security", "Event Type", "Amount", "Currency", "Quantity", "Price", "Notes"],
    row: ["2024-01-15", "US91282CJJ18", "COUPON", "125", "USD", "1000", "100", "Coupon"],
    fieldMappings: { date: "Date", activityType: "Event Type", symbol: "Security", amount: "Amount", currency: "Currency", quantity: "Quantity", unitPrice: "Price" },
    requiredFields: ["date", "activityType", "symbol", "amount", "currency"],
  },
];

function toMapping(fixture: GoldenFixture): ImportMappingData {
  const goldenTemplate: GoldenImportTemplateConfig = {
    id: fixture.id,
    displayName: fixture.id,
    strictHeaders: fixture.headers,
    requiredHeaders: fixture.requiredFields
      .map((field) => fixture.fieldMappings[field])
      .filter((header): header is string => Boolean(header)),
    requiredFields: fixture.requiredFields,
    noAiMapping: true,
    dryRunPreviewRequired: true,
  };

  return {
    accountId: "acc-1",
    importType: fixture.kind,
    name: fixture.id,
    fieldMappings: fixture.fieldMappings,
    activityMappings: {},
    symbolMappings: {},
    accountMappings: {},
    symbolMappingMeta: {},
    goldenTemplate,
  };
}

describe("golden import template fixtures", () => {
  it.each(fixtures)("accepts the $id fixture with strict headers", (fixture) => {
    const mapping = toMapping(fixture);
    const headers = validateGoldenHeaders(fixture.headers, mapping.goldenTemplate);
    const rowIssues = buildGoldenRowIssues(fixture.headers, [fixture.row], mapping);

    expect(headers.errors).toEqual([]);
    expect(headers.warnings).toEqual([]);
    expect(rowIssues.size).toBe(0);
  });
});
