use serde::{Deserialize, Serialize};

use crate::{Error, Result};

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
}
