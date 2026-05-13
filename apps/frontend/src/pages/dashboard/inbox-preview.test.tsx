import type { HealthIssue, HealthStatus } from "@/lib/types";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen } from "@testing-library/react";
import React from "react";
import { MemoryRouter } from "react-router-dom";
import { beforeEach, describe, expect, it, vi } from "vitest";

vi.mock("@mizan/ui", () => ({
  Icons: {
    CheckCircle: () => <span>CheckCircle</span>,
  },
}));

vi.mock("@mizan/ui/components/ui/card", () => ({
  Card: ({ children, ...props }: { children: React.ReactNode; [k: string]: unknown }) => (
    <section {...props}>{children}</section>
  ),
  CardHeader: ({ children }: { children: React.ReactNode }) => <header>{children}</header>,
  CardContent: ({ children }: { children: React.ReactNode }) => <div>{children}</div>,
}));

const mockUseHealthStatus = vi.fn();
vi.mock("@/hooks/use-health", () => ({
  useHealthStatus: () => mockUseHealthStatus(),
}));

import { InboxPreview } from "./inbox-preview";

function renderPreview() {
  const client = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  });
  return render(
    <QueryClientProvider client={client}>
      <MemoryRouter>
        <InboxPreview />
      </MemoryRouter>
    </QueryClientProvider>,
  );
}

function makeIssue(overrides: Partial<HealthIssue> = {}): HealthIssue {
  return {
    id: "h:1",
    severity: "WARNING",
    category: "PRICE_STALENESS",
    title: "Stale price",
    message: "Quote older than 7 days",
    affectedCount: 1,
    dataHash: "x",
    timestamp: "2026-05-01T00:00:00Z",
    ...overrides,
  };
}

beforeEach(() => {
  mockUseHealthStatus.mockReset();
});

describe("InboxPreview", () => {
  it("renders the empty state when there are no health issues", () => {
    mockUseHealthStatus.mockReturnValue({
      data: {
        overallSeverity: "INFO",
        issueCounts: {},
        issues: [],
        checkedAt: "2026-05-01T00:00:00Z",
        isStale: false,
      } satisfies HealthStatus,
      isLoading: false,
    });
    renderPreview();
    expect(screen.getByTestId("inbox-preview-empty")).toBeInTheDocument();
  });

  it("renders at most 3 rows and indicates how many are hidden", () => {
    mockUseHealthStatus.mockReturnValue({
      data: {
        overallSeverity: "CRITICAL",
        issueCounts: { CRITICAL: 1, ERROR: 1, WARNING: 3 },
        issues: [
          makeIssue({ id: "w1", severity: "WARNING" }),
          makeIssue({ id: "c", severity: "CRITICAL", title: "Missing FX" }),
          makeIssue({ id: "w2", severity: "WARNING" }),
          makeIssue({ id: "e", severity: "ERROR", title: "Bad classification" }),
          makeIssue({ id: "w3", severity: "WARNING" }),
        ],
        checkedAt: "2026-05-01T00:00:00Z",
        isStale: false,
      } satisfies HealthStatus,
      isLoading: false,
    });
    renderPreview();
    const rows = screen.getAllByTestId("inbox-preview-item");
    expect(rows).toHaveLength(3);
    // Severity ordering: CRITICAL, ERROR, then WARNING
    expect(rows[0]).toHaveAttribute("data-severity", "CRITICAL");
    expect(rows[1]).toHaveAttribute("data-severity", "ERROR");
    expect(rows[2]).toHaveAttribute("data-severity", "WARNING");
    expect(screen.getByText(/\+ 2 more in inbox/)).toBeInTheDocument();
  });

  it("links to the full inbox view", () => {
    mockUseHealthStatus.mockReturnValue({
      data: {
        overallSeverity: "INFO",
        issueCounts: {},
        issues: [],
        checkedAt: "2026-05-01T00:00:00Z",
        isStale: false,
      } satisfies HealthStatus,
      isLoading: false,
    });
    renderPreview();
    expect(screen.getByTestId("inbox-preview-open")).toHaveAttribute("href", "/inbox");
  });
});
