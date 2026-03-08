use super::*;

#[cfg(feature = "bot")]
fn single_attempt_bootstrap_policy() -> BootstrapRetryPolicy {
    BootstrapRetryPolicy {
        max_attempts: 1,
        continue_on_failure: false,
        ..BootstrapRetryPolicy::default()
    }
}

/// Control-plane facade for setup and runtime orchestration.
#[cfg(feature = "_async")]
#[derive(Clone)]
pub struct ControlApi {
    client: Client,
}

#[cfg(feature = "_async")]
impl ControlApi {
    pub(crate) fn new(client: Client) -> Self {
        Self { client }
    }

    pub fn setup(&self) -> SetupApi {
        SetupApi::new(self.client.clone())
    }

    #[cfg(feature = "bot")]
    pub fn spawn_outbox(&self, config: crate::bot::OutboxConfig) -> crate::bot::BotOutbox {
        crate::bot::BotOutbox::spawn(self.client.clone(), config)
    }

    #[cfg(feature = "bot")]
    pub async fn bootstrap_router(
        &self,
        router: &crate::bot::Router,
        plan: &BootstrapPlan,
    ) -> BootstrapOutcome {
        self.bootstrap_router_with_retry(router, plan, single_attempt_bootstrap_policy())
            .await
    }

    #[cfg(feature = "bot")]
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

/// Blocking control-plane facade for setup orchestration.
#[cfg(feature = "_blocking")]
#[derive(Clone)]
pub struct BlockingControlApi {
    client: BlockingClient,
}

#[cfg(feature = "_blocking")]
impl BlockingControlApi {
    pub(crate) fn new(client: BlockingClient) -> Self {
        Self { client }
    }

    pub fn setup(&self) -> BlockingSetupApi {
        BlockingSetupApi::new(self.client.clone())
    }
}
