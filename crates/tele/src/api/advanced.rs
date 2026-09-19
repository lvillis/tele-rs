use serde::de::DeserializeOwned;

use crate::Result;
use crate::types::advanced::*;

#[cfg(feature = "_blocking")]
use crate::BlockingClient;
#[cfg(feature = "_async")]
use crate::Client;

#[cfg(feature = "_async")]
macro_rules! define_async_methods {
    ($(($fn_name:ident, $typed_name:ident, $method:literal, $request_ty:ty)),* $(,)?) => {
        $(
            pub async fn $fn_name<R>(&self, request: &$request_ty) -> Result<R>
            where
                R: DeserializeOwned,
            {
                request.validate()?;
                self.client.call_method($method, request).await
            }

            pub async fn $typed_name(
                &self,
                request: &$request_ty,
            ) -> Result<<$request_ty as AdvancedRequest>::Response> {
                self.call_typed(request).await
            }
        )*
    };
}

#[cfg(feature = "_blocking")]
macro_rules! define_blocking_methods {
    ($(($fn_name:ident, $typed_name:ident, $method:literal, $request_ty:ty)),* $(,)?) => {
        $(
            pub fn $fn_name<R>(&self, request: &$request_ty) -> Result<R>
            where
                R: DeserializeOwned,
            {
                request.validate()?;
                self.client.call_method($method, request)
            }

            pub fn $typed_name(
                &self,
                request: &$request_ty,
            ) -> Result<<$request_ty as AdvancedRequest>::Response> {
                self.call_typed(request)
            }
        )*
    };
}

include!("advanced_methods.inc.rs");

/// Additional Telegram Bot API methods with typed request models.
#[cfg(feature = "_async")]
#[derive(Clone)]
pub struct AdvancedService {
    client: Client,
}

#[cfg(feature = "_async")]
impl AdvancedService {
    pub(crate) fn new(client: Client) -> Self {
        Self { client }
    }

    /// Calls advanced methods using request-associated response type.
    pub async fn call_typed<Q>(&self, request: &Q) -> Result<Q::Response>
    where
        Q: AdvancedRequest,
    {
        request.validate()?;
        self.client.call_method(Q::METHOD, request).await
    }

    /// Sends a typed advanced request with named `attach://` file parts.
    pub async fn call_typed_with_files<Q>(
        &self,
        request: &Q,
        files: &[crate::types::UploadPart],
    ) -> Result<Q::Response>
    where
        Q: AdvancedRequest,
    {
        request.validate_for_upload()?;
        let skip_fields = prepare_attachments(request, files)?;
        let skip_fields = skip_fields.iter().map(String::as_str).collect::<Vec<_>>();
        self.client
            .call_method_multipart_files(Q::METHOD, request, &skip_fields, files)
            .await
    }

    with_advanced_methods!(define_async_methods);
}

/// Blocking additional Telegram Bot API methods with typed request models.
#[cfg(feature = "_blocking")]
#[derive(Clone)]
pub struct BlockingAdvancedService {
    client: BlockingClient,
}

#[cfg(feature = "_blocking")]
impl BlockingAdvancedService {
    pub(crate) fn new(client: BlockingClient) -> Self {
        Self { client }
    }

    /// Calls advanced methods using request-associated response type.
    pub fn call_typed<Q>(&self, request: &Q) -> Result<Q::Response>
    where
        Q: AdvancedRequest,
    {
        request.validate()?;
        self.client.call_method(Q::METHOD, request)
    }

    /// Sends a typed advanced request with named `attach://` file parts.
    pub fn call_typed_with_files<Q>(
        &self,
        request: &Q,
        files: &[crate::types::UploadPart],
    ) -> Result<Q::Response>
    where
        Q: AdvancedRequest,
    {
        request.validate_for_upload()?;
        let skip_fields = prepare_attachments(request, files)?;
        let skip_fields = skip_fields.iter().map(String::as_str).collect::<Vec<_>>();
        self.client
            .call_method_multipart_files(Q::METHOD, request, &skip_fields, files)
    }

    with_advanced_methods!(define_blocking_methods);
}

#[cfg(any(feature = "_async", feature = "_blocking"))]
fn prepare_attachments<Q: AdvancedRequest>(
    request: &Q,
    files: &[crate::types::UploadPart],
) -> Result<Vec<String>> {
    use std::collections::BTreeSet;
    let value = serde_json::to_value(request)
        .map_err(|source| crate::Error::SerializeRequest { source })?;
    let mut referenced = BTreeSet::new();
    crate::types::upload::visit_file_references(&value, &mut |reference| {
        if let Some(name) = reference.strip_prefix("attach://") {
            referenced.insert(name.to_owned());
        }
    });
    let provided = files
        .iter()
        .map(|file| file.field_name().to_owned())
        .collect::<BTreeSet<_>>();
    if files.is_empty() || provided.len() != files.len() || provided != referenced {
        return Err(crate::Error::InvalidRequest {
            reason:
                "multipart files must match attach:// references exactly, without duplicate names"
                    .to_owned(),
        });
    }
    // A top-level file parameter can be supplied directly under its own name.
    // Omit the attach:// placeholder in that case to avoid duplicate form fields.
    let skip_fields = value
        .as_object()
        .into_iter()
        .flatten()
        .filter_map(|(name, value)| {
            match value.as_str().and_then(|s| s.strip_prefix("attach://")) {
                Some(reference)
                    if reference == name
                        && provided.contains(name)
                        && crate::types::upload::is_file_reference_field(name) =>
                {
                    Some(name.clone())
                }
                _ => None,
            }
        })
        .collect();
    Ok(skip_fields)
}
