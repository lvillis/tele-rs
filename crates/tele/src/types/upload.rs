use std::path::Path;
use std::sync::Arc;

use crate::{Error, Result};

/// Binary file payload for Telegram multipart uploads.
#[derive(Clone, Debug)]
pub struct UploadFile {
    file_name: String,
    content_type: Option<String>,
    data: Arc<[u8]>,
}

impl UploadFile {
    /// Build from in-memory bytes.
    pub fn from_bytes(file_name: impl Into<String>, data: Vec<u8>) -> Result<Self> {
        let file_name = file_name.into();
        if file_name.trim().is_empty() {
            return Err(Error::InvalidRequest {
                reason: "upload file_name cannot be empty".to_owned(),
            });
        }

        Ok(Self {
            file_name,
            content_type: None,
            data: data.into(),
        })
    }

    /// Load file content from local filesystem.
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let data = std::fs::read(path).map_err(|source| Error::ReadLocalFile {
            path: path.display().to_string(),
            source,
        })?;

        let file_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| Error::InvalidRequest {
                reason: "path does not contain a valid UTF-8 file name".to_owned(),
            })?
            .to_owned();

        Self::from_bytes(file_name, data)
    }

    /// Attach a specific content-type for this file part.
    pub fn with_content_type(mut self, content_type: impl Into<String>) -> Self {
        self.content_type = Some(content_type.into());
        self
    }

    pub(crate) fn file_name(&self) -> &str {
        &self.file_name
    }

    pub(crate) fn content_type(&self) -> Option<&str> {
        self.content_type.as_deref()
    }

    pub(crate) fn data_arc(&self) -> Arc<[u8]> {
        Arc::clone(&self.data)
    }
}
