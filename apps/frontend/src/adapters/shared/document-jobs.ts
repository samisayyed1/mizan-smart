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

export interface DocumentParserCapabilities {
  text: boolean;
  layout: boolean;
  tables: boolean;
  ocr: boolean;
}

export interface DocumentBoundingBox {
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface ParsedDocumentPage {
  pageNumber: number;
  width: number | null;
  height: number | null;
  rotation: number | null;
}

export interface ParsedTextBlock {
  pageNumber: number;
  text: string;
  boundingBox: DocumentBoundingBox | null;
  blockOrder: number;
  confidence: number | null;
}

export interface ParsedTableCell {
  rowIndex: number;
  columnIndex: number;
  text: string;
  boundingBox: DocumentBoundingBox | null;
  confidence: number | null;
}

export interface ParsedTable {
  pageNumber: number;
  boundingBox: DocumentBoundingBox | null;
  cells: ParsedTableCell[];
}

export interface ParsedDocument {
  documentId: string;
  pages: ParsedDocumentPage[];
  textBlocks: ParsedTextBlock[];
  tables: ParsedTable[];
}

export async function enqueueDocumentJob(
  request: EnqueueDocumentJobRequest,
): Promise<DocumentProcessingJob> {
  return invoke<DocumentProcessingJob>("enqueue_document_job", { request });
}

export async function listDocumentJobs(documentId?: string): Promise<DocumentProcessingJob[]> {
  return invoke<DocumentProcessingJob[]>("list_document_jobs", { documentId });
}

export async function getDocumentParserCapabilities(): Promise<DocumentParserCapabilities> {
  return invoke<DocumentParserCapabilities>("get_document_parser_capabilities", {});
}

export async function getParsedDocument(documentId: string): Promise<ParsedDocument> {
  return invoke<ParsedDocument>("get_parsed_document", { documentId });
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
