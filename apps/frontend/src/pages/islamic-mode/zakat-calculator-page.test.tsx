import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { MemoryRouter } from "react-router-dom";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { calculateZakatSnapshot } from "@/adapters";
import { useSettingsContext } from "@/lib/settings-provider";
import ZakatCalculatorPage from "./zakat-calculator-page";

vi.mock("@/adapters", () => ({
  calculateZakatSnapshot: vi.fn(),
}));

vi.mock("@/lib/settings-provider", () => ({
  useSettingsContext: vi.fn(),
}));

const mockUseSettingsContext = vi.mocked(useSettingsContext);
const mockCalculateZakatSnapshot = vi.mocked(calculateZakatSnapshot);

function settings(shariahModeEnabled: boolean) {
  return {
    theme: "light",
    font: "font-mono",
    baseCurrency: "USD",
    timezone: "UTC",
    instanceId: "test-instance",
    onboardingCompleted: true,
    autoUpdateCheckEnabled: true,
    menuBarVisible: true,
    syncEnabled: true,
    shariahModeEnabled,
  };
}

function renderPage() {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  });
  return render(
    <QueryClientProvider client={queryClient}>
      <MemoryRouter>
        <ZakatCalculatorPage />
      </MemoryRouter>
    </QueryClientProvider>,
  );
}

describe("ZakatCalculatorPage", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("blocks the calculator when Islamic mode is disabled", () => {
    mockUseSettingsContext.mockReturnValue({
      settings: settings(false),
      isLoading: false,
      isError: false,
      updateBaseCurrency: vi.fn(),
      updateSettings: vi.fn(),
      refetch: vi.fn(),
      accountsGrouped: true,
      setAccountsGrouped: vi.fn(),
    });

    renderPage();

    expect(screen.getByText("Islamic finance tools are disabled for this profile.")).toBeVisible();
    expect(screen.queryByText("Review Lines")).not.toBeInTheDocument();
    expect(mockCalculateZakatSnapshot).not.toHaveBeenCalled();
  });

  it("submits user-selected lines with manual nisab and displays the summary", async () => {
    mockUseSettingsContext.mockReturnValue({
      settings: settings(true),
      isLoading: false,
      isError: false,
      updateBaseCurrency: vi.fn(),
      updateSettings: vi.fn(),
      refetch: vi.fn(),
      accountsGrouped: true,
      setAccountsGrouped: vi.fn(),
    });
    mockCalculateZakatSnapshot.mockResolvedValue({
      id: "zakat-1",
      snapshotDate: "2026-05-15",
      baseCurrency: "USD",
      totalZakatableAssets: "10000",
      deductibleLiabilities: "2000",
      netZakatableWealth: "8000",
      nisabValue: "5000",
      zakatDue: "200",
      notes: "Annual review",
      createdAt: "2026-05-15T00:00:00Z",
      lines: [
        {
          id: "line-1",
          snapshotId: "zakat-1",
          assetId: "asset-1",
          category: "short_term_asset",
          amount: "10000",
          included: true,
          explanation: "Included latest USD valuation dated 2026-05-14 for asset asset-1",
          sourceCitationId: null,
        },
      ],
    });

    renderPage();

    fireEvent.change(screen.getByPlaceholderText("asset id with stored valuation"), {
      target: { value: "asset-1" },
    });
    fireEvent.change(screen.getByPlaceholderText("how this line was determined"), {
      target: { value: "Use latest market value" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Continue" }));
    fireEvent.change(screen.getByPlaceholderText("required"), {
      target: { value: "5000" },
    });
    fireEvent.change(screen.getByPlaceholderText("snapshot notes"), {
      target: { value: "Annual review" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Calculate Snapshot" }));

    await waitFor(() => {
      expect(mockCalculateZakatSnapshot).toHaveBeenCalledWith({
        snapshotDate: expect.any(String) as string,
        baseCurrency: "USD",
        nisabValue: "5000",
        notes: "Annual review",
        lines: [
          {
            assetId: "asset-1",
            category: "short_term_asset",
            amount: null,
            included: true,
            explanation: "Use latest market value",
            sourceCitationId: null,
          },
        ],
      });
    });
    expect(await screen.findByText("Zakat due")).toBeVisible();
    expect(screen.getByText("200")).toBeVisible();
    expect(screen.getByText(/Included latest USD valuation/)).toBeVisible();
  });
});
