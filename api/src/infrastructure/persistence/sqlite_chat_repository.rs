use async_trait::async_trait;
use chrono::{TimeZone, Utc};
use sqlx::SqlitePool;
use uuid::Uuid;

use std::collections::HashMap;

use crate::domain::entities::{
    Channel, ChannelType, Member, MemberRole, Message, Presence, PresenceStatus, ReactionSummary,
    Server, TypingIndicator, User, VoiceParticipant,
};
use crate::domain::ports::{ChatRepository, ServerDetail};
use crate::error::{AppError, AppResult};

pub struct SqliteChatRepository {
    pool: SqlitePool,
}

impl SqliteChatRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

fn ts(ms: i64) -> AppResult<chrono::DateTime<Utc>> {
    Utc.timestamp_millis_opt(ms)
        .single()
        .ok_or_else(|| AppError::Internal("invalid timestamp".into()))
}

fn opt_ts(ms: Option<i64>) -> AppResult<Option<chrono::DateTime<Utc>>> {
    match ms {
        Some(v) => Ok(Some(ts(v)?)),
        None => Ok(None),
    }
}

#[async_trait]
impl ChatRepository for SqliteChatRepository {
    async fn list_servers_for_user(&self, user_id: &str) -> AppResult<Vec<Server>> {
        let rows = sqlx::query_as::<_, (String, String, Option<String>, String, i64, i64)>(
            r#"
            SELECT s.id, s.name, s.icon_url, s.owner_user_id, s.created_at, s.updated_at
            FROM servers s
            INNER JOIN members m ON m.server_id = s.id
            WHERE m.user_id = ?
            ORDER BY s.name
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        rows.into_iter()
            .map(
                |(id, name, icon_url, owner_user_id, created_at, updated_at)| {
                    Ok(Server {
                        id,
                        name,
                        icon_url,
                        owner_user_id,
                        created_at: ts(created_at)?,
                        updated_at: ts(updated_at)?,
                    })
                },
            )
            .collect()
    }

    async fn get_server(&self, server_id: &str) -> AppResult<Option<Server>> {
        let row = sqlx::query_as::<_, (String, String, Option<String>, String, i64, i64)>(
            "SELECT id, name, icon_url, owner_user_id, created_at, updated_at FROM servers WHERE id = ?",
        )
        .bind(server_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        row.map(
            |(id, name, icon_url, owner_user_id, created_at, updated_at)| {
                Ok(Server {
                    id,
                    name,
                    icon_url,
                    owner_user_id,
                    created_at: ts(created_at)?,
                    updated_at: ts(updated_at)?,
                })
            },
        )
        .transpose()
    }

    async fn get_server_detail(&self, server_id: &str) -> AppResult<Option<ServerDetail>> {
        let server = self.get_server(server_id).await?;
        let Some(server) = server else {
            return Ok(None);
        };
        let channels = self.list_channels_for_server(server_id).await?;
        Ok(Some(ServerDetail {
            server,
            channels,
            unread_by_channel_id: HashMap::new(),
        }))
    }

    async fn create_server(&self, name: &str, owner: &User) -> AppResult<Server> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().timestamp_millis();
        sqlx::query(
            "INSERT INTO servers (id, name, icon_url, owner_user_id, created_at, updated_at) VALUES (?, ?, NULL, ?, ?, ?)",
        )
        .bind(&id)
        .bind(name)
        .bind(&owner.id)
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        self.get_server(&id)
            .await?
            .ok_or_else(|| AppError::Internal("server insert failed".into()))
    }

    async fn get_member(&self, server_id: &str, user_id: &str) -> AppResult<Option<Member>> {
        let row = sqlx::query_as::<_, (String, String, String, i64)>(
            "SELECT server_id, user_id, role, joined_at FROM members WHERE server_id = ? AND user_id = ?",
        )
        .bind(server_id)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        row.map(|(server_id, user_id, role, joined_at)| {
            Ok(Member {
                server_id,
                user_id,
                role: MemberRole::from_str(&role)
                    .ok_or_else(|| AppError::Internal("invalid role".into()))?,
                joined_at: ts(joined_at)?,
                display_name: None,
            })
        })
        .transpose()
    }

    async fn add_member(
        &self,
        server_id: &str,
        user_id: &str,
        role: MemberRole,
    ) -> AppResult<Member> {
        let now = Utc::now().timestamp_millis();
        sqlx::query(
            "INSERT INTO members (server_id, user_id, role, joined_at) VALUES (?, ?, ?, ?) ON CONFLICT DO NOTHING",
        )
        .bind(server_id)
        .bind(user_id)
        .bind(role.as_str())
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        self.get_member(server_id, user_id)
            .await?
            .ok_or_else(|| AppError::Internal("member insert failed".into()))
    }

    async fn list_members(&self, server_id: &str) -> AppResult<Vec<Member>> {
        let rows = sqlx::query_as::<_, (String, String, String, i64, Option<String>)>(
            r#"
            SELECT m.server_id, m.user_id, m.role, m.joined_at, u.display_name
            FROM members m
            INNER JOIN users u ON u.id = m.user_id
            WHERE m.server_id = ?
            ORDER BY m.joined_at
            "#,
        )
        .bind(server_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        rows.into_iter()
            .map(|(server_id, user_id, role, joined_at, display_name)| {
                Ok(Member {
                    server_id,
                    user_id,
                    role: MemberRole::from_str(&role)
                        .ok_or_else(|| AppError::Internal("invalid role".into()))?,
                    joined_at: ts(joined_at)?,
                    display_name,
                })
            })
            .collect()
    }

    async fn create_channel(
        &self,
        server_id: &str,
        name: &str,
        channel_type: ChannelType,
    ) -> AppResult<Channel> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().timestamp_millis();
        let rows = sqlx::query_as::<_, (i64,)>(
            "SELECT COALESCE(MAX(position), -1) + 1 AS pos FROM channels WHERE server_id = ?",
        )
        .bind(server_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
        let position = rows.0 as i32;

        sqlx::query(
            r#"
            INSERT INTO channels (id, server_id, category_id, type, name, position, created_at, updated_at)
            VALUES (?, ?, NULL, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&id)
        .bind(server_id)
        .bind(channel_type.as_str())
        .bind(name)
        .bind(position)
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        self.get_channel(&id)
            .await?
            .ok_or_else(|| AppError::Internal("channel insert failed".into()))
    }

    async fn get_channel(&self, channel_id: &str) -> AppResult<Option<Channel>> {
        let row = sqlx::query_as::<_, (String, Option<String>, Option<String>, String, String, i32, i64, i64)>(
            "SELECT id, server_id, category_id, type, name, position, created_at, updated_at FROM channels WHERE id = ?",
        )
        .bind(channel_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        row.map(
            |(id, server_id, category_id, ty, name, position, created_at, updated_at)| {
                Ok(Channel {
                    id,
                    server_id,
                    category_id,
                    channel_type: ChannelType::from_str(&ty)
                        .ok_or_else(|| AppError::Internal("invalid channel type".into()))?,
                    name,
                    position,
                    created_at: ts(created_at)?,
                    updated_at: ts(updated_at)?,
                })
            },
        )
        .transpose()
    }

    async fn list_channels_for_server(&self, server_id: &str) -> AppResult<Vec<Channel>> {
        let rows = sqlx::query_as::<_, (String, Option<String>, Option<String>, String, String, i32, i64, i64)>(
            "SELECT id, server_id, category_id, type, name, position, created_at, updated_at FROM channels WHERE server_id = ? ORDER BY position, name",
        )
        .bind(server_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        rows.into_iter()
            .map(
                |(id, server_id, category_id, ty, name, position, created_at, updated_at)| {
                    Ok(Channel {
                        id,
                        server_id,
                        category_id,
                        channel_type: ChannelType::from_str(&ty)
                            .ok_or_else(|| AppError::Internal("invalid channel type".into()))?,
                        name,
                        position,
                        created_at: ts(created_at)?,
                        updated_at: ts(updated_at)?,
                    })
                },
            )
            .collect()
    }

    async fn list_messages(
        &self,
        channel_id: &str,
        before: Option<&str>,
        limit: u32,
    ) -> AppResult<Vec<Message>> {
        let rows = if let Some(before_id) = before {
            let before_created =
                sqlx::query_as::<_, (i64,)>("SELECT created_at FROM messages WHERE id = ?")
                    .bind(before_id)
                    .fetch_optional(&self.pool)
                    .await
                    .map_err(|e| AppError::Internal(e.to_string()))?;

            let Some((created_at,)) = before_created else {
                return Ok(vec![]);
            };

            sqlx::query_as::<
                _,
                (
                    String,
                    String,
                    String,
                    Option<String>,
                    String,
                    i64,
                    Option<i64>,
                    Option<i64>,
                ),
            >(
                r#"
                SELECT m.id, m.channel_id, m.author_user_id, u.display_name, m.content, m.created_at, m.edited_at, m.deleted_at
                FROM messages m
                INNER JOIN users u ON u.id = m.author_user_id
                WHERE m.channel_id = ? AND m.created_at < ?
                ORDER BY m.created_at DESC
                LIMIT ?
                "#,
            )
            .bind(channel_id)
            .bind(created_at)
            .bind(limit)
            .fetch_all(&self.pool)
            .await
        } else {
            sqlx::query_as::<
                _,
                (
                    String,
                    String,
                    String,
                    Option<String>,
                    String,
                    i64,
                    Option<i64>,
                    Option<i64>,
                ),
            >(
                r#"
                SELECT m.id, m.channel_id, m.author_user_id, u.display_name, m.content, m.created_at, m.edited_at, m.deleted_at
                FROM messages m
                INNER JOIN users u ON u.id = m.author_user_id
                WHERE m.channel_id = ?
                ORDER BY m.created_at DESC
                LIMIT ?
                "#,
            )
            .bind(channel_id)
            .bind(limit)
            .fetch_all(&self.pool)
            .await
        }
        .map_err(|e| AppError::Internal(e.to_string()))?;

        let mut messages: Vec<Message> = rows
            .into_iter()
            .map(
                |(
                    id,
                    channel_id,
                    author_user_id,
                    author_display_name,
                    content,
                    created_at,
                    edited_at,
                    deleted_at,
                )| {
                    Ok(Message {
                        id,
                        channel_id,
                        author_user_id,
                        author_display_name,
                        content,
                        created_at: ts(created_at)?,
                        edited_at: opt_ts(edited_at)?,
                        deleted_at: opt_ts(deleted_at)?,
                        reactions: vec![],
                        attachment: None,
                    })
                },
            )
            .collect::<AppResult<_>>()?;
        messages.reverse();
        Ok(messages)
    }

    async fn insert_message(
        &self,
        channel_id: &str,
        author: &User,
        content: &str,
    ) -> AppResult<Message> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().timestamp_millis();
        sqlx::query(
            "INSERT INTO messages (id, channel_id, author_user_id, content, created_at) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(channel_id)
        .bind(&author.id)
        .bind(content)
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        self.get_message(&id)
            .await?
            .ok_or_else(|| AppError::Internal("message insert failed".into()))
    }

    async fn get_message(&self, message_id: &str) -> AppResult<Option<Message>> {
        let row = sqlx::query_as::<
            _,
            (
                String,
                String,
                String,
                Option<String>,
                String,
                i64,
                Option<i64>,
                Option<i64>,
            ),
        >(
            r#"
            SELECT m.id, m.channel_id, m.author_user_id, u.display_name, m.content, m.created_at, m.edited_at, m.deleted_at
            FROM messages m
            INNER JOIN users u ON u.id = m.author_user_id
            WHERE m.id = ?
            "#,
        )
        .bind(message_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        row.map(
            |(
                id,
                channel_id,
                author_user_id,
                author_display_name,
                content,
                created_at,
                edited_at,
                deleted_at,
            )| {
                Ok(Message {
                    id,
                    channel_id,
                    author_user_id,
                    author_display_name,
                    content,
                    created_at: ts(created_at)?,
                    edited_at: opt_ts(edited_at)?,
                    deleted_at: opt_ts(deleted_at)?,
                    reactions: vec![],
                    attachment: None,
                })
            },
        )
        .transpose()
    }

    async fn update_message_content(
        &self,
        message_id: &str,
        content: &str,
        edited_at: chrono::DateTime<Utc>,
    ) -> AppResult<Message> {
        let ms = edited_at.timestamp_millis();
        sqlx::query("UPDATE messages SET content = ?, edited_at = ? WHERE id = ?")
            .bind(content)
            .bind(ms)
            .bind(message_id)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;

        self.get_message(message_id)
            .await?
            .ok_or_else(|| AppError::NotFound("message not found".into()))
    }

    async fn soft_delete_message(
        &self,
        message_id: &str,
        deleted_at: chrono::DateTime<Utc>,
    ) -> AppResult<Message> {
        let ms = deleted_at.timestamp_millis();
        sqlx::query("UPDATE messages SET deleted_at = ? WHERE id = ?")
            .bind(ms)
            .bind(message_id)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;

        self.get_message(message_id)
            .await?
            .ok_or_else(|| AppError::NotFound("message not found".into()))
    }

    async fn toggle_reaction(
        &self,
        message_id: &str,
        user_id: &str,
        emoji: &str,
    ) -> AppResult<bool> {
        let existing = sqlx::query_as::<_, (i64,)>(
            "SELECT 1 FROM reactions WHERE message_id = ? AND user_id = ? AND emoji = ?",
        )
        .bind(message_id)
        .bind(user_id)
        .bind(emoji)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        if existing.is_some() {
            sqlx::query("DELETE FROM reactions WHERE message_id = ? AND user_id = ? AND emoji = ?")
                .bind(message_id)
                .bind(user_id)
                .bind(emoji)
                .execute(&self.pool)
                .await
                .map_err(|e| AppError::Internal(e.to_string()))?;
            Ok(false)
        } else {
            let now = Utc::now().timestamp_millis();
            sqlx::query(
                "INSERT INTO reactions (message_id, user_id, emoji, created_at) VALUES (?, ?, ?, ?)",
            )
            .bind(message_id)
            .bind(user_id)
            .bind(emoji)
            .bind(now)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
            Ok(true)
        }
    }

    async fn list_reactions_for_messages(
        &self,
        message_ids: &[String],
    ) -> AppResult<HashMap<String, Vec<ReactionSummary>>> {
        if message_ids.is_empty() {
            return Ok(HashMap::new());
        }
        let placeholders = message_ids
            .iter()
            .map(|_| "?")
            .collect::<Vec<_>>()
            .join(",");
        let sql = format!(
            "SELECT message_id, emoji, user_id FROM reactions WHERE message_id IN ({placeholders}) ORDER BY emoji"
        );
        let mut q = sqlx::query_as::<_, (String, String, String)>(&sql);
        for id in message_ids {
            q = q.bind(id);
        }
        let rows = q
            .fetch_all(&self.pool)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;

        let mut grouped: HashMap<(String, String), Vec<String>> = HashMap::new();
        for (message_id, emoji, user_id) in rows {
            grouped
                .entry((message_id, emoji))
                .or_default()
                .push(user_id);
        }

        let mut by_message: HashMap<String, Vec<ReactionSummary>> = HashMap::new();
        for ((message_id, emoji), user_ids) in grouped {
            let count = user_ids.len() as u32;
            by_message
                .entry(message_id)
                .or_default()
                .push(ReactionSummary {
                    emoji,
                    count,
                    user_ids,
                });
        }
        Ok(by_message)
    }

    async fn unread_counts_for_server(
        &self,
        server_id: &str,
        user_id: &str,
    ) -> AppResult<HashMap<String, u32>> {
        let rows = sqlx::query_as::<_, (String, i64)>(
            r#"
            SELECT m.channel_id, COUNT(*) AS cnt
            FROM messages m
            INNER JOIN channels c ON c.id = m.channel_id AND c.server_id = ?
            WHERE m.author_user_id != ?
            AND (
                NOT EXISTS (
                    SELECT 1 FROM read_states rs
                    WHERE rs.channel_id = m.channel_id AND rs.user_id = ?
                )
                OR m.created_at > COALESCE((
                    SELECT m2.created_at
                    FROM read_states rs
                    LEFT JOIN messages m2 ON m2.id = rs.last_read_message_id
                    WHERE rs.channel_id = m.channel_id AND rs.user_id = ?
                ), 0)
            )
            GROUP BY m.channel_id
            "#,
        )
        .bind(server_id)
        .bind(user_id)
        .bind(user_id)
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        Ok(rows
            .into_iter()
            .map(|(channel_id, cnt)| (channel_id, cnt as u32))
            .collect())
    }

    async fn mark_read(&self, channel_id: &str, user_id: &str, message_id: &str) -> AppResult<()> {
        let now = Utc::now().timestamp_millis();
        sqlx::query(
            r#"
            INSERT INTO read_states (channel_id, user_id, last_read_message_id, updated_at)
            VALUES (?, ?, ?, ?)
            ON CONFLICT(channel_id, user_id) DO UPDATE SET
                last_read_message_id = excluded.last_read_message_id,
                updated_at = excluded.updated_at
            "#,
        )
        .bind(channel_id)
        .bind(user_id)
        .bind(message_id)
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
        Ok(())
    }

    async fn find_or_create_dm(&self, user_a: &str, user_b: &str) -> AppResult<Channel> {
        let (a, b) = if user_a < user_b {
            (user_a, user_b)
        } else {
            (user_b, user_a)
        };

        let existing = sqlx::query_as::<_, (String,)>(
            r#"
            SELECT c.id FROM channels c
            INNER JOIN dm_participants p1 ON p1.channel_id = c.id AND p1.user_id = ?
            INNER JOIN dm_participants p2 ON p2.channel_id = c.id AND p2.user_id = ?
            WHERE c.type = 'dm'
            LIMIT 1
            "#,
        )
        .bind(a)
        .bind(b)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        if let Some((id,)) = existing {
            return self
                .get_channel(&id)
                .await?
                .ok_or_else(|| AppError::Internal("dm channel missing".into()));
        }

        let id = Uuid::new_v4().to_string();
        let now = Utc::now().timestamp_millis();
        let name = format!("dm-{a}-{b}");

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;

        sqlx::query(
            "INSERT INTO channels (id, server_id, category_id, type, name, position, created_at, updated_at) VALUES (?, NULL, NULL, 'dm', ?, 0, ?, ?)",
        )
        .bind(&id)
        .bind(&name)
        .bind(now)
        .bind(now)
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        for uid in [a, b] {
            sqlx::query("INSERT INTO dm_participants (channel_id, user_id) VALUES (?, ?)")
                .bind(&id)
                .bind(uid)
                .execute(&mut *tx)
                .await
                .map_err(|e| AppError::Internal(e.to_string()))?;
        }

        tx.commit()
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;

        self.get_channel(&id)
            .await?
            .ok_or_else(|| AppError::Internal("dm channel insert failed".into()))
    }

    async fn is_dm_participant(&self, channel_id: &str, user_id: &str) -> AppResult<bool> {
        let row = sqlx::query_as::<_, (i64,)>(
            "SELECT 1 FROM dm_participants WHERE channel_id = ? AND user_id = ?",
        )
        .bind(channel_id)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
        Ok(row.is_some())
    }

    async fn upsert_presence(
        &self,
        server_id: &str,
        user_id: &str,
        status: PresenceStatus,
        updated_at: chrono::DateTime<Utc>,
    ) -> AppResult<Presence> {
        let ms = updated_at.timestamp_millis();
        sqlx::query(
            r#"
            INSERT INTO presence_states (server_id, user_id, status, updated_at)
            VALUES (?, ?, ?, ?)
            ON CONFLICT(server_id, user_id) DO UPDATE SET
                status = excluded.status,
                updated_at = excluded.updated_at
            "#,
        )
        .bind(server_id)
        .bind(user_id)
        .bind(status.as_str())
        .bind(ms)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        Ok(Presence {
            server_id: server_id.to_string(),
            user_id: user_id.to_string(),
            status,
            updated_at,
        })
    }

    async fn list_presence(&self, server_id: &str) -> AppResult<Vec<Presence>> {
        let rows = sqlx::query_as::<_, (String, String, String, i64)>(
            "SELECT server_id, user_id, status, updated_at FROM presence_states WHERE server_id = ? ORDER BY updated_at DESC",
        )
        .bind(server_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        rows.into_iter()
            .map(
                |(server_id, user_id, status, updated_at)| {
                    Ok(Presence {
                        server_id,
                        user_id,
                        status: PresenceStatus::from_str(&status)
                            .ok_or_else(|| AppError::Internal("invalid presence status".into()))?,
                        updated_at: ts(updated_at)?,
                    })
                },
            )
            .collect()
    }

    async fn upsert_typing(
        &self,
        channel_id: &str,
        user_id: &str,
        expires_at: chrono::DateTime<Utc>,
    ) -> AppResult<TypingIndicator> {
        let ms = expires_at.timestamp_millis();
        sqlx::query(
            r#"
            INSERT INTO typing_states (channel_id, user_id, expires_at)
            VALUES (?, ?, ?)
            ON CONFLICT(channel_id, user_id) DO UPDATE SET
                expires_at = excluded.expires_at
            "#,
        )
        .bind(channel_id)
        .bind(user_id)
        .bind(ms)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        let display_name = sqlx::query_as::<_, (Option<String>,)>(
            "SELECT display_name FROM users WHERE id = ?",
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .and_then(|(name,)| name);

        Ok(TypingIndicator {
            channel_id: channel_id.to_string(),
            user_id: user_id.to_string(),
            display_name,
            expires_at,
        })
    }

    async fn delete_typing(&self, channel_id: &str, user_id: &str) -> AppResult<()> {
        sqlx::query("DELETE FROM typing_states WHERE channel_id = ? AND user_id = ?")
            .bind(channel_id)
            .bind(user_id)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        Ok(())
    }

    async fn list_typing(
        &self,
        channel_id: &str,
        now: chrono::DateTime<Utc>,
    ) -> AppResult<Vec<TypingIndicator>> {
        let now_ms = now.timestamp_millis();
        sqlx::query("DELETE FROM typing_states WHERE channel_id = ? AND expires_at <= ?")
            .bind(channel_id)
            .bind(now_ms)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        let rows = sqlx::query_as::<_, (String, String, Option<String>, i64)>(
            r#"
            SELECT t.channel_id, t.user_id, u.display_name, t.expires_at
            FROM typing_states t
            INNER JOIN users u ON u.id = t.user_id
            WHERE t.channel_id = ?
            ORDER BY t.expires_at DESC
            "#,
        )
        .bind(channel_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
        rows.into_iter()
            .map(|(channel_id, user_id, display_name, expires_at)| {
                Ok(TypingIndicator {
                    channel_id,
                    user_id,
                    display_name,
                    expires_at: ts(expires_at)?,
                })
            })
            .collect()
    }

    async fn upsert_voice_participant(
        &self,
        channel_id: &str,
        user_id: &str,
        muted: bool,
        deafened: bool,
        updated_at: chrono::DateTime<Utc>,
    ) -> AppResult<VoiceParticipant> {
        let ms = updated_at.timestamp_millis();
        sqlx::query(
            r#"
            INSERT INTO voice_participants (channel_id, user_id, muted, deafened, updated_at)
            VALUES (?, ?, ?, ?, ?)
            ON CONFLICT(channel_id, user_id) DO UPDATE SET
                muted = excluded.muted,
                deafened = excluded.deafened,
                updated_at = excluded.updated_at
            "#,
        )
        .bind(channel_id)
        .bind(user_id)
        .bind(if muted { 1 } else { 0 })
        .bind(if deafened { 1 } else { 0 })
        .bind(ms)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        let display_name = sqlx::query_as::<_, (Option<String>,)>(
            "SELECT display_name FROM users WHERE id = ?",
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .and_then(|(name,)| name);

        Ok(VoiceParticipant {
            channel_id: channel_id.to_string(),
            user_id: user_id.to_string(),
            display_name,
            muted,
            deafened,
            updated_at,
        })
    }

    async fn delete_voice_participant(&self, channel_id: &str, user_id: &str) -> AppResult<()> {
        sqlx::query("DELETE FROM voice_participants WHERE channel_id = ? AND user_id = ?")
            .bind(channel_id)
            .bind(user_id)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        Ok(())
    }

    async fn list_voice_participants(&self, channel_id: &str) -> AppResult<Vec<VoiceParticipant>> {
        let rows = sqlx::query_as::<_, (String, String, Option<String>, i64, i64, i64)>(
            r#"
            SELECT v.channel_id, v.user_id, u.display_name, v.muted, v.deafened, v.updated_at
            FROM voice_participants v
            INNER JOIN users u ON u.id = v.user_id
            WHERE v.channel_id = ?
            ORDER BY v.updated_at DESC
            "#,
        )
        .bind(channel_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        rows.into_iter()
            .map(
                |(channel_id, user_id, display_name, muted, deafened, updated_at)| {
                    Ok(VoiceParticipant {
                        channel_id,
                        user_id,
                        display_name,
                        muted: muted != 0,
                        deafened: deafened != 0,
                        updated_at: ts(updated_at)?,
                    })
                },
            )
            .collect()
    }
}
