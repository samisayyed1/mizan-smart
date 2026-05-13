import type { HealthIssue, HealthStatus } from "@/lib/types";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen } from "@testing-library/react";
import React from "react";
import { MemoryRouter } from "react-router-dom";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

vi.mock("@mizan/ui", () => ({
  Badge: ({ children }: { children: React.ReactNode }) => <span>{children}</span>,
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
    CheckCircle: () => <span>CheckCircle</span>,
  },
  Page: ({ children }: { children: React.ReactNode }) => <div>{children}</div>,
  PageContent: ({ children }: { children: React.ReactNode }) => <div>{children}</div>,
  PageHeader: ({ heading, text }: { heading?: string; text?: string }) => (
    <header>
      {heading}
      {text}
    </header>
  ),
  Skeleton: () => <div data-testid="skel" />,
}));

const mockUseHealthStatus = vi.fn();
vi.mock("@/hooks/use-health", () => ({
  useHealthStatus: () => mockUseHealthStatus(),
}));

import InboxPage from "./inbox-page";

function renderInbox() {
  const client = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  });
  return render(
    <QueryClientProvider client={client}>
      <MemoryRouter>
        <InboxPage />
      </MemoryRouter>
    </QueryClientProvider>,
  );
}

function makeIssue(overrides: Partial<HealthIssue> = {}): HealthIssue {
  return {
    id: "issue:1",
    severity: "WARNING",
    category: "PRICE_STALENESS",
    title: "Stale prices",
    message: "Some quotes are older than 7 days.",
    affectedCount: 3,
    dataHash: "h1",
    timestamp: "2026-05-01T00:00:00Z",
    ...overrides,
  };
}

beforeEach(() => {
  mockUseHealthStatus.mockReset();
});

afterEach(() => {
  vi.clearAllMocks();
});

describe("InboxPage", () => {
  it("renders the empty state when there are no health issues", () => {
    const status: HealthStatus = {
      overallSeverity: "INFO",
      issueCounts: {},
      issues: [],
      checkedAt: "2026-05-01T00:00:00Z",
      isStale: false,
    };
    mockUseHealthStatus.mockReturnValue({
      data: status,
      isLoading: false,
    });
    renderInbox();
    expect(screen.getByTestId("inbox-empty")).toBeInTheDocument();
    expect(screen.getByText(/Nothing needs attention/i)).toBeInTheDocument();
  });

  it("renders one row per active health issue, sorted by severity", () => {
    const status: HealthStatus = {
      overallSeverity: "CRITICAL",
      issueCounts: { CRITICAL: 1, WARNING: 1, ERROR: 1 },
      issues: [
        makeIssue({ id: "w", severity: "WARNING", title: "Stale prices" }),
        makeIssue({ id: "c", severity: "CRITICAL", title: "Missing FX" }),
        makeIssue({ id: "e", severity: "ERROR", title: "Bad classification" }),
      ],
      checkedAt: "2026-05-01T00:00:00Z",
      isStale: false,
    };
    mockUseHealthStatus.mockReturnValue({
      data: status,
      isLoading: false,
    });
    renderInbox();
    const rows = screen.getAllByTestId("inbox-item");
    expect(rows).toHaveLength(3);
    // CRITICAL first, then ERROR, then WARNING.
    expect(rows[0]).toHaveAttribute("data-severity", "CRITICAL");
    expect(rows[1]).toHaveAttribute("data-severity", "ERROR");
    expect(rows[2]).toHaveAttribute("data-severity", "WARNING");
  });

  it("renders a loading skeleton while the health query is pending", () => {
    mockUseHealthStatus.mockReturnValue({
      data: undefined,
      isLoading: true,
    });
    renderInbox();
    expect(screen.getByTestId("inbox-loading")).toBeInTheDocument();
  });
});
