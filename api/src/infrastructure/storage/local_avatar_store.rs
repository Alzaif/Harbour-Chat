use std::path::PathBuf;

use async_trait::async_trait;
use infer::Infer;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::domain::ports::{AvatarMeta, AvatarStore};
use crate::error::{AppError, AppResult};
use crate::infrastructure::security::EnvelopeCrypto;

const ALLOWED_MIME: &[&str] = &["image/jpeg", "image/png", "image/gif", "image/webp"];

/// Filesystem-backed avatar store: bytes are encrypted at rest under
/// `{data_dir}/avatars/{storage_key}` with metadata in the `user_avatars` table.
/// Mirrors `LocalAttachmentStore` but is keyed by user and restricted to images.
pub struct LocalAvatarStore {
    pool: SqlitePool,
    data_dir: PathBuf,
    max_bytes: u64,
    crypto: EnvelopeCrypto,
}

impl LocalAvatarStore {
    pub fn new(pool: SqlitePool, data_dir: PathBuf, max_bytes: u64, crypto: EnvelopeCrypto) -> Self {
        Self {
            pool,
            data_dir,
            max_bytes,
            crypto,
        }
    }

    fn file_path(&self, storage_key: &str) -> PathBuf {
        self.data_dir.join("avatars").join(storage_key)
    }

    fn validate_mime(mime: &str) -> AppResult<()> {
        if ALLOWED_MIME.contains(&mime) {
            Ok(())
        } else {
            Err(AppError::Validation(format!(
                "avatar must be an image (got {mime})"
            )))
        }
    }

    fn detect_mime(data: &[u8]) -> Option<&'static str> {
        Infer::new().get(data).map(|k| k.mime_type())
    }
}

#[async_trait]
impl AvatarStore for LocalAvatarStore {
    async fn save(&self, user_id: &str, mime_type: &str, data: &[u8]) -> AppResult<AvatarMeta> {
        if data.is_empty() {
            return Err(AppError::Validation("avatar is empty".into()));
        }
        if data.len() as u64 > self.max_bytes {
            return Err(AppError::Validation(format!(
                "avatar exceeds max size of {} bytes",
                self.max_bytes
            )));
        }
        Self::validate_mime(mime_type)?;

        let detected = Self::detect_mime(data)
            .ok_or_else(|| AppError::Validation("avatar type not recognized".into()))?;
        if mime_type != detected {
            return Err(AppError::Validation(format!(
                "avatar content type mismatch: declared={mime_type} detected={detected}"
            )));
        }

        let dir = self.data_dir.join("avatars");
        tokio::fs::create_dir_all(&dir)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;

        let storage_key = Uuid::new_v4().to_string();
        let encrypted = self.crypto.encrypt_bytes(data)?;
        tokio::fs::write(self.file_path(&storage_key), encrypted)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;

        // Remove the previous file (best effort) so replaced avatars don't leak.
        let previous = sqlx::query_as::<_, (String,)>(
            "SELECT storage_key FROM user_avatars WHERE user_id = ?",
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        let now = chrono::Utc::now().timestamp_millis();
        sqlx::query(
            r#"
            INSERT INTO user_avatars (user_id, storage_key, mime_type, size_bytes, updated_at)
            VALUES (?, ?, ?, ?, ?)
            ON CONFLICT(user_id) DO UPDATE SET
                storage_key = excluded.storage_key,
                mime_type = excluded.mime_type,
                size_bytes = excluded.size_bytes,
                updated_at = excluded.updated_at
            "#,
        )
        .bind(user_id)
        .bind(&storage_key)
        .bind(mime_type)
        .bind(data.len() as i64)
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        if let Some((old_key,)) = previous {
            let _ = tokio::fs::remove_file(self.file_path(&old_key)).await;
        }

        Ok(AvatarMeta {
            mime_type: mime_type.to_string(),
            size_bytes: data.len() as u64,
            updated_at: now,
        })
    }

    async fn read(&self, user_id: &str) -> AppResult<Option<(String, Vec<u8>)>> {
        let row = sqlx::query_as::<_, (String, String)>(
            "SELECT storage_key, mime_type FROM user_avatars WHERE user_id = ?",
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        let Some((storage_key, mime_type)) = row else {
            return Ok(None);
        };
        match tokio::fs::read(self.file_path(&storage_key)).await {
            Ok(bytes) => Ok(Some((mime_type, self.crypto.decrypt_bytes(&bytes)?))),
            Err(_) => Ok(None),
        }
    }

    async fn meta(&self, user_id: &str) -> AppResult<Option<AvatarMeta>> {
        let row = sqlx::query_as::<_, (String, i64, i64)>(
            "SELECT mime_type, size_bytes, updated_at FROM user_avatars WHERE user_id = ?",
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        Ok(row.map(|(mime_type, size_bytes, updated_at)| AvatarMeta {
            mime_type,
            size_bytes: size_bytes as u64,
            updated_at,
        }))
    }
}
