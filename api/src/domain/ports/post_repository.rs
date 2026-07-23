use async_trait::async_trait;

use crate::domain::entities::{Post, PostComment};
use crate::error::AppResult;

#[async_trait]
pub trait PostRepository: Send + Sync {
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
    ) -> AppResult<Post>;

    async fn list_posts_top(
        &self,
        since_ms: Option<i64>,
        viewer_user_id: &str,
        limit: u32,
    ) -> AppResult<Vec<Post>>;

    async fn list_posts_older(
        &self,
        before_ms: i64,
        viewer_user_id: &str,
        limit: u32,
    ) -> AppResult<Vec<Post>>;

    async fn get_post(&self, post_id: &str, viewer_user_id: Option<&str>) -> AppResult<Option<Post>>;

    async fn get_vote(&self, post_id: &str, user_id: &str) -> AppResult<Option<i8>>;

    async fn set_vote(&self, post_id: &str, user_id: &str, value: i8) -> AppResult<Post>;

    async fn insert_comment(
        &self,
        post_id: &str,
        author_user_id: &str,
        parent_comment_id: Option<&str>,
        body: &str,
    ) -> AppResult<PostComment>;

    async fn list_comments_flat(&self, post_id: &str) -> AppResult<Vec<PostComment>>;

    async fn get_comment(&self, comment_id: &str) -> AppResult<Option<PostComment>>;
}
