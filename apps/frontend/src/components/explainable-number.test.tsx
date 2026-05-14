import type { DataLineageResponse, GetDataLineageRequest } from "@/adapters";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import React from "react";
import { MemoryRouter } from "react-router-dom";
import { beforeEach, describe, expect, it, vi } from "vitest";

const getDataLineageMock = vi.fn<(request: GetDataLineageRequest) => Promise<DataLineageResponse>>();

vi.mock("@/adapters", () => ({
  getDataLineage: (request: GetDataLineageRequest) => getDataLineageMock(request),
}));

vi.mock("@mizan/ui", () => ({
  Button: ({
    children,
    ...props
  }: React.ButtonHTMLAttributes<HTMLButtonElement> & { children: React.ReactNode }) => (
    <button type="button" {...props}>
      {children}
    </button>
  ),
  Dialog: ({ children, open }: { children: React.ReactNode; open: boolean }) =>
    open ? <div>{children}</div> : null,
  DialogContent: ({ children }: { children: React.ReactNode }) => <section>{children}</section>,
  DialogDescription: ({ children }: { children: React.ReactNode }) => <p>{children}</p>,
  DialogHeader: ({ children }: { children: React.ReactNode }) => <header>{children}</header>,
  DialogTitle: ({ children }: { children: React.ReactNode }) => <h2>{children}</h2>,
  Icons: {
    Info: () => <span>Info</span>,
  },
}));

import { ExplainableNumber } from "./explainable-number";

function lineage(overrides: Partial<DataLineageResponse> = {}): DataLineageResponse {
  return {
    entityType: "valuation",
    entityId: "valuation-1",
    metricType: "valuation",
    displayedValue: "1250.00",
    currency: "USD",
    formulaName: "Valuation",
    formulaDescription: "Stored asset valuation value from the valuations table.",
    inputRows: [
      {
        sourceTable: "valuations",
        sourceId: "valuation-1",
        label: "Villa",
        value: "1250.00",
        currency: "USD",
        asOfDate: "2026-05-14",
        notes: "asset_id=asset-1",
      },
    ],
    sourceCitations: [
      {
        id: "citation-1",
        label: "statement.pdf p.3",
        sourceType: "document",
        sourceId: "doc-1",
        documentId: "doc-1",
        extractedFactId: "fact-1",
        pageNumber: 3,
        boundingBoxJson: null,
      },
    ],
    sourceDocuments: [{ id: "doc-1", name: "statement.pdf", pageNumber: 3 }],
    fxRatesUsed: [],
    valuationDates: ["2026-05-14"],
    roundingPolicy: "Stored Decimal values are returned without display rounding.",
    warnings: [],
    confidence: "0.90",
    freshness: "fresh",
    lastUpdated: "2026-05-14T00:00:00Z",
    ...overrides,
  };
}

function renderExplainableNumber() {
  return render(
    <MemoryRouter>
      <ExplainableNumber entityType="valuation" entityId="valuation-1" metricType="valuation" />
    </MemoryRouter>,
  );
}

describe("ExplainableNumber", () => {
  beforeEach(() => {
    getDataLineageMock.mockReset().mockResolvedValue(lineage());
  });

  it("renders formula, inputs, citations, and source document links", async () => {
    const user = userEvent.setup();
    renderExplainableNumber();

    await user.click(screen.getByRole("button", { name: "Explain this number" }));

    expect(getDataLineageMock).toHaveBeenCalledWith({
      entityType: "valuation",
      entityId: "valuation-1",
      metricType: "valuation",
    });
    expect(await screen.findByText("Explain This Number")).toBeInTheDocument();
    expect(screen.getByText("Stored asset valuation value from the valuations table.")).toBeInTheDocument();
    expect(screen.getByText("Villa")).toBeInTheDocument();
    expect(screen.getByText("statement.pdf p.3, page 3")).toBeInTheDocument();
    expect(screen.getByRole("link", { name: "statement.pdf" })).toHaveAttribute(
      "href",
      "/documents/review-queue?documentId=doc-1",
    );
  });

  it("renders the missing-source copy when no citation exists", async () => {
    getDataLineageMock.mockResolvedValueOnce(
      lineage({
        sourceCitations: [],
        sourceDocuments: [],
        warnings: ["No source document linked yet."],
      }),
    );
    const user = userEvent.setup();
    renderExplainableNumber();

    await user.click(screen.getByRole("button", { name: "Explain this number" }));

    expect(await screen.findAllByText("No source document linked yet")).toHaveLength(2);
    expect(screen.getByText("No source document linked yet.")).toBeInTheDocument();
  });
});

