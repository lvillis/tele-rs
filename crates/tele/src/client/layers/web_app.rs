use super::*;

fn build_answer_web_app_request<T>(
    web_app_query_id: impl Into<String>,
    result: T,
) -> Result<AdvancedAnswerWebAppQueryRequest>
where
    T: Serialize,
{
    let result = InlineQueryResult::from_typed(result).map_err(|source| Error::InvalidRequest {
        reason: format!("failed to serialize WebApp inline result: {source}"),
    })?;
    Ok(AdvancedAnswerWebAppQueryRequest::new(
        web_app_query_id,
        result,
    ))
}

#[cfg(feature = "_async")]
#[derive(Clone)]
pub struct WebAppApi {
    client: Client,
}

#[cfg(feature = "_async")]
impl WebAppApi {
    pub(crate) fn new(client: Client) -> Self {
        Self { client }
    }

    pub fn parse_query_payload<T>(&self, web_app_data: &WebAppData) -> Result<WebAppQueryPayload<T>>
    where
        T: DeserializeOwned,
    {
        WebAppQueryPayload::parse(web_app_data)
    }

    pub async fn set_menu_button(
        &self,
        text: impl Into<String>,
        web_app: impl Into<WebAppInfo>,
    ) -> Result<bool> {
        let request = crate::types::advanced::AdvancedSetChatMenuButtonRequest::from(
            MenuButtonConfig::web_app(text, web_app),
        );
        self.client
            .advanced()
            .set_chat_menu_button_typed(&request)
            .await
    }

    pub async fn set_chat_menu_button(
        &self,
        chat_id: i64,
        text: impl Into<String>,
        web_app: impl Into<WebAppInfo>,
    ) -> Result<bool> {
        let request = crate::types::advanced::AdvancedSetChatMenuButtonRequest::from(
            MenuButtonConfig::for_chat_web_app(chat_id, text, web_app),
        );
        self.client
            .advanced()
            .set_chat_menu_button_typed(&request)
            .await
    }

    pub async fn answer_query<T>(
        &self,
        web_app_query_id: impl Into<String>,
        result: T,
    ) -> Result<SentWebAppMessage>
    where
        T: Serialize,
    {
        let request = build_answer_web_app_request(web_app_query_id, result)?;
        self.client
            .advanced()
            .answer_web_app_query_typed(&request)
            .await
    }

    pub async fn answer_query_result(
        &self,
        web_app_query_id: impl Into<String>,
        result: InlineQueryResult,
    ) -> Result<SentWebAppMessage> {
        let request = AdvancedAnswerWebAppQueryRequest::new(web_app_query_id, result);
        self.client
            .advanced()
            .answer_web_app_query_typed(&request)
            .await
    }

    pub async fn answer_query_from_payload<T, R>(
        &self,
        web_app_data: &WebAppData,
        result: R,
    ) -> Result<SentWebAppMessage>
    where
        T: DeserializeOwned,
        R: Serialize,
    {
        let envelope = self.parse_query_payload::<T>(web_app_data)?;
        self.answer_query(envelope.query_id, result).await
    }
}

#[cfg(feature = "_blocking")]
#[derive(Clone)]
pub struct BlockingWebAppApi {
    client: BlockingClient,
}

#[cfg(feature = "_blocking")]
impl BlockingWebAppApi {
    pub(crate) fn new(client: BlockingClient) -> Self {
        Self { client }
    }

    pub fn parse_query_payload<T>(&self, web_app_data: &WebAppData) -> Result<WebAppQueryPayload<T>>
    where
        T: DeserializeOwned,
    {
        WebAppQueryPayload::parse(web_app_data)
    }

    pub fn set_menu_button(
        &self,
        text: impl Into<String>,
        web_app: impl Into<WebAppInfo>,
    ) -> Result<bool> {
        let request = crate::types::advanced::AdvancedSetChatMenuButtonRequest::from(
            MenuButtonConfig::web_app(text, web_app),
        );
        self.client.advanced().set_chat_menu_button_typed(&request)
    }

    pub fn set_chat_menu_button(
        &self,
        chat_id: i64,
        text: impl Into<String>,
        web_app: impl Into<WebAppInfo>,
    ) -> Result<bool> {
        let request = crate::types::advanced::AdvancedSetChatMenuButtonRequest::from(
            MenuButtonConfig::for_chat_web_app(chat_id, text, web_app),
        );
        self.client.advanced().set_chat_menu_button_typed(&request)
    }

    pub fn answer_query<T>(
        &self,
        web_app_query_id: impl Into<String>,
        result: T,
    ) -> Result<SentWebAppMessage>
    where
        T: Serialize,
    {
        let request = build_answer_web_app_request(web_app_query_id, result)?;
        self.client.advanced().answer_web_app_query_typed(&request)
    }

    pub fn answer_query_result(
        &self,
        web_app_query_id: impl Into<String>,
        result: InlineQueryResult,
    ) -> Result<SentWebAppMessage> {
        let request = AdvancedAnswerWebAppQueryRequest::new(web_app_query_id, result);
        self.client.advanced().answer_web_app_query_typed(&request)
    }

    pub fn answer_query_from_payload<T, R>(
        &self,
        web_app_data: &WebAppData,
        result: R,
    ) -> Result<SentWebAppMessage>
    where
        T: DeserializeOwned,
        R: Serialize,
    {
        let envelope = self.parse_query_payload::<T>(web_app_data)?;
        self.answer_query(envelope.query_id, result)
    }
}
