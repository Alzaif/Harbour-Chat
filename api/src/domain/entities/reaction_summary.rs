use serde::Serialize;

#[derive(Clone, Debug, Default, Serialize)]
pub struct ReactionSummary {
    pub emoji: String,
    pub count: u32,
    #[serde(rename = "userIds")]
    pub user_ids: Vec<String>,
}
