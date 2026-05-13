import { render, screen } from "@testing-library/react";
import React from "react";
import { MemoryRouter } from "react-router-dom";
import { describe, expect, it, vi } from "vitest";

vi.mock("@mizan/ui", () => ({
  Button: ({
    children,
    asChild,
    ...props
  }: {
    children: React.ReactNode;
    asChild?: boolean;
    [key: string]: unknown;
  }) => {
    if (asChild && React.isValidElement(children)) {
      return React.cloneElement(children, props);
    }
    return (
      <button type="button" {...props}>
        {children}
      </button>
    );
  },
  Icons: {
    TrendingUp: () => <span>TrendingUp</span>,
    HandCoins: () => <span>HandCoins</span>,
    PieChart: () => <span>PieChart</span>,
    ShieldCheck: () => <span>ShieldCheck</span>,
  },
  Page: ({ children }: { children: React.ReactNode }) => <div>{children}</div>,
  PageContent: ({ children }: { children: React.ReactNode }) => <div>{children}</div>,
  PageHeader: ({ heading }: { heading?: string }) => <header>{heading}</header>,
}));

import ReportsPage from "./reports-page";

describe("ReportsPage", () => {
  it("renders the deterministic-builder notice", () => {
    render(
      <MemoryRouter>
        <ReportsPage />
      </MemoryRouter>,
    );
    expect(screen.getByTestId("reports-builder-notice")).toBeInTheDocument();
    expect(screen.getByText(/Deterministic Report Builder/i)).toBeInTheDocument();
  });

  it("lists the four existing real report-like surfaces with correct hrefs", () => {
    render(
      <MemoryRouter>
        <ReportsPage />
      </MemoryRouter>,
    );
    const perf = screen.getByRole("link", { name: /Performance/i });
    expect(perf).toHaveAttribute("href", "/performance");
    const income = screen.getByRole("link", { name: /Income/i });
    expect(income).toHaveAttribute("href", "/income");
    const breakdown = screen.getByRole("link", { name: /Holdings breakdown/i });
    expect(breakdown).toHaveAttribute("href", "/insights");
    const health = screen.getByRole("link", { name: /Data health/i });
    expect(health).toHaveAttribute("href", "/health");
  });
});
