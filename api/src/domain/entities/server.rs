use chrono::{DateTime, Utc};
use serde::Serialize;

use super::member::MemberRole;

#[derive(Clone, Debug, Serialize)]
pub struct Server {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub icon_url: Option<String>,
    #[serde(rename = "cardColor")]
    pub card_color: Option<String>,
    pub owner_user_id: String,
    #[serde(rename = "myRole", skip_serializing_if = "Option::is_none")]
    pub my_role: Option<MemberRole>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
