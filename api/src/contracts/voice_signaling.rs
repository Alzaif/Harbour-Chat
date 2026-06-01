use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum VoiceSignalKind {
    SessionBootstrap,
    CreateTransport,
    ConnectTransport,
    CreateProducer,
    CreateConsumer,
    AddIceCandidate,
    RestartIce,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceSignalEnvelope<T> {
    pub request_id: String,
    pub kind: VoiceSignalKind,
    pub payload: T,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceSignalRequest {
    #[serde(rename = "type")]
    pub frame_type: String,
    #[serde(flatten)]
    pub envelope: VoiceSignalEnvelope<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceSignalResponse {
    #[serde(rename = "type")]
    pub frame_type: String,
    pub request_id: String,
    pub kind: VoiceSignalKind,
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl VoiceSignalResponse {
    pub fn ok(request_id: String, kind: VoiceSignalKind, payload: Value) -> Self {
        Self {
            frame_type: "signal_response".into(),
            request_id,
            kind,
            ok: true,
            payload: Some(payload),
            error: None,
        }
    }

    pub fn err(request_id: String, kind: VoiceSignalKind, error: String) -> Self {
        Self {
            frame_type: "signal_response".into(),
            request_id,
            kind,
            ok: false,
            payload: None,
            error: Some(error),
        }
    }
}
