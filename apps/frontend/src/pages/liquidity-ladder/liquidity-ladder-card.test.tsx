import type { LiquidityLadderReport } from "@/adapters";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen } from "@testing-library/react";
import React from "react";
import { MemoryRouter } from "react-router-dom";
import { beforeEach, describe, expect, it, vi } from "vitest";

const getLiquidityLadderMock = vi.fn<() => Promise<LiquidityLadderReport>>();

vi.mock("@/adapters", () => ({
  getLiquidityLadder: () => getLiquidityLadderMock(),
}));

vi.mock("@mizan/ui", () => ({
  Icons: {
    ArrowRight: ({ className }: { className?: string }) => <span className={className}>Arrow</span>,
  },
}));

vi.mock("@mizan/ui/components/ui/card", () => ({
  Card: ({ children, ...props }: { children: React.ReactNode; [k: string]: unknown }) => (
    <section {...props}>{children}</section>
  ),
  CardHeader: ({ children }: { children: React.ReactNode }) => <header>{children}</header>,
  CardContent: ({ children }: { children: React.ReactNode }) => <div>{children}</div>,
}));

import { LiquidityLadderCard } from "./liquidity-ladder-card";

function renderCard() {
  const client = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  });
  return render(
    <QueryClientProvider client={client}>
      <MemoryRouter>
        <LiquidityLadderCard />
      </MemoryRouter>
    </QueryClientProvider>,
  );
}

beforeEach(() => {
  getLiquidityLadderMock.mockReset();
});

describe("LiquidityLadderCard", () => {
  it("renders an honest empty state", async () => {
    getLiquidityLadderMock.mockResolvedValueOnce({
      asOf: "2026-05-15",
      views: [{ window: "next_30_days", startDate: "2026-05-15", endDate: "2026-06-14", currencyGroups: [], warnings: [] }],
    });

    renderCard();

    expect(await screen.findByTestId("liquidity-ladder-empty")).toHaveTextContent(
      "No cash balances or dated cashflows",
    );
  });

  it("shows expected and confirmed labels separately", async () => {
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
              confirmedIncoming: "0",
              expectedIncoming: "25",
              confirmedOutgoing: "0",
              expectedOutgoing: "200",
              netConfirmed: "1000",
              netExpected: "825",
              items: [
                {
                  id: "cash",
                  date: "2026-05-15",
                  currency: "USD",
                  amount: "1000",
                  direction: "balance",
                  confidence: "confirmed",
                  itemType: "cash_balance",
                  label: "Available cash balance",
                },
                {
                  id: "coupon",
                  date: "2026-05-30",
                  currency: "USD",
                  amount: "25",
                  direction: "incoming",
                  confidence: "expected",
                  itemType: "fixed_income_cashflow",
                  label: "Fixed income coupon",
                },
              ],
            },
          ],
          warnings: [],
        },
      ],
    });

    renderCard();

    expect(await screen.findByText(/Expected after scheduled items/)).toBeInTheDocument();
    expect(screen.getByText(/Expected and confirmed cashflows/)).toBeInTheDocument();
    expect(screen.getByTestId("liquidity-ladder-open")).toHaveAttribute("href", "/liquidity-ladder");
  });
});
