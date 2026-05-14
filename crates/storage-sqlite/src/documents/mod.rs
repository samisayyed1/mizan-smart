//! SQLite-backed encrypted Document Vault storage.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use chacha20poly1305::aead::{Aead, KeyInit};
use chacha20poly1305::{ChaCha20Poly1305, Nonce};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel::r2d2::{self, Pool};
use diesel::SqliteConnection;
use rand::RngCore;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use mizan_core::documents::{
    DocumentFileMetadata, DocumentMetadata, DocumentRecord, DocumentStatus, UploadDocumentRequest,
};
use mizan_core::errors::{DatabaseError, Error};
use mizan_core::secrets::SecretStore;
use mizan_core::Result;

use crate::db::{get_connection, WriteHandle};
use crate::errors::StorageError;
use crate::schema::{document_files, documents};

pub const DOCUMENT_VAULT_KEY_SECRET: &str = "document_vault_key_v1";
const ENCRYPTION_VERSION: i32 = 1;
const NONCE_LEN: usize = 12;

#[derive(Debug, Clone, Queryable, Identifiable)]
#[diesel(table_name = documents)]
struct DocumentRow {
    id: String,
    file_hash: String,
    original_name: String,
    mime_type: String,
    file_size_bytes: i64,
    encrypted_storage_path: String,
    status: String,
    source_type: Option<String>,
    error_message: Option<String>,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = documents)]
struct NewDocumentRow {
    id: String,
    file_hash: String,
    original_name: String,
    mime_type: String,
    file_size_bytes: i64,
    encrypted_storage_path: String,
    status: String,
    source_type: Option<String>,
    error_message: Option<String>,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, Clone, Queryable, Identifiable, Associations)]
#[diesel(belongs_to(DocumentRow, foreign_key = document_id))]
#[diesel(table_name = document_files)]
struct DocumentFileRow {
    id: String,
    document_id: String,
    encryption_version: i32,
    nonce: String,
    checksum_sha256: String,
    storage_path: String,
    created_at: String,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = document_files)]
struct NewDocumentFileRow {
    id: String,
    document_id: String,
    encryption_version: i32,
    nonce: String,
    checksum_sha256: String,
    storage_path: String,
    created_at: String,
}

impl TryFrom<DocumentRow> for DocumentMetadata {
    type Error = Error;

    fn try_from(row: DocumentRow) -> std::result::Result<Self, Self::Error> {
        Ok(Self {
            id: row.id,
            file_hash: row.file_hash,
            original_name: row.original_name,
            mime_type: row.mime_type,
            file_size_bytes: row.file_size_bytes,
            encrypted_storage_path: row.encrypted_storage_path,
            status: DocumentStatus::try_from(row.status.as_str())
                .map_err(|err| Error::Database(DatabaseError::Internal(err)))?,
            source_type: row.source_type,
            error_message: row.error_message,
            created_at: parse_rfc3339(&row.created_at)?,
            updated_at: parse_rfc3339(&row.updated_at)?,
        })
    }
}

impl TryFrom<DocumentFileRow> for DocumentFileMetadata {
    type Error = Error;

    fn try_from(row: DocumentFileRow) -> std::result::Result<Self, Self::Error> {
        Ok(Self {
            id: row.id,
            document_id: row.document_id,
            encryption_version: row.encryption_version,
            nonce: row.nonce,
            checksum_sha256: row.checksum_sha256,
            storage_path: row.storage_path,
            created_at: parse_rfc3339(&row.created_at)?,
        })
    }
}

pub fn generate_document_vault_key_hex() -> String {
    let mut key = [0_u8; 32];
    rand::thread_rng().fill_bytes(&mut key);
    hex::encode(key)
}

pub fn parse_document_vault_key_hex(raw: &str) -> Result<[u8; 32]> {
    let decoded = hex::decode(raw.trim()).map_err(|err| {
        Error::Secret(format!(
            "Invalid Document Vault encryption key encoding: {err}"
        ))
    })?;
    if decoded.len() != 32 {
        return Err(Error::Secret(format!(
            "Invalid Document Vault encryption key length: expected 32 bytes, got {}",
            decoded.len()
        )));
    }
    let mut key = [0_u8; 32];
    key.copy_from_slice(&decoded);
    Ok(key)
}

pub fn load_or_create_document_vault_key(secret_store: &dyn SecretStore) -> Result<[u8; 32]> {
    match secret_store.get_secret(DOCUMENT_VAULT_KEY_SECRET)? {
        Some(raw) => parse_document_vault_key_hex(&raw),
        None => {
            let raw = generate_document_vault_key_hex();
            secret_store.set_secret(DOCUMENT_VAULT_KEY_SECRET, &raw)?;
            parse_document_vault_key_hex(&raw)
        }
    }
}

pub struct DocumentVaultRepository {
    pool: Arc<Pool<r2d2::ConnectionManager<SqliteConnection>>>,
    writer: WriteHandle,
    vault_dir: PathBuf,
    key: [u8; 32],
}

impl DocumentVaultRepository {
    pub fn new(
        pool: Arc<Pool<r2d2::ConnectionManager<SqliteConnection>>>,
        writer: WriteHandle,
        vault_dir: impl Into<PathBuf>,
        key: [u8; 32],
    ) -> Result<Self> {
        let vault_dir = vault_dir.into();
        fs::create_dir_all(&vault_dir)?;
        Ok(Self {
            pool,
            writer,
            vault_dir,
            key,
        })
    }

    pub async fn upload(&self, request: UploadDocumentRequest) -> Result<DocumentRecord> {
        let original_name = request.original_name.trim().to_string();
        if original_name.is_empty() {
            return Err(Error::Validation(
                mizan_core::errors::ValidationError::MissingField("original_name".into()),
            ));
        }

        let mime_type = if request.mime_type.trim().is_empty() {
            "application/octet-stream".to_string()
        } else {
            request.mime_type.trim().to_string()
        };
        let source_type = request.source_type.and_then(|value| {
            let trimmed = value.trim().to_string();
            (!trimmed.is_empty()).then_some(trimmed)
        });

        let file_hash = sha256_hex(&request.content);
        if self.find_by_hash(&file_hash)?.is_some() {
            return Err(Error::ConstraintViolation(
                "Duplicate document already exists in the Document Vault".into(),
            ));
        }

        let encrypted = encrypt_bytes(&self.key, &request.content)?;
        let encrypted_checksum = sha256_hex(&encrypted.ciphertext);
        let document_id = Uuid::new_v4().to_string();
        let file_id = Uuid::new_v4().to_string();
        let storage_path = format!("{document_id}.mizdoc");
        let absolute_path = self.vault_dir.join(&storage_path);
        fs::write(&absolute_path, &encrypted.ciphertext)?;

        let now = Utc::now();
        let now_rfc = now.to_rfc3339();
        let document_row = NewDocumentRow {
            id: document_id.clone(),
            file_hash: file_hash.clone(),
            original_name,
            mime_type,
            file_size_bytes: request.content.len() as i64,
            encrypted_storage_path: storage_path.clone(),
            status: DocumentStatus::Ingested.as_str().to_string(),
            source_type,
            error_message: None,
            created_at: now_rfc.clone(),
            updated_at: now_rfc.clone(),
        };
        let file_row = NewDocumentFileRow {
            id: file_id,
            document_id: document_id.clone(),
            encryption_version: ENCRYPTION_VERSION,
            nonce: hex::encode(encrypted.nonce),
            checksum_sha256: encrypted_checksum,
            storage_path: storage_path.clone(),
            created_at: now_rfc,
        };

        let insert_result = self
            .writer
            .exec_tx(move |tx| -> Result<()> {
                let conn = tx.conn();
                diesel::insert_into(documents::table)
                    .values(&document_row)
                    .execute(conn)
                    .map_err(StorageError::from)?;
                diesel::insert_into(document_files::table)
                    .values(&file_row)
                    .execute(conn)
                    .map_err(StorageError::from)?;
                Ok(())
            })
            .await;

        if let Err(err) = insert_result {
            let _ = fs::remove_file(&absolute_path);
            if err.to_string().contains("UNIQUE constraint failed") {
                return Err(Error::ConstraintViolation(
                    "Duplicate document already exists in the Document Vault".into(),
                ));
            }
            return Err(err);
        }

        self.get_metadata(&document_id)
    }

    pub fn list(&self) -> Result<Vec<DocumentMetadata>> {
        let mut conn = get_connection(&self.pool)?;
        let rows = documents::table
            .order(documents::created_at.desc())
            .load::<DocumentRow>(&mut conn)
            .map_err(StorageError::from)?;
        rows.into_iter().map(DocumentMetadata::try_from).collect()
    }

    pub fn get_metadata(&self, document_id: &str) -> Result<DocumentRecord> {
        let mut conn = get_connection(&self.pool)?;
        let document = documents::table
            .find(document_id)
            .first::<DocumentRow>(&mut conn)
            .optional()
            .map_err(StorageError::from)?
            .ok_or_else(|| Error::Database(DatabaseError::NotFound(document_id.to_string())))?;
        let file = document_files::table
            .filter(document_files::document_id.eq(document_id))
            .first::<DocumentFileRow>(&mut conn)
            .map_err(StorageError::from)?;
        Ok(DocumentRecord {
            document: DocumentMetadata::try_from(document)?,
            file: DocumentFileMetadata::try_from(file)?,
        })
    }

    pub fn read_decrypted(&self, document_id: &str) -> Result<Vec<u8>> {
        let record = self.get_metadata(document_id)?;
        let encrypted_path = self.path_for(&record.file.storage_path);
        let ciphertext = fs::read(encrypted_path)?;
        let checksum = sha256_hex(&ciphertext);
        if checksum != record.file.checksum_sha256 {
            return Err(Error::Validation(
                mizan_core::errors::ValidationError::InvalidInput(
                    "Encrypted document checksum mismatch".into(),
                ),
            ));
        }
        let nonce = parse_nonce_hex(&record.file.nonce)?;
        decrypt_bytes(&self.key, &nonce, &ciphertext)
    }

    pub async fn delete(&self, document_id: &str) -> Result<()> {
        let record = self.get_metadata(document_id)?;
        let storage_path = record.file.storage_path.clone();
        let document_id = document_id.to_string();
        self.writer
            .exec(move |conn| -> Result<()> {
                diesel::delete(documents::table.find(&document_id))
                    .execute(conn)
                    .map_err(StorageError::from)?;
                Ok(())
            })
            .await?;
        let _ = fs::remove_file(self.path_for(&storage_path));
        Ok(())
    }

    pub fn encrypted_path_for_metadata(&self, storage_path: &str) -> PathBuf {
        self.path_for(storage_path)
    }

    fn find_by_hash(&self, hash: &str) -> Result<Option<DocumentMetadata>> {
        let mut conn = get_connection(&self.pool)?;
        let row = documents::table
            .filter(documents::file_hash.eq(hash))
            .first::<DocumentRow>(&mut conn)
            .optional()
            .map_err(StorageError::from)?;
        row.map(DocumentMetadata::try_from).transpose()
    }

    fn path_for(&self, storage_path: &str) -> PathBuf {
        self.vault_dir.join(Path::new(storage_path))
    }
}

struct EncryptedBytes {
    nonce: [u8; NONCE_LEN],
    ciphertext: Vec<u8>,
}

fn encrypt_bytes(key: &[u8; 32], plaintext: &[u8]) -> Result<EncryptedBytes> {
    let mut nonce = [0_u8; NONCE_LEN];
    rand::thread_rng().fill_bytes(&mut nonce);
    let cipher = ChaCha20Poly1305::new_from_slice(key).map_err(|err| {
        Error::Validation(mizan_core::errors::ValidationError::InvalidInput(
            err.to_string(),
        ))
    })?;
    let ciphertext = cipher
        .encrypt(Nonce::from_slice(&nonce), plaintext)
        .map_err(|_| Error::Unexpected("Document encryption failed".into()))?;
    Ok(EncryptedBytes { nonce, ciphertext })
}

fn decrypt_bytes(key: &[u8; 32], nonce: &[u8; NONCE_LEN], ciphertext: &[u8]) -> Result<Vec<u8>> {
    let cipher = ChaCha20Poly1305::new_from_slice(key).map_err(|err| {
        Error::Validation(mizan_core::errors::ValidationError::InvalidInput(
            err.to_string(),
        ))
    })?;
    cipher
        .decrypt(Nonce::from_slice(nonce), ciphertext)
        .map_err(|_| Error::Unexpected("Document decryption failed".into()))
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    format!("{digest:x}")
}

fn parse_nonce_hex(value: &str) -> Result<[u8; NONCE_LEN]> {
    let decoded = hex::decode(value).map_err(|err| {
        Error::Validation(mizan_core::errors::ValidationError::InvalidInput(
            err.to_string(),
        ))
    })?;
    if decoded.len() != NONCE_LEN {
        return Err(Error::Validation(
            mizan_core::errors::ValidationError::InvalidInput(format!(
                "Invalid document nonce length: expected {NONCE_LEN}, got {}",
                decoded.len()
            )),
        ));
    }
    let mut nonce = [0_u8; NONCE_LEN];
    nonce.copy_from_slice(&decoded);
    Ok(nonce)
}

fn parse_rfc3339(value: &str) -> Result<DateTime<Utc>> {
    Ok(DateTime::parse_from_rfc3339(value)?.with_timezone(&Utc))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::write_actor;
    use crate::db::{create_pool, init, run_migrations};
    use tempfile::tempdir;

    struct TestVault {
        repo: DocumentVaultRepository,
        _app_data: tempfile::TempDir,
    }

    fn request(name: &str, content: &[u8]) -> UploadDocumentRequest {
        UploadDocumentRequest {
            original_name: name.to_string(),
            mime_type: "application/pdf".to_string(),
            content: content.to_vec(),
            source_type: None,
        }
    }

    fn setup() -> TestVault {
        let app_data = tempdir().expect("tempdir");
        let db_path = init(app_data.path().to_str().expect("path")).expect("init");
        run_migrations(&db_path).expect("migrate");
        let pool = create_pool(&db_path).expect("pool");
        let writer = write_actor::spawn_writer(pool.as_ref().clone()).expect("writer");
        let key = parse_document_vault_key_hex(&generate_document_vault_key_hex()).expect("key");
        let repo =
            DocumentVaultRepository::new(pool, writer, app_data.path().join("document-vault"), key)
                .expect("repo");
        TestVault {
            repo,
            _app_data: app_data,
        }
    }

    #[tokio::test]
    async fn duplicate_documents_are_rejected() {
        let vault = setup();
        vault
            .repo
            .upload(request("statement.pdf", b"same-bytes"))
            .await
            .expect("first upload");
        let err = vault
            .repo
            .upload(request("copy.pdf", b"same-bytes"))
            .await
            .expect_err("duplicate");
        assert!(err.to_string().contains("Duplicate document"));
    }

    #[tokio::test]
    async fn encrypted_file_exists_without_plaintext() {
        let vault = setup();
        let record = vault
            .repo
            .upload(request("statement.pdf", b"plain document bytes"))
            .await
            .expect("upload");
        let path = vault
            .repo
            .encrypted_path_for_metadata(&record.file.storage_path);
        assert!(path.exists());
        let encrypted = fs::read(path).expect("read encrypted");
        assert_ne!(encrypted, b"plain document bytes");
    }

    #[tokio::test]
    async fn decrypt_round_trip_returns_original_bytes() {
        let vault = setup();
        let record = vault
            .repo
            .upload(request("statement.pdf", b"round trip bytes"))
            .await
            .expect("upload");
        let decrypted = vault
            .repo
            .read_decrypted(&record.document.id)
            .expect("decrypt");
        assert_eq!(decrypted, b"round trip bytes");
    }

    #[tokio::test]
    async fn delete_removes_encrypted_file_and_metadata() {
        let vault = setup();
        let record = vault
            .repo
            .upload(request("statement.pdf", b"delete me"))
            .await
            .expect("upload");
        let path = vault
            .repo
            .encrypted_path_for_metadata(&record.file.storage_path);
        vault
            .repo
            .delete(&record.document.id)
            .await
            .expect("delete");
        assert!(!path.exists());
        assert!(vault.repo.get_metadata(&record.document.id).is_err());
    }

    #[tokio::test]
    async fn metadata_is_persisted() {
        let vault = setup();
        let record = vault
            .repo
            .upload(request("broker-statement.pdf", b"metadata"))
            .await
            .expect("upload");
        let listed = vault.repo.list().expect("list");
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].id, record.document.id);
        assert_eq!(listed[0].original_name, "broker-statement.pdf");
        assert_eq!(listed[0].mime_type, "application/pdf");
        assert_eq!(listed[0].status, DocumentStatus::Ingested);
        assert_eq!(listed[0].file_size_bytes, 8);
    }
}
