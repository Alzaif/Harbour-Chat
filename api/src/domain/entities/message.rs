use chrono::{DateTime, Utc};
use serde::Serialize;

use super::{MessageAttachment, ReactionSummary};

#[derive(Clone, Debug, Serialize)]
pub struct Message {
    pub id: String,
    pub channel_id: String,
    pub author_user_id: String,
    pub author_display_name: Option<String>,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub edited_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub reactions: Vec<ReactionSummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attachment: Option<MessageAttachment>,
}
