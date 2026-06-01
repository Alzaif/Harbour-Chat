use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct MessageAttachment {
    pub id: String,
    pub filename: String,
    pub mime_type: String,
    pub size_bytes: u64,
}
