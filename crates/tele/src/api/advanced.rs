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
        self.client.call_method(Q::METHOD, request).await
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
        self.client.call_method(Q::METHOD, request)
    }

    with_advanced_methods!(define_blocking_methods);
}
