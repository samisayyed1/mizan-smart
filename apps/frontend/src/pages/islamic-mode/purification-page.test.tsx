import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { MemoryRouter } from "react-router-dom";
import { beforeEach, describe, expect, it, vi } from "vitest";

import {
  getPurificationPeriodSummary,
  markPurificationPaid,
  upsertPurificationEntry,
} from "@/adapters";
import { useSettingsContext } from "@/lib/settings-provider";
import PurificationPage from "./purification-page";

vi.mock("@/adapters", () => ({
  getPurificationPeriodSummary: vi.fn(),
  markPurificationPaid: vi.fn(),
  upsertPurificationEntry: vi.fn(),
}));

vi.mock("@/lib/settings-provider", () => ({
  useSettingsContext: vi.fn(),
}));

const mockUseSettingsContext = vi.mocked(useSettingsContext);
const mockSummary = vi.mocked(getPurificationPeriodSummary);
const mockUpsert = vi.mocked(upsertPurificationEntry);
const mockMarkPaid = vi.mocked(markPurificationPaid);

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
        <PurificationPage />
      </MemoryRouter>
    </QueryClientProvider>,
  );
}

describe("PurificationPage", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    URL.createObjectURL = vi.fn(() => "blob:purification-summary");
    URL.revokeObjectURL = vi.fn();
    vi.spyOn(HTMLAnchorElement.prototype, "click").mockImplementation(() => undefined);
  });

  it("blocks purification while Islamic mode is disabled", () => {
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
    expect(screen.queryByText("Add Review Entry")).not.toBeInTheDocument();
    expect(mockSummary).not.toHaveBeenCalled();
  });

  it("saves a review entry and marks an existing entry paid", async () => {
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
    mockSummary.mockResolvedValue({
      periodStart: "2026-01-01",
      periodEnd: "2026-12-31",
      totalCalculated: "20",
      totalPaid: "0",
      entries: [
        {
          id: "purification-1",
          assetId: "asset-1",
          periodStart: "2026-01-01",
          periodEnd: "2026-12-31",
          totalImpureIncome: null,
          outstandingShares: null,
          userShares: null,
          dividendReceived: "400",
          impureIncomeRatio: "0.05",
          purificationAmount: "20",
          calculationMethod: "dividend_ratio",
          status: "calculated",
          sourceCitationId: null,
          notes: "Dividend ratio",
          createdAt: "2026-05-15T00:00:00Z",
          updatedAt: "2026-05-15T00:00:00Z",
        },
      ],
    });
    mockUpsert.mockResolvedValue({
      id: "purification-2",
      assetId: "asset-2",
      periodStart: "2026-01-01",
      periodEnd: "2026-12-31",
      totalImpureIncome: "1000",
      outstandingShares: "10000",
      userShares: "50",
      dividendReceived: null,
      impureIncomeRatio: null,
      purificationAmount: "5",
      calculationMethod: "impure_income_per_share",
      status: "calculated",
      sourceCitationId: null,
      notes: "Per-share method",
      createdAt: "2026-05-15T00:00:00Z",
      updatedAt: "2026-05-15T00:00:00Z",
    });
    mockMarkPaid.mockResolvedValue({
      id: "purification-1",
      assetId: "asset-1",
      periodStart: "2026-01-01",
      periodEnd: "2026-12-31",
      purificationAmount: "20",
      calculationMethod: "dividend_ratio",
      status: "paid",
      createdAt: "2026-05-15T00:00:00Z",
      updatedAt: "2026-05-15T00:00:00Z",
    });

    renderPage();

    expect(await screen.findByText("Purification Summary")).toBeVisible();
    expect(await screen.findByText("dividend ratio")).toBeVisible();
    fireEvent.click(screen.getByRole("button", { name: "Export summary" }));
    expect(URL.createObjectURL).toHaveBeenCalledTimes(1);
    fireEvent.click(screen.getByRole("button", { name: "Mark paid" }));
    await waitFor(() => expect(mockMarkPaid).toHaveBeenCalledWith("purification-1"));

    fireEvent.change(screen.getByPlaceholderText("asset id"), { target: { value: "asset-2" } });
    fireEvent.change(screen.getByPlaceholderText("total impure income"), {
      target: { value: "1000" },
    });
    fireEvent.change(screen.getByPlaceholderText("outstanding shares"), {
      target: { value: "10000" },
    });
    fireEvent.change(screen.getByPlaceholderText("user shares"), { target: { value: "50" } });
    fireEvent.click(screen.getByRole("button", { name: "Save Entry" }));

    await waitFor(() => {
      expect(mockUpsert).toHaveBeenCalledWith({
        assetId: "asset-2",
        periodStart: "2026-01-01",
        periodEnd: "2026-12-31",
        totalImpureIncome: "1000",
        outstandingShares: "10000",
        userShares: "50",
        dividendReceived: null,
        impureIncomeRatio: null,
        status: null,
        sourceCitationId: null,
        notes: null,
      });
    });
  });
});
