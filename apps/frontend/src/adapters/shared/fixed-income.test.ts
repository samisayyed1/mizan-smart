import { describe, expect, it, vi } from "vitest";

import { getFixedIncomeProjection, upsertFixedIncomeDetails } from "./fixed-income";
import { invoke } from "./platform";

vi.mock("./platform", () => ({
  invoke: vi.fn<(...args: unknown[]) => Promise<unknown>>(),
}));

const invokeMock = vi.mocked(invoke);

describe("fixed income adapter", () => {
  it("sends upsert requests through the shared invoke channel", async () => {
    invokeMock.mockResolvedValueOnce({
      details: {
        assetId: "asset-1",
        instrumentType: "sukuk",
        issuer: "Issuer",
        faceValue: "1000",
        currency: "USD",
        maturityDate: "2027-01-01",
        dayCountConvention: "ACT_365",
        isSukuk: true,
      },
      accruedAmount: "0",
      cashflows: [],
      warnings: [],
    });

    await upsertFixedIncomeDetails({
      assetId: "asset-1",
      instrumentType: "sukuk",
      issuer: "Issuer",
      faceValue: "1000",
      currency: "USD",
      purchaseDate: "2026-01-01",
      maturityDate: "2027-01-01",
      couponOrProfitRate: "0.06",
      paymentFrequency: "semi_annual",
      dayCountConvention: "ACT_365",
      isSukuk: true,
    });

    expect(invokeMock).toHaveBeenCalledWith("upsert_fixed_income_details", {
      request: {
        assetId: "asset-1",
        instrumentType: "sukuk",
        issuer: "Issuer",
        faceValue: "1000",
        currency: "USD",
        purchaseDate: "2026-01-01",
        maturityDate: "2027-01-01",
        couponOrProfitRate: "0.06",
        paymentFrequency: "semi_annual",
        dayCountConvention: "ACT_365",
        isSukuk: true,
      },
    });
  });

  it("requests projections without injecting cashflows", async () => {
    invokeMock.mockResolvedValueOnce(null);

    await getFixedIncomeProjection("asset-1");

    expect(invokeMock).toHaveBeenCalledWith("get_fixed_income_projection", {
      assetId: "asset-1",
    });
  });
});
