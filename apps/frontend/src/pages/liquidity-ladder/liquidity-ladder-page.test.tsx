import type { LiquidityLadderReport } from "@/adapters";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen } from "@testing-library/react";
import React from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";

const getLiquidityLadderMock = vi.fn<() => Promise<LiquidityLadderReport>>();

vi.mock("@/adapters", () => ({
  getLiquidityLadder: () => getLiquidityLadderMock(),
}));

vi.mock("@mizan/ui", () => ({
  Button: ({ children, ...props }: React.ButtonHTMLAttributes<HTMLButtonElement>) => (
    <button {...props}>{children}</button>
  ),
  Icons: {
    Calendar: ({ className }: { className?: string }) => <span className={className}>Calendar</span>,
  },
  Page: ({ children }: { children: React.ReactNode }) => <main>{children}</main>,
  PageContent: ({ children, className }: { children: React.ReactNode; className?: string }) => (
    <div className={className}>{children}</div>
  ),
  PageHeader: ({ heading }: { heading: string }) => <h1>{heading}</h1>,
}));

vi.mock("@mizan/ui/components/ui/card", () => ({
  Card: ({ children, ...props }: { children: React.ReactNode; [k: string]: unknown }) => (
    <section {...props}>{children}</section>
  ),
  CardHeader: ({ children }: { children: React.ReactNode }) => <header>{children}</header>,
  CardContent: ({ children }: { children: React.ReactNode }) => <div>{children}</div>,
}));

import LiquidityLadderPage from "./liquidity-ladder-page";

function renderPage() {
  const client = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  });
  return render(
    <QueryClientProvider client={client}>
      <LiquidityLadderPage />
    </QueryClientProvider>,
  );
}

beforeEach(() => {
  getLiquidityLadderMock.mockReset();
});

describe("LiquidityLadderPage", () => {
  it("renders empty state copy without invented dividends", async () => {
    getLiquidityLadderMock.mockResolvedValueOnce({
      asOf: "2026-05-15",
      views: [{ window: "next_30_days", startDate: "2026-05-15", endDate: "2026-06-14", currencyGroups: [], warnings: [] }],
    });

    renderPage();

    expect(await screen.findByTestId("liquidity-ladder-empty")).toHaveTextContent(
      "no cash balances or dated cashflows",
    );
    expect(screen.getByText(/not estimated unless they are already recorded/i)).toBeInTheDocument();
  });

  it("renders expected and confirmed cashflow labels in the table", async () => {
    getLiquidityLadderMock.mockResolvedValueOnce({
      asOf: "2026-05-15",
      views: [
        {
          window: "next_30_days",
          startDate: "2026-05-15",
          endDate: "2026-06-14",
          currencyGroups: [
            {
              currency: "USD",
              availableCash: "1000",
              confirmedIncoming: "100",
              expectedIncoming: "25",
              confirmedOutgoing: "0",
              expectedOutgoing: "200",
              netConfirmed: "1100",
              netExpected: "925",
              items: [
                {
                  id: "distribution",
                  date: "2026-05-18",
                  currency: "USD",
                  amount: "100",
                  direction: "incoming",
                  confidence: "confirmed",
                  itemType: "private_distribution",
                  label: "Private distribution",
                },
                {
                  id: "call",
                  date: "2026-05-25",
                  currency: "USD",
                  amount: "200",
                  direction: "outgoing",
                  confidence: "expected",
                  itemType: "private_capital_call",
                  label: "Private capital call",
                },
              ],
            },
          ],
          warnings: ["Future dividends are included only when recorded."],
        },
      ],
    });

    renderPage();

    expect(await screen.findAllByText("Private distribution")).toHaveLength(2);
    expect(screen.getAllByText("Private capital call")).toHaveLength(2);
    expect(screen.getByTestId("confidence-confirmed")).toHaveTextContent("Confirmed");
    expect(screen.getByTestId("confidence-expected")).toHaveTextContent("Expected");
  });
});
