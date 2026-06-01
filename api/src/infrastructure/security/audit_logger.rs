use chrono::Utc;
use serde_json::Value;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::error::{AppError, AppResult};

#[derive(Clone)]
pub struct AuditLogger {
    pool: SqlitePool,
    enabled: bool,
}

impl AuditLogger {
    pub fn new(pool: SqlitePool, enabled: bool) -> Self {
        Self { pool, enabled }
    }

    pub async fn log(
        &self,
        event_type: &str,
        user_id: Option<&str>,
        resource_type: Option<&str>,
        resource_id: Option<&str>,
        metadata: Value,
    ) -> AppResult<()> {
        if !self.enabled {
            return Ok(());
        }
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().timestamp_millis();
        let metadata_json = serde_json::to_string(&metadata)
            .map_err(|e| AppError::Internal(e.to_string()))?;

        sqlx::query(
            "INSERT INTO audit_events (id, event_type, user_id, resource_type, resource_id, metadata_json, created_at) VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(id)
        .bind(event_type)
        .bind(user_id)
        .bind(resource_type)
        .bind(resource_id)
        .bind(metadata_json)
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        tracing::info!(target: "security_audit", event_type, user_id, resource_type, resource_id, "audit event");
        Ok(())
    }
}
