use std::sync::Arc;

use chrono::{Duration, Utc};
use serde_json::json;

use crate::domain::board_feed::BOARD_FEED_TOPIC;
use crate::domain::entities::{
    Channel, ChannelType, Member, MemberRole, Message, Presence, PresenceStatus, ReplyPreview,
    Server, TypingIndicator, User, VoiceParticipant,
};
use crate::domain::ports::{
    AttachmentStore, ChatRepository, RealtimeEvent, RealtimePublisher, ServerDetail, VoiceMediaPort,
};
use crate::domain::MAX_MESSAGE_LENGTH;
use crate::error::{AppError, AppResult};
use crate::infrastructure::security::{AuditLogger, EnvelopeCrypto};

pub const HARBOUR_HOME_SERVER_ID: &str = "00000000-0000-4000-8000-000000000001";

pub struct ChatService {
    chat: Arc<dyn ChatRepository>,
    realtime: Arc<dyn RealtimePublisher>,
    attachments: Arc<dyn AttachmentStore>,
    voice_media: Arc<dyn VoiceMediaPort>,
    crypto: EnvelopeCrypto,
    audit: AuditLogger,
}

impl ChatService {
    pub fn new(
        chat: Arc<dyn ChatRepository>,
        realtime: Arc<dyn RealtimePublisher>,
        attachments: Arc<dyn AttachmentStore>,
        voice_media: Arc<dyn VoiceMediaPort>,
        crypto: EnvelopeCrypto,
        audit: AuditLogger,
    ) -> Self {
        Self {
            chat,
            realtime,
            attachments,
            voice_media,
            crypto,
            audit,
        }
    }

    pub async fn list_servers(&self, user: &User) -> AppResult<Vec<Server>> {
        self.chat.list_servers_for_user(&user.id).await
    }

    pub async fn get_server_detail(&self, user: &User, server_id: &str) -> AppResult<ServerDetail> {
        if self.chat.get_server(server_id).await?.is_none() {
            return Err(AppError::NotFound("server not found".into()));
        }
        self.require_member(server_id, &user.id).await?;
        let mut detail = self
            .chat
            .get_server_detail(server_id)
            .await?
            .ok_or_else(|| AppError::NotFound("server not found".into()))?;
        detail.unread_by_channel_id = self
            .chat
            .unread_counts_for_server(server_id, &user.id)
            .await?;
        Ok(detail)
    }

    pub async fn create_server(
        &self,
        user: &User,
        name: &str,
        description: Option<&str>,
    ) -> AppResult<Server> {
        let name = name.trim();
        if name.is_empty() {
            return Err(AppError::Validation("name is required".into()));
        }
        let mut server = self.chat.create_server(name, user).await?;
        if let Some(desc) = description.map(str::trim).filter(|d| !d.is_empty()) {
            server = self
                .chat
                .update_server(&server.id, None, Some(Some(desc)), None, None)
                .await?;
        }
        self.chat
            .add_member(&server.id, &user.id, MemberRole::Owner)
            .await?;
        self.chat
            .create_channel(&server.id, "general", ChannelType::Text)
            .await?;
        Ok(server)
    }

    pub async fn update_server(
        &self,
        user: &User,
        server_id: &str,
        name: Option<&str>,
        description: Option<Option<&str>>,
        icon_url: Option<Option<&str>>,
        card_color: Option<Option<&str>>,
    ) -> AppResult<Server> {
        let member = self.require_member(server_id, &user.id).await?;
        if !member.role.can_moderate() {
            return Err(AppError::Forbidden);
        }
        if let Some(name) = name {
            if name.trim().is_empty() {
                return Err(AppError::Validation("name is required".into()));
            }
        }
        self.chat
            .update_server(
                server_id,
                name.map(str::trim),
                description,
                icon_url,
                card_color,
            )
            .await
    }

    pub async fn delete_server(&self, user: &User, server_id: &str) -> AppResult<()> {
        if server_id == HARBOUR_HOME_SERVER_ID {
            return Err(AppError::Forbidden);
        }
        let server = self
            .chat
            .get_server(server_id)
            .await?
            .ok_or_else(|| AppError::NotFound("server not found".into()))?;
        self.require_member(server_id, &user.id).await?;
        if server.owner_user_id != user.id {
            return Err(AppError::Forbidden);
        }
        self.chat.delete_server(server_id).await
    }

    pub async fn add_member(
        &self,
        actor: &User,
        server_id: &str,
        user_id: &str,
    ) -> AppResult<Member> {
        let actor_member = self.require_member(server_id, &actor.id).await?;
        if !actor_member.role.can_moderate() {
            return Err(AppError::Forbidden);
        }
        if user_id.trim().is_empty() {
            return Err(AppError::Validation("userId is required".into()));
        }
        self.chat
            .add_member(server_id, user_id, MemberRole::Member)
            .await
    }

    pub async fn list_members(&self, user: &User, server_id: &str) -> AppResult<Vec<Member>> {
        self.require_member(server_id, &user.id).await?;
        self.chat.list_members(server_id).await
    }

    pub async fn create_channel(
        &self,
        user: &User,
        server_id: &str,
        name: &str,
        channel_type: ChannelType,
    ) -> AppResult<Channel> {
        let member = self.require_member(server_id, &user.id).await?;
        // Harbour Home is a shared workspace where members can create channels.
        // Other servers keep moderator-only channel creation.
        if server_id != HARBOUR_HOME_SERVER_ID && !member.role.can_moderate() {
            return Err(AppError::Forbidden);
        }
        let name = name.trim();
        if name.is_empty() {
            return Err(AppError::Validation("name is required".into()));
        }
        self.chat
            .create_channel(server_id, name, channel_type)
            .await
    }

    pub async fn list_messages(
        &self,
        user: &User,
        channel_id: &str,
        before: Option<&str>,
        limit: u32,
    ) -> AppResult<Vec<Message>> {
        self.require_channel_access(user, channel_id).await?;
        let limit = limit.clamp(1, 100);
        let mut messages = self.chat.list_messages(channel_id, before, limit).await?;
        self.enrich_messages(&mut messages).await?;
        self.decrypt_messages(&mut messages)?;
        Ok(messages)
    }

    pub async fn send_message(
        &self,
        user: &User,
        channel_id: &str,
        content: &str,
        reply_to_message_id: Option<&str>,
    ) -> AppResult<Message> {
        self.require_channel_access(user, channel_id).await?;
        let content = content.trim();
        if content.is_empty() {
            return Err(AppError::Validation("content is required".into()));
        }
        if content.len() > MAX_MESSAGE_LENGTH {
            return Err(AppError::Validation(format!(
                "content must be at most {MAX_MESSAGE_LENGTH} characters"
            )));
        }
        if let Some(reply_id) = reply_to_message_id {
            let parent = self
                .chat
                .get_message(reply_id)
                .await?
                .ok_or_else(|| AppError::NotFound("reply target not found".into()))?;
            if parent.channel_id != channel_id {
                return Err(AppError::Validation(
                    "reply must reference a message in the same channel".into(),
                ));
            }
        }
        let encrypted = self.crypto.encrypt_text(content)?;
        let mut message = self
            .chat
            .insert_message(channel_id, user, &encrypted, reply_to_message_id)
            .await?;
        self.enrich_messages(std::slice::from_mut(&mut message)).await?;
        self.decrypt_messages(std::slice::from_mut(&mut message))?;
        self.audit
            .log(
                "message.sent",
                Some(&user.id),
                Some("channel"),
                Some(channel_id),
                json!({ "messageId": message.id }),
            )
            .await?;
        let payload =
            serde_json::to_value(&message).map_err(|e| AppError::Internal(e.to_string()))?;
        self.realtime
            .publish(
                channel_id,
                RealtimeEvent::MessageCreated { message: payload },
            )
            .await?;
        Ok(message)
    }

    pub async fn edit_message(
        &self,
        user: &User,
        message_id: &str,
        content: &str,
    ) -> AppResult<Message> {
        let content = content.trim();
        if content.is_empty() {
            return Err(AppError::Validation("content is required".into()));
        }
        if content.len() > MAX_MESSAGE_LENGTH {
            return Err(AppError::Validation(format!(
                "content must be at most {MAX_MESSAGE_LENGTH} characters"
            )));
        }
        let existing = self
            .chat
            .get_message(message_id)
            .await?
            .ok_or_else(|| AppError::NotFound("message not found".into()))?;
        if existing.deleted_at.is_some() {
            return Err(AppError::Conflict("message is deleted".into()));
        }
        self.require_channel_access(user, &existing.channel_id)
            .await?;
        if existing.author_user_id != user.id
            && !self
                .can_moderate_channel(user, &existing.channel_id)
                .await?
        {
            return Err(AppError::Forbidden);
        }
        let mut updated = self
            .chat
            .update_message_content(
                message_id,
                &self.crypto.encrypt_text(content)?,
                Utc::now(),
            )
            .await?;
        self.enrich_messages(std::slice::from_mut(&mut updated))
            .await?;
        self.decrypt_messages(std::slice::from_mut(&mut updated))?;
        self.audit
            .log(
                "message.edited",
                Some(&user.id),
                Some("message"),
                Some(message_id),
                json!({ "channelId": updated.channel_id }),
            )
            .await?;
        let payload =
            serde_json::to_value(&updated).map_err(|e| AppError::Internal(e.to_string()))?;
        self.realtime
            .publish(
                &updated.channel_id,
                RealtimeEvent::MessageUpdated { message: payload },
            )
            .await?;
        Ok(updated)
    }

    pub async fn delete_message(&self, user: &User, message_id: &str) -> AppResult<Message> {
        let existing = self
            .chat
            .get_message(message_id)
            .await?
            .ok_or_else(|| AppError::NotFound("message not found".into()))?;
        if existing.deleted_at.is_some() {
            return Ok(existing);
        }
        self.require_channel_access(user, &existing.channel_id)
            .await?;
        if existing.author_user_id != user.id
            && !self
                .can_moderate_channel(user, &existing.channel_id)
                .await?
        {
            return Err(AppError::Forbidden);
        }
        let mut deleted = self
            .chat
            .soft_delete_message(message_id, Utc::now())
            .await?;
        self.enrich_messages(std::slice::from_mut(&mut deleted))
            .await?;
        self.decrypt_messages(std::slice::from_mut(&mut deleted))?;
        self.audit
            .log(
                "message.deleted",
                Some(&user.id),
                Some("message"),
                Some(message_id),
                json!({ "channelId": deleted.channel_id }),
            )
            .await?;
        self.realtime
            .publish(
                &deleted.channel_id,
                RealtimeEvent::MessageDeleted {
                    message_id: deleted.id.clone(),
                    channel_id: deleted.channel_id.clone(),
                },
            )
            .await?;
        Ok(deleted)
    }

    pub async fn toggle_reaction(
        &self,
        user: &User,
        message_id: &str,
        emoji: &str,
    ) -> AppResult<bool> {
        let emoji = emoji.trim();
        if emoji.is_empty() {
            return Err(AppError::Validation("emoji is required".into()));
        }
        let message = self
            .chat
            .get_message(message_id)
            .await?
            .ok_or_else(|| AppError::NotFound("message not found".into()))?;
        self.require_channel_access(user, &message.channel_id)
            .await?;
        let added = self
            .chat
            .toggle_reaction(message_id, &user.id, emoji)
            .await?;
        self.realtime
            .publish(
                &message.channel_id,
                RealtimeEvent::ReactionUpdated {
                    message_id: message_id.into(),
                    channel_id: message.channel_id.clone(),
                },
            )
            .await?;
        Ok(added)
    }

    pub async fn upload_attachment(
        &self,
        user: &User,
        message_id: &str,
        filename: &str,
        mime_type: &str,
        data: &[u8],
    ) -> AppResult<Message> {
        let message = self
            .chat
            .get_message(message_id)
            .await?
            .ok_or_else(|| AppError::NotFound("message not found".into()))?;
        if message.author_user_id != user.id {
            return Err(AppError::Forbidden);
        }
        self.require_channel_access(user, &message.channel_id)
            .await?;
        let attachment = self
            .attachments
            .save(message_id, filename, mime_type, data)
            .await?;
        let mut updated = message;
        updated.content = self.crypto.decrypt_text(&updated.content)?;
        updated.attachment = Some(attachment);
        self.audit
            .log(
                "attachment.uploaded",
                Some(&user.id),
                Some("message"),
                Some(message_id),
                json!({ "mimeType": mime_type, "sizeBytes": data.len() }),
            )
            .await?;
        let payload =
            serde_json::to_value(&updated).map_err(|e| AppError::Internal(e.to_string()))?;
        self.realtime
            .publish(
                &updated.channel_id,
                RealtimeEvent::MessageUpdated { message: payload },
            )
            .await?;
        Ok(updated)
    }

    pub async fn get_attachment(
        &self,
        user: &User,
        attachment_id: &str,
    ) -> AppResult<(String, Vec<u8>)> {
        let message_id = self
            .attachments
            .find_message_id(attachment_id)
            .await?
            .ok_or_else(|| AppError::NotFound("attachment not found".into()))?;
        let message = self
            .chat
            .get_message(&message_id)
            .await?
            .ok_or_else(|| AppError::NotFound("message not found".into()))?;
        self.require_channel_access(user, &message.channel_id)
            .await?;
        let meta = self
            .attachments
            .find(attachment_id)
            .await?
            .ok_or_else(|| AppError::NotFound("attachment not found".into()))?;
        let bytes = self
            .attachments
            .read_content(attachment_id)
            .await?
            .ok_or_else(|| AppError::NotFound("attachment not found".into()))?;
        self.audit
            .log(
                "attachment.downloaded",
                Some(&user.id),
                Some("attachment"),
                Some(attachment_id),
                json!({ "messageId": message_id }),
            )
            .await?;
        Ok((meta.mime_type, bytes))
    }

    pub async fn mark_read(
        &self,
        user: &User,
        channel_id: &str,
        message_id: &str,
    ) -> AppResult<()> {
        self.require_channel_access(user, channel_id).await?;
        self.chat.mark_read(channel_id, &user.id, message_id).await
    }

    pub async fn open_dm(&self, user: &User, other_user_id: &str) -> AppResult<Channel> {
        if other_user_id == user.id {
            return Err(AppError::Validation("cannot DM yourself".into()));
        }
        self.chat.find_or_create_dm(&user.id, other_user_id).await
    }

    pub async fn list_dms(&self, user: &User) -> AppResult<Vec<crate::domain::entities::DmInboxEntry>> {
        let mut entries = self.chat.list_dm_inbox_for_user(&user.id).await?;
        for entry in &mut entries {
            if let Some(ref encrypted) = entry.last_message_preview {
                entry.last_message_preview = Some(
                    self.crypto
                        .decrypt_text(encrypted)
                        .unwrap_or_else(|_| "[encrypted]".into()),
                );
            }
        }
        Ok(entries)
    }

    pub async fn list_dm_peers(&self, user: &User) -> AppResult<Vec<crate::domain::entities::DmPeer>> {
        self.chat.list_dm_peers_for_user(&user.id).await
    }

    pub async fn set_presence(
        &self,
        user: &User,
        server_id: &str,
        status: PresenceStatus,
    ) -> AppResult<Presence> {
        self.require_member(server_id, &user.id).await?;
        let now = Utc::now();
        let presence = self
            .chat
            .upsert_presence(server_id, &user.id, status, now)
            .await?;
        self.publish_presence(server_id, user, status, now).await?;
        Ok(presence)
    }

    pub async fn list_presence(&self, user: &User, server_id: &str) -> AppResult<Vec<Presence>> {
        self.require_member(server_id, &user.id).await?;
        self.chat.list_presence(server_id).await
    }

    pub async fn set_typing(
        &self,
        user: &User,
        channel_id: &str,
        is_typing: bool,
    ) -> AppResult<Vec<TypingIndicator>> {
        self.require_channel_access(user, channel_id).await?;
        if is_typing {
            let expires_at = Utc::now() + Duration::seconds(8);
            let indicator = self
                .chat
                .upsert_typing(channel_id, &user.id, expires_at)
                .await?;
            self.realtime
                .publish(
                    channel_id,
                    RealtimeEvent::TypingStarted {
                        channel_id: channel_id.to_string(),
                        user_id: user.id.clone(),
                        display_name: indicator.display_name.clone(),
                        expires_at: indicator.expires_at.to_rfc3339(),
                    },
                )
                .await?;
        } else {
            self.chat.delete_typing(channel_id, &user.id).await?;
            self.realtime
                .publish(
                    channel_id,
                    RealtimeEvent::TypingStopped {
                        channel_id: channel_id.to_string(),
                        user_id: user.id.clone(),
                    },
                )
                .await?;
        }
        self.chat.list_typing(channel_id, Utc::now()).await
    }

    pub async fn list_typing(&self, user: &User, channel_id: &str) -> AppResult<Vec<TypingIndicator>> {
        self.require_channel_access(user, channel_id).await?;
        self.chat.list_typing(channel_id, Utc::now()).await
    }

    pub async fn join_voice(
        &self,
        user: &User,
        channel_id: &str,
        muted: bool,
        deafened: bool,
    ) -> AppResult<VoiceParticipant> {
        let channel = self
            .chat
            .get_channel(channel_id)
            .await?
            .ok_or_else(|| AppError::NotFound("channel not found".into()))?;
        if channel.channel_type != ChannelType::Voice {
            return Err(AppError::Validation("channel is not voice".into()));
        }
        self.require_channel_access(user, channel_id).await?;
        let now = Utc::now();
        let participant = self
            .chat
            .upsert_voice_participant(channel_id, &user.id, muted, deafened, now)
            .await?;
        self.audit
            .log(
                "voice.participant.updated",
                Some(&user.id),
                Some("channel"),
                Some(channel_id),
                json!({ "connected": true, "muted": muted, "deafened": deafened }),
            )
            .await?;
        self.realtime
            .publish(
                channel_id,
                RealtimeEvent::VoiceParticipantUpdated {
                    channel_id: channel_id.to_string(),
                    user_id: user.id.clone(),
                    display_name: participant.display_name.clone(),
                    connected: true,
                    muted,
                    deafened,
                    updated_at: now.to_rfc3339(),
                },
            )
            .await?;
        Ok(participant)
    }

    pub async fn leave_voice(&self, user: &User, channel_id: &str) -> AppResult<()> {
        self.require_channel_access(user, channel_id).await?;
        self.chat.delete_voice_participant(channel_id, &user.id).await?;
        let now = Utc::now();
        self.audit
            .log(
                "voice.participant.updated",
                Some(&user.id),
                Some("channel"),
                Some(channel_id),
                json!({ "connected": false }),
            )
            .await?;
        self.realtime
            .publish(
                channel_id,
                RealtimeEvent::VoiceParticipantUpdated {
                    channel_id: channel_id.to_string(),
                    user_id: user.id.clone(),
                    display_name: user.display_name.clone(),
                    connected: false,
                    muted: false,
                    deafened: false,
                    updated_at: now.to_rfc3339(),
                },
            )
            .await?;
        Ok(())
    }

    pub async fn update_voice_state(
        &self,
        user: &User,
        channel_id: &str,
        muted: bool,
        deafened: bool,
    ) -> AppResult<VoiceParticipant> {
        self.require_channel_access(user, channel_id).await?;
        let now = Utc::now();
        let participant = self
            .chat
            .upsert_voice_participant(channel_id, &user.id, muted, deafened, now)
            .await?;
        self.realtime
            .publish(
                channel_id,
                RealtimeEvent::VoiceParticipantUpdated {
                    channel_id: channel_id.to_string(),
                    user_id: user.id.clone(),
                    display_name: participant.display_name.clone(),
                    connected: true,
                    muted,
                    deafened,
                    updated_at: now.to_rfc3339(),
                },
            )
            .await?;
        Ok(participant)
    }

    pub async fn list_voice_participants(
        &self,
        user: &User,
        channel_id: &str,
    ) -> AppResult<Vec<VoiceParticipant>> {
        self.require_channel_access(user, channel_id).await?;
        self.chat.list_voice_participants(channel_id).await
    }

    pub async fn bootstrap_voice_session(
        &self,
        user: &User,
        channel_id: &str,
    ) -> AppResult<crate::domain::entities::VoiceSessionBootstrap> {
        let channel = self
            .chat
            .get_channel(channel_id)
            .await?
            .ok_or_else(|| AppError::NotFound("channel not found".into()))?;
        if channel.channel_type != ChannelType::Voice {
            return Err(AppError::Validation("channel is not voice".into()));
        }
        self.require_channel_access(user, channel_id).await?;
        self.voice_media.bootstrap_session(&user.id, channel_id).await
    }

    pub async fn create_voice_transport(
        &self,
        user: &User,
        channel_id: &str,
        session_id: &str,
        direction: &str,
    ) -> AppResult<crate::domain::entities::VoiceTransport> {
        self.require_channel_access(user, channel_id).await?;
        self.voice_media.create_transport(session_id, direction).await
    }

    pub async fn connect_voice_transport(
        &self,
        user: &User,
        channel_id: &str,
        session_id: &str,
        transport_id: &str,
        dtls_parameters: serde_json::Value,
    ) -> AppResult<()> {
        self.require_channel_access(user, channel_id).await?;
        self.voice_media
            .connect_transport(session_id, transport_id, dtls_parameters)
            .await
    }

    pub async fn create_voice_producer(
        &self,
        user: &User,
        channel_id: &str,
        session_id: &str,
        transport_id: &str,
        kind: &str,
        rtp_parameters: serde_json::Value,
    ) -> AppResult<crate::domain::entities::VoiceProducer> {
        self.require_channel_access(user, channel_id).await?;
        self.voice_media
            .create_producer(session_id, transport_id, kind, rtp_parameters)
            .await
    }

    pub async fn create_voice_consumer(
        &self,
        user: &User,
        channel_id: &str,
        session_id: &str,
        transport_id: &str,
        producer_id: &str,
        rtp_capabilities: serde_json::Value,
    ) -> AppResult<crate::domain::entities::VoiceConsumer> {
        self.require_channel_access(user, channel_id).await?;
        self.voice_media
            .create_consumer(session_id, transport_id, producer_id, rtp_capabilities)
            .await
    }

    pub async fn add_voice_ice_candidate(
        &self,
        user: &User,
        channel_id: &str,
        session_id: &str,
        transport_id: &str,
        candidate: serde_json::Value,
    ) -> AppResult<()> {
        self.require_channel_access(user, channel_id).await?;
        self.voice_media
            .add_ice_candidate(session_id, transport_id, candidate)
            .await
    }

    pub async fn restart_voice_ice(
        &self,
        user: &User,
        channel_id: &str,
        session_id: &str,
        transport_id: &str,
    ) -> AppResult<serde_json::Value> {
        self.require_channel_access(user, channel_id).await?;
        self.voice_media.restart_ice(session_id, transport_id).await
    }

    pub async fn list_remote_voice_producers(
        &self,
        user: &User,
        channel_id: &str,
        session_id: &str,
    ) -> AppResult<Vec<crate::domain::entities::VoiceRemoteProducer>> {
        self.require_channel_access(user, channel_id).await?;
        self.voice_media.list_remote_producers(session_id).await
    }

    pub async fn close_voice_session(
        &self,
        user: &User,
        channel_id: &str,
        session_id: &str,
    ) -> AppResult<()> {
        self.require_channel_access(user, channel_id).await?;
        self.voice_media.close_session(session_id).await
    }

    async fn enrich_messages(&self, messages: &mut [Message]) -> AppResult<()> {
        if messages.is_empty() {
            return Ok(());
        }
        let ids: Vec<String> = messages.iter().map(|m| m.id.clone()).collect();
        let reactions = self.chat.list_reactions_for_messages(&ids).await?;
        let attachments = self.attachments.list_for_messages(&ids).await?;
        for m in messages.iter_mut() {
            m.reactions = reactions.get(&m.id).cloned().unwrap_or_default();
            m.attachment = attachments.get(&m.id).cloned();
            if let Some(ref reply_id) = m.reply_to_message_id.clone() {
                if let Some(parent) = self.chat.get_message(reply_id).await? {
                    m.reply_to = Some(ReplyPreview {
                        id: parent.id,
                        author_user_id: parent.author_user_id,
                        author_display_name: parent.author_display_name,
                        content: parent.content,
                        deleted_at: parent.deleted_at,
                    });
                }
            }
        }
        Ok(())
    }

    fn decrypt_messages(&self, messages: &mut [Message]) -> AppResult<()> {
        for m in messages {
            m.content = self.crypto.decrypt_text(&m.content)?;
            if let Some(ref mut reply) = m.reply_to {
                if reply.deleted_at.is_none() {
                    reply.content = self.crypto.decrypt_text(&reply.content)?;
                } else {
                    reply.content.clear();
                }
            }
        }
        Ok(())
    }

    async fn require_member(&self, server_id: &str, user_id: &str) -> AppResult<Member> {
        if let Some(member) = self.chat.get_member(server_id, user_id).await? {
            return Ok(member);
        }
        if server_id == HARBOUR_HOME_SERVER_ID && self.chat.get_server(server_id).await?.is_some() {
            return self
                .chat
                .add_member(server_id, user_id, MemberRole::Member)
                .await;
        }
        Err(AppError::Forbidden)
    }

    pub async fn authorize_realtime_subscribe(&self, user: &User, topic_id: &str) -> AppResult<()> {
        if topic_id == BOARD_FEED_TOPIC {
            return Ok(());
        }
        self.require_channel_access(user, topic_id).await
    }

    async fn require_channel_access(&self, user: &User, channel_id: &str) -> AppResult<()> {
        let channel = self
            .chat
            .get_channel(channel_id)
            .await?
            .ok_or_else(|| AppError::NotFound("channel not found".into()))?;

        if channel.channel_type == ChannelType::Dm {
            if !self.chat.is_dm_participant(channel_id, &user.id).await? {
                return Err(AppError::Forbidden);
            }
            return Ok(());
        }

        let server_id = channel
            .server_id
            .as_deref()
            .ok_or(AppError::NotFound("channel not found".into()))?;
        self.require_member(server_id, &user.id).await?;
        Ok(())
    }

    async fn can_moderate_channel(&self, user: &User, channel_id: &str) -> AppResult<bool> {
        let channel = self
            .chat
            .get_channel(channel_id)
            .await?
            .ok_or_else(|| AppError::NotFound("channel not found".into()))?;
        if channel.channel_type == ChannelType::Dm {
            return Ok(false);
        }
        let server_id = channel
            .server_id
            .as_deref()
            .ok_or(AppError::NotFound("channel not found".into()))?;
        let member = self.require_member(server_id, &user.id).await?;
        Ok(member.role.can_moderate())
    }

    async fn publish_presence(
        &self,
        server_id: &str,
        user: &User,
        status: PresenceStatus,
        now: chrono::DateTime<Utc>,
    ) -> AppResult<()> {
        let channels = self.chat.list_channels_for_server(server_id).await?;
        for channel in channels {
            self.realtime
                .publish(
                    &channel.id,
                    RealtimeEvent::PresenceChanged {
                        server_id: server_id.to_string(),
                        user_id: user.id.clone(),
                        status,
                        updated_at: now.to_rfc3339(),
                    },
                )
                .await?;
        }
        Ok(())
    }
}
