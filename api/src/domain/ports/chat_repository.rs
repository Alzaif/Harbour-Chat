use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::Serialize;
use std::collections::HashMap;

use crate::domain::entities::{
    Channel, ChannelType, Member, MemberRole, Message, Presence, PresenceStatus, ReactionSummary,
    Server, TypingIndicator, User, VoiceParticipant,
};
use crate::error::AppResult;

#[derive(Clone, Debug, Serialize)]
pub struct ServerDetail {
    pub server: Server,
    pub channels: Vec<Channel>,
    #[serde(rename = "unreadByChannelId")]
    pub unread_by_channel_id: HashMap<String, u32>,
}

#[async_trait]
pub trait ChatRepository: Send + Sync {
    async fn list_servers_for_user(&self, user_id: &str) -> AppResult<Vec<Server>>;
    async fn get_server(&self, server_id: &str) -> AppResult<Option<Server>>;
    async fn get_server_detail(&self, server_id: &str) -> AppResult<Option<ServerDetail>>;
    async fn create_server(&self, name: &str, owner: &User) -> AppResult<Server>;
    async fn get_member(&self, server_id: &str, user_id: &str) -> AppResult<Option<Member>>;
    async fn add_member(
        &self,
        server_id: &str,
        user_id: &str,
        role: MemberRole,
    ) -> AppResult<Member>;
    async fn list_members(&self, server_id: &str) -> AppResult<Vec<Member>>;
    async fn create_channel(
        &self,
        server_id: &str,
        name: &str,
        channel_type: ChannelType,
    ) -> AppResult<Channel>;
    async fn get_channel(&self, channel_id: &str) -> AppResult<Option<Channel>>;
    async fn list_channels_for_server(&self, server_id: &str) -> AppResult<Vec<Channel>>;
    async fn list_messages(
        &self,
        channel_id: &str,
        before: Option<&str>,
        limit: u32,
    ) -> AppResult<Vec<Message>>;
    async fn insert_message(
        &self,
        channel_id: &str,
        author: &User,
        content: &str,
    ) -> AppResult<Message>;
    async fn get_message(&self, message_id: &str) -> AppResult<Option<Message>>;
    async fn update_message_content(
        &self,
        message_id: &str,
        content: &str,
        edited_at: DateTime<Utc>,
    ) -> AppResult<Message>;
    async fn soft_delete_message(
        &self,
        message_id: &str,
        deleted_at: DateTime<Utc>,
    ) -> AppResult<Message>;
    async fn toggle_reaction(
        &self,
        message_id: &str,
        user_id: &str,
        emoji: &str,
    ) -> AppResult<bool>;
    async fn list_reactions_for_messages(
        &self,
        message_ids: &[String],
    ) -> AppResult<HashMap<String, Vec<ReactionSummary>>>;
    async fn unread_counts_for_server(
        &self,
        server_id: &str,
        user_id: &str,
    ) -> AppResult<HashMap<String, u32>>;
    async fn mark_read(&self, channel_id: &str, user_id: &str, message_id: &str) -> AppResult<()>;
    async fn find_or_create_dm(&self, user_a: &str, user_b: &str) -> AppResult<Channel>;
    async fn is_dm_participant(&self, channel_id: &str, user_id: &str) -> AppResult<bool>;
    async fn upsert_presence(
        &self,
        server_id: &str,
        user_id: &str,
        status: PresenceStatus,
        updated_at: DateTime<Utc>,
    ) -> AppResult<Presence>;
    async fn list_presence(&self, server_id: &str) -> AppResult<Vec<Presence>>;
    async fn upsert_typing(
        &self,
        channel_id: &str,
        user_id: &str,
        expires_at: DateTime<Utc>,
    ) -> AppResult<TypingIndicator>;
    async fn delete_typing(&self, channel_id: &str, user_id: &str) -> AppResult<()>;
    async fn list_typing(&self, channel_id: &str, now: DateTime<Utc>) -> AppResult<Vec<TypingIndicator>>;
    async fn upsert_voice_participant(
        &self,
        channel_id: &str,
        user_id: &str,
        muted: bool,
        deafened: bool,
        updated_at: DateTime<Utc>,
    ) -> AppResult<VoiceParticipant>;
    async fn delete_voice_participant(&self, channel_id: &str, user_id: &str) -> AppResult<()>;
    async fn list_voice_participants(&self, channel_id: &str) -> AppResult<Vec<VoiceParticipant>>;
}
