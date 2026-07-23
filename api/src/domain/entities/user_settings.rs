use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserSettings {
    #[serde(rename = "pushToTalk")]
    pub push_to_talk: bool,
    #[serde(rename = "pushToTalkKey")]
    pub push_to_talk_key: String,
}

impl Default for UserSettings {
    fn default() -> Self {
        Self {
            push_to_talk: false,
            push_to_talk_key: "Space".into(),
        }
    }
}
