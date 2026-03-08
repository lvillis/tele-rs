use std::sync::Arc;
use std::time::Duration;

use crate::ErrorClass;

/// One completed Telegram API request observation.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct ClientMetric {
    pub method: String,
    pub success: bool,
    pub latency: Duration,
    pub status: Option<u16>,
    pub classification: Option<ErrorClass>,
    pub retryable: bool,
    pub request_id: Option<String>,
}

/// Hook called whenever one Telegram API request completes.
pub type ClientMetricHook = Arc<dyn Fn(&ClientMetric) + Send + Sync + 'static>;

#[derive(Clone, Default)]
pub(crate) struct ClientObservability {
    pub(crate) on_metric: Option<ClientMetricHook>,
}

pub(crate) fn emit_client_metric(observability: &ClientObservability, metric: ClientMetric) {
    if let Some(hook) = observability.on_metric.as_ref() {
        hook(&metric);
    }

    #[cfg(feature = "tracing")]
    tracing::debug!(
        target: "tele::client",
        method = metric.method,
        success = metric.success,
        latency_ms = metric.latency.as_millis() as u64,
        status = metric.status,
        classification = ?metric.classification,
        retryable = metric.retryable,
        request_id = metric.request_id,
        "telegram api request completed"
    );
}
