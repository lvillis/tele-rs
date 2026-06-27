use std::path::Path;
use std::sync::Arc;

use http::header::HeaderValue;

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
        validate_upload_file_name(&file_name)?;

        Ok(Self {
            file_name,
            content_type: None,
            data: data.into(),
        })
    }

    /// Load file content from local filesystem.
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let file_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| Error::InvalidRequest {
                reason: "path does not contain a valid UTF-8 file name".to_owned(),
            })?
            .to_owned();
        validate_upload_file_name(&file_name)?;

        let data = std::fs::read(path).map_err(|source| Error::ReadLocalFile {
            path: path.display().to_string(),
            source,
        })?;

        Ok(Self {
            file_name,
            content_type: None,
            data: data.into(),
        })
    }

    /// Attach a specific content-type for this file part.
    pub fn with_content_type(mut self, content_type: impl Into<String>) -> Result<Self> {
        let content_type = content_type.into();
        if content_type.trim().is_empty() {
            return Err(Error::InvalidRequest {
                reason: "upload content_type cannot be empty".to_owned(),
            });
        }
        validate_multipart_header_fragment("upload content_type", &content_type)?;
        HeaderValue::from_str(&content_type).map_err(|source| Error::InvalidHeaderValue {
            name: "content-type".to_owned(),
            source,
        })?;
        self.content_type = Some(content_type);
        Ok(self)
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

/// Named multipart file part for methods that upload multiple files.
#[derive(Clone, Debug)]
pub struct UploadPart {
    field_name: String,
    file: UploadFile,
}

impl UploadPart {
    /// Build a named multipart file part.
    ///
    /// The `field_name` is the name referenced by Telegram `attach://field_name` values.
    pub fn new(field_name: impl Into<String>, file: UploadFile) -> Result<Self> {
        let field_name = field_name.into();
        validate_upload_part_name("upload part field_name", &field_name)?;

        Ok(Self { field_name, file })
    }

    /// Build a named multipart file part from in-memory bytes.
    pub fn from_bytes(
        field_name: impl Into<String>,
        file_name: impl Into<String>,
        data: Vec<u8>,
    ) -> Result<Self> {
        Self::new(field_name, UploadFile::from_bytes(file_name, data)?)
    }

    /// Build a named multipart file part from a local file path.
    pub fn from_path(field_name: impl Into<String>, path: impl AsRef<Path>) -> Result<Self> {
        Self::new(field_name, UploadFile::from_path(path)?)
    }

    /// Returns the multipart field name.
    pub fn field_name(&self) -> &str {
        &self.field_name
    }

    /// Returns the file payload.
    pub fn file(&self) -> &UploadFile {
        &self.file
    }

    /// Returns the Telegram attach URI for this part.
    pub fn attach_uri(&self) -> String {
        format!("attach://{}", self.field_name)
    }
}

fn validate_upload_file_name(file_name: &str) -> Result<()> {
    if file_name.trim().is_empty() {
        return Err(Error::InvalidRequest {
            reason: "upload file_name cannot be empty".to_owned(),
        });
    }
    if file_name == "." || file_name == ".." || file_name.contains(['/', '\\']) {
        return Err(Error::InvalidRequest {
            reason: "upload file_name must be a plain file name without path separators".to_owned(),
        });
    }
    validate_multipart_header_fragment("upload file_name", file_name)
}

fn validate_multipart_header_fragment(label: &str, value: &str) -> Result<()> {
    if value.chars().any(char::is_control) {
        return Err(Error::InvalidRequest {
            reason: format!("{label} must not contain control characters"),
        });
    }
    Ok(())
}

pub(crate) fn validate_upload_part_name(label: &str, value: &str) -> Result<()> {
    if value.is_empty() {
        return Err(Error::InvalidRequest {
            reason: format!("{label} cannot be empty"),
        });
    }
    if !value
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.'))
    {
        return Err(Error::InvalidRequest {
            reason: format!("{label} must contain only ASCII letters, digits, `_`, `-`, or `.`"),
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::Error;

    use super::{UploadFile, UploadPart};

    #[test]
    fn rejects_header_injection_metadata() -> std::result::Result<(), Box<dyn std::error::Error>> {
        assert!(UploadFile::from_bytes("a\r\nx: y", Vec::new()).is_err());
        let file = UploadFile::from_bytes("a.txt", Vec::new())?;
        assert!(file.clone().with_content_type("").is_err());
        assert!(file.with_content_type("text/plain\r\nx: y").is_err());
        Ok(())
    }

    #[test]
    fn rejects_upload_file_names_with_path_semantics() {
        for file_name in [
            "../secret.txt",
            "nested/photo.jpg",
            r"nested\photo.jpg",
            ".",
            "..",
        ] {
            assert!(matches!(
                UploadFile::from_bytes(file_name, Vec::new()),
                Err(Error::InvalidRequest { .. })
            ));
        }
    }

    #[test]
    fn validates_upload_part_names() -> crate::Result<()> {
        let file = UploadFile::from_bytes("a.txt", Vec::new())?;
        let part = UploadPart::new("photo_0", file.clone())?;
        assert_eq!(part.field_name(), "photo_0");
        assert_eq!(part.attach_uri(), "attach://photo_0");

        assert!(UploadPart::new("", file.clone()).is_err());
        assert!(UploadPart::new("bad/name", file.clone()).is_err());
        assert!(UploadPart::new("bad\r\nname", file).is_err());
        Ok(())
    }

    #[test]
    fn from_path_rejects_missing_file_name_before_reading()
    -> std::result::Result<(), Box<dyn std::error::Error>> {
        let error = match UploadFile::from_path(Path::new("")) {
            Ok(file) => return Err(format!("expected invalid upload path, got {file:?}").into()),
            Err(error) => error,
        };

        assert!(matches!(error, Error::InvalidRequest { .. }));
        Ok(())
    }
}
