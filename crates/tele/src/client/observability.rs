use std::sync::Arc;
use std::time::Duration;

use crate::ErrorClass;
use crate::Result;

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
        latency_ms = crate::util::duration_millis_u64(metric.latency),
        status = metric.status,
        classification = ?metric.classification,
        retryable = metric.retryable,
        request_id = metric.request_id,
        "telegram api request completed"
    );
}

pub(crate) fn emit_client_result_metric<R>(
    observability: &ClientObservability,
    method: &str,
    latency: Duration,
    result: &Result<R>,
) {
    let (success, status, classification, retryable, request_id) = match result {
        Ok(_) => (true, None, None, false, None),
        Err(error) => (
            false,
            error.status().map(|status| status.as_u16()),
            Some(error.classification()),
            error.is_retryable(),
            error.request_id().map(ToOwned::to_owned),
        ),
    };

    emit_client_metric(
        observability,
        ClientMetric {
            method: method.to_owned(),
            success,
            latency,
            status,
            classification,
            retryable,
            request_id,
        },
    );
}
