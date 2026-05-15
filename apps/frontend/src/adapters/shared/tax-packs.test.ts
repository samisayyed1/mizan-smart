import { describe, expect, it, vi } from "vitest";

import { generateTaxPack, generateTaxPackExport, getTaxPack } from "./tax-packs";
import { invoke } from "./platform";

vi.mock("./platform", () => ({
  invoke: vi.fn<(...args: unknown[]) => Promise<unknown>>(),
}));

const invokeMock = vi.mocked(invoke);

describe("tax pack adapter", () => {
  it("generates tax packs through the shared invoke channel", async () => {
    invokeMock.mockResolvedValueOnce({ id: "pack-1", lines: [], missingDataChecklist: [] });
    const request = { taxYear: 2026, jurisdiction: "General" as const, baseCurrency: "USD" };

    await generateTaxPack(request);

    expect(invokeMock).toHaveBeenCalledWith("generate_tax_pack", { request });
  });

  it("loads tax packs by id", async () => {
    invokeMock.mockResolvedValueOnce(null);

    await getTaxPack("pack-1");

    expect(invokeMock).toHaveBeenCalledWith("get_tax_pack", { taxPackId: "pack-1" });
  });

  it("requests tax pack export bundles by id", async () => {
    invokeMock.mockResolvedValueOnce({ fileName: "tax-pack.zip", bytes: [] });

    await generateTaxPackExport("pack-1");

    expect(invokeMock).toHaveBeenCalledWith("generate_tax_pack_export", {
      taxPackId: "pack-1",
    });
  });
});
