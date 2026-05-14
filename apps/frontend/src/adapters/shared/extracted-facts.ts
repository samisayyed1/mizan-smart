import type { DocumentBoundingBox } from "./document-jobs";
import { invoke } from "./platform";

export type ExtractionMethod = "parser" | "ocr" | "vlm" | "manual";
export type ExtractedFactStatus = "pending" | "approved" | "rejected" | "superseded";
export type SourceCitationType = "document" | "manual" | "import" | "web_evidence" | "calculated";
export type ExtractedFactLinkEntityType = "asset" | "account";

export interface CreateExtractedFactRequest {
  documentId: string;
  pageNumber: number | null;
  factType: string;
  rawValue: string;
  normalizedValue: string | null;
  currency: string | null;
  dateValue: string | null;
  confidenceScore: number | null;
  boundingBox: DocumentBoundingBox | null;
  extractionMethod: ExtractionMethod;
  extractionVersion: string;
  citationLabel: string;
}

export interface ExtractedFact {
  id: string;
  documentId: string;
  pageNumber: number | null;
  factType: string;
  rawValue: string;
  normalizedValue: string | null;
  currency: string | null;
  dateValue: string | null;
  confidenceScore: number | null;
  boundingBox: DocumentBoundingBox | null;
  extractionMethod: ExtractionMethod;
  extractionVersion: string;
  status: ExtractedFactStatus;
  createdAt: string;
  reviewedAt: string | null;
  reviewNotes: string | null;
}

export interface SourceCitation {
  id: string;
  sourceType: SourceCitationType;
  sourceId: string | null;
  documentId: string | null;
  extractedFactId: string | null;
  pageNumber: number | null;
  boundingBox: DocumentBoundingBox | null;
  citationLabel: string;
  createdAt: string;
}

export interface CreateExtractedFactResult {
  fact: ExtractedFact;
  citation: SourceCitation;
}

export interface ReviewExtractedFactRequest {
  reviewNotes: string | null;
}

export interface UpdateExtractedFactRequest {
  normalizedValue: string | null;
  currency: string | null;
  dateValue: string | null;
  reviewNotes: string | null;
}

export interface LinkExtractedFactRequest {
  entityType: ExtractedFactLinkEntityType;
  entityId: string;
  reviewNotes: string | null;
}

export interface DeferExtractedFactRequest {
  reviewNotes: string | null;
}

export interface ExtractedFactEntityLink {
  id: string;
  extractedFactId: string;
  entityType: ExtractedFactLinkEntityType;
  entityId: string;
  createdAt: string;
}

export async function createExtractedFact(
  request: CreateExtractedFactRequest,
): Promise<CreateExtractedFactResult> {
  return invoke<CreateExtractedFactResult>("create_extracted_fact", { request });
}

export async function listPendingExtractedFacts(): Promise<ExtractedFact[]> {
  return invoke<ExtractedFact[]>("list_pending_extracted_facts", {});
}

export async function getSourceCitation(citationId: string): Promise<SourceCitation> {
  return invoke<SourceCitation>("get_source_citation", { citationId });
}

export async function approveExtractedFact(
  factId: string,
  request: ReviewExtractedFactRequest,
): Promise<ExtractedFact> {
  return invoke<ExtractedFact>("approve_extracted_fact", { factId, request });
}

export async function updateExtractedFactBeforeApproval(
  factId: string,
  request: UpdateExtractedFactRequest,
): Promise<ExtractedFact> {
  return invoke<ExtractedFact>("update_extracted_fact_before_approval", { factId, request });
}

export async function linkExtractedFactToEntity(
  factId: string,
  request: LinkExtractedFactRequest,
): Promise<ExtractedFactEntityLink> {
  return invoke<ExtractedFactEntityLink>("link_extracted_fact_to_entity", { factId, request });
}

export async function deferExtractedFact(
  factId: string,
  request: DeferExtractedFactRequest,
): Promise<ExtractedFact> {
  return invoke<ExtractedFact>("defer_extracted_fact", { factId, request });
}

export async function rejectExtractedFact(
  factId: string,
  request: ReviewExtractedFactRequest,
): Promise<ExtractedFact> {
  return invoke<ExtractedFact>("reject_extracted_fact", { factId, request });
}
