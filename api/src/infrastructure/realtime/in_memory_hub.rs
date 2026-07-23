use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use axum::extract::ws::{Message as WsMessage, WebSocket};
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use tokio::sync::{broadcast, RwLock};

use crate::application::ChatService;
use crate::contracts::voice_signaling::{VoiceSignalRequest, VoiceSignalResponse};
use crate::domain::entities::User;
use crate::domain::ports::{RealtimeEvent, RealtimePublisher};
use crate::error::{AppError, AppResult};

type Sender = broadcast::Sender<String>;

#[derive(Default)]
pub struct InMemoryRealtimeHub {
    channels: RwLock<HashMap<String, Sender>>,
}

impl InMemoryRealtimeHub {
    pub fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }

    async fn sender_for(&self, channel_id: &str) -> Sender {
        let mut guard = self.channels.write().await;
        guard
            .entry(channel_id.to_string())
            .or_insert_with(|| broadcast::channel(256).0)
            .clone()
    }

    pub async fn handle_socket(
        self: &Arc<Self>,
        socket: WebSocket,
        user: User,
        chat: Arc<ChatService>,
    ) {
        let (mut ws_tx, mut ws_rx) = socket.split();
        let hub = Arc::clone(self);
        let mut receivers: Vec<broadcast::Receiver<String>> = Vec::new();
        let user_id = user.id.clone();

        loop {
            tokio::select! {
                incoming = ws_rx.next() => {
                    match incoming {
                        Some(Ok(WsMessage::Text(text))) => {
                            match parse_incoming_text(&text, &user_id) {
                                IncomingTextAction::Subscribe(channel_ids) => {
                                    for channel_id in channel_ids {
                                        if chat
                                            .authorize_realtime_subscribe(&user, &channel_id)
                                            .await
                                            .is_err()
                                        {
                                            continue;
                                        }
                                        let tx = hub.sender_for(&channel_id).await;
                                        receivers.push(tx.subscribe());
                                    }
                                }
                                IncomingTextAction::SignalAck(frame) => {
                                    if ws_tx.send(WsMessage::Text(frame.into())).await.is_err() {
                                        break;
                                    }
                                }
                                IncomingTextAction::Ignore => {}
                            }
                        }
                        Some(Ok(WsMessage::Ping(payload))) => {
                            if ws_tx.send(WsMessage::Pong(payload)).await.is_err() {
                                break;
                            }
                        }
                        Some(Ok(WsMessage::Pong(_))) => {}
                        Some(Ok(WsMessage::Close(_))) | None => break,
                        _ => {}
                    }
                }
                _ = async {
                    for rx in &mut receivers {
                        if let Ok(payload) = rx.try_recv() {
                            if ws_tx.send(WsMessage::Text(payload.into())).await.is_err() {
                                return;
                            }
                        }
                    }
                    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                } => {}
            }
        }
    }
}

enum IncomingTextAction {
    Subscribe(Vec<String>),
    SignalAck(String),
    Ignore,
}

fn parse_incoming_text(text: &str, user_id: &str) -> IncomingTextAction {
    if let Ok(parsed) = serde_json::from_str::<SubscribeMessage>(text) {
        if parsed.kind == "subscribe" {
            return IncomingTextAction::Subscribe(parsed.channel_ids);
        }
    } else if let Ok(signal) = serde_json::from_str::<VoiceSignalRequest>(text) {
        let payload = serde_json::json!({
            "ack": true,
            "userId": user_id,
            "note": "use HTTP signaling endpoints for Phase 4 MVP control-plane operations"
        });
        let response = VoiceSignalResponse::ok(
            signal.envelope.request_id,
            signal.envelope.kind,
            payload,
        );
        if let Ok(frame) = serde_json::to_string(&response) {
            return IncomingTextAction::SignalAck(frame);
        }
    }
    IncomingTextAction::Ignore
}

#[derive(Deserialize)]
struct SubscribeMessage {
    #[serde(rename = "type")]
    kind: String,
    #[serde(alias = "channelIds")]
    channel_ids: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::{parse_incoming_text, IncomingTextAction};

    #[test]
    fn subscribe_message_supports_snake_case_channel_ids() {
        let action = parse_incoming_text(
            r#"{"type":"subscribe","channel_ids":["c1","c2"]}"#,
            "user-1",
        );
        match action {
            IncomingTextAction::Subscribe(ids) => assert_eq!(ids, vec!["c1", "c2"]),
            _ => panic!("expected subscribe action"),
        }
    }

    #[test]
    fn subscribe_message_supports_camel_case_channel_ids() {
        let action = parse_incoming_text(
            r#"{"type":"subscribe","channelIds":["c1"]}"#,
            "user-1",
        );
        match action {
            IncomingTextAction::Subscribe(ids) => assert_eq!(ids, vec!["c1"]),
            _ => panic!("expected subscribe action"),
        }
    }

    #[test]
    fn voice_signal_request_returns_signal_response_ack() {
        let action = parse_incoming_text(
            r#"{"type":"signal_request","request_id":"r-1","kind":"session_bootstrap","payload":{}}"#,
            "user-1",
        );
        match action {
            IncomingTextAction::SignalAck(payload) => {
                let value: serde_json::Value = serde_json::from_str(&payload).expect("valid json");
                assert_eq!(value["type"], "signal_response");
                assert_eq!(value["request_id"], "r-1");
                assert_eq!(value["kind"], "session_bootstrap");
                assert_eq!(value["ok"], true);
                assert_eq!(value["payload"]["ack"], true);
                assert_eq!(value["payload"]["userId"], "user-1");
            }
            _ => panic!("expected signal ack"),
        }
    }
}

#[async_trait]
impl RealtimePublisher for InMemoryRealtimeHub {
    async fn publish(&self, channel_id: &str, event: RealtimeEvent) -> AppResult<()> {
        let payload =
            serde_json::to_string(&event).map_err(|e| AppError::Internal(e.to_string()))?;
        let tx = self.sender_for(channel_id).await;
        let _ = tx.send(payload);
        Ok(())
    }
}
