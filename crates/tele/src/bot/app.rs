use super::*;

mod engine;
mod webhook;

pub use engine::*;
pub use webhook::*;

async fn bootstrap_router(
    client: &Client,
    router: &Router,
    plan: &BootstrapPlan,
) -> BootstrapOutcome {
    bootstrap_router_with_retry(
        client,
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

async fn bootstrap_router_with_retry(
    client: &Client,
    router: &Router,
    plan: &BootstrapPlan,
    policy: BootstrapRetryPolicy,
) -> BootstrapOutcome {
    BotControl::new(client.clone())
        .bootstrap_router_with_retry(router, plan, policy)
        .await
}
