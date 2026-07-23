use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct PostComment {
    pub id: String,
    #[serde(rename = "postId")]
    pub post_id: String,
    #[serde(rename = "authorUserId")]
    pub author_user_id: String,
    #[serde(rename = "authorDisplayName")]
    pub author_display_name: Option<String>,
    #[serde(rename = "parentCommentId")]
    pub parent_comment_id: Option<String>,
    pub body: String,
    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,
    #[serde(rename = "editedAt")]
    pub edited_at: Option<DateTime<Utc>>,
    #[serde(rename = "deletedAt")]
    pub deleted_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub replies: Vec<PostComment>,
}
