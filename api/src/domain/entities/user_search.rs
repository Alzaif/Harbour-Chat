use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct UserSearchResult {
    pub id: String,
    pub email: String,
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
}
