import { describe, expect, it, vi } from "vitest";

import { evaluateShariahScreeningRatios, listShariahScreeningProfiles } from "./islamic-mode";
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
});
