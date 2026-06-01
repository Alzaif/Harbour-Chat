use std::collections::HashMap;
use std::path::PathBuf;

use async_trait::async_trait;
use infer::Infer;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::domain::entities::MessageAttachment;
use crate::domain::ports::AttachmentStore;
use crate::error::{AppError, AppResult};
use crate::infrastructure::security::EnvelopeCrypto;

const ALLOWED_MIME: &[&str] = &[
    "image/jpeg",
    "image/png",
    "image/gif",
    "image/webp",
    "application/pdf",
];

const EICAR_MARKER: &str = "X5O!P%@AP[4\\PZX54(P^)7CC)7}$EICAR-STANDARD-ANTIVIRUS-TEST-FILE!$H+H*";

pub struct LocalAttachmentStore {
    pool: SqlitePool,
    data_dir: PathBuf,
    max_bytes: u64,
    crypto: EnvelopeCrypto,
    quarantine_suspicious: bool,
}

impl LocalAttachmentStore {
    pub fn new(
        pool: SqlitePool,
        data_dir: PathBuf,
        max_bytes: u64,
        crypto: EnvelopeCrypto,
        quarantine_suspicious: bool,
    ) -> Self {
        Self {
            pool,
            data_dir,
            max_bytes,
            crypto,
            quarantine_suspicious,
        }
    }

    fn file_path(&self, storage_key: &str) -> PathBuf {
        self.data_dir.join("attachments").join(storage_key)
    }

    fn quarantine_path(&self, storage_key: &str) -> PathBuf {
        self.data_dir.join("attachments-quarantine").join(storage_key)
    }

    fn validate_mime(mime: &str) -> AppResult<()> {
        if ALLOWED_MIME.contains(&mime) {
            Ok(())
        } else {
            Err(AppError::Validation(format!("mime type not allowed: {mime}")))
        }
    }

    fn detect_mime(data: &[u8]) -> Option<&'static str> {
        let infer = Infer::new();
        infer.get(data).map(|k| k.mime_type())
    }

    fn is_eicar(data: &[u8]) -> bool {
        std::str::from_utf8(data)
            .map(|s| s.contains(EICAR_MARKER))
            .unwrap_or(false)
    }

    fn normalize_filename(name: &str) -> String {
        let mut out = name
            .chars()
            .map(|c| if matches!(c, '/' | '\\' | '\0') { '_' } else { c })
            .collect::<String>();
        if out.is_empty() {
            out = "upload.bin".to_string();
        }
        if out.len() > 200 {
            out.truncate(200);
        }
        out
    }
}

#[async_trait]
impl AttachmentStore for LocalAttachmentStore {
    async fn save(
        &self,
        message_id: &str,
        filename: &str,
        mime_type: &str,
        data: &[u8],
    ) -> AppResult<MessageAttachment> {
        if data.len() as u64 > self.max_bytes {
            return Err(AppError::Validation(format!(
                "attachment exceeds max size of {} bytes",
                self.max_bytes
            )));
        }
        Self::validate_mime(mime_type)?;

        let detected = Self::detect_mime(data)
            .ok_or_else(|| AppError::Validation("attachment type not recognized".into()))?;
        if mime_type != detected {
            return Err(AppError::Validation(format!(
                "attachment content type mismatch: declared={mime_type} detected={detected}"
            )));
        }

        let id = Uuid::new_v4().to_string();
        let storage_key = Uuid::new_v4().to_string();
        let now = chrono::Utc::now().timestamp_millis();
        let safe_filename = Self::normalize_filename(filename);

        let mut dir = self.data_dir.join("attachments");
        if self.quarantine_suspicious && Self::is_eicar(data) {
            dir = self.data_dir.join("attachments-quarantine");
            tokio::fs::create_dir_all(&dir)
                .await
                .map_err(|e| AppError::Internal(e.to_string()))?;
            let quarantined_path = self.quarantine_path(&storage_key);
            let encrypted = self.crypto.encrypt_bytes(data)?;
            tokio::fs::write(quarantined_path, encrypted)
                .await
                .map_err(|e| AppError::Internal(e.to_string()))?;
            return Err(AppError::Validation("attachment flagged by malware detector".into()));
        }

        tokio::fs::create_dir_all(&dir)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;

        let encrypted = self.crypto.encrypt_bytes(data)?;
        tokio::fs::write(self.file_path(&storage_key), encrypted)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;

        sqlx::query(
            "INSERT INTO attachments (id, message_id, storage_key, filename, mime_type, size_bytes, created_at) VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(message_id)
        .bind(&storage_key)
        .bind(&safe_filename)
        .bind(mime_type)
        .bind(data.len() as i64)
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        Ok(MessageAttachment {
            id,
            filename: safe_filename,
            mime_type: mime_type.to_string(),
            size_bytes: data.len() as u64,
        })
    }

    async fn find(&self, id: &str) -> AppResult<Option<MessageAttachment>> {
        let row = sqlx::query_as::<_, (String, String, String, i64)>(
            "SELECT id, filename, mime_type, size_bytes FROM attachments WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        Ok(row.map(|(id, filename, mime_type, size_bytes)| MessageAttachment {
            id,
            filename,
            mime_type,
            size_bytes: size_bytes as u64,
        }))
    }

    async fn find_message_id(&self, attachment_id: &str) -> AppResult<Option<String>> {
        let row = sqlx::query_as::<_, (String,)>(
            "SELECT message_id FROM attachments WHERE id = ?",
        )
        .bind(attachment_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
        Ok(row.map(|(m,)| m))
    }

    async fn read_content(&self, id: &str) -> AppResult<Option<Vec<u8>>> {
        let row = sqlx::query_as::<_, (String,)>(
            "SELECT storage_key FROM attachments WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        let Some((storage_key,)) = row else {
            return Ok(None);
        };
        match tokio::fs::read(self.file_path(&storage_key)).await {
            Ok(bytes) => Ok(Some(self.crypto.decrypt_bytes(&bytes)?)),
            Err(_) => Ok(None),
        }
    }

    async fn list_for_messages(
        &self,
        message_ids: &[String],
    ) -> AppResult<HashMap<String, MessageAttachment>> {
        if message_ids.is_empty() {
            return Ok(HashMap::new());
        }
        let placeholders = message_ids
            .iter()
            .map(|_| "?")
            .collect::<Vec<_>>()
            .join(",");
        let sql = format!(
            "SELECT message_id, id, filename, mime_type, size_bytes FROM attachments WHERE message_id IN ({placeholders})"
        );
        let mut q = sqlx::query_as::<_, (String, String, String, String, i64)>(&sql);
        for id in message_ids {
            q = q.bind(id);
        }
        let rows = q
            .fetch_all(&self.pool)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;

        let mut map = HashMap::new();
        for (message_id, id, filename, mime_type, size_bytes) in rows {
            map.insert(
                message_id,
                MessageAttachment {
                    id,
                    filename,
                    mime_type,
                    size_bytes: size_bytes as u64,
                },
            );
        }
        Ok(map)
    }
}
