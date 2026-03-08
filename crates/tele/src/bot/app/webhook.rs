use super::*;

#[derive(Clone, Debug, Default)]
pub struct WebhookConfig {
    pub expected_secret_token: Option<String>,
    pub continue_on_handler_error: bool,
}

/// Webhook dispatcher that can be embedded into any HTTP framework.
pub struct WebhookRunner {
    client: Client,
    router: Router,
    config: WebhookConfig,
    on_handler_error: Option<HandlerErrorHook>,
}

impl WebhookRunner {
    pub fn new(client: Client, router: Router) -> Self {
        Self {
            client,
            router,
            config: WebhookConfig::default(),
            on_handler_error: None,
        }
    }

    pub fn with_config(mut self, config: WebhookConfig) -> Self {
        self.config = config;
        self
    }

    pub fn expected_secret_token(mut self, secret_token: impl Into<String>) -> Self {
        self.config.expected_secret_token = Some(secret_token.into());
        self
    }

    pub fn continue_on_handler_error(mut self, enabled: bool) -> Self {
        self.config.continue_on_handler_error = enabled;
        self
    }

    pub fn on_handler_error<F>(mut self, hook: F) -> Self
    where
        F: Fn(i64, &Error) + Send + Sync + 'static,
    {
        self.on_handler_error = Some(Arc::new(hook));
        self
    }

    /// Runs setup bootstrap and prepares webhook router state.
    pub async fn bootstrap(&self, plan: &BootstrapPlan) -> BootstrapOutcome {
        super::bootstrap_router(&self.client, &self.router, plan).await
    }

    /// Runs setup bootstrap with retry/backoff and prepares webhook router state.
    pub async fn bootstrap_with_retry(
        &self,
        plan: &BootstrapPlan,
        policy: BootstrapRetryPolicy,
    ) -> BootstrapOutcome {
        super::bootstrap_router_with_retry(&self.client, &self.router, plan, policy).await
    }

    pub fn verify_secret_token(&self, incoming_secret: Option<&str>) -> bool {
        match self.config.expected_secret_token.as_deref() {
            None => true,
            Some(expected) => {
                incoming_secret.is_some_and(|incoming| constant_time_eq_str(incoming, expected))
            }
        }
    }

    /// Calls Telegram `setWebhook` with runner's secret token config.
    pub async fn configure_webhook(
        &self,
        url: impl Into<String>,
        drop_pending_updates: bool,
    ) -> Result<bool> {
        let mut request = SetWebhookRequest::new(url);
        request.secret_token = self.config.expected_secret_token.clone();
        request.drop_pending_updates = drop_pending_updates.then_some(true);
        self.client.updates().set_webhook(&request).await
    }

    /// Calls Telegram `deleteWebhook`.
    pub async fn delete_webhook(&self, drop_pending_updates: bool) -> Result<bool> {
        let request = DeleteWebhookRequest {
            drop_pending_updates: drop_pending_updates.then_some(true),
        };
        self.client.updates().delete_webhook(&request).await
    }

    /// Dispatches one already-deserialized update and returns structured outcome.
    pub async fn dispatch_update_outcome(&self, update: Update) -> Result<DispatchOutcome> {
        let update_id = update.update_id;
        let context = BotContext::new(self.client.clone());
        let result = self.router.dispatch_prepared(context, update).await;

        match result {
            Ok(true) => Ok(DispatchOutcome::Handled { update_id }),
            Ok(false) => Ok(DispatchOutcome::Ignored { update_id }),
            Err(error) => {
                self.notify_handler_error(update_id, &error);
                if self.config.continue_on_handler_error {
                    return Ok(DispatchOutcome::Ignored { update_id });
                }
                Err(error)
            }
        }
    }

    /// Verifies secret token and parses JSON into an `Update` without dispatching.
    pub fn parse_update_json(
        &self,
        payload: &[u8],
        incoming_secret: Option<&str>,
    ) -> Result<Update> {
        if !self.verify_secret_token(incoming_secret) {
            return Err(invalid_request("invalid webhook secret token"));
        }

        serde_json::from_slice(payload).map_err(|source| {
            invalid_request(format!(
                "failed to deserialize webhook update payload: {source}"
            ))
        })
    }

    /// Verifies secret token, parses JSON update and dispatches it with structured outcome.
    pub async fn dispatch_json_outcome(
        &self,
        payload: &[u8],
        incoming_secret: Option<&str>,
    ) -> Result<DispatchOutcome> {
        let update = self.parse_update_json(payload, incoming_secret)?;

        self.dispatch_update_outcome(update).await
    }

    fn notify_handler_error(&self, update_id: i64, error: &Error) {
        if let Some(hook) = self.on_handler_error.as_ref() {
            hook(update_id, error);
        }
    }
}

fn constant_time_eq_str(left: &str, right: &str) -> bool {
    if left.len() != right.len() {
        return false;
    }

    let mut diff = 0_u8;
    for (lhs, rhs) in left.as_bytes().iter().zip(right.as_bytes().iter()) {
        diff |= lhs ^ rhs;
    }
    diff == 0
}
