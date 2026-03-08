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

    /// Returns the request-scoped high-level facade for handler code.
    pub fn app(&self) -> ContextAppApi {
        ContextAppApi::new(self.client.clone())
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

    /// Converts high-level handler error into transportable SDK result.
    pub async fn resolve_handler_error(&self, update: &Update, error: HandlerError) -> Result<()> {
        match error {
            HandlerError::Rejected(rejection) => {
                let _ = self.app().reply_text(update, rejection.message()).await?;
                Ok(())
            }
            HandlerError::Internal(error) => Err(error),
        }
    }
}
