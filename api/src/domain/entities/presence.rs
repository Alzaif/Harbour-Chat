use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PresenceStatus {
    Online,
    Idle,
    Dnd,
    Offline,
}

impl PresenceStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            PresenceStatus::Online => "online",
            PresenceStatus::Idle => "idle",
            PresenceStatus::Dnd => "dnd",
            PresenceStatus::Offline => "offline",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "online" => Some(PresenceStatus::Online),
            "idle" => Some(PresenceStatus::Idle),
            "dnd" => Some(PresenceStatus::Dnd),
            "offline" => Some(PresenceStatus::Offline),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct Presence {
    pub server_id: String,
    pub user_id: String,
    pub status: PresenceStatus,
    pub updated_at: DateTime<Utc>,
}
