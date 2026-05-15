import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { MemoryRouter } from "react-router-dom";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { generateTaxPack, generateTaxPackExport } from "@/adapters";
import TaxPackPage from "./tax-pack-page";

vi.mock("@/adapters", () => ({
  generateTaxPack: vi.fn(),
  generateTaxPackExport: vi.fn(),
}));

const mockGenerateTaxPack = vi.mocked(generateTaxPack);
const mockGenerateTaxPackExport = vi.mocked(generateTaxPackExport);

function renderPage() {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  });
  return render(
    <QueryClientProvider client={queryClient}>
      <MemoryRouter>
        <TaxPackPage />
      </MemoryRouter>
    </QueryClientProvider>,
  );
}

describe("TaxPackPage", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    URL.createObjectURL = vi.fn(() => "blob:tax-pack");
    URL.revokeObjectURL = vi.fn();
    HTMLAnchorElement.prototype.click = vi.fn();
  });

  it("submits draft generation inputs and displays generated lines", async () => {
    mockGenerateTaxPack.mockResolvedValue({
      id: "pack-1",
      taxYear: 2026,
      jurisdiction: "General",
      baseCurrency: "USD",
      status: "draft",
      createdAt: "2026-05-16T00:00:00Z",
      finalizedAt: null,
      disclaimer: "Data preparation only. Mizan does not provide tax advice.",
      missingDataChecklist: [
        {
          id: "missing-1",
          taxPackId: "pack-1",
          severity: "warning",
          message: "Tax pack line has no source citation.",
          relatedActivityId: "activity-1",
          relatedAssetId: "asset-1",
        },
      ],
      lines: [
        {
          id: "line-1",
          taxPackId: "pack-1",
          category: "dividend",
          assetId: "asset-1",
          activityId: "activity-1",
          amount: "25",
          currency: "USD",
          taxableDate: "2026-01-15",
          sourceCitationId: null,
          notes: null,
        },
      ],
    });

    renderPage();

    fireEvent.change(screen.getByLabelText("Tax year"), { target: { value: "2026" } });
    fireEvent.change(screen.getByLabelText("Base currency"), { target: { value: "usd" } });
    fireEvent.click(screen.getByRole("button", { name: "Generate Tax Pack" }));

    await waitFor(() => {
      expect(mockGenerateTaxPack).toHaveBeenCalledWith({
        taxYear: 2026,
        jurisdiction: "General",
        baseCurrency: "USD",
      });
    });
    expect(await screen.findByText("Draft Summary")).toBeVisible();
    expect(screen.getByText("25 USD")).toBeVisible();
    expect(screen.getByText("Tax pack line has no source citation.")).toBeVisible();
  });

  it("downloads the generated export bundle", async () => {
    mockGenerateTaxPack.mockResolvedValue({
      id: "pack-1",
      taxYear: 2026,
      jurisdiction: "General",
      baseCurrency: "USD",
      status: "draft",
      createdAt: "2026-05-16T00:00:00Z",
      finalizedAt: null,
      disclaimer: "Data preparation only. Mizan does not provide tax advice.",
      missingDataChecklist: [],
      lines: [],
    });
    mockGenerateTaxPackExport.mockResolvedValue({
      fileName: "tax-pack-2026-general-pack-1.zip",
      mimeType: "application/zip",
      bytes: [80, 75, 3, 4],
      manifest: {
        taxPackId: "pack-1",
        files: ["DISCLAIMER.txt"],
        sourceDocuments: [],
        missingSources: ["Line has no source citation"],
        disclaimer: "Data preparation only. Mizan does not provide tax advice.",
      },
    });

    renderPage();

    fireEvent.click(screen.getByRole("button", { name: "Generate Tax Pack" }));
    fireEvent.click(await screen.findByRole("button", { name: "Export ZIP" }));

    await waitFor(() => {
      expect(mockGenerateTaxPackExport).toHaveBeenCalledWith("pack-1");
    });
    expect(URL.createObjectURL).toHaveBeenCalled();
    expect(await screen.findByText(/Export manifest prepared 1 files/)).toBeVisible();
  });

  it("renders an honest empty draft state", async () => {
    mockGenerateTaxPack.mockResolvedValue({
      id: "pack-empty",
      taxYear: 2026,
      jurisdiction: "General",
      baseCurrency: "USD",
      status: "draft",
      createdAt: "2026-05-16T00:00:00Z",
      finalizedAt: null,
      disclaimer: "Data preparation only. Mizan does not provide tax advice.",
      lines: [],
      missingDataChecklist: [
        {
          id: "missing-empty",
          taxPackId: "pack-empty",
          severity: "info",
          message: "No taxable ledger activity was found for this tax year.",
          relatedActivityId: null,
          relatedAssetId: null,
        },
      ],
    });

    renderPage();

    fireEvent.click(screen.getByRole("button", { name: "Generate Tax Pack" }));

    expect(await screen.findByText("No tax pack lines were generated.")).toBeVisible();
    expect(screen.getByText("No taxable ledger activity was found for this tax year.")).toBeVisible();
  });
});
