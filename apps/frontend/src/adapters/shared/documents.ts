import { invoke } from "./platform";

export type DocumentStatus =
  | "ingested"
  | "queued"
  | "processing"
  | "processed"
  | "reviewed"
  | "error";

export interface UploadDocumentRequest {
  originalName: string;
  mimeType: string;
  content: number[];
  sourceType?: string | null;
}

export interface DocumentMetadata {
  id: string;
  fileHash: string;
  originalName: string;
  mimeType: string;
  fileSizeBytes: number;
  encryptedStoragePath: string;
  status: DocumentStatus;
  sourceType: string | null;
  errorMessage: string | null;
  createdAt: string;
  updatedAt: string;
}

export interface DocumentFileMetadata {
  id: string;
  documentId: string;
  encryptionVersion: number;
  nonce: string;
  checksumSha256: string;
  storagePath: string;
  createdAt: string;
}

export interface DocumentRecord {
  document: DocumentMetadata;
  file: DocumentFileMetadata;
}

async function fileToContent(file: File): Promise<number[]> {
  return Array.from(new Uint8Array(await file.arrayBuffer()));
}

export async function uploadDocument(file: File): Promise<DocumentRecord> {
  const request: UploadDocumentRequest = {
    originalName: file.name,
    mimeType: file.type || "application/octet-stream",
    content: await fileToContent(file),
    sourceType: null,
  };
  return invoke<DocumentRecord>("upload_document", { request });
}

export async function listDocuments(): Promise<DocumentMetadata[]> {
  return invoke<DocumentMetadata[]>("list_documents", {});
}

export async function getDocumentMetadata(documentId: string): Promise<DocumentRecord> {
  return invoke<DocumentRecord>("get_document_metadata", { documentId });
}

export async function deleteDocument(documentId: string): Promise<void> {
  return invoke<void>("delete_document", { documentId });
}

export async function readDocumentBytes(documentId: string): Promise<number[]> {
  return invoke<number[]>("read_document_bytes", { documentId });
}
