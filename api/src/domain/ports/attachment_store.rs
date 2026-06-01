use async_trait::async_trait;
use std::collections::HashMap;

use crate::domain::entities::MessageAttachment;
use crate::error::AppResult;

#[async_trait]
pub trait AttachmentStore: Send + Sync {
    async fn save(
        &self,
        message_id: &str,
        filename: &str,
        mime_type: &str,
        data: &[u8],
    ) -> AppResult<MessageAttachment>;

    async fn find(&self, id: &str) -> AppResult<Option<MessageAttachment>>;

    async fn find_message_id(&self, attachment_id: &str) -> AppResult<Option<String>>;

    async fn read_content(&self, id: &str) -> AppResult<Option<Vec<u8>>>;

    async fn list_for_messages(
        &self,
        message_ids: &[String],
    ) -> AppResult<HashMap<String, MessageAttachment>>;
}
