use serde::Serialize;

use crate::domain::entities::Post;

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum FeedSectionKind {
    Top,
    Older,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FeedSection {
    pub kind: FeedSectionKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    pub posts: Vec<Post>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BoardFeed {
    pub period: String,
    pub sections: Vec<FeedSection>,
}
