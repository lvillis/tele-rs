use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{Error, Result};

/// Telegram file object.
#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct File {
    pub file_id: String,
    pub file_unique_id: String,
    #[serde(default)]
    pub file_size: Option<u64>,
    #[serde(default)]
    pub file_path: Option<String>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
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

    pub fn validate(&self) -> Result<()> {
        if self.file_id.trim().is_empty() {
            return Err(Error::InvalidRequest {
                reason: "file_id cannot be empty".to_owned(),
            });
        }
        if self.file_id.chars().any(char::is_control) {
            return Err(Error::InvalidRequest {
                reason: "file_id must not contain control characters".to_owned(),
            });
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_get_file_request() {
        assert!(GetFileRequest::new("file-1").validate().is_ok());

        for file_id in ["", " ", "file\n1"] {
            assert!(matches!(
                GetFileRequest::new(file_id).validate(),
                Err(Error::InvalidRequest { .. })
            ));
        }
    }

    #[test]
    fn file_preserves_future_fields() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let file: File = serde_json::from_value(serde_json::json!({
            "file_id": "file-id",
            "file_unique_id": "unique-id",
            "file_size": 42,
            "file_path": "documents/file.txt",
            "future_field": {"kept": true}
        }))?;

        assert_eq!(file.file_id, "file-id");
        assert_eq!(file.file_unique_id, "unique-id");
        assert_eq!(file.file_size, Some(42));
        assert_eq!(file.file_path.as_deref(), Some("documents/file.txt"));
        assert_eq!(
            file.extra["future_field"],
            serde_json::json!({"kept": true})
        );

        Ok(())
    }
}
