use crate::Result;
use crate::types::file::{File, GetFileRequest};

#[cfg(feature = "_blocking")]
use crate::BlockingClient;
#[cfg(feature = "_async")]
use crate::Client;

/// File API methods.
#[cfg(feature = "_async")]
#[derive(Clone)]
pub struct FilesService {
    client: Client,
}

#[cfg(feature = "_async")]
impl FilesService {
    pub(crate) fn new(client: Client) -> Self {
        Self { client }
    }

    /// Calls `getFile`.
    pub async fn get_file(&self, request: &GetFileRequest) -> Result<File> {
        self.client.call_method("getFile", request).await
    }
}

/// Blocking file methods.
#[cfg(feature = "_blocking")]
#[derive(Clone)]
pub struct BlockingFilesService {
    client: BlockingClient,
}

#[cfg(feature = "_blocking")]
impl BlockingFilesService {
    pub(crate) fn new(client: BlockingClient) -> Self {
        Self { client }
    }

    /// Calls `getFile`.
    pub fn get_file(&self, request: &GetFileRequest) -> Result<File> {
        self.client.call_method("getFile", request)
    }
}
