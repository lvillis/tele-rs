use serde::{Deserialize, Serialize};

/// Telegram file object.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct File {
    pub file_id: String,
    pub file_unique_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_size: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_path: Option<String>,
}

/// `getFile` request.
#[derive(Clone, Debug, Serialize)]
pub struct GetFileRequest {
    pub file_id: String,
}

impl GetFileRequest {
    pub fn new(file_id: impl Into<String>) -> Self {
        Self {
            file_id: file_id.into(),
        }
    }
}
