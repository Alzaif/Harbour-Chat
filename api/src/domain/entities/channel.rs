use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChannelType {
    Text,
    Voice,
    Dm,
}

impl ChannelType {
    pub fn as_str(self) -> &'static str {
        match self {
            ChannelType::Text => "text",
            ChannelType::Voice => "voice",
            ChannelType::Dm => "dm",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "text" => Some(ChannelType::Text),
            "voice" => Some(ChannelType::Voice),
            "dm" => Some(ChannelType::Dm),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct Channel {
    pub id: String,
    pub server_id: Option<String>,
    pub category_id: Option<String>,
    #[serde(rename = "type")]
    pub channel_type: ChannelType,
    pub name: String,
    pub position: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
