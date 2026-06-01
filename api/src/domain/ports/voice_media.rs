use async_trait::async_trait;

use crate::domain::entities::{
    VoiceConsumer, VoiceProducer, VoiceRemoteProducer, VoiceSessionBootstrap, VoiceTransport,
};
use crate::error::AppResult;

#[async_trait]
pub trait VoiceMediaPort: Send + Sync {
    async fn bootstrap_session(&self, user_id: &str, channel_id: &str) -> AppResult<VoiceSessionBootstrap>;
    async fn create_transport(&self, session_id: &str, direction: &str) -> AppResult<VoiceTransport>;
    async fn connect_transport(
        &self,
        session_id: &str,
        transport_id: &str,
        dtls_parameters: serde_json::Value,
    ) -> AppResult<()>;
    async fn create_producer(
        &self,
        session_id: &str,
        transport_id: &str,
        kind: &str,
        rtp_parameters: serde_json::Value,
    ) -> AppResult<VoiceProducer>;
    async fn create_consumer(
        &self,
        session_id: &str,
        transport_id: &str,
        producer_id: &str,
        rtp_capabilities: serde_json::Value,
    ) -> AppResult<VoiceConsumer>;
    async fn add_ice_candidate(
        &self,
        session_id: &str,
        transport_id: &str,
        candidate: serde_json::Value,
    ) -> AppResult<()>;
    async fn restart_ice(&self, session_id: &str, transport_id: &str) -> AppResult<serde_json::Value>;
    async fn list_remote_producers(&self, session_id: &str) -> AppResult<Vec<VoiceRemoteProducer>>;
    async fn close_session(&self, session_id: &str) -> AppResult<()>;
}
