import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { MemoryRouter } from "react-router-dom";
import { beforeEach, describe, expect, it, vi } from "vitest";

import {
  getAssetShariahScreening,
  listShariahScreeningAudit,
  listShariahScreeningProfiles,
  upsertAssetShariahScreening,
} from "@/adapters";
import { useSettingsContext } from "@/lib/settings-provider";
import ShariahScreeningPage from "./shariah-screening-page";

vi.mock("@/adapters", () => ({
  getAssetShariahScreening: vi.fn(),
  listShariahScreeningAudit: vi.fn(),
  listShariahScreeningProfiles: vi.fn(),
  upsertAssetShariahScreening: vi.fn(),
}));

vi.mock("@/lib/settings-provider", () => ({
  useSettingsContext: vi.fn(),
}));

const mockUseSettingsContext = vi.mocked(useSettingsContext);
const mockListProfiles = vi.mocked(listShariahScreeningProfiles);
const mockGetScreening = vi.mocked(getAssetShariahScreening);
const mockListAudit = vi.mocked(listShariahScreeningAudit);
const mockUpsertScreening = vi.mocked(upsertAssetShariahScreening);

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
        <ShariahScreeningPage />
      </MemoryRouter>
    </QueryClientProvider>,
  );
}

describe("ShariahScreeningPage", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("hides screening, zakat, and purification sections when mode is disabled", () => {
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
    expect(screen.queryByText("Asset Screening Review")).not.toBeInTheDocument();
    expect(mockListProfiles).not.toHaveBeenCalled();
  });

  it("shows profiles and the auditable review form when mode is enabled", async () => {
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
    mockListProfiles.mockResolvedValue([
      {
        id: "system_default_shariah_screening_profile",
        name: "Default screening profile",
        debtThreshold: "0.30",
        liquidAssetsThreshold: "0.30",
        impureIncomeThreshold: "0.05",
        isDefault: true,
        createdAt: "2026-05-15T00:00:00Z",
        updatedAt: "2026-05-15T00:00:00Z",
      },
    ]);

    renderPage();

    expect(await screen.findByText("Screening Profiles")).toBeVisible();
    expect(screen.getByText("Asset Screening Review")).toBeVisible();
    expect(screen.getByText("Review Status")).toBeVisible();
    expect(screen.getByText("No screening review has been saved yet.")).toBeVisible();
  });

  it("saves user-entered ratios and displays audit history", async () => {
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
    mockListProfiles.mockResolvedValue([
      {
        id: "system_default_shariah_screening_profile",
        name: "Default screening profile",
        debtThreshold: "0.30",
        liquidAssetsThreshold: "0.30",
        impureIncomeThreshold: "0.05",
        isDefault: true,
        createdAt: "2026-05-15T00:00:00Z",
        updatedAt: "2026-05-15T00:00:00Z",
      },
    ]);
    mockUpsertScreening.mockResolvedValue({
      id: "screening-1",
      assetId: "asset-1",
      profileId: "system_default_shariah_screening_profile",
      status: "compliant",
      debtRatio: "0.1",
      liquidAssetsRatio: "0.2",
      impureIncomeRatio: "0.01",
      sourceCitationId: "citation-1",
      manualOverrideReason: null,
      reviewedAt: "2026-05-15T00:00:00Z",
      notes: "Reviewed from document-backed ratios",
      createdAt: "2026-05-15T00:00:00Z",
      updatedAt: "2026-05-15T00:00:00Z",
    });
    mockGetScreening.mockResolvedValue({
      id: "screening-1",
      assetId: "asset-1",
      profileId: "system_default_shariah_screening_profile",
      status: "compliant",
      debtRatio: "0.1",
      liquidAssetsRatio: "0.2",
      impureIncomeRatio: "0.01",
      sourceCitationId: "citation-1",
      manualOverrideReason: null,
      reviewedAt: "2026-05-15T00:00:00Z",
      notes: "Reviewed from document-backed ratios",
      createdAt: "2026-05-15T00:00:00Z",
      updatedAt: "2026-05-15T00:00:00Z",
    });
    mockListAudit.mockResolvedValue([
      {
        id: "audit-1",
        screeningId: "screening-1",
        assetId: "asset-1",
        profileId: "system_default_shariah_screening_profile",
        previousStatus: null,
        newStatus: "compliant",
        notes: "Reviewed from document-backed ratios",
        createdAt: "2026-05-15T00:00:00Z",
      },
    ]);

    renderPage();

    fireEvent.change(await screen.findByPlaceholderText("asset id"), {
      target: { value: "asset-1" },
    });
    fireEvent.change(screen.getByPlaceholderText("profile id"), {
      target: { value: "system_default_shariah_screening_profile" },
    });
    fireEvent.change(screen.getByPlaceholderText("0.10"), { target: { value: "0.10" } });
    fireEvent.change(screen.getByPlaceholderText("0.20"), { target: { value: "0.20" } });
    fireEvent.change(screen.getByPlaceholderText("0.01"), { target: { value: "0.01" } });
    fireEvent.change(screen.getByPlaceholderText("optional citation id"), {
      target: { value: "citation-1" },
    });
    fireEvent.change(screen.getByPlaceholderText("screening notes"), {
      target: { value: "Reviewed from document-backed ratios" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Save Review" }));

    await waitFor(() => {
      expect(mockUpsertScreening).toHaveBeenCalledWith({
        assetId: "asset-1",
        profileId: "system_default_shariah_screening_profile",
        ratios: {
          debtRatio: "0.10",
          liquidAssetsRatio: "0.20",
          impureIncomeRatio: "0.01",
        },
        sourceCitationId: "citation-1",
        notes: "Reviewed from document-backed ratios",
        manualOverrideStatus: null,
        manualOverrideReason: null,
      });
    });
    expect(await screen.findByText("Document-backed citation: citation-1")).toBeVisible();
    expect(screen.getByText("not reviewed to compliant")).toBeVisible();
  });
});
