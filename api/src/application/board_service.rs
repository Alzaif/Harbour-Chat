use std::sync::Arc;

use chrono::{Duration, Utc};

use crate::application::ChatService;
use crate::contracts::events::PostCreatedV1;
use crate::domain::board_feed::BOARD_FEED_TOPIC;
use crate::domain::entities::{
    BoardFeed, ChannelType, FeedSection, FeedSectionKind, Message, Post, PostComment, ShareTarget,
    User,
};
use crate::domain::ports::{PostRepository, RealtimeEvent, RealtimePublisher};
use crate::error::{AppError, AppResult};

const MAX_COMMENT_LENGTH: usize = 4000;
const MAX_COMMENT_DEPTH: usize = 8;

fn site_name_from_url(url: &str) -> Option<&str> {
    let rest = url.split("://").nth(1).unwrap_or(url);
    let host = rest.split('/').next()?.split(':').next()?;
    if host.is_empty() {
        None
    } else {
        Some(host)
    }
}

pub fn format_share_content(post: &Post) -> String {
    let mut lines = vec!["📌 Shared from Board".to_string()];
    if let Some(title) = &post.title {
        if !title.is_empty() {
            lines.push(title.clone());
        }
    }
    let excerpt: String = post.body.chars().take(280).collect();
    if post.body.chars().count() > 280 {
        lines.push(format!("{excerpt}…"));
    } else {
        lines.push(excerpt);
    }
    lines.push(format!("/board/feed/{}", post.id));
    if let Some(url) = &post.link_url {
        if !url.is_empty() {
            lines.push(url.clone());
        }
    }
    lines.join("\n")
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FeedPeriod {
    Hour,
    Day,
    Week,
    Month,
    Year,
    All,
}

impl FeedPeriod {
    pub fn parse(raw: &str) -> AppResult<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "hour" | "1h" | "1hour" => Ok(Self::Hour),
            "day" | "24h" | "24hours" => Ok(Self::Day),
            "week" => Ok(Self::Week),
            "month" => Ok(Self::Month),
            "year" => Ok(Self::Year),
            "all" | "alltime" | "all_time" => Ok(Self::All),
            _ => Err(AppError::Validation(
                "period must be one of: hour, day, week, month, year, all".into(),
            )),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Hour => "hour",
            Self::Day => "day",
            Self::Week => "week",
            Self::Month => "month",
            Self::Year => "year",
            Self::All => "all",
        }
    }

    pub fn older_than_label(self) -> Option<&'static str> {
        match self {
            Self::Hour => Some("Posts older than 1 hour"),
            Self::Day => Some("Posts older than 24 hours"),
            Self::Week => Some("Posts older than week"),
            Self::Month => Some("Posts older than month"),
            Self::Year => Some("Posts older than year"),
            Self::All => None,
        }
    }

    pub fn window_start_ms(self, now_ms: i64) -> Option<i64> {
        let duration = match self {
            Self::Hour => Duration::hours(1),
            Self::Day => Duration::hours(24),
            Self::Week => Duration::days(7),
            Self::Month => Duration::days(30),
            Self::Year => Duration::days(365),
            Self::All => return None,
        };
        Some(now_ms - duration.num_milliseconds())
    }
}

pub struct BoardService {
    posts: Arc<dyn PostRepository>,
    realtime: Arc<dyn RealtimePublisher>,
    chat: Arc<ChatService>,
}

impl BoardService {
    pub fn new(
        posts: Arc<dyn PostRepository>,
        realtime: Arc<dyn RealtimePublisher>,
        chat: Arc<ChatService>,
    ) -> Self {
        Self {
            posts,
            realtime,
            chat,
        }
    }

    pub async fn create_post(
        &self,
        user: &User,
        title: Option<&str>,
        body: &str,
        link_url: Option<&str>,
    ) -> AppResult<Post> {
        let body = body.trim();
        if body.is_empty() {
            return Err(AppError::Validation("body is required".into()));
        }
        if body.len() > 8000 {
            return Err(AppError::Validation("body must be at most 8000 characters".into()));
        }
        let title = title.map(str::trim).filter(|t| !t.is_empty());
        let link_url = link_url.map(str::trim).filter(|u| !u.is_empty());

        let post = self
            .posts
            .insert_post(
                &user.id,
                title,
                body,
                link_url,
                None,
                None,
                None,
                link_url.and_then(site_name_from_url),
            )
            .await?;

        let post_json = serde_json::to_value(&post)
            .map_err(|e| AppError::Internal(e.to_string()))?;
        self.realtime
            .publish(
                BOARD_FEED_TOPIC,
                RealtimeEvent::PostCreated {
                    post: post_json.clone(),
                },
            )
            .await?;

        let _contract = PostCreatedV1::new(
            post.id.clone(),
            post.author_user_id.clone(),
            post.created_at.to_rfc3339(),
        );

        Ok(post)
    }

    pub async fn list_feed(
        &self,
        user: &User,
        period: FeedPeriod,
        limit: u32,
    ) -> AppResult<BoardFeed> {
        let limit = limit.clamp(1, 50);
        let now_ms = Utc::now().timestamp_millis();
        let since_ms = period.window_start_ms(now_ms);

        let top = self
            .posts
            .list_posts_top(since_ms, &user.id, limit)
            .await?;

        let mut sections = vec![FeedSection {
            kind: FeedSectionKind::Top,
            label: None,
            posts: top,
        }];

        if let (Some(cutoff), Some(label)) = (since_ms, period.older_than_label()) {
            let older = self
                .posts
                .list_posts_older(cutoff, &user.id, limit)
                .await?;
            if !older.is_empty() {
                sections.push(FeedSection {
                    kind: FeedSectionKind::Older,
                    label: Some(label.to_string()),
                    posts: older,
                });
            }
        }

        Ok(BoardFeed {
            period: period.as_str().to_string(),
            sections,
        })
    }

    pub async fn get_post(&self, user: &User, post_id: &str) -> AppResult<Post> {
        self.posts
            .get_post(post_id, Some(&user.id))
            .await?
            .ok_or_else(|| AppError::NotFound("post not found".into()))
    }

    /// Vote value: 1 upvote, -1 downvote, 0 clear. Same value again clears.
    pub async fn vote_post(&self, user: &User, post_id: &str, value: i8) -> AppResult<Post> {
        if !(-1..=1).contains(&value) {
            return Err(AppError::Validation("value must be -1, 0, or 1".into()));
        }
        let _ = self.get_post(user, post_id).await?;
        let existing = self.posts.get_vote(post_id, &user.id).await?.unwrap_or(0);
        let next = if value != 0 && existing == value { 0 } else { value };
        self.posts.set_vote(post_id, &user.id, next).await
    }

    pub async fn list_comments(&self, user: &User, post_id: &str) -> AppResult<Vec<PostComment>> {
        let _ = self.get_post(user, post_id).await?;
        let flat = self.posts.list_comments_flat(post_id).await?;
        Ok(nest_comments(flat))
    }

    pub async fn create_comment(
        &self,
        user: &User,
        post_id: &str,
        body: &str,
        parent_comment_id: Option<&str>,
    ) -> AppResult<PostComment> {
        let _ = self.get_post(user, post_id).await?;
        let body = body.trim();
        if body.is_empty() {
            return Err(AppError::Validation("body is required".into()));
        }
        if body.len() > MAX_COMMENT_LENGTH {
            return Err(AppError::Validation(format!(
                "body must be at most {MAX_COMMENT_LENGTH} characters"
            )));
        }

        if let Some(parent_id) = parent_comment_id {
            let parent = self
                .posts
                .get_comment(parent_id)
                .await?
                .ok_or_else(|| AppError::NotFound("parent comment not found".into()))?;
            if parent.post_id != post_id {
                return Err(AppError::Validation(
                    "parent comment must belong to the same post".into(),
                ));
            }
            let depth = comment_depth(&parent, &*self.posts).await?;
            if depth >= MAX_COMMENT_DEPTH {
                return Err(AppError::Validation(format!(
                    "comment nesting is limited to {MAX_COMMENT_DEPTH} levels"
                )));
            }
        }

        self.posts
            .insert_comment(post_id, &user.id, parent_comment_id, body)
            .await
    }

    pub async fn list_share_targets(&self, user: &User) -> AppResult<Vec<ShareTarget>> {
        let mut targets = Vec::new();

        for dm in self.chat.list_dms(user).await? {
            let label = dm
                .other_display_name
                .clone()
                .unwrap_or_else(|| dm.other_user_id.clone());
            targets.push(ShareTarget {
                channel_id: dm.channel_id,
                label: format!("Direct · {label}"),
                kind: "dm".into(),
                server_name: None,
            });
        }

        for server in self.chat.list_servers(user).await? {
            let detail = self.chat.get_server_detail(user, &server.id).await?;
            for channel in detail.channels {
                if channel.channel_type != ChannelType::Text {
                    continue;
                }
                targets.push(ShareTarget {
                    channel_id: channel.id,
                    label: format!("{} / #{}", server.name, channel.name),
                    kind: "channel".into(),
                    server_name: Some(server.name.clone()),
                });
            }
        }

        Ok(targets)
    }

    pub async fn share_post(
        &self,
        user: &User,
        post_id: &str,
        channel_id: &str,
    ) -> AppResult<Message> {
        let post = self.get_post(user, post_id).await?;
        let content = format_share_content(&post);
        self.chat.send_message(user, channel_id, &content, None).await
    }
}

fn nest_comments(flat: Vec<PostComment>) -> Vec<PostComment> {
    use std::collections::HashMap;

    let mut by_parent: HashMap<Option<String>, Vec<PostComment>> = HashMap::new();
    for mut c in flat {
        // Soft-deleted comments keep tree shape with placeholder body.
        if c.deleted_at.is_some() {
            c.body = "[deleted]".into();
        }
        c.replies = vec![];
        by_parent
            .entry(c.parent_comment_id.clone())
            .or_default()
            .push(c);
    }

    fn attach(
        parent_id: Option<String>,
        by_parent: &mut HashMap<Option<String>, Vec<PostComment>>,
    ) -> Vec<PostComment> {
        let mut children = by_parent.remove(&parent_id).unwrap_or_default();
        for child in &mut children {
            child.replies = attach(Some(child.id.clone()), by_parent);
        }
        children
    }

    attach(None, &mut by_parent)
}

async fn comment_depth(
    comment: &PostComment,
    posts: &dyn PostRepository,
) -> AppResult<usize> {
    let mut depth = 1usize;
    let mut current = comment.parent_comment_id.clone();
    while let Some(id) = current {
        depth += 1;
        if depth > MAX_COMMENT_DEPTH {
            break;
        }
        current = posts
            .get_comment(&id)
            .await?
            .and_then(|c| c.parent_comment_id);
    }
    Ok(depth)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn sample_post() -> Post {
        Post {
            id: "post-1".into(),
            author_user_id: "u1".into(),
            author_display_name: Some("Alice".into()),
            title: Some("News".into()),
            body: "Hello world".into(),
            link_url: Some("https://example.com".into()),
            preview_title: None,
            preview_description: None,
            preview_image_url: None,
            preview_site_name: None,
            upvotes: 0,
            downvotes: 0,
            score: 0,
            comment_count: 0,
            my_vote: 0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn format_share_content_includes_feed_link() {
        let text = format_share_content(&sample_post());
        assert!(text.contains("Shared from Board"));
        assert!(text.contains("News"));
        assert!(text.contains("/board/feed/post-1"));
        assert!(text.contains("https://example.com"));
    }

    #[test]
    fn feed_period_window_and_labels() {
        assert!(FeedPeriod::parse("day").unwrap().older_than_label().is_some());
        assert!(FeedPeriod::parse("all").unwrap().older_than_label().is_none());
        let now = 1_700_000_000_000i64;
        let since = FeedPeriod::Hour.window_start_ms(now).unwrap();
        assert_eq!(now - since, Duration::hours(1).num_milliseconds());
    }

    #[test]
    fn nest_comments_builds_tree() {
        let flat = vec![
            PostComment {
                id: "c1".into(),
                post_id: "p".into(),
                author_user_id: "u".into(),
                author_display_name: None,
                parent_comment_id: None,
                body: "root".into(),
                created_at: Utc::now(),
                edited_at: None,
                deleted_at: None,
                replies: vec![],
            },
            PostComment {
                id: "c2".into(),
                post_id: "p".into(),
                author_user_id: "u".into(),
                author_display_name: None,
                parent_comment_id: Some("c1".into()),
                body: "child".into(),
                created_at: Utc::now(),
                edited_at: None,
                deleted_at: None,
                replies: vec![],
            },
        ];
        let tree = nest_comments(flat);
        assert_eq!(tree.len(), 1);
        assert_eq!(tree[0].replies.len(), 1);
        assert_eq!(tree[0].replies[0].body, "child");
    }
}
