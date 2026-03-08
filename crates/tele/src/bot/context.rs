use super::*;

/// Request-scoped dispatch context passed to handlers and middlewares.
#[derive(Clone)]
pub struct BotContext {
    client: Client,
    request_state: RequestState,
}

impl BotContext {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            request_state: RequestState::default(),
        }
    }

    pub fn client(&self) -> &Client {
        &self.client
    }

    /// Returns a control-plane facade for setup/bootstrap helpers.
    pub fn control(&self) -> BotControl {
        BotControl::new(self.client.clone())
    }

    /// Returns the stable high-level Web App facade scoped to this bot client.
    pub fn web_app(&self) -> crate::client::WebAppApi {
        self.client.app().web_app()
    }

    pub fn request_state(&self) -> &RequestState {
        &self.request_state
    }

    pub fn bot(&self) -> BotService {
        self.client.bot()
    }

    pub fn messages(&self) -> MessagesService {
        self.client.messages()
    }

    pub fn chats(&self) -> ChatsService {
        self.client.chats()
    }

    pub fn files(&self) -> FilesService {
        self.client.files()
    }

    pub fn stickers(&self) -> StickersService {
        self.client.stickers()
    }

    pub fn payments(&self) -> PaymentsService {
        self.client.payments()
    }

    pub fn advanced(&self) -> AdvancedService {
        self.client.advanced()
    }

    pub fn updates(&self) -> UpdatesService {
        self.client.updates()
    }

    /// Sends plain text to a target chat.
    pub async fn send_text(
        &self,
        chat_id: impl Into<ChatId>,
        text: impl Into<String>,
    ) -> Result<Message> {
        let request = SendMessageRequest::new(chat_id, text)?;
        self.messages().send_message(&request).await
    }

    /// Replies with plain text using the canonical chat id extracted from update.
    pub async fn reply_text(&self, update: &Update, text: impl Into<String>) -> Result<Message> {
        let chat_id = crate::client::reply_chat_id(update)?;
        self.send_text(chat_id, text).await
    }

    /// Answers callback query by id.
    pub async fn answer_callback(
        &self,
        callback_query_id: impl Into<String>,
        text: Option<String>,
    ) -> Result<bool> {
        let request = AnswerCallbackQueryRequest {
            callback_query_id: callback_query_id.into(),
            text,
            show_alert: None,
            url: None,
            cache_time: None,
        };
        self.updates().answer_callback_query(&request).await
    }

    /// Answers callback query using id extracted from update.
    pub async fn answer_callback_from_update(
        &self,
        update: &Update,
        text: Option<String>,
    ) -> Result<bool> {
        let Some(callback_query_id) = update.callback_query.as_ref().map(|query| query.id.clone())
        else {
            return Err(invalid_request(
                "update does not contain callback query for answerCallbackQuery",
            ));
        };

        self.answer_callback(callback_query_id, text).await
    }

    /// Converts high-level handler error into transportable SDK result.
    pub async fn resolve_handler_error(&self, update: &Update, error: HandlerError) -> Result<()> {
        match error {
            HandlerError::Rejected(rejection) => {
                let _ = self.reply_text(update, rejection.message()).await?;
                Ok(())
            }
            HandlerError::Internal(error) => Err(error),
        }
    }
}
