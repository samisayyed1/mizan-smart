import { describe, expect, it, vi } from "vitest";

import {
  evaluateShariahCompliance,
  evaluateShariahScreeningRatios,
  listShariahScreeningProfiles,
  upsertAssetShariahScreening,
} from "./islamic-mode";
import { invoke } from "./platform";

vi.mock("./platform", () => ({
  invoke: vi.fn<(...args: unknown[]) => Promise<unknown>>(),
}));

const invokeMock = vi.mocked(invoke);

describe("islamic mode adapter", () => {
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
});
