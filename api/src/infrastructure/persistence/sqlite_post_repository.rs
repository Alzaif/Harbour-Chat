use async_trait::async_trait;
use chrono::{TimeZone, Utc};
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::domain::entities::{Post, PostComment};
use crate::domain::ports::PostRepository;
use crate::error::{AppError, AppResult};

pub struct SqlitePostRepository {
    pool: SqlitePool,
}

impl SqlitePostRepository {
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
        None => Ok(None),
        Some(v) => Ok(Some(ts(v)?)),
    }
}

#[derive(sqlx::FromRow)]
struct PostRow {
    id: String,
    author_user_id: String,
    author_display_name: Option<String>,
    title: Option<String>,
    body: String,
    link_url: Option<String>,
    preview_title: Option<String>,
    preview_description: Option<String>,
    preview_image_url: Option<String>,
    preview_site_name: Option<String>,
    upvotes: i64,
    downvotes: i64,
    score: i64,
    comment_count: i64,
    my_vote: Option<i64>,
    created_at: i64,
    updated_at: i64,
}

impl SqlitePostRepository {
    fn row_to_post(row: PostRow) -> AppResult<Post> {
        Ok(Post {
            id: row.id,
            author_user_id: row.author_user_id,
            author_display_name: row.author_display_name,
            title: row.title,
            body: row.body,
            link_url: row.link_url,
            preview_title: row.preview_title,
            preview_description: row.preview_description,
            preview_image_url: row.preview_image_url,
            preview_site_name: row.preview_site_name,
            upvotes: row.upvotes,
            downvotes: row.downvotes,
            score: row.score,
            comment_count: row.comment_count,
            my_vote: row.my_vote.unwrap_or(0) as i8,
            created_at: ts(row.created_at)?,
            updated_at: ts(row.updated_at)?,
        })
    }

    const SELECT_POST: &'static str = r#"
        SELECT p.id, p.author_user_id,
               u.display_name AS author_display_name,
               p.title, p.body, p.link_url,
               p.preview_title, p.preview_description, p.preview_image_url, p.preview_site_name,
               p.upvotes, p.downvotes, p.score,
               (SELECT COUNT(*) FROM post_comments c WHERE c.post_id = p.id AND c.deleted_at IS NULL) AS comment_count,
               (SELECT v.value FROM post_votes v WHERE v.post_id = p.id AND v.user_id = ?) AS my_vote,
               p.created_at, p.updated_at
        FROM posts p
        LEFT JOIN users u ON u.id = p.author_user_id
    "#;
}

#[async_trait]
impl PostRepository for SqlitePostRepository {
    async fn insert_post(
        &self,
        author_user_id: &str,
        title: Option<&str>,
        body: &str,
        link_url: Option<&str>,
        preview_title: Option<&str>,
        preview_description: Option<&str>,
        preview_image_url: Option<&str>,
        preview_site_name: Option<&str>,
    ) -> AppResult<Post> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().timestamp_millis();
        sqlx::query(
            r#"
            INSERT INTO posts (
                id, author_user_id, title, body, link_url,
                preview_title, preview_description, preview_image_url, preview_site_name,
                upvotes, downvotes, score, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, 0, 0, 0, ?, ?)
            "#,
        )
        .bind(&id)
        .bind(author_user_id)
        .bind(title)
        .bind(body)
        .bind(link_url)
        .bind(preview_title)
        .bind(preview_description)
        .bind(preview_image_url)
        .bind(preview_site_name)
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        self.get_post(&id, Some(author_user_id))
            .await?
            .ok_or_else(|| AppError::Internal("post insert failed".into()))
    }

    async fn list_posts_top(
        &self,
        since_ms: Option<i64>,
        viewer_user_id: &str,
        limit: u32,
    ) -> AppResult<Vec<Post>> {
        let sql = if since_ms.is_some() {
            format!(
                "{} WHERE p.created_at >= ? ORDER BY p.score DESC, p.created_at DESC LIMIT ?",
                Self::SELECT_POST
            )
        } else {
            format!(
                "{} ORDER BY p.score DESC, p.created_at DESC LIMIT ?",
                Self::SELECT_POST
            )
        };

        let rows = if let Some(since) = since_ms {
            sqlx::query_as::<_, PostRow>(&sql)
                .bind(viewer_user_id)
                .bind(since)
                .bind(limit)
                .fetch_all(&self.pool)
                .await
        } else {
            sqlx::query_as::<_, PostRow>(&sql)
                .bind(viewer_user_id)
                .bind(limit)
                .fetch_all(&self.pool)
                .await
        }
        .map_err(|e| AppError::Internal(e.to_string()))?;

        rows.into_iter().map(Self::row_to_post).collect()
    }

    async fn list_posts_older(
        &self,
        before_ms: i64,
        viewer_user_id: &str,
        limit: u32,
    ) -> AppResult<Vec<Post>> {
        let sql = format!(
            "{} WHERE p.created_at < ? ORDER BY p.created_at DESC LIMIT ?",
            Self::SELECT_POST
        );
        let rows = sqlx::query_as::<_, PostRow>(&sql)
            .bind(viewer_user_id)
            .bind(before_ms)
            .bind(limit)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;

        rows.into_iter().map(Self::row_to_post).collect()
    }

    async fn get_post(&self, post_id: &str, viewer_user_id: Option<&str>) -> AppResult<Option<Post>> {
        let viewer = viewer_user_id.unwrap_or("");
        let sql = format!("{} WHERE p.id = ?", Self::SELECT_POST);
        let row = sqlx::query_as::<_, PostRow>(&sql)
            .bind(viewer)
            .bind(post_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;

        row.map(Self::row_to_post).transpose()
    }

    async fn get_vote(&self, post_id: &str, user_id: &str) -> AppResult<Option<i8>> {
        let row = sqlx::query_as::<_, (i64,)>(
            "SELECT value FROM post_votes WHERE post_id = ? AND user_id = ?",
        )
        .bind(post_id)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
        Ok(row.map(|(v,)| v as i8))
    }

    async fn set_vote(&self, post_id: &str, user_id: &str, value: i8) -> AppResult<Post> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;

        let existing = sqlx::query_as::<_, (i64,)>(
            "SELECT value FROM post_votes WHERE post_id = ? AND user_id = ?",
        )
        .bind(post_id)
        .bind(user_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        let prev = existing.map(|(v,)| v as i8).unwrap_or(0);
        let next = value;

        if prev == next {
            // no-op path still returns post
        } else if next == 0 {
            sqlx::query("DELETE FROM post_votes WHERE post_id = ? AND user_id = ?")
                .bind(post_id)
                .bind(user_id)
                .execute(&mut *tx)
                .await
                .map_err(|e| AppError::Internal(e.to_string()))?;
        } else if prev == 0 {
            sqlx::query(
                "INSERT INTO post_votes (post_id, user_id, value) VALUES (?, ?, ?)",
            )
            .bind(post_id)
            .bind(user_id)
            .bind(next as i64)
            .execute(&mut *tx)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        } else {
            sqlx::query("UPDATE post_votes SET value = ? WHERE post_id = ? AND user_id = ?")
                .bind(next as i64)
                .bind(post_id)
                .bind(user_id)
                .execute(&mut *tx)
                .await
                .map_err(|e| AppError::Internal(e.to_string()))?;
        }

        // Recompute denormalized counts from votes table
        sqlx::query(
            r#"
            UPDATE posts SET
              upvotes = (SELECT COUNT(*) FROM post_votes WHERE post_id = ? AND value = 1),
              downvotes = (SELECT COUNT(*) FROM post_votes WHERE post_id = ? AND value = -1),
              score = (SELECT COALESCE(SUM(value), 0) FROM post_votes WHERE post_id = ?),
              updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(post_id)
        .bind(post_id)
        .bind(post_id)
        .bind(Utc::now().timestamp_millis())
        .bind(post_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        tx.commit()
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;

        self.get_post(post_id, Some(user_id))
            .await?
            .ok_or_else(|| AppError::NotFound("post not found".into()))
    }

    async fn insert_comment(
        &self,
        post_id: &str,
        author_user_id: &str,
        parent_comment_id: Option<&str>,
        body: &str,
    ) -> AppResult<PostComment> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().timestamp_millis();
        sqlx::query(
            r#"
            INSERT INTO post_comments (id, post_id, author_user_id, parent_comment_id, body, created_at)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&id)
        .bind(post_id)
        .bind(author_user_id)
        .bind(parent_comment_id)
        .bind(body)
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        self.get_comment(&id)
            .await?
            .ok_or_else(|| AppError::Internal("comment insert failed".into()))
    }

    async fn list_comments_flat(&self, post_id: &str) -> AppResult<Vec<PostComment>> {
        let rows = sqlx::query_as::<
            _,
            (
                String,
                String,
                String,
                Option<String>,
                Option<String>,
                String,
                i64,
                Option<i64>,
                Option<i64>,
            ),
        >(
            r#"
            SELECT c.id, c.post_id, c.author_user_id, u.display_name, c.parent_comment_id,
                   c.body, c.created_at, c.edited_at, c.deleted_at
            FROM post_comments c
            LEFT JOIN users u ON u.id = c.author_user_id
            WHERE c.post_id = ?
            ORDER BY c.created_at ASC
            "#,
        )
        .bind(post_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        rows.into_iter()
            .map(
                |(
                    id,
                    post_id,
                    author_user_id,
                    author_display_name,
                    parent_comment_id,
                    body,
                    created_at,
                    edited_at,
                    deleted_at,
                )| {
                    Ok(PostComment {
                        id,
                        post_id,
                        author_user_id,
                        author_display_name,
                        parent_comment_id,
                        body,
                        created_at: ts(created_at)?,
                        edited_at: opt_ts(edited_at)?,
                        deleted_at: opt_ts(deleted_at)?,
                        replies: vec![],
                    })
                },
            )
            .collect()
    }

    async fn get_comment(&self, comment_id: &str) -> AppResult<Option<PostComment>> {
        let row = sqlx::query_as::<
            _,
            (
                String,
                String,
                String,
                Option<String>,
                Option<String>,
                String,
                i64,
                Option<i64>,
                Option<i64>,
            ),
        >(
            r#"
            SELECT c.id, c.post_id, c.author_user_id, u.display_name, c.parent_comment_id,
                   c.body, c.created_at, c.edited_at, c.deleted_at
            FROM post_comments c
            LEFT JOIN users u ON u.id = c.author_user_id
            WHERE c.id = ?
            "#,
        )
        .bind(comment_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        row.map(
            |(
                id,
                post_id,
                author_user_id,
                author_display_name,
                parent_comment_id,
                body,
                created_at,
                edited_at,
                deleted_at,
            )| {
                Ok(PostComment {
                    id,
                    post_id,
                    author_user_id,
                    author_display_name,
                    parent_comment_id,
                    body,
                    created_at: ts(created_at)?,
                    edited_at: opt_ts(edited_at)?,
                    deleted_at: opt_ts(deleted_at)?,
                    replies: vec![],
                })
            },
        )
        .transpose()
    }
}
