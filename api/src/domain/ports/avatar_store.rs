use async_trait::async_trait;

use crate::error::AppResult;

/// Metadata about a stored user avatar. `updated_at` is a millisecond epoch
/// timestamp that clients can use as a cache-busting version.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AvatarMeta {
    pub mime_type: String,
    pub size_bytes: u64,
    pub updated_at: i64,
}

/// Stores a single profile avatar per user. Implementations own validation
/// (image type, size) and at-rest handling of the bytes.
#[async_trait]
pub trait AvatarStore: Send + Sync {
    /// Persist (replacing any existing) the avatar for `user_id`.
    async fn save(&self, user_id: &str, mime_type: &str, data: &[u8]) -> AppResult<AvatarMeta>;

    /// Read the decrypted avatar bytes and mime type, if one is set.
    async fn read(&self, user_id: &str) -> AppResult<Option<(String, Vec<u8>)>>;

    /// Read only the avatar metadata, if one is set.
    async fn meta(&self, user_id: &str) -> AppResult<Option<AvatarMeta>>;
}
