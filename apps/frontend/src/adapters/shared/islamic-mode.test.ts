import { beforeEach, describe, expect, it, vi } from "vitest";

import {
  calculateZakatSnapshot,
  evaluateShariahCompliance,
  evaluateShariahScreeningRatios,
  getPurificationPeriodSummary,
  listShariahScreeningProfiles,
  markPurificationPaid,
  upsertAssetShariahScreening,
  upsertPurificationEntry,
} from "./islamic-mode";
import { invoke } from "./platform";

vi.mock("./platform", () => ({
  invoke: vi.fn<(...args: unknown[]) => Promise<unknown>>(),
}));

const invokeMock = vi.mocked(invoke);

describe("islamic mode adapter", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("lists screening profiles through the shared invoke channel", async () => {
    invokeMock.mockResolvedValueOnce([]);

    await listShariahScreeningProfiles();

    expect(invokeMock).toHaveBeenCalledWith("list_shariah_screening_profiles");
  });

  it("evaluates ratios without fabricating missing values", async () => {
    invokeMock.mockResolvedValueOnce({ status: "unknown", missingFields: ["debtRatio"] });

    await evaluateShariahScreeningRatios({
      debtRatio: null,
      liquidAssetsRatio: "0.20",
      impureIncomeRatio: "0.01",
    });

    expect(invokeMock).toHaveBeenCalledWith("evaluate_shariah_screening_ratios", {
      ratios: {
        debtRatio: null,
        liquidAssetsRatio: "0.20",
        impureIncomeRatio: "0.01",
      },
    });
  });

  it("evaluates a stored asset screening by asset and profile", async () => {
    invokeMock.mockResolvedValueOnce({ status: "compliant", missingFields: [] });

    await evaluateShariahCompliance("asset-1", "profile-1");

    expect(invokeMock).toHaveBeenCalledWith("evaluate_shariah_compliance", {
      assetId: "asset-1",
      profileId: "profile-1",
    });
  });

  it("upserts user-entered ratios without altering the request", async () => {
    invokeMock.mockResolvedValueOnce({ id: "screening-1" });
    const request = {
      assetId: "asset-1",
      profileId: "profile-1",
      ratios: {
        debtRatio: "0.10",
        liquidAssetsRatio: "0.20",
        impureIncomeRatio: "0.01",
      },
      sourceCitationId: "citation-1",
      notes: "Reviewed from document-backed ratios",
      manualOverrideStatus: null,
      manualOverrideReason: null,
    };

    await upsertAssetShariahScreening(request);

    expect(invokeMock).toHaveBeenCalledWith("upsert_asset_shariah_screening", { request });
  });

  it("calculates a zakat snapshot with user-controlled lines and manual nisab", async () => {
    invokeMock.mockResolvedValueOnce({ id: "zakat-1" });
    const request = {
      snapshotDate: "2026-05-15",
      baseCurrency: "USD",
      nisabValue: "5000",
      notes: "Annual review",
      lines: [
        {
          assetId: "asset-1",
          category: "short_term_asset",
          amount: null,
          included: true,
          explanation: null,
          sourceCitationId: null,
        },
      ],
    };

    await calculateZakatSnapshot(request);

    expect(invokeMock).toHaveBeenCalledWith("calculate_zakat_snapshot", { request });
  });

  it("upserts and marks purification entries through the shared invoke channel", async () => {
    invokeMock.mockResolvedValueOnce({ id: "purification-1" }).mockResolvedValueOnce({
      id: "purification-1",
      status: "paid",
    });
    const request = {
      assetId: "asset-1",
      periodStart: "2026-01-01",
      periodEnd: "2026-12-31",
      totalImpureIncome: "1000",
      outstandingShares: "10000",
      userShares: "50",
      dividendReceived: null,
      impureIncomeRatio: null,
      status: null,
      sourceCitationId: null,
      notes: "Annual purification review",
    };

    await upsertPurificationEntry(request);
    await markPurificationPaid("purification-1");

    expect(invokeMock).toHaveBeenNthCalledWith(1, "upsert_purification_entry", { request });
    expect(invokeMock).toHaveBeenNthCalledWith(2, "mark_purification_paid", {
      entryId: "purification-1",
    });
  });

  it("loads purification period summaries by date range", async () => {
    invokeMock.mockResolvedValueOnce({ entries: [] });

    await getPurificationPeriodSummary("2026-01-01", "2026-12-31");

    expect(invokeMock).toHaveBeenCalledWith("get_purification_period_summary", {
      periodStart: "2026-01-01",
      periodEnd: "2026-12-31",
    });
  });
});
