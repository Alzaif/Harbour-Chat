use serde::{Deserialize, Serialize};

pub const MESSAGE_SENT_V1: &str = "harbour.message-sent.v1";
pub const TYPING_STARTED_V1: &str = "harbour.typing-started.v1";
pub const TYPING_STOPPED_V1: &str = "harbour.typing-stopped.v1";
pub const PRESENCE_CHANGED_V1: &str = "harbour.presence-changed.v1";
pub const VOICE_PARTICIPANT_UPDATED_V1: &str = "harbour.voice-participant-updated.v1";
pub const VOICE_SESSION_CREATED_V1: &str = "harbour.voice-session-created.v1";
pub const VOICE_TRANSPORT_CREATED_V1: &str = "harbour.voice-transport-created.v1";
pub const VOICE_PRODUCER_CREATED_V1: &str = "harbour.voice-producer-created.v1";
pub const VOICE_CONSUMER_CREATED_V1: &str = "harbour.voice-consumer-created.v1";
pub const VOICE_ICE_CANDIDATE_V1: &str = "harbour.voice-ice-candidate.v1";
pub const VOICE_ERROR_V1: &str = "harbour.voice-error.v1";
pub const POST_CREATED_V1: &str = "harbour.post-created.v1";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PostCreatedV1 {
    pub schema: String,
    pub post_id: String,
    pub author_user_id: String,
    pub occurred_at: String,
}

impl PostCreatedV1 {
    pub fn new(post_id: String, author_user_id: String, occurred_at: String) -> Self {
        Self {
            schema: POST_CREATED_V1.into(),
            post_id,
            author_user_id,
            occurred_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MessageSentV1 {
    pub schema: String,
    pub message_id: String,
    pub channel_id: String,
    pub server_id: String,
    pub author_user_id: String,
    pub occurred_at: String,
}

impl MessageSentV1 {
    pub fn new(
        message_id: String,
        channel_id: String,
        server_id: String,
        author_user_id: String,
        occurred_at: String,
    ) -> Self {
        Self {
            schema: MESSAGE_SENT_V1.into(),
            message_id,
            channel_id,
            server_id,
            author_user_id,
            occurred_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TypingStartedV1 {
    pub schema: String,
    pub channel_id: String,
    pub user_id: String,
    pub occurred_at: String,
    pub expires_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TypingStoppedV1 {
    pub schema: String,
    pub channel_id: String,
    pub user_id: String,
    pub occurred_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PresenceChangedV1 {
    pub schema: String,
    pub server_id: String,
    pub user_id: String,
    pub status: String,
    pub occurred_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VoiceParticipantUpdatedV1 {
    pub schema: String,
    pub channel_id: String,
    pub user_id: String,
    pub connected: bool,
    pub muted: bool,
    pub deafened: bool,
    pub occurred_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VoiceSessionCreatedV1 {
    pub schema: String,
    pub session_id: String,
    pub channel_id: String,
    pub user_id: String,
    pub occurred_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VoiceTransportCreatedV1 {
    pub schema: String,
    pub session_id: String,
    pub transport_id: String,
    pub direction: String,
    pub occurred_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VoiceProducerCreatedV1 {
    pub schema: String,
    pub session_id: String,
    pub producer_id: String,
    pub transport_id: String,
    pub kind: String,
    pub occurred_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VoiceConsumerCreatedV1 {
    pub schema: String,
    pub session_id: String,
    pub consumer_id: String,
    pub producer_id: String,
    pub transport_id: String,
    pub kind: String,
    pub occurred_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VoiceIceCandidateV1 {
    pub schema: String,
    pub session_id: String,
    pub transport_id: String,
    pub candidate: String,
    pub sdp_mid: Option<String>,
    pub sdp_mline_index: Option<u16>,
    pub occurred_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VoiceErrorV1 {
    pub schema: String,
    pub code: String,
    pub message: String,
    pub occurred_at: String,
}
