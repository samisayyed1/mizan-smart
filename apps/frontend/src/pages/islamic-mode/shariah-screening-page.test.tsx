import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen } from "@testing-library/react";
import { MemoryRouter } from "react-router-dom";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { listShariahScreeningProfiles } from "@/adapters";
import { useSettingsContext } from "@/lib/settings-provider";
import ShariahScreeningPage from "./shariah-screening-page";

vi.mock("@/adapters", () => ({
  listShariahScreeningProfiles: vi.fn(),
}));

vi.mock("@/lib/settings-provider", () => ({
  useSettingsContext: vi.fn(),
}));

const mockUseSettingsContext = vi.mocked(useSettingsContext);
const mockListProfiles = vi.mocked(listShariahScreeningProfiles);

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
    expect(screen.queryByText("Zakat")).not.toBeInTheDocument();
    expect(screen.queryByText("Purification")).not.toBeInTheDocument();
    expect(mockListProfiles).not.toHaveBeenCalled();
  });

  it("shows screening, zakat, and purification sections when mode is enabled", async () => {
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

    expect(await screen.findByText("Default Screening Profile")).toBeVisible();
    expect(screen.getByText("Zakat")).toBeVisible();
    expect(screen.getByText("Purification")).toBeVisible();
  });
});
