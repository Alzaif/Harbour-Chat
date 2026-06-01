use std::sync::Arc;

use async_trait::async_trait;
use base64::Engine;
use chrono::{Duration, Utc};
use hmac::{Hmac, Mac};
use reqwest::Client;
use serde::Deserialize;
use sqlx::SqlitePool;
use sha1::Sha1;
use uuid::Uuid;

use crate::domain::entities::{
    IceServer, VoiceConsumer, VoiceProducer, VoiceRemoteProducer, VoiceSessionBootstrap,
    VoiceTransport,
};
use crate::domain::ports::VoiceMediaPort;
use crate::error::{AppError, AppResult};

#[derive(Deserialize)]
struct BootstrapPayload {
    session_id: String,
    channel_id: String,
    user_id: String,
    router_rtp_capabilities: serde_json::Value,
}

#[derive(Deserialize)]
struct TransportPayload {
    session_id: String,
    transport_id: String,
    direction: String,
    ice_parameters: serde_json::Value,
    ice_candidates: serde_json::Value,
    dtls_parameters: serde_json::Value,
}

#[derive(Deserialize)]
struct ProducerPayload {
    session_id: String,
    producer_id: String,
    transport_id: String,
    kind: String,
}

#[derive(Deserialize)]
struct ConsumerPayload {
    session_id: String,
    consumer_id: String,
    producer_id: String,
    transport_id: String,
    kind: String,
    rtp_parameters: serde_json::Value,
}

#[derive(Deserialize)]
struct ProducersPayload {
    producers: Vec<VoiceRemoteProducer>,
}

pub struct InMemoryVoiceMediaAdapter {
    sfu_base_url: String,
    turn_urls: Vec<String>,
    turn_secret: String,
    turn_ttl_seconds: i64,
    http: Client,
    pool: SqlitePool,
}

impl InMemoryVoiceMediaAdapter {
    pub fn new(
        sfu_base_url: String,
        turn_urls: Vec<String>,
        turn_secret: String,
        turn_ttl_seconds: i64,
        pool: SqlitePool,
    ) -> Arc<Self> {
        Arc::new(Self {
            sfu_base_url,
            turn_urls,
            turn_secret,
            turn_ttl_seconds,
            http: Client::new(),
            pool,
        })
    }

    async fn require_session_row(&self, session_id: &str) -> AppResult<()> {
        let exists = sqlx::query_as::<_, (i64,)>(
            "SELECT 1 FROM voice_media_sessions WHERE session_id = ? AND closed_at IS NULL",
        )
        .bind(session_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
        if exists.is_some() {
            Ok(())
        } else {
            Err(AppError::NotFound("voice session not found".into()))
        }
    }

    fn signed_turn_credentials(&self, user_id: &str) -> (String, String) {
        let expiry = Utc::now() + Duration::seconds(self.turn_ttl_seconds.max(60));
        let username = format!("{}:{}", expiry.timestamp(), user_id);
        let mut mac = Hmac::<Sha1>::new_from_slice(self.turn_secret.as_bytes())
            .expect("HMAC can take key of any size");
        mac.update(username.as_bytes());
        let credential = base64::engine::general_purpose::STANDARD.encode(mac.finalize().into_bytes());
        (username, credential)
    }

    async fn response_error(context: &str, response: reqwest::Response) -> AppError {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        AppError::Internal(format!("{context}: {status} {body}"))
    }
}

#[async_trait]
impl VoiceMediaPort for InMemoryVoiceMediaAdapter {
    async fn bootstrap_session(&self, user_id: &str, channel_id: &str) -> AppResult<VoiceSessionBootstrap> {
        let send_result = self
            .http
            .post(format!("{}/v1/sessions/bootstrap", self.sfu_base_url.trim_end_matches('/')))
            .json(&serde_json::json!({ "userId": user_id, "channelId": channel_id }))
            .send()
            .await;
        let payload: BootstrapPayload = match send_result {
            Ok(response) if response.status().is_success() => response
                .json()
                .await
                .map_err(|e| AppError::Internal(format!("invalid sfu bootstrap payload: {e}")))?,
            Ok(response) => return Err(Self::response_error("sfu bootstrap rejected", response).await),
            Err(_) => BootstrapPayload {
                session_id: Uuid::new_v4().to_string(),
                channel_id: channel_id.to_string(),
                user_id: user_id.to_string(),
                router_rtp_capabilities: serde_json::json!({
                    "codecs":[{"kind":"audio","mimeType":"audio/opus","clockRate":48000,"channels":2}],
                    "headerExtensions":[]
                }),
            },
        };
        let now = Utc::now();
        let expires_at = now + Duration::minutes(15);
        sqlx::query(
            r#"
            INSERT INTO voice_media_sessions (session_id, channel_id, user_id, created_at, expires_at, closed_at)
            VALUES (?, ?, ?, ?, ?, NULL)
            "#,
        )
        .bind(&payload.session_id)
        .bind(channel_id)
        .bind(user_id)
        .bind(now.timestamp_millis())
        .bind(expires_at.timestamp_millis())
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
        let (turn_username, turn_credential) = self.signed_turn_credentials(user_id);
        let ice_servers = if self.turn_urls.is_empty() {
            vec![IceServer {
                urls: vec!["stun:stun.l.google.com:19302".into()],
                username: None,
                credential: None,
            }]
        } else {
            vec![IceServer {
                urls: self.turn_urls.clone(),
                username: Some(turn_username),
                credential: Some(turn_credential),
            }]
        };
        Ok(VoiceSessionBootstrap {
            session_id: payload.session_id,
            channel_id: payload.channel_id,
            user_id: payload.user_id,
            sfu_base_url: self.sfu_base_url.clone(),
            router_rtp_capabilities: payload.router_rtp_capabilities,
            ice_servers,
            expires_at,
        })
    }

    async fn create_transport(&self, session_id: &str, direction: &str) -> AppResult<VoiceTransport> {
        self.require_session_row(session_id).await?;
        let send_result = self
            .http
            .post(format!("{}/v1/transports", self.sfu_base_url.trim_end_matches('/')))
            .json(&serde_json::json!({ "sessionId": session_id, "direction": direction }))
            .send()
            .await;
        let payload: TransportPayload = match send_result {
            Ok(response) if response.status().is_success() => response
                .json()
                .await
                .map_err(|e| AppError::Internal(format!("invalid transport payload: {e}")))?,
            Ok(response) => return Err(Self::response_error("sfu create transport rejected", response).await),
            Err(_) => TransportPayload {
                session_id: session_id.to_string(),
                transport_id: Uuid::new_v4().to_string(),
                direction: direction.to_string(),
                ice_parameters: serde_json::json!({"usernameFragment":"local","password":"local","iceLite":false}),
                ice_candidates: serde_json::json!([]),
                dtls_parameters: serde_json::json!({"role":"auto","fingerprints":[]}),
            },
        };
        sqlx::query(
            r#"
            INSERT INTO voice_media_transports (transport_id, session_id, direction, created_at, connected_at)
            VALUES (?, ?, ?, ?, NULL)
            "#,
        )
        .bind(&payload.transport_id)
        .bind(session_id)
        .bind(direction)
        .bind(Utc::now().timestamp_millis())
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
        Ok(VoiceTransport {
            session_id: payload.session_id,
            transport_id: payload.transport_id,
            direction: payload.direction,
            ice_parameters: payload.ice_parameters,
            ice_candidates: payload.ice_candidates,
            dtls_parameters: payload.dtls_parameters,
        })
    }

    async fn connect_transport(
        &self,
        session_id: &str,
        transport_id: &str,
        dtls_parameters: serde_json::Value,
    ) -> AppResult<()> {
        self.require_session_row(session_id).await?;
        let response = self
            .http
            .post(format!(
                "{}/v1/transports/{transport_id}/connect",
                self.sfu_base_url.trim_end_matches('/')
            ))
            .json(&serde_json::json!({ "dtlsParameters": dtls_parameters }))
            .send()
            .await;
        if let Ok(response) = response {
            if !response.status().is_success() {
                return Err(Self::response_error("sfu connect rejected", response).await);
            }
        }
        sqlx::query(
            "UPDATE voice_media_transports SET connected_at = ? WHERE transport_id = ? AND session_id = ?",
        )
        .bind(Utc::now().timestamp_millis())
        .bind(transport_id)
        .bind(session_id)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
        Ok(())
    }

    async fn create_producer(
        &self,
        session_id: &str,
        transport_id: &str,
        kind: &str,
        _rtp_parameters: serde_json::Value,
    ) -> AppResult<VoiceProducer> {
        self.require_session_row(session_id).await?;
        let send_result = self
            .http
            .post(format!("{}/v1/producers", self.sfu_base_url.trim_end_matches('/')))
            .json(&serde_json::json!({
                "sessionId": session_id,
                "transportId": transport_id,
                "kind": kind,
                "rtpParameters": _rtp_parameters
            }))
            .send()
            .await;
        let payload: ProducerPayload = match send_result {
            Ok(response) if response.status().is_success() => response
                .json()
                .await
                .map_err(|e| AppError::Internal(format!("invalid producer payload: {e}")))?,
            Ok(response) => return Err(Self::response_error("sfu create producer rejected", response).await),
            Err(_) => ProducerPayload {
                session_id: session_id.to_string(),
                producer_id: Uuid::new_v4().to_string(),
                transport_id: transport_id.to_string(),
                kind: kind.to_string(),
            },
        };
        sqlx::query(
            r#"
            INSERT INTO voice_media_producers (producer_id, session_id, transport_id, kind, created_at, closed_at)
            VALUES (?, ?, ?, ?, ?, NULL)
            "#,
        )
        .bind(&payload.producer_id)
        .bind(session_id)
        .bind(transport_id)
        .bind(kind)
        .bind(Utc::now().timestamp_millis())
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
        Ok(VoiceProducer {
            session_id: payload.session_id,
            producer_id: payload.producer_id,
            transport_id: payload.transport_id,
            kind: payload.kind,
        })
    }

    async fn create_consumer(
        &self,
        session_id: &str,
        transport_id: &str,
        producer_id: &str,
        _rtp_capabilities: serde_json::Value,
    ) -> AppResult<VoiceConsumer> {
        self.require_session_row(session_id).await?;
        let send_result = self
            .http
            .post(format!("{}/v1/consumers", self.sfu_base_url.trim_end_matches('/')))
            .json(&serde_json::json!({
                "sessionId": session_id,
                "transportId": transport_id,
                "producerId": producer_id,
                "rtpCapabilities": _rtp_capabilities
            }))
            .send()
            .await;
        let payload: ConsumerPayload = match send_result {
            Ok(response) if response.status().is_success() => response
                .json()
                .await
                .map_err(|e| AppError::Internal(format!("invalid consumer payload: {e}")))?,
            Ok(response) => return Err(Self::response_error("sfu create consumer rejected", response).await),
            Err(_) => ConsumerPayload {
                session_id: session_id.to_string(),
                consumer_id: Uuid::new_v4().to_string(),
                producer_id: producer_id.to_string(),
                transport_id: transport_id.to_string(),
                kind: "audio".into(),
                rtp_parameters: serde_json::json!({}),
            },
        };
        sqlx::query(
            r#"
            INSERT INTO voice_media_consumers (consumer_id, session_id, transport_id, producer_id, kind, created_at, closed_at)
            VALUES (?, ?, ?, ?, ?, ?, NULL)
            "#,
        )
        .bind(&payload.consumer_id)
        .bind(session_id)
        .bind(transport_id)
        .bind(producer_id)
        .bind(&payload.kind)
        .bind(Utc::now().timestamp_millis())
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
        Ok(VoiceConsumer {
            session_id: payload.session_id,
            consumer_id: payload.consumer_id,
            producer_id: payload.producer_id,
            transport_id: payload.transport_id,
            kind: payload.kind,
            rtp_parameters: payload.rtp_parameters,
        })
    }

    async fn add_ice_candidate(
        &self,
        session_id: &str,
        _transport_id: &str,
        _candidate: serde_json::Value,
    ) -> AppResult<()> {
        self.require_session_row(session_id).await?;
        let response = self
            .http
            .post(format!(
                "{}/v1/transports/{_transport_id}/ice-candidates",
                self.sfu_base_url.trim_end_matches('/')
            ))
            .json(&serde_json::json!({ "candidate": _candidate }))
            .send()
            .await;
        if let Ok(response) = response {
            if !response.status().is_success() {
                return Err(Self::response_error("sfu add candidate rejected", response).await);
            }
        }
        Ok(())
    }

    async fn restart_ice(&self, session_id: &str, transport_id: &str) -> AppResult<serde_json::Value> {
        self.require_session_row(session_id).await?;
        let response = self
            .http
            .post(format!(
                "{}/v1/transports/{transport_id}/restart-ice",
                self.sfu_base_url.trim_end_matches('/')
            ))
            .send()
            .await;
        let response = match response {
            Ok(r) => r,
            Err(_) => {
                return Ok(serde_json::json!({
                    "usernameFragment":"local",
                    "password":"local"
                }))
            }
        };
        if !response.status().is_success() {
            return Err(Self::response_error("sfu restart ice rejected", response).await);
        }
        response
            .json()
            .await
            .map_err(|e| AppError::Internal(format!("invalid restart ice payload: {e}")))
    }

    async fn list_remote_producers(&self, session_id: &str) -> AppResult<Vec<VoiceRemoteProducer>> {
        self.require_session_row(session_id).await?;
        let response = self
            .http
            .get(format!(
                "{}/v1/sessions/{session_id}/producers",
                self.sfu_base_url.trim_end_matches('/')
            ))
            .send()
            .await;
        let response = match response {
            Ok(r) => r,
            Err(_) => return Ok(vec![]),
        };
        if !response.status().is_success() {
            return Err(Self::response_error("sfu list producers rejected", response).await);
        }
        let payload: ProducersPayload = response
            .json()
            .await
            .map_err(|e| AppError::Internal(format!("invalid producers payload: {e}")))?;
        Ok(payload.producers)
    }

    async fn close_session(&self, session_id: &str) -> AppResult<()> {
        let _ = self
            .http
            .delete(format!(
                "{}/v1/sessions/{session_id}",
                self.sfu_base_url.trim_end_matches('/')
            ))
            .send()
            .await;
        sqlx::query("UPDATE voice_media_sessions SET closed_at = ? WHERE session_id = ? AND closed_at IS NULL")
            .bind(Utc::now().timestamp_millis())
            .bind(session_id)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        Ok(())
    }
}
