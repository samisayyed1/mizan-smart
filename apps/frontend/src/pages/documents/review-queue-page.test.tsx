import type {
  ExtractedFact,
  ExtractedFactEntityLink,
  ParsedDocument,
  ReviewExtractedFactRequest,
  UpdateExtractedFactRequest,
  LinkExtractedFactRequest,
  DeferExtractedFactRequest,
} from "@/adapters";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import React from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";

const listPendingExtractedFactsMock = vi.fn<() => Promise<ExtractedFact[]>>();
const getParsedDocumentMock = vi.fn<(documentId: string) => Promise<ParsedDocument>>();
const approveExtractedFactMock = vi.fn<
  (factId: string, request: ReviewExtractedFactRequest) => Promise<ExtractedFact>
>();
const rejectExtractedFactMock = vi.fn<
  (factId: string, request: ReviewExtractedFactRequest) => Promise<ExtractedFact>
>();
const updateExtractedFactBeforeApprovalMock = vi.fn<
  (factId: string, request: UpdateExtractedFactRequest) => Promise<ExtractedFact>
>();
const linkExtractedFactToEntityMock = vi.fn<
  (factId: string, request: LinkExtractedFactRequest) => Promise<ExtractedFactEntityLink>
>();
const deferExtractedFactMock = vi.fn<
  (factId: string, request: DeferExtractedFactRequest) => Promise<ExtractedFact>
>();

vi.mock("@/adapters", () => ({
  listPendingExtractedFacts: () => listPendingExtractedFactsMock(),
  getParsedDocument: (documentId: string) => getParsedDocumentMock(documentId),
  approveExtractedFact: (factId: string, request: ReviewExtractedFactRequest) =>
    approveExtractedFactMock(factId, request),
  rejectExtractedFact: (factId: string, request: ReviewExtractedFactRequest) =>
    rejectExtractedFactMock(factId, request),
  updateExtractedFactBeforeApproval: (factId: string, request: UpdateExtractedFactRequest) =>
    updateExtractedFactBeforeApprovalMock(factId, request),
  linkExtractedFactToEntity: (factId: string, request: LinkExtractedFactRequest) =>
    linkExtractedFactToEntityMock(factId, request),
  deferExtractedFact: (factId: string, request: DeferExtractedFactRequest) =>
    deferExtractedFactMock(factId, request),
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
  Icons: {
    Loader: () => <span>Loader</span>,
  },
  Page: ({ children }: { children: React.ReactNode }) => <div>{children}</div>,
  PageContent: ({ children }: { children: React.ReactNode }) => <main>{children}</main>,
  PageHeader: ({ heading }: { heading?: string }) => <header>{heading}</header>,
}));

import DocumentReviewQueuePage from "./review-queue-page";

function fact(overrides: Partial<ExtractedFact> = {}): ExtractedFact {
  return {
    id: "fact-1",
    documentId: "doc-1",
    pageNumber: 1,
    factType: "statement_balance",
    rawValue: "$1,250.00",
    normalizedValue: "1250.00",
    currency: "USD",
    dateValue: null,
    confidenceScore: 0.91,
    boundingBox: null,
    extractionMethod: "parser",
    extractionVersion: "local-parser-v1",
    status: "pending",
    createdAt: "2026-05-14T09:02:00Z",
    reviewedAt: null,
    reviewNotes: null,
    ...overrides,
  };
}

function parsedDocument(): ParsedDocument {
  return {
    documentId: "doc-1",
    pages: [{ pageNumber: 1, width: null, height: null, rotation: null }],
    textBlocks: [
      {
        pageNumber: 1,
        text: "Statement balance $1,250.00",
        boundingBox: null,
        blockOrder: 0,
        confidence: 0.91,
      },
    ],
    tables: [],
  };
}

describe("DocumentReviewQueuePage", () => {
  beforeEach(() => {
    listPendingExtractedFactsMock.mockReset().mockResolvedValue([fact()]);
    getParsedDocumentMock.mockReset().mockResolvedValue(parsedDocument());
    approveExtractedFactMock.mockReset().mockImplementation((_, request) =>
      Promise.resolve(
        fact({
          status: "approved",
          reviewNotes: request.reviewNotes,
          reviewedAt: "2026-05-14T10:00:00Z",
        }),
      ),
    );
    rejectExtractedFactMock.mockReset().mockImplementation((_, request) =>
      Promise.resolve(
        fact({
          status: "rejected",
          reviewNotes: request.reviewNotes,
          reviewedAt: "2026-05-14T10:00:00Z",
        }),
      ),
    );
    updateExtractedFactBeforeApprovalMock
      .mockReset()
      .mockImplementation((_, request) =>
        Promise.resolve(fact({ normalizedValue: request.normalizedValue, currency: request.currency })),
      );
    linkExtractedFactToEntityMock.mockReset().mockResolvedValue({
      id: "link-1",
      extractedFactId: "fact-1",
      entityType: "asset",
      entityId: "asset-1",
      createdAt: "2026-05-14T10:00:00Z",
    });
    deferExtractedFactMock.mockReset().mockResolvedValue(fact({ reviewNotes: "Later" }));
  });

  it("renders pending facts with document text", async () => {
    render(<DocumentReviewQueuePage />);

    expect(await screen.findByText("statement_balance")).toBeInTheDocument();
    expect(screen.getByText("$1,250.00")).toBeInTheDocument();
    expect(await screen.findByText("Statement balance $1,250.00")).toBeInTheDocument();
    expect(screen.getByText("No suggested target mapping")).toBeInTheDocument();
  });

  it("approves a selected fact", async () => {
    const user = userEvent.setup();
    render(<DocumentReviewQueuePage />);

    await user.click(await screen.findByRole("button", { name: "Approve" }));

    expect(approveExtractedFactMock).toHaveBeenCalledWith("fact-1", { reviewNotes: null });
    expect(await screen.findByText("approved")).toBeInTheDocument();
  });

  it("rejects a selected fact", async () => {
    const user = userEvent.setup();
    render(<DocumentReviewQueuePage />);

    await user.click(await screen.findByRole("button", { name: "Reject" }));

    expect(rejectExtractedFactMock).toHaveBeenCalledWith("fact-1", { reviewNotes: null });
    expect(await screen.findByText("rejected")).toBeInTheDocument();
  });

  it("validates normalized money value before edit and approve", async () => {
    const user = userEvent.setup();
    render(<DocumentReviewQueuePage />);

    await user.clear(await screen.findByLabelText("Normalized value"));
    await user.type(screen.getByLabelText("Normalized value"), "not-money");
    await user.click(screen.getByRole("button", { name: "Edit and approve" }));

    expect(updateExtractedFactBeforeApprovalMock).not.toHaveBeenCalled();
    expect(await screen.findByRole("alert")).toHaveTextContent(
      "Normalized value must be a decimal amount before approval.",
    );
  });

  it("links a selected fact to an asset", async () => {
    const user = userEvent.setup();
    render(<DocumentReviewQueuePage />);

    await user.type(await screen.findByLabelText("Link entity id"), "asset-1");
    await user.click(screen.getByRole("button", { name: "Link" }));

    expect(linkExtractedFactToEntityMock).toHaveBeenCalledWith("fact-1", {
      entityType: "asset",
      entityId: "asset-1",
      reviewNotes: null,
    });
    expect(await screen.findByText("Linked to asset")).toBeInTheDocument();
  });
});
