import { describe, expect, it } from "vitest";

import { reportTotalMatchesLineSum, sumDecimalStrings } from "./accuracy-invariants";

describe("accuracy invariants", () => {
  it("checks report totals using decimal strings without numeric coercion", () => {
    expect(
      reportTotalMatchesLineSum({
        reportName: "net-worth",
        lineAmounts: ["125.10", "374.90", "500.00"],
        reportedTotal: "1000.00",
      }),
    ).toEqual({ ok: true });
  });

  it("reports mismatched line sums", () => {
    const result = reportTotalMatchesLineSum({
      reportName: "tax-pack",
      lineAmounts: ["0.10", "0.20"],
      reportedTotal: "0.31",
    });

    expect(result.ok).toBe(false);
    expect(result.code).toBe("report_total_line_sum_mismatch");
    expect(result.message).toContain("0.3");
  });

  it("preserves fractional precision beyond JavaScript number-safe cents", () => {
    expect(sumDecimalStrings(["0.000000000001", "0.000000000002"])).toBe("0.000000000003");
  });

  it("rejects invalid decimal strings explicitly", () => {
    expect(() => sumDecimalStrings(["not-money"])).toThrow("Invalid decimal value");
  });
});
