import { describe, expect, it, vi } from "vitest";

import {
  addCapitalCall,
  getPrivateInvestmentDetail,
  getPrivateInvestmentSummary,
  upsertPrivateInvestment,
} from "./private-investments";
import { invoke } from "./platform";

vi.mock("./platform", () => ({
  invoke: vi.fn<(...args: unknown[]) => Promise<unknown>>(),
}));

const invokeMock = vi.mocked(invoke);

describe("private investment adapter", () => {
  it("sends upsert requests through the shared invoke channel", async () => {
    invokeMock.mockResolvedValueOnce({
      assetId: "asset-1",
      manager: "Acme",
      strategy: "Buyout",
      commitmentAmount: "1000",
      commitmentCurrency: "USD",
    });

    await upsertPrivateInvestment({
      assetId: "asset-1",
      manager: "Acme",
      strategy: "Buyout",
      commitmentAmount: "1000",
      commitmentCurrency: "USD",
    });

    expect(invokeMock).toHaveBeenCalledWith("upsert_private_investment", {
      request: {
        assetId: "asset-1",
        manager: "Acme",
        strategy: "Buyout",
        commitmentAmount: "1000",
        commitmentCurrency: "USD",
      },
    });
  });

  it("passes capital call and summary commands without fake data", async () => {
    invokeMock.mockResolvedValueOnce({
      id: "call-1",
      assetId: "asset-1",
      noticeDate: "2026-05-14",
      dueDate: "2026-05-31",
      amount: "200",
      currency: "USD",
      status: "paid",
    });
    await addCapitalCall({
      assetId: "asset-1",
      noticeDate: "2026-05-14",
      dueDate: "2026-05-31",
      amount: "200",
      currency: "USD",
      status: "paid",
    });
    expect(invokeMock).toHaveBeenLastCalledWith("add_capital_call", {
      request: {
        assetId: "asset-1",
        noticeDate: "2026-05-14",
        dueDate: "2026-05-31",
        amount: "200",
        currency: "USD",
        status: "paid",
      },
    });

    invokeMock.mockResolvedValueOnce(null);
    await getPrivateInvestmentSummary("asset-1");
    expect(invokeMock).toHaveBeenLastCalledWith("get_private_investment_summary", {
      assetId: "asset-1",
    });

    invokeMock.mockResolvedValueOnce({ jCurve: [] });
    await getPrivateInvestmentDetail("asset-1");
    expect(invokeMock).toHaveBeenLastCalledWith("get_private_investment_detail", {
      assetId: "asset-1",
    });
  });
});
