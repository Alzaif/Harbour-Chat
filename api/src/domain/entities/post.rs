use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct Post {
    pub id: String,
    #[serde(rename = "authorUserId")]
    pub author_user_id: String,
    #[serde(rename = "authorDisplayName")]
    pub author_display_name: Option<String>,
    pub title: Option<String>,
    pub body: String,
    #[serde(rename = "linkUrl")]
    pub link_url: Option<String>,
    #[serde(rename = "previewTitle")]
    pub preview_title: Option<String>,
    #[serde(rename = "previewDescription")]
    pub preview_description: Option<String>,
    #[serde(rename = "previewImageUrl")]
    pub preview_image_url: Option<String>,
    #[serde(rename = "previewSiteName")]
    pub preview_site_name: Option<String>,
    pub upvotes: i64,
    pub downvotes: i64,
    pub score: i64,
    #[serde(rename = "commentCount")]
    pub comment_count: i64,
    /// Current user's vote: -1, 0, or 1.
    #[serde(rename = "myVote")]
    pub my_vote: i8,
    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,
    #[serde(rename = "updatedAt")]
    pub updated_at: DateTime<Utc>,
}
