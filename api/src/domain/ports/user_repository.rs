use async_trait::async_trait;

use crate::domain::entities::{User, UserSearchResult, UserSettings};
use crate::error::AppResult;

#[derive(Clone, Debug)]
pub struct GatewayIdentity {
    pub id: String,
    pub email: String,
    pub display_name: Option<String>,
}

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn upsert_from_gateway(&self, identity: GatewayIdentity) -> AppResult<User>;
    async fn find_by_id(&self, id: &str) -> AppResult<Option<User>>;
    async fn search(
        &self,
        exclude_user_id: &str,
        query: &str,
        exclude_server_id: Option<&str>,
        limit: u32,
    ) -> AppResult<Vec<UserSearchResult>>;
    async fn get_settings(&self, user_id: &str) -> AppResult<UserSettings>;
    async fn upsert_settings(&self, user_id: &str, settings: &UserSettings) -> AppResult<UserSettings>;
}
