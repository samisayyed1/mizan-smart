import { render, screen } from "@testing-library/react";
import React from "react";
import { MemoryRouter } from "react-router-dom";
import { describe, expect, it, vi } from "vitest";

vi.mock("@mizan/ui", () => ({
  Icons: {
    Plus: () => <span>Plus</span>,
    RefreshCw: () => <span>RefreshCw</span>,
    FileText: () => <span>FileText</span>,
    Inbox: () => <span>Inbox</span>,
    PieChart: () => <span>PieChart</span>,
  },
}));

vi.mock("@mizan/ui/components/ui/card", () => ({
  Card: ({ children, ...props }: { children: React.ReactNode; [k: string]: unknown }) => (
    <section {...props}>{children}</section>
  ),
  CardHeader: ({ children }: { children: React.ReactNode }) => <header>{children}</header>,
  CardContent: ({ children }: { children: React.ReactNode }) => <div>{children}</div>,
}));

import { QuickActions } from "./quick-actions";

describe("QuickActions (Home dashboard)", () => {
  it("renders five senior-friendly actions in canonical order", () => {
    render(
      <MemoryRouter>
        <QuickActions />
      </MemoryRouter>,
    );
    const card = screen.getByTestId("quick-actions");
    const titles = Array.from(card.querySelectorAll("a")).map(
      (a) => a.textContent?.split("\n")[0]?.trim() ?? "",
    );
    // The action titles are followed by a description so we match the first
    // line. The order must be: Add asset, Update values, Upload document,
    // Review inbox, Generate report.
    expect(titles).toHaveLength(5);
    expect(titles[0]).toMatch(/Add asset/i);
    expect(titles[1]).toMatch(/Update values/i);
    expect(titles[2]).toMatch(/Upload document/i);
    expect(titles[3]).toMatch(/Review inbox/i);
    expect(titles[4]).toMatch(/Generate report/i);
  });

  it("routes each action to a real, existing route", () => {
    render(
      <MemoryRouter>
        <QuickActions />
      </MemoryRouter>,
    );
    expect(screen.getByTestId("quick-action-plus")).toHaveAttribute("href", "/holdings");
    expect(screen.getByTestId("quick-action-refreshcw")).toHaveAttribute("href", "/holdings");
    expect(screen.getByTestId("quick-action-filetext")).toHaveAttribute("href", "/documents");
    expect(screen.getByTestId("quick-action-inbox")).toHaveAttribute("href", "/inbox");
    expect(screen.getByTestId("quick-action-piechart")).toHaveAttribute("href", "/reports");
  });
});
