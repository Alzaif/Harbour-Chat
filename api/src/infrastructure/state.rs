use std::sync::Arc;

use sqlx::SqlitePool;

use crate::application::ChatService;
use crate::config::Config;
use crate::domain::entities::User;
use crate::error::AppResult;
use crate::domain::ports::AttachmentStore;
use crate::infrastructure::persistence::{create_pool, SqliteChatRepository, SqliteUserRepository};
use crate::infrastructure::realtime::InMemoryRealtimeHub;
use crate::infrastructure::security::{AuditLogger, EnvelopeCrypto};
use crate::infrastructure::storage::LocalAttachmentStore;
use crate::infrastructure::voice::InMemoryVoiceMediaAdapter;

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub users: Arc<SqliteUserRepository>,
    pub chat: Arc<ChatService>,
    pub realtime: Arc<InMemoryRealtimeHub>,
    #[allow(dead_code)]
    pool: SqlitePool,
}

impl AppState {
    pub async fn new(config: Config) -> AppResult<Self> {
        tokio::fs::create_dir_all(&config.data_dir)
            .await
            .map_err(|e| crate::error::AppError::Internal(e.to_string()))?;

        let pool = create_pool(&config).await?;
        let users = Arc::new(SqliteUserRepository::new(pool.clone()));
        let chat_repo = Arc::new(SqliteChatRepository::new(pool.clone()));
        let crypto = EnvelopeCrypto::new(config.master_key_id.clone(), config.master_key_b64.clone())?;
        let audit = AuditLogger::new(pool.clone(), config.enable_security_audit_log);
        let attachments: Arc<dyn AttachmentStore> = Arc::new(LocalAttachmentStore::new(
            pool.clone(),
            config.data_dir.clone(),
            config.max_attachment_bytes,
            crypto.clone(),
            config.quarantine_suspicious_attachments,
        ));
        let realtime = InMemoryRealtimeHub::new();
        let voice_media = InMemoryVoiceMediaAdapter::new(
            config.voice_sfu_base_url.clone(),
            config.voice_turn_urls.clone(),
            config.voice_turn_secret.clone(),
            config.voice_turn_ttl_seconds,
            pool.clone(),
        );
        let chat = Arc::new(ChatService::new(
            chat_repo,
            realtime.clone(),
            attachments,
            voice_media,
            crypto,
            audit,
        ));

        Ok(Self {
            config,
            users,
            chat,
            realtime,
            pool,
        })
    }

    pub async fn new_for_test(config: Config) -> AppResult<Self> {
        let _ = tokio::fs::remove_file(&config.db_path).await;
        Self::new(config).await
    }
}

pub type AuthenticatedUser = User;
