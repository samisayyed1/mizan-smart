import { describe, expect, it, vi } from "vitest";

import {
  applyCorporateAction,
  listCorporateActions,
  previewCorporateAction,
} from "./corporate-actions";
import { invoke } from "./platform";

vi.mock("./platform", () => ({
  invoke: vi.fn<(...args: unknown[]) => Promise<unknown>>(),
}));

const invokeMock = vi.mocked(invoke);

describe("corporate actions adapter", () => {
  it("previews reviewed split requests through the shared invoke channel", async () => {
    const request = {
      assetId: "asset-1",
      actionType: "split" as const,
      effectiveDate: "2026-01-15",
      ratioNumerator: "2",
      ratioDenominator: "1",
    };
    invokeMock.mockResolvedValueOnce({
      assetId: "asset-1",
      actionType: "split",
      effectiveDate: "2026-01-15",
      ratio: "2",
      positions: [],
      warnings: [],
    });

    await previewCorporateAction(request);

    expect(invokeMock).toHaveBeenCalledWith("preview_corporate_action", { request });
  });

  it("applies user-confirmed actions without adding fake evidence", async () => {
    const request = {
      assetId: "asset-1",
      actionType: "symbol_change" as const,
      effectiveDate: "2026-01-15",
      newSymbol: "META",
    };
    invokeMock.mockResolvedValueOnce({
      action: {
        id: "action-1",
        assetId: "asset-1",
        actionType: "symbol_change",
        effectiveDate: "2026-01-15",
        newSymbol: "META",
        createdAt: "2026-01-15T00:00:00Z",
      },
      preview: {
        assetId: "asset-1",
        actionType: "symbol_change",
        effectiveDate: "2026-01-15",
        newSymbol: "META",
        positions: [],
        warnings: [],
      },
    });

    await applyCorporateAction(request);

    expect(invokeMock).toHaveBeenCalledWith("apply_corporate_action", { request });
  });

  it("lists corporate action history for one asset", async () => {
    invokeMock.mockResolvedValueOnce([]);

    await listCorporateActions("asset-1");

    expect(invokeMock).toHaveBeenCalledWith("list_corporate_actions", {
      assetId: "asset-1",
    });
  });
});
