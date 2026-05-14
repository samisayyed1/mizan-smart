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
