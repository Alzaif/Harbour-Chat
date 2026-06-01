use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IceServer {
    pub urls: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credential: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VoiceSessionBootstrap {
    pub session_id: String,
    pub channel_id: String,
    pub user_id: String,
    pub sfu_base_url: String,
    pub router_rtp_capabilities: serde_json::Value,
    pub ice_servers: Vec<IceServer>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VoiceTransport {
    pub session_id: String,
    pub transport_id: String,
    pub direction: String,
    pub ice_parameters: serde_json::Value,
    pub ice_candidates: serde_json::Value,
    pub dtls_parameters: serde_json::Value,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VoiceProducer {
    pub session_id: String,
    pub producer_id: String,
    pub transport_id: String,
    pub kind: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VoiceConsumer {
    pub session_id: String,
    pub consumer_id: String,
    pub producer_id: String,
    pub transport_id: String,
    pub kind: String,
    pub rtp_parameters: serde_json::Value,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VoiceRemoteProducer {
    pub producer_id: String,
    pub kind: String,
    pub user_id: String,
}
