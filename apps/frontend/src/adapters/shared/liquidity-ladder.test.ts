import { describe, expect, it, vi } from "vitest";

import { getLiquidityLadder } from "./liquidity-ladder";
import { invoke } from "./platform";

vi.mock("./platform", () => ({
  invoke: vi.fn<(...args: unknown[]) => Promise<unknown>>(),
}));

const invokeMock = vi.mocked(invoke);

describe("liquidity ladder adapter", () => {
  it("requests the current liquidity ladder without fake payload data", async () => {
    invokeMock.mockResolvedValueOnce({ asOf: "2026-05-15", views: [] });

    await getLiquidityLadder();

    expect(invokeMock).toHaveBeenCalledWith("get_liquidity_ladder", undefined);
  });

  it("passes an explicit as-of date through the shared invoke channel", async () => {
    invokeMock.mockResolvedValueOnce({ asOf: "2026-05-15", views: [] });

    await getLiquidityLadder("2026-05-15");

    expect(invokeMock).toHaveBeenCalledWith("get_liquidity_ladder", {
      asOf: "2026-05-15",
    });
  });
});
