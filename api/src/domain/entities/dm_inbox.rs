use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct DmInboxEntry {
    #[serde(rename = "channelId")]
    pub channel_id: String,
    #[serde(rename = "otherUserId")]
    pub other_user_id: String,
    #[serde(rename = "otherDisplayName")]
    pub other_display_name: Option<String>,
    #[serde(rename = "lastMessagePreview")]
    pub last_message_preview: Option<String>,
    #[serde(rename = "unreadCount")]
    pub unread_count: u32,
    #[serde(rename = "updatedAt")]
    pub updated_at: i64,
}

#[derive(Clone, Debug, Serialize)]
pub struct DmPeer {
    pub id: String,
    pub email: String,
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
}
