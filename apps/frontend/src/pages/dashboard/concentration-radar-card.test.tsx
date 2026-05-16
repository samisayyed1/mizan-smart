import { render, screen } from "@testing-library/react";
import React from "react";
import { describe, expect, it, vi } from "vitest";

import { ConcentrationRadarCard } from "./concentration-radar-card";

vi.mock("@mizan/ui/components/ui/card", () => ({
  Card: ({ children, ...props }: { children: React.ReactNode; [k: string]: unknown }) => (
    <section {...props}>{children}</section>
  ),
  CardContent: ({ children }: { children: React.ReactNode }) => <div>{children}</div>,
}));

describe("ConcentrationRadarCard", () => {
  it("renders plain-English findings without advice", () => {
    render(
      <ConcentrationRadarCard
        summary={{
          asOfDate: "2026-05-16",
          baseCurrency: "USD",
          totalWealth: "1000",
          exposures: [],
          findings: [
            {
              dimension: "asset",
              label: "Asset",
              message: "42% of recorded wealth is in ACME.",
              amount: "420",
              currency: "USD",
              percent: "42",
              thresholdPercent: "25",
            },
          ],
          emptyState: false,
          islamicModeEnabled: false,
          taxonomyState: "missing",
        }}
      />,
    );

    expect(screen.getByTestId("concentration-radar-card")).toBeInTheDocument();
    expect(screen.getByText("42% of recorded wealth is in ACME.")).toBeInTheDocument();
    expect(screen.getByText(/taxonomy exposure is unavailable/i)).toBeInTheDocument();
    expect(screen.queryByText(/buy|sell|rebalance/i)).not.toBeInTheDocument();
  });

  it("does not render for an honest empty state", () => {
    const { container } = render(
      <ConcentrationRadarCard
        summary={{
          asOfDate: "2026-05-16",
          baseCurrency: "USD",
          totalWealth: "0",
          exposures: [],
          findings: [],
          emptyState: true,
          islamicModeEnabled: false,
          taxonomyState: "missing",
        }}
      />,
    );

    expect(container).toBeEmptyDOMElement();
  });
});
