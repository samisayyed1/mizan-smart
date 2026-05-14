import { invoke } from "./platform";

export type DocumentJobType =
  | "parse_text"
  | "extract_layout"
  | "extract_tables"
  | "ocr"
  | "vlm_extract"
  | "embed";

export type DocumentJobStatus = "queued" | "running" | "succeeded" | "failed" | "cancelled";

export interface EnqueueDocumentJobRequest {
  documentId: string;
  jobType: DocumentJobType;
  priority: number;
  maxAttempts: number;
}

export interface DocumentProcessingJob {
  id: string;
  documentId: string;
  jobType: DocumentJobType;
  status: DocumentJobStatus;
  priority: number;
  attempts: number;
  maxAttempts: number;
  errorMessage: string | null;
  startedAt: string | null;
  completedAt: string | null;
  createdAt: string;
}

export interface RunDocumentJobResult {
  job: DocumentProcessingJob | null;
}

export async function enqueueDocumentJob(
  request: EnqueueDocumentJobRequest,
): Promise<DocumentProcessingJob> {
  return invoke<DocumentProcessingJob>("enqueue_document_job", { request });
}

export async function listDocumentJobs(documentId?: string): Promise<DocumentProcessingJob[]> {
  return invoke<DocumentProcessingJob[]>("list_document_jobs", { documentId });
}

export async function runNextDocumentJob(): Promise<RunDocumentJobResult> {
  return invoke<RunDocumentJobResult>("run_next_document_job", {});
}

export async function cancelDocumentJob(jobId: string): Promise<DocumentProcessingJob> {
  return invoke<DocumentProcessingJob>("cancel_document_job", { jobId });
}

export async function retryDocumentJob(jobId: string): Promise<DocumentProcessingJob> {
  return invoke<DocumentProcessingJob>("retry_document_job", { jobId });
}
