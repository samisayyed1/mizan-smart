import type {
  DocumentMetadata,
  DocumentParserCapabilities,
  DocumentProcessingJob,
  DocumentRecord,
  ExtractedFact,
} from "@/adapters";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import React from "react";
import { MemoryRouter } from "react-router-dom";
import { beforeEach, describe, expect, it, vi } from "vitest";

const listDocumentsMock = vi.fn<() => Promise<DocumentMetadata[]>>();
const listDocumentJobsMock = vi.fn<() => Promise<DocumentProcessingJob[]>>();
const getDocumentParserCapabilitiesMock = vi.fn<() => Promise<DocumentParserCapabilities>>();
const listPendingExtractedFactsMock = vi.fn<() => Promise<ExtractedFact[]>>();
const uploadDocumentMock = vi.fn<(file: File) => Promise<DocumentRecord>>();
const deleteDocumentMock = vi.fn<(documentId: string) => Promise<void>>();
const retryDocumentJobMock = vi.fn<(jobId: string) => Promise<DocumentProcessingJob>>();

vi.mock("@/adapters", () => ({
  listDocumentJobs: () => listDocumentJobsMock(),
  getDocumentParserCapabilities: () => getDocumentParserCapabilitiesMock(),
  listPendingExtractedFacts: () => listPendingExtractedFactsMock(),
  listDocuments: () => listDocumentsMock(),
  uploadDocument: (file: File) => uploadDocumentMock(file),
  deleteDocument: (documentId: string) => deleteDocumentMock(documentId),
  retryDocumentJob: (jobId: string) => retryDocumentJobMock(jobId),
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
    FileText: () => <span>FileText</span>,
    Loader: () => <span>Loader</span>,
    Refresh: () => <span>Refresh</span>,
    Trash2: () => <span>Trash2</span>,
    Upload: () => <span>Upload</span>,
  },
  Page: ({ children }: { children: React.ReactNode }) => <div>{children}</div>,
  PageContent: ({ children }: { children: React.ReactNode }) => <main>{children}</main>,
  PageHeader: ({ heading }: { heading?: string }) => <header>{heading}</header>,
}));

import DocumentsPage from "./documents-page";

function renderDocumentsPage(): ReturnType<typeof render> {
  return render(
    <MemoryRouter>
      <DocumentsPage />
    </MemoryRouter>,
  );
}

function metadata(overrides: Partial<DocumentMetadata> = {}): DocumentMetadata {
  return {
    id: "doc-1",
    fileHash: "hash-1",
    originalName: "statement.pdf",
    mimeType: "application/pdf",
    fileSizeBytes: 2048,
    encryptedStoragePath: "doc-1.mizdoc",
    status: "ingested",
    sourceType: null,
    errorMessage: null,
    createdAt: "2026-05-14T09:00:00Z",
    updatedAt: "2026-05-14T09:00:00Z",
    ...overrides,
  };
}

function record(document: DocumentMetadata): DocumentRecord {
  return {
    document,
    file: {
      id: "file-1",
      documentId: document.id,
      encryptionVersion: 1,
      nonce: "abc",
      checksumSha256: "checksum",
      storagePath: document.encryptedStoragePath,
      createdAt: document.createdAt,
    },
  };
}

function job(overrides: Partial<DocumentProcessingJob> = {}): DocumentProcessingJob {
  return {
    id: "job-1",
    documentId: "doc-1",
    jobType: "parse_text",
    status: "queued",
    priority: 0,
    attempts: 0,
    maxAttempts: 3,
    errorMessage: null,
    startedAt: null,
    completedAt: null,
    createdAt: "2026-05-14T09:01:00Z",
    ...overrides,
  };
}

function extractedFact(overrides: Partial<ExtractedFact> = {}): ExtractedFact {
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

describe("DocumentsPage", () => {
  beforeEach(() => {
    listDocumentsMock.mockReset().mockResolvedValue([]);
    listDocumentJobsMock.mockReset().mockResolvedValue([]);
    getDocumentParserCapabilitiesMock
      .mockReset()
      .mockResolvedValue({ text: true, layout: false, tables: false, ocr: false });
    listPendingExtractedFactsMock.mockReset().mockResolvedValue([]);
    uploadDocumentMock
      .mockReset()
      .mockImplementation((file) => Promise.resolve(record(metadata({ originalName: file.name }))));
    deleteDocumentMock.mockReset().mockResolvedValue();
    retryDocumentJobMock.mockReset().mockResolvedValue(job());
  });

  it("renders an honest empty state when no documents exist", async () => {
    renderDocumentsPage();
    expect(await screen.findByTestId("documents-empty")).toBeInTheDocument();
    expect(screen.getByText("No documents in the vault")).toBeInTheDocument();
    expect(
      screen.getByText("Local text extraction available; layout and table extraction unavailable"),
    ).toBeInTheDocument();
    expect(screen.getByText("0 extracted facts awaiting review")).toBeInTheDocument();
  });

  it("shows the pending extracted fact count", async () => {
    listPendingExtractedFactsMock.mockResolvedValueOnce([
      extractedFact({ id: "fact-1" }),
      extractedFact({ id: "fact-2" }),
    ]);

    renderDocumentsPage();

    expect(await screen.findByText("2 extracted facts awaiting review")).toBeInTheDocument();
  });

  it("lists persisted document metadata", async () => {
    listDocumentsMock.mockResolvedValueOnce([
      metadata({ id: "doc-1", originalName: "statement.pdf", fileSizeBytes: 2048 }),
    ]);
    renderDocumentsPage();
    expect(await screen.findByText("statement.pdf")).toBeInTheDocument();
    expect(screen.getByText("application/pdf")).toBeInTheDocument();
    expect(screen.getByText("2.0 KB")).toBeInTheDocument();
    expect(screen.getByText("ingested")).toBeInTheDocument();
    expect(screen.getByText("No job")).toBeInTheDocument();
  });

  it("uploads a selected file and refreshes the list", async () => {
    const uploaded = metadata({ id: "doc-2", originalName: "factsheet.pdf" });
    listDocumentsMock.mockResolvedValueOnce([]).mockResolvedValueOnce([uploaded]);
    uploadDocumentMock.mockResolvedValueOnce(record(uploaded));
    const user = userEvent.setup();

    renderDocumentsPage();
    const file = new File(["facts"], "factsheet.pdf", { type: "application/pdf" });
    await user.upload(await screen.findByLabelText("Choose document"), file);

    expect(uploadDocumentMock).toHaveBeenCalledWith(file);
    expect(await screen.findByText("factsheet.pdf")).toBeInTheDocument();
  });

  it("shows duplicate upload errors", async () => {
    uploadDocumentMock.mockRejectedValueOnce(new Error("Duplicate document already exists"));
    const user = userEvent.setup();

    renderDocumentsPage();
    const file = new File(["same"], "statement.pdf", { type: "application/pdf" });
    await user.upload(await screen.findByLabelText("Choose document"), file);

    expect(await screen.findByRole("alert")).toHaveTextContent(
      "This file is already in the Document Vault.",
    );
  });

  it("shows generic upload errors", async () => {
    uploadDocumentMock.mockRejectedValueOnce(new Error("Disk is full"));
    const user = userEvent.setup();

    renderDocumentsPage();
    const file = new File(["bytes"], "statement.pdf", { type: "application/pdf" });
    await user.upload(await screen.findByLabelText("Choose document"), file);

    expect(await screen.findByRole("alert")).toHaveTextContent("Disk is full");
  });

  it("deletes a document from the list", async () => {
    listDocumentsMock.mockResolvedValueOnce([metadata({ id: "doc-1", originalName: "statement.pdf" })]);
    const user = userEvent.setup();

    renderDocumentsPage();
    await user.click(await screen.findByRole("button", { name: "Delete statement.pdf" }));

    expect(deleteDocumentMock).toHaveBeenCalledWith("doc-1");
    await waitFor(() => expect(screen.queryByText("statement.pdf")).not.toBeInTheDocument());
  });

  it("shows failed processing jobs and retries them", async () => {
    const failedJob = job({
      id: "job-1",
      documentId: "doc-1",
      status: "failed",
      attempts: 1,
      maxAttempts: 3,
      errorMessage: "Document text parser is not available on this machine",
    });
    listDocumentsMock.mockResolvedValue([metadata({ id: "doc-1", originalName: "statement.pdf" })]);
    listDocumentJobsMock
      .mockResolvedValueOnce([failedJob])
      .mockResolvedValueOnce([job({ ...failedJob, status: "queued", errorMessage: null })]);
    retryDocumentJobMock.mockResolvedValueOnce(job({ ...failedJob, status: "queued", errorMessage: null }));
    const user = userEvent.setup();

    renderDocumentsPage();
    expect(await screen.findByText("failed")).toBeInTheDocument();
    expect(screen.getByText("Document text parser is not available on this machine")).toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: "Retry" }));

    expect(retryDocumentJobMock).toHaveBeenCalledWith("job-1");
    expect(await screen.findByText("queued")).toBeInTheDocument();
  });
});
