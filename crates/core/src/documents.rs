use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::Result;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentParserCapabilities {
    pub text: bool,
    pub layout: bool,
    pub tables: bool,
    pub ocr: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentBoundingBox {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParsedDocumentPage {
    pub page_number: i32,
    pub width: Option<f64>,
    pub height: Option<f64>,
    pub rotation: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParsedTextBlock {
    pub page_number: i32,
    pub text: String,
    pub bounding_box: Option<DocumentBoundingBox>,
    pub block_order: i32,
    pub confidence: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParsedTableCell {
    pub row_index: i32,
    pub column_index: i32,
    pub text: String,
    pub bounding_box: Option<DocumentBoundingBox>,
    pub confidence: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParsedTable {
    pub page_number: i32,
    pub bounding_box: Option<DocumentBoundingBox>,
    pub cells: Vec<ParsedTableCell>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParsedDocument {
    pub document_id: String,
    pub pages: Vec<ParsedDocumentPage>,
    pub text_blocks: Vec<ParsedTextBlock>,
    pub tables: Vec<ParsedTable>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExtractionMethod {
    Parser,
    Ocr,
    Vlm,
    Manual,
}

impl ExtractionMethod {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Parser => "parser",
            Self::Ocr => "ocr",
            Self::Vlm => "vlm",
            Self::Manual => "manual",
        }
    }
}

impl TryFrom<&str> for ExtractionMethod {
    type Error = String;

    fn try_from(value: &str) -> std::result::Result<Self, String> {
        match value {
            "parser" => Ok(Self::Parser),
            "ocr" => Ok(Self::Ocr),
            "vlm" => Ok(Self::Vlm),
            "manual" => Ok(Self::Manual),
            other => Err(format!("Unknown extraction method: {other}")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExtractedFactStatus {
    Pending,
    Approved,
    Rejected,
    Superseded,
}

impl ExtractedFactStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Superseded => "superseded",
        }
    }
}

impl TryFrom<&str> for ExtractedFactStatus {
    type Error = String;

    fn try_from(value: &str) -> std::result::Result<Self, String> {
        match value {
            "pending" => Ok(Self::Pending),
            "approved" => Ok(Self::Approved),
            "rejected" => Ok(Self::Rejected),
            "superseded" => Ok(Self::Superseded),
            other => Err(format!("Unknown extracted fact status: {other}")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceCitationType {
    Document,
    Manual,
    Import,
    WebEvidence,
    Calculated,
}

impl SourceCitationType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Document => "document",
            Self::Manual => "manual",
            Self::Import => "import",
            Self::WebEvidence => "web_evidence",
            Self::Calculated => "calculated",
        }
    }
}

impl TryFrom<&str> for SourceCitationType {
    type Error = String;

    fn try_from(value: &str) -> std::result::Result<Self, String> {
        match value {
            "document" => Ok(Self::Document),
            "manual" => Ok(Self::Manual),
            "import" => Ok(Self::Import),
            "web_evidence" => Ok(Self::WebEvidence),
            "calculated" => Ok(Self::Calculated),
            other => Err(format!("Unknown source citation type: {other}")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateExtractedFactRequest {
    pub document_id: String,
    pub page_number: Option<i32>,
    pub fact_type: String,
    pub raw_value: String,
    pub normalized_value: Option<String>,
    pub currency: Option<String>,
    pub date_value: Option<String>,
    pub confidence_score: Option<f64>,
    pub bounding_box: Option<DocumentBoundingBox>,
    pub extraction_method: ExtractionMethod,
    pub extraction_version: String,
    pub citation_label: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtractedFact {
    pub id: String,
    pub document_id: String,
    pub page_number: Option<i32>,
    pub fact_type: String,
    pub raw_value: String,
    pub normalized_value: Option<String>,
    pub currency: Option<String>,
    pub date_value: Option<String>,
    pub confidence_score: Option<f64>,
    pub bounding_box: Option<DocumentBoundingBox>,
    pub extraction_method: ExtractionMethod,
    pub extraction_version: String,
    pub status: ExtractedFactStatus,
    pub created_at: DateTime<Utc>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub review_notes: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceCitation {
    pub id: String,
    pub source_type: SourceCitationType,
    pub source_id: Option<String>,
    pub document_id: Option<String>,
    pub extracted_fact_id: Option<String>,
    pub page_number: Option<i32>,
    pub bounding_box: Option<DocumentBoundingBox>,
    pub citation_label: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateExtractedFactResult {
    pub fact: ExtractedFact,
    pub citation: SourceCitation,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewExtractedFactRequest {
    pub review_notes: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExtractedFactLinkEntityType {
    Asset,
    Account,
}

impl ExtractedFactLinkEntityType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Asset => "asset",
            Self::Account => "account",
        }
    }
}

impl TryFrom<&str> for ExtractedFactLinkEntityType {
    type Error = String;

    fn try_from(value: &str) -> std::result::Result<Self, String> {
        match value {
            "asset" => Ok(Self::Asset),
            "account" => Ok(Self::Account),
            other => Err(format!("Unknown extracted fact link entity type: {other}")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateExtractedFactRequest {
    pub normalized_value: Option<String>,
    pub currency: Option<String>,
    pub date_value: Option<String>,
    pub review_notes: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LinkExtractedFactRequest {
    pub entity_type: ExtractedFactLinkEntityType,
    pub entity_id: String,
    pub review_notes: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeferExtractedFactRequest {
    pub review_notes: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtractedFactEntityLink {
    pub id: String,
    pub extracted_fact_id: String,
    pub entity_type: ExtractedFactLinkEntityType,
    pub entity_id: String,
    pub created_at: DateTime<Utc>,
}

#[async_trait::async_trait]
pub trait DocumentParser: Send + Sync {
    fn capabilities(&self) -> DocumentParserCapabilities;
    async fn parse_document(&self, document_id: &str) -> Result<ParsedDocument>;
    async fn parse_text(&self, document_id: &str) -> Result<ParsedDocument>;
    async fn parse_layout(&self, document_id: &str) -> Result<ParsedDocument>;
    async fn parse_tables(&self, document_id: &str) -> Result<ParsedDocument>;
}
