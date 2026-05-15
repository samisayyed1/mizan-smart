import { render, screen } from "@testing-library/react";
import { MemoryRouter } from "react-router-dom";
import { describe, expect, it, vi } from "vitest";

import { useSettingsContext } from "@/lib/settings-provider";
import { IslamicModeSettings } from "./islamic-mode-settings";

vi.mock("@/lib/settings-provider", () => ({
  useSettingsContext: vi.fn(),
}));

const mockUseSettingsContext = vi.mocked(useSettingsContext);

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

function renderComponent() {
  return render(
    <MemoryRouter>
      <IslamicModeSettings />
    </MemoryRouter>,
  );
}

describe("IslamicModeSettings", () => {
  it("does not show the screening link while disabled", () => {
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

    renderComponent();

    expect(screen.getByLabelText("Enable Islamic finance tools")).not.toBeChecked();
    expect(screen.queryByText("Open screening page")).not.toBeInTheDocument();
    expect(screen.queryByText("Open Zakat calculator")).not.toBeInTheDocument();
    expect(screen.queryByText("Open purification tracker")).not.toBeInTheDocument();
  });

  it("shows the screening link while enabled", () => {
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

    renderComponent();

    expect(screen.getByLabelText("Enable Islamic finance tools")).toBeChecked();
    expect(screen.getByText("Open screening page")).toHaveAttribute("href", "/shariah-screening");
    expect(screen.getByText("Open Zakat calculator")).toHaveAttribute("href", "/zakat");
    expect(screen.getByText("Open purification tracker")).toHaveAttribute("href", "/purification");
  });
});
