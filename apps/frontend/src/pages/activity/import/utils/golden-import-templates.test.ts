import { describe, expect, it } from "vitest";
import { ImportFormat } from "@/lib/constants";
import type { ImportMappingData } from "@/lib/types";
import type { DraftActivity } from "../context";
import {
  buildGoldenRowIssues,
  getGoldenRequiredFields,
  mergeGoldenIssuesIntoDrafts,
  validateGoldenHeaders,
} from "./golden-import-templates";

const mapping: ImportMappingData = {
  accountId: "acc-1",
  importType: "CSV_ACTIVITY",
  name: "Generic Bank CSV (Golden)",
  fieldMappings: {
    [ImportFormat.DATE]: "Date",
    [ImportFormat.ACTIVITY_TYPE]: "Type",
    [ImportFormat.AMOUNT]: "Amount",
    [ImportFormat.CURRENCY]: "Currency",
  },
  activityMappings: {
    DEPOSIT: ["DEPOSIT"],
    WITHDRAWAL: ["WITHDRAWAL"],
  },
  symbolMappings: {},
  accountMappings: {},
  symbolMappingMeta: {},
  goldenTemplate: {
    id: "generic_bank",
    displayName: "generic bank CSV",
    strictHeaders: ["Date", "Type", "Description", "Amount", "Currency"],
    requiredHeaders: ["Date", "Type", "Amount", "Currency"],
    requiredFields: ["date", "activityType", "amount", "currency"],
    noAiMapping: true,
    dryRunPreviewRequired: true,
  },
};

function draft(rowIndex: number): DraftActivity {
  return {
    rowIndex,
    rawRow: [],
    activityDate: "2024-01-01T00:00:00.000Z",
    activityType: "DEPOSIT",
    amount: "10",
    currency: "USD",
    fee: "0",
    accountId: "acc-1",
    status: "valid",
    errors: {},
    warnings: {},
    isEdited: false,
  };
}

describe("golden-import-templates", () => {
  it("warns on unknown columns and errors on missing strict required headers", () => {
    const result = validateGoldenHeaders(["Date", "Type", "Amount", "Memo"], mapping.goldenTemplate);

    expect(result.errors.map((issue) => issue.message)).toContain(
      'generic bank CSV is missing required column "Currency".',
    );
    expect(result.warnings.map((issue) => issue.message)).toContain(
      'generic bank CSV does not define column "Memo". Review it before importing.',
    );
  });

  it("enforces required fields and detects duplicate rows during dry-run preview", () => {
    const issues = buildGoldenRowIssues(
      ["Date", "Type", "Description", "Amount", "Currency"],
      [
        ["2024-01-01", "DEPOSIT", "Payroll", "1000", "USD"],
        ["2024-01-01", "DEPOSIT", "Payroll", "1000", "USD"],
        ["2024-01-02", "WITHDRAWAL", "ATM", "", "USD"],
      ],
      mapping,
    );

    expect(issues.get(1)?.duplicateOfLineNumber).toBe(1);
    expect(issues.get(2)?.errors.amount).toContain("amount is required by generic bank CSV.");
  });

  it("merges golden errors into drafts so invalid rows cannot silently pass", () => {
    const issues = buildGoldenRowIssues(
      ["Date", "Type", "Description", "Amount", "Currency"],
      [["2024-01-02", "WITHDRAWAL", "ATM", "", "USD"]],
      mapping,
    );

    const [reviewed] = mergeGoldenIssuesIntoDrafts([draft(0)], issues);

    expect(reviewed.status).toBe("error");
    expect(reviewed.errors.amount).toContain("amount is required by generic bank CSV.");
    expect(getGoldenRequiredFields(mapping)).toEqual([
      "date",
      "activityType",
      "amount",
      "currency",
    ]);
  });
});
