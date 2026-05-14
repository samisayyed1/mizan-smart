import {
  deleteDocument,
  getDocumentParserCapabilities,
  listDocumentJobs,
  listDocuments,
  retryDocumentJob,
  uploadDocument,
} from "@/adapters";
import type { DocumentMetadata, DocumentParserCapabilities, DocumentProcessingJob } from "@/adapters";
import { Button, Icons, Page, PageContent, PageHeader } from "@mizan/ui";
import { useEffect, useRef, useState } from "react";

function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

function statusLabel(status: DocumentMetadata["status"]): string {
  return status.replaceAll("_", " ");
}

function jobStatusLabel(status: DocumentProcessingJob["status"]): string {
  return status.replaceAll("_", " ");
}

function uploadMessage(error: unknown): string {
  const message = error instanceof Error ? error.message : String(error);
  if (message.toLowerCase().includes("duplicate")) {
    return "This file is already in the Document Vault.";
  }
  return message || "Upload failed.";
}

export default function DocumentsPage() {
  const inputRef = useRef<HTMLInputElement | null>(null);
  const [documents, setDocuments] = useState<DocumentMetadata[]>([]);
  const [jobs, setJobs] = useState<DocumentProcessingJob[]>([]);
  const [parserCapabilities, setParserCapabilities] = useState<DocumentParserCapabilities | null>(
    null,
  );
  const [loading, setLoading] = useState(true);
  const [uploading, setUploading] = useState(false);
  const [deletingId, setDeletingId] = useState<string | null>(null);
  const [retryingJobId, setRetryingJobId] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  async function refreshDocuments(): Promise<void> {
    setLoading(true);
    try {
      const [loadedDocuments, loadedJobs, loadedParserCapabilities] = await Promise.all([
        listDocuments(),
        listDocumentJobs(),
        getDocumentParserCapabilities(),
      ]);
      setDocuments(loadedDocuments);
      setJobs(loadedJobs);
      setParserCapabilities(loadedParserCapabilities);
      setError(null);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Could not load documents.");
    } finally {
      setLoading(false);
    }
  }

  useEffect(() => {
    void refreshDocuments();
  }, []);

  async function handleFiles(files: FileList | File[]): Promise<void> {
    const file = Array.from(files)[0];
    if (!file) return;
    setUploading(true);
    setError(null);
    try {
      await uploadDocument(file);
      await refreshDocuments();
    } catch (err) {
      setError(uploadMessage(err));
    } finally {
      setUploading(false);
      if (inputRef.current) inputRef.current.value = "";
    }
  }

  async function handleDelete(documentId: string): Promise<void> {
    setDeletingId(documentId);
    setError(null);
    try {
      await deleteDocument(documentId);
      setDocuments((current) => current.filter((document) => document.id !== documentId));
    } catch (err) {
      setError(err instanceof Error ? err.message : "Could not delete document.");
    } finally {
      setDeletingId(null);
    }
  }

  async function handleRetry(jobId: string): Promise<void> {
    setRetryingJobId(jobId);
    setError(null);
    try {
      await retryDocumentJob(jobId);
      await refreshDocuments();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Could not retry document job.");
    } finally {
      setRetryingJobId(null);
    }
  }

  const latestJobByDocument = new Map<string, DocumentProcessingJob>();
  for (const job of jobs) {
    const current = latestJobByDocument.get(job.documentId);
    if (!current || job.createdAt > current.createdAt) {
      latestJobByDocument.set(job.documentId, job);
    }
  }

  return (
    <Page>
      <PageHeader heading="Documents" text="Encrypted statements, factsheets, and source files." />
      <PageContent>
        <div className="space-y-4">
          <div
            data-testid="document-dropzone"
            className="border-border bg-muted/20 flex min-h-40 flex-col items-center justify-center rounded-lg border border-dashed px-6 py-8 text-center"
            onDragOver={(event) => event.preventDefault()}
            onDrop={(event) => {
              event.preventDefault();
              void handleFiles(event.dataTransfer.files);
            }}
          >
            <Icons.Upload className="text-muted-foreground size-10" aria-hidden="true" />
            <p className="mt-3 text-sm font-medium">Drop a file here</p>
            <p className="text-muted-foreground mt-1 text-sm">PDFs, statements, and source files</p>
            <input
              ref={inputRef}
              aria-label="Choose document"
              className="sr-only"
              type="file"
              onChange={(event) => void handleFiles(event.currentTarget.files ?? [])}
            />
            <Button
              className="mt-4"
              type="button"
              onClick={() => inputRef.current?.click()}
              disabled={uploading}
            >
              {uploading ? (
                <Icons.Loader className="mr-2 size-4 animate-spin" aria-hidden="true" />
              ) : (
                <Icons.Upload className="mr-2 size-4" aria-hidden="true" />
              )}
              Upload
            </Button>
          </div>

          {parserCapabilities ? (
            <p className="text-muted-foreground text-sm">
              {parserCapabilities.text
                ? "Local text extraction available"
                : "Local text extraction unavailable"}
              {parserCapabilities.layout || parserCapabilities.tables
                ? ""
                : "; layout and table extraction unavailable"}
            </p>
          ) : null}

          {error ? (
            <div
              role="alert"
              className="border-destructive/30 bg-destructive/10 text-destructive rounded-md border px-4 py-3 text-sm"
            >
              {error}
            </div>
          ) : null}

          {loading ? (
            <div className="text-muted-foreground flex items-center gap-2 text-sm">
              <Icons.Loader className="size-4 animate-spin" aria-hidden="true" />
              Loading documents
            </div>
          ) : documents.length === 0 ? (
            <div
              data-testid="documents-empty"
              className="border-border bg-card rounded-lg border px-6 py-10 text-center"
            >
              <Icons.FileText className="text-muted-foreground mx-auto size-10" aria-hidden="true" />
              <p className="mt-4 text-base font-medium">No documents in the vault</p>
            </div>
          ) : (
            <div className="border-border overflow-hidden rounded-lg border">
              <div className="bg-muted/40 text-muted-foreground grid grid-cols-[minmax(0,1fr)_120px_140px_120px_88px] gap-3 px-4 py-2 text-xs font-medium uppercase">
                <span>Name</span>
                <span>Status</span>
                <span>Processing</span>
                <span className="text-right">Size</span>
                <span className="text-right">Action</span>
              </div>
              <ul className="divide-border divide-y">
                {documents.map((document) => {
                  const job = latestJobByDocument.get(document.id);
                  return (
                    <li
                      key={document.id}
                      className="grid grid-cols-[minmax(0,1fr)_120px_140px_120px_88px] items-center gap-3 px-4 py-3 text-sm"
                    >
                      <div className="min-w-0">
                        <p className="truncate font-medium">{document.originalName}</p>
                        <p className="text-muted-foreground truncate text-xs">{document.mimeType}</p>
                      </div>
                      <span className="capitalize">{statusLabel(document.status)}</span>
                      <div className="min-w-0">
                        {job ? (
                          <>
                            <p className="capitalize">{jobStatusLabel(job.status)}</p>
                            {job.status === "failed" && job.errorMessage ? (
                              <p className="text-destructive truncate text-xs">{job.errorMessage}</p>
                            ) : null}
                            {job.status === "failed" && job.attempts < job.maxAttempts ? (
                              <Button
                                className="mt-1 h-7 px-2 text-xs"
                                type="button"
                                variant="secondary"
                                aria-label="Retry"
                                onClick={() => void handleRetry(job.id)}
                                disabled={retryingJobId === job.id}
                              >
                                {retryingJobId === job.id ? (
                                  <Icons.Loader className="mr-1 size-3 animate-spin" aria-hidden="true" />
                                ) : (
                                  <Icons.Refresh className="mr-1 size-3" aria-hidden="true" />
                                )}
                                Retry
                              </Button>
                            ) : null}
                          </>
                        ) : (
                          <span className="text-muted-foreground">No job</span>
                        )}
                      </div>
                      <span className="text-right">{formatBytes(document.fileSizeBytes)}</span>
                      <div className="text-right">
                        <Button
                          aria-label={`Delete ${document.originalName}`}
                          type="button"
                          variant="ghost"
                          size="icon"
                          onClick={() => void handleDelete(document.id)}
                          disabled={deletingId === document.id}
                        >
                          {deletingId === document.id ? (
                            <Icons.Loader className="size-4 animate-spin" aria-hidden="true" />
                          ) : (
                            <Icons.Trash2 className="size-4" aria-hidden="true" />
                          )}
                        </Button>
                      </div>
                    </li>
                  );
                })}
              </ul>
            </div>
          )}
        </div>
      </PageContent>
    </Page>
  );
}
