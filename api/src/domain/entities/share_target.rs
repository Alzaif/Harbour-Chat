use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct ShareTarget {
    #[serde(rename = "channelId")]
    pub channel_id: String,
    pub label: String,
    pub kind: String,
    #[serde(rename = "serverName")]
    pub server_name: Option<String>,
}
