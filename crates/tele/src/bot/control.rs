use super::*;

/// Control-plane helper facade built from a client.
#[derive(Clone)]
pub struct BotControl {
    client: Client,
}

impl BotControl {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    pub fn client(&self) -> &Client {
        &self.client
    }

    /// App setup facade for commands, menu buttons and bootstrap.
    pub fn setup(&self) -> crate::client::SetupApi {
        self.client.app().setup()
    }

    /// Stable Web App facade for menu button setup and query responses.
    pub fn web_app(&self) -> crate::client::WebAppApi {
        self.client.app().web_app()
    }

    /// Spawns a reliable outbox worker for send-side retry, throttling and idempotency.
    pub fn spawn_outbox(&self, config: OutboxConfig) -> BotOutbox {
        BotOutbox::spawn(self.client.clone(), config)
    }

    /// Runs setup bootstrap and prepares router command-target state when `getMe` succeeded.
    pub async fn bootstrap_router(
        &self,
        router: &crate::bot::Router,
        plan: &BootstrapPlan,
    ) -> BootstrapOutcome {
        self.bootstrap_router_with_retry(
            router,
            plan,
            BootstrapRetryPolicy {
                max_attempts: 1,
                continue_on_failure: false,
                ..BootstrapRetryPolicy::default()
            },
        )
        .await
    }

    /// Runs setup bootstrap with retry/backoff and prepares router state when possible.
    pub async fn bootstrap_router_with_retry(
        &self,
        router: &crate::bot::Router,
        plan: &BootstrapPlan,
        policy: BootstrapRetryPolicy,
    ) -> BootstrapOutcome {
        let mut outcome = self.setup().bootstrap_with_retry(plan, policy).await;
        if outcome.error.is_some() {
            return outcome;
        }

        // Do not trigger a second `getMe`; router prep should honor the bootstrap step policy.
        if let Some(me) = outcome.report.me.value.as_ref()
            && let Err(error) = router.prepare_with_user(me)
        {
            outcome.error = Some(error);
        } else if outcome.report.me.value.is_none() {
            let _ = router.disable_auto_command_target();
        }

        outcome
    }
}
