use async_trait::async_trait;

use crate::domain::entities::User;
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
}
