import type { InboxItem } from "@/adapters";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import React from "react";
import { MemoryRouter } from "react-router-dom";
import { beforeEach, describe, expect, it, vi } from "vitest";

const listWealthInboxItemsMock = vi.fn<() => Promise<InboxItem[]>>();

vi.mock("@/adapters", () => ({
  listWealthInboxItems: () => listWealthInboxItemsMock(),
}));

vi.mock("@mizan/ui", () => ({
  Badge: ({ children }: { children: React.ReactNode }) => <span>{children}</span>,
  Button: ({
    children,
    asChild,
  }: {
    children: React.ReactNode;
    asChild?: boolean;
    [key: string]: unknown;
  }) => {
    if (asChild && React.isValidElement(children)) {
      return children;
    }
    return <button type="button">{children}</button>;
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

import InboxPage from "./inbox-page";

function renderInbox() {
  return render(
    <MemoryRouter>
      <InboxPage />
    </MemoryRouter>,
  );
}

function item(overrides: Partial<InboxItem>): InboxItem {
  return {
    id: "alert:1",
    itemType: "security",
    title: "Missing FX rate",
    description: "A deterministic alert needs review.",
    severity: "warning",
    dueDate: null,
    sourceEntityType: "alert",
    sourceEntityId: "alert-1",
    actionRoute: "/health",
    status: "active",
    createdAt: "2026-05-01T00:00:00Z",
    ...overrides,
  };
}

beforeEach(() => {
  listWealthInboxItemsMock.mockReset();
});

describe("InboxPage", () => {
  it("renders the empty state when no real inbox items exist", async () => {
    listWealthInboxItemsMock.mockResolvedValue([]);

    renderInbox();

    expect(await screen.findByTestId("inbox-empty")).toBeInTheDocument();
    expect(screen.getByText("Nothing needs attention")).toBeInTheDocument();
  });

  it("renders alerts and stale valuation tasks from the normalized inbox model", async () => {
    listWealthInboxItemsMock.mockResolvedValue([
      item({ id: "alert:1", itemType: "security", title: "Missing FX rate" }),
      item({
        id: "valuation:asset-1",
        itemType: "valuation",
        title: "Update value for Primary residence",
        actionRoute: "/holdings/update-values",
      }),
    ]);

    renderInbox();

    expect(await screen.findByText("Missing FX rate")).toBeInTheDocument();
    expect(screen.getByText("Update value for Primary residence")).toBeInTheDocument();
  });

  it("sorts critical first by default and can switch to newest", async () => {
    const user = userEvent.setup();
    listWealthInboxItemsMock.mockResolvedValue([
      item({
        id: "warning",
        title: "Older warning",
        severity: "warning",
        createdAt: "2026-05-01T00:00:00Z",
      }),
      item({
        id: "critical",
        title: "Older critical",
        severity: "critical",
        createdAt: "2026-04-01T00:00:00Z",
      }),
      item({
        id: "newest",
        title: "Newest info",
        severity: "info",
        createdAt: "2026-05-14T00:00:00Z",
      }),
    ]);

    renderInbox();

    let rows = await screen.findAllByTestId("inbox-item");
    expect(rows[0]).toHaveTextContent("Older critical");

    await user.selectOptions(screen.getByLabelText(/Sort/i), "newest");
    rows = await screen.findAllByTestId("inbox-item");
    expect(rows[0]).toHaveTextContent("Newest info");
  });

  it("filters by item type", async () => {
    const user = userEvent.setup();
    listWealthInboxItemsMock.mockResolvedValue([
      item({ id: "security", itemType: "security", title: "Security alert" }),
      item({ id: "valuation", itemType: "valuation", title: "Valuation task" }),
    ]);

    renderInbox();

    expect(await screen.findByText("Security alert")).toBeInTheDocument();
    await user.selectOptions(screen.getByLabelText(/Filter/i), "valuation");

    expect(screen.queryByText("Security alert")).not.toBeInTheDocument();
    expect(screen.getByText("Valuation task")).toBeInTheDocument();
  });

  it("uses backend-provided action routes", async () => {
    listWealthInboxItemsMock.mockResolvedValue([
      item({
        id: "valuation",
        itemType: "valuation",
        title: "Valuation task",
        actionRoute: "/holdings/update-values",
      }),
    ]);

    renderInbox();

    await waitFor(() =>
      expect(screen.getByTestId("inbox-action")).toHaveAttribute(
        "href",
        "/holdings/update-values",
      ),
    );
  });
});
