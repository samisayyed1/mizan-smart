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
    FileText: () => <span>FileText</span>,
  },
  Page: ({ children }: { children: React.ReactNode }) => <div>{children}</div>,
  PageContent: ({ children }: { children: React.ReactNode }) => <div>{children}</div>,
  PageHeader: ({ heading }: { heading?: string }) => <header>{heading}</header>,
}));

import DocumentsPage from "./documents-page";

describe("DocumentsPage", () => {
  it("renders an honest empty state explaining Phase 2 will deliver the vault", () => {
    render(
      <MemoryRouter>
        <DocumentsPage />
      </MemoryRouter>,
    );
    expect(screen.getByTestId("documents-empty")).toBeInTheDocument();
    expect(screen.getByText(/Document Vault is being built/i)).toBeInTheDocument();
  });

  it("offers links to the closest existing surfaces (Activities, Holdings)", () => {
    render(
      <MemoryRouter>
        <DocumentsPage />
      </MemoryRouter>,
    );
    const activities = screen.getByRole("link", { name: /Open Activities/i });
    expect(activities).toHaveAttribute("href", "/activities");
    const holdings = screen.getByRole("link", { name: /View Holdings/i });
    expect(holdings).toHaveAttribute("href", "/holdings");
  });
});
