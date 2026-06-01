use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MemberRole {
    Owner,
    Admin,
    Member,
}

impl MemberRole {
    pub fn as_str(self) -> &'static str {
        match self {
            MemberRole::Owner => "owner",
            MemberRole::Admin => "admin",
            MemberRole::Member => "member",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "owner" => Some(MemberRole::Owner),
            "admin" => Some(MemberRole::Admin),
            "member" => Some(MemberRole::Member),
            _ => None,
        }
    }

    pub fn can_moderate(self) -> bool {
        matches!(self, MemberRole::Owner | MemberRole::Admin)
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct Member {
    pub server_id: String,
    pub user_id: String,
    pub role: MemberRole,
    pub joined_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
}
