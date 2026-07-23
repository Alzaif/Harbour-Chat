use async_trait::async_trait;
use serde::Serialize;

use crate::domain::entities::PresenceStatus;
use crate::error::AppResult;

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RealtimeEvent {
    PostCreated {
        post: serde_json::Value,
    },
    MessageCreated {
        message: serde_json::Value,
    },
    MessageUpdated {
        message: serde_json::Value,
    },
    MessageDeleted {
        message_id: String,
        channel_id: String,
    },
    ReactionUpdated {
        message_id: String,
        channel_id: String,
    },
    TypingStarted {
        channel_id: String,
        user_id: String,
        display_name: Option<String>,
        expires_at: String,
    },
    TypingStopped {
        channel_id: String,
        user_id: String,
    },
    PresenceChanged {
        server_id: String,
        user_id: String,
        status: PresenceStatus,
        updated_at: String,
    },
    VoiceParticipantUpdated {
        channel_id: String,
        user_id: String,
        display_name: Option<String>,
        connected: bool,
        muted: bool,
        deafened: bool,
        updated_at: String,
    },
}

#[async_trait]
pub trait RealtimePublisher: Send + Sync {
    async fn publish(&self, channel_id: &str, event: RealtimeEvent) -> AppResult<()>;
}
