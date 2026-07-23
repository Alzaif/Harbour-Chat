use async_trait::async_trait;
use chrono::{TimeZone, Utc};
use sqlx::SqlitePool;

use crate::domain::entities::{User, UserSearchResult, UserSettings};
use crate::domain::ports::{GatewayIdentity, UserRepository};
use crate::error::{AppError, AppResult};

pub struct SqliteUserRepository {
    pool: SqlitePool,
}

impl SqliteUserRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

fn row_to_user(
    id: String,
    email: String,
    display_name: Option<String>,
    created_at: i64,
    updated_at: i64,
) -> AppResult<User> {
    Ok(User {
        id,
        email,
        display_name,
        created_at: Utc
            .timestamp_millis_opt(created_at)
            .single()
            .ok_or_else(|| AppError::Internal("invalid created_at".into()))?,
        updated_at: Utc
            .timestamp_millis_opt(updated_at)
            .single()
            .ok_or_else(|| AppError::Internal("invalid updated_at".into()))?,
    })
}

#[async_trait]
impl UserRepository for SqliteUserRepository {
    async fn upsert_from_gateway(&self, identity: GatewayIdentity) -> AppResult<User> {
        let now = Utc::now().timestamp_millis();
        sqlx::query(
            r#"
            INSERT INTO users (id, email, display_name, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?)
            ON CONFLICT(id) DO UPDATE SET
                email = excluded.email,
                display_name = excluded.display_name,
                updated_at = excluded.updated_at
            "#,
        )
        .bind(&identity.id)
        .bind(&identity.email)
        .bind(&identity.display_name)
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        self.find_by_id(&identity.id)
            .await?
            .ok_or_else(|| AppError::Internal("user upsert failed".into()))
    }

    async fn find_by_id(&self, id: &str) -> AppResult<Option<User>> {
        let row = sqlx::query_as::<_, (String, String, Option<String>, i64, i64)>(
            "SELECT id, email, display_name, created_at, updated_at FROM users WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        row.map(|(id, email, display_name, created_at, updated_at)| {
            row_to_user(id, email, display_name, created_at, updated_at)
        })
        .transpose()
    }

    async fn search(
        &self,
        exclude_user_id: &str,
        query: &str,
        exclude_server_id: Option<&str>,
        limit: u32,
    ) -> AppResult<Vec<UserSearchResult>> {
        let trimmed = query.trim();
        if trimmed.len() < 2 {
            return Ok(Vec::new());
        }
        let pattern = format!("%{}%", trimmed.to_lowercase());
        let limit = limit.clamp(1, 50);

        let rows = if let Some(server_id) = exclude_server_id {
            sqlx::query_as::<_, (String, String, Option<String>)>(
                r#"
                SELECT u.id, u.email, u.display_name
                FROM users u
                WHERE u.id != ?
                  AND (
                    LOWER(u.email) LIKE ?
                    OR LOWER(COALESCE(u.display_name, '')) LIKE ?
                    OR LOWER(u.id) LIKE ?
                  )
                  AND u.id NOT IN (
                    SELECT user_id FROM members WHERE server_id = ?
                  )
                ORDER BY COALESCE(u.display_name, u.email)
                LIMIT ?
                "#,
            )
            .bind(exclude_user_id)
            .bind(&pattern)
            .bind(&pattern)
            .bind(&pattern)
            .bind(server_id)
            .bind(limit as i64)
            .fetch_all(&self.pool)
            .await
        } else {
            sqlx::query_as::<_, (String, String, Option<String>)>(
                r#"
                SELECT u.id, u.email, u.display_name
                FROM users u
                WHERE u.id != ?
                  AND (
                    LOWER(u.email) LIKE ?
                    OR LOWER(COALESCE(u.display_name, '')) LIKE ?
                    OR LOWER(u.id) LIKE ?
                  )
                ORDER BY COALESCE(u.display_name, u.email)
                LIMIT ?
                "#,
            )
            .bind(exclude_user_id)
            .bind(&pattern)
            .bind(&pattern)
            .bind(&pattern)
            .bind(limit as i64)
            .fetch_all(&self.pool)
            .await
        }
        .map_err(|e| AppError::Internal(e.to_string()))?;

        Ok(rows
            .into_iter()
            .map(|(id, email, display_name)| UserSearchResult {
                id,
                email,
                display_name,
            })
            .collect())
    }

    async fn get_settings(&self, user_id: &str) -> AppResult<UserSettings> {
        let row = sqlx::query_as::<_, (i64, String)>(
            "SELECT push_to_talk, push_to_talk_key FROM user_preferences WHERE user_id = ?",
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        Ok(match row {
            Some((push_to_talk, push_to_talk_key)) => UserSettings {
                push_to_talk: push_to_talk != 0,
                push_to_talk_key,
            },
            None => UserSettings::default(),
        })
    }

    async fn upsert_settings(&self, user_id: &str, settings: &UserSettings) -> AppResult<UserSettings> {
        let now = Utc::now().timestamp_millis();
        sqlx::query(
            r#"
            INSERT INTO user_preferences (user_id, push_to_talk, push_to_talk_key, updated_at)
            VALUES (?, ?, ?, ?)
            ON CONFLICT(user_id) DO UPDATE SET
                push_to_talk = excluded.push_to_talk,
                push_to_talk_key = excluded.push_to_talk_key,
                updated_at = excluded.updated_at
            "#,
        )
        .bind(user_id)
        .bind(if settings.push_to_talk { 1 } else { 0 })
        .bind(&settings.push_to_talk_key)
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
        Ok(settings.clone())
    }
}
