use async_trait::async_trait;
use chrono::{TimeZone, Utc};
use sqlx::SqlitePool;

use crate::domain::entities::User;
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
}
