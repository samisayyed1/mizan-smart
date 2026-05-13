import { renderHook } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { useNavigation } from "./app-navigation";

// The addons runtime adds extra items dynamically. Stub it so the test
// only asserts on the static senior-friendly primary navigation contract.
vi.mock("@/addons/addons-runtime-context", () => ({
  getDynamicNavItems: () => [],
  subscribeToNavigationUpdates: () => () => undefined,
}));

describe("useNavigation (mizan-smart senior-friendly nav)", () => {
  it("exposes exactly the six primary sections in canonical order", () => {
    const { result } = renderHook(() => useNavigation());
    expect(result.current.primary.map((item) => item.title)).toEqual([
      "Home",
      "Portfolio",
      "Documents",
      "Reports",
      "Inbox",
      "Settings",
    ]);
  });

  it("routes the primary sections to their canonical paths", () => {
    const { result } = renderHook(() => useNavigation());
    const map = Object.fromEntries(
      result.current.primary.map((item) => [item.title, item.href] as const),
    );
    expect(map).toEqual({
      Home: "/dashboard",
      Portfolio: "/holdings",
      Documents: "/documents",
      Reports: "/reports",
      Inbox: "/inbox",
      Settings: "/settings",
    });
  });

  it("keeps advanced pages reachable through secondary navigation", () => {
    const { result } = renderHook(() => useNavigation());
    const secondaryTitles = (result.current.secondary ?? []).map((item) => item.title);
    // Each advanced surface that used to be in the primary nav must remain
    // navigable — the spec forbids hiding existing functionality.
    for (const title of ["Activities", "Insights", "Goals", "Assistant", "Connect"]) {
      expect(secondaryTitles).toContain(title);
    }
  });

  it("gives every primary item a plain-English label", () => {
    const { result } = renderHook(() => useNavigation());
    for (const item of result.current.primary) {
      expect(item.title.length).toBeGreaterThan(0);
      expect(item.title).toMatch(/^[A-Z][a-z]+$/);
    }
  });
});
