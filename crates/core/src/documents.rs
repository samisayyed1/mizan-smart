use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DocumentStatus {
    Ingested,
    Queued,
    Processing,
    Processed,
    Reviewed,
    Error,
}

impl DocumentStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ingested => "ingested",
            Self::Queued => "queued",
            Self::Processing => "processing",
            Self::Processed => "processed",
            Self::Reviewed => "reviewed",
            Self::Error => "error",
        }
    }
}

impl TryFrom<&str> for DocumentStatus {
    type Error = String;

    fn try_from(value: &str) -> std::result::Result<Self, String> {
        match value {
            "ingested" => Ok(Self::Ingested),
            "queued" => Ok(Self::Queued),
            "processing" => Ok(Self::Processing),
            "processed" => Ok(Self::Processed),
            "reviewed" => Ok(Self::Reviewed),
            "error" => Ok(Self::Error),
            other => Err(format!("Unknown document status: {other}")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UploadDocumentRequest {
    pub original_name: String,
    pub mime_type: String,
    pub content: Vec<u8>,
    pub source_type: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentMetadata {
    pub id: String,
    pub file_hash: String,
    pub original_name: String,
    pub mime_type: String,
    pub file_size_bytes: i64,
    pub encrypted_storage_path: String,
    pub status: DocumentStatus,
    pub source_type: Option<String>,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentFileMetadata {
    pub id: String,
    pub document_id: String,
    pub encryption_version: i32,
    pub nonce: String,
    pub checksum_sha256: String,
    pub storage_path: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentLink {
    pub id: String,
    pub document_id: String,
    pub linked_entity_type: String,
    pub linked_entity_id: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentRecord {
    pub document: DocumentMetadata,
    pub file: DocumentFileMetadata,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DocumentJobType {
    ParseText,
    ExtractLayout,
    ExtractTables,
    Ocr,
    VlmExtract,
    Embed,
}

impl DocumentJobType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ParseText => "parse_text",
            Self::ExtractLayout => "extract_layout",
            Self::ExtractTables => "extract_tables",
            Self::Ocr => "ocr",
            Self::VlmExtract => "vlm_extract",
            Self::Embed => "embed",
        }
    }
}

impl TryFrom<&str> for DocumentJobType {
    type Error = String;

    fn try_from(value: &str) -> std::result::Result<Self, String> {
        match value {
            "parse_text" => Ok(Self::ParseText),
            "extract_layout" => Ok(Self::ExtractLayout),
            "extract_tables" => Ok(Self::ExtractTables),
            "ocr" => Ok(Self::Ocr),
            "vlm_extract" => Ok(Self::VlmExtract),
            "embed" => Ok(Self::Embed),
            other => Err(format!("Unknown document job type: {other}")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DocumentJobStatus {
    Queued,
    Running,
    Succeeded,
    Failed,
    Cancelled,
}

impl DocumentJobStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Succeeded => "succeeded",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl TryFrom<&str> for DocumentJobStatus {
    type Error = String;

    fn try_from(value: &str) -> std::result::Result<Self, String> {
        match value {
            "queued" => Ok(Self::Queued),
            "running" => Ok(Self::Running),
            "succeeded" => Ok(Self::Succeeded),
            "failed" => Ok(Self::Failed),
            "cancelled" => Ok(Self::Cancelled),
            other => Err(format!("Unknown document job status: {other}")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnqueueDocumentJobRequest {
    pub document_id: String,
    pub job_type: DocumentJobType,
    pub priority: i32,
    pub max_attempts: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentProcessingJob {
    pub id: String,
    pub document_id: String,
    pub job_type: DocumentJobType,
    pub status: DocumentJobStatus,
    pub priority: i32,
    pub attempts: i32,
    pub max_attempts: i32,
    pub error_message: Option<String>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunDocumentJobResult {
    pub job: Option<DocumentProcessingJob>,
}
