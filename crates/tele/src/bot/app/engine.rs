use super::*;
#[cfg(feature = "tracing")]
use tracing::Instrument;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PollFailureKind {
    Source,
    Dispatch,
    Fatal,
}

#[derive(Debug)]
struct PollFailure {
    kind: PollFailureKind,
    error: Error,
}

impl PollFailure {
    fn source(error: Error) -> Self {
        Self {
            kind: PollFailureKind::Source,
            error,
        }
    }

    fn dispatch(error: Error) -> Self {
        Self {
            kind: PollFailureKind::Dispatch,
            error,
        }
    }

    fn fatal(error: Error) -> Self {
        Self {
            kind: PollFailureKind::Fatal,
            error,
        }
    }
}

#[derive(Debug)]
struct DispatchFailure {
    error: Error,
    outcomes: Vec<DispatchOutcome>,
}

impl DispatchFailure {
    fn new(error: Error, outcomes: Vec<DispatchOutcome>) -> Self {
        Self { error, outcomes }
    }
}

fn successful_source_order_prefix(outcomes: &[Option<DispatchOutcome>]) -> Vec<DispatchOutcome> {
    let mut prefix = Vec::new();

    for outcome in outcomes {
        let Some(outcome) = *outcome else {
            break;
        };
        if outcome.is_failed() {
            break;
        }
        prefix.push(outcome);
    }

    prefix
}

/// Source-agnostic bot engine that handles dispatching, backpressure and error policy.
pub struct BotEngine<S>
where
    S: UpdateSource,
{
    client: Client,
    source: S,
    router: Router,
    config: EngineConfig,
    on_source_error: Option<SourceErrorHook>,
    on_handler_error: Option<HandlerErrorHook>,
    on_event: Option<EngineEventHook>,
    on_source_error_async: Option<AsyncSourceErrorHook>,
    on_handler_error_async: Option<AsyncHandlerErrorHook>,
    on_event_async: Option<AsyncEngineEventHook>,
    on_metric: Option<EngineMetricHook>,
    on_metric_async: Option<AsyncEngineMetricHook>,
    source_error_streak: usize,
}

impl<S> BotEngine<S>
where
    S: UpdateSource,
{
    pub fn new(client: Client, source: S, router: Router) -> Self {
        Self {
            client,
            source,
            router,
            config: EngineConfig::default(),
            on_source_error: None,
            on_handler_error: None,
            on_event: None,
            on_source_error_async: None,
            on_handler_error_async: None,
            on_event_async: None,
            on_metric: None,
            on_metric_async: None,
            source_error_streak: 0,
        }
    }

    pub fn with_config(mut self, config: EngineConfig) -> Self {
        self.set_config(config);
        self
    }

    pub fn with_config_checked(mut self, config: EngineConfig) -> Result<Self> {
        config.validate()?;
        self.set_config(config);
        Ok(self)
    }

    /// Returns the current engine configuration.
    ///
    /// Use [`Self::set_config`] or [`Self::with_config_checked`] to change values so derived
    /// runtime state stays consistent.
    pub fn config(&self) -> &EngineConfig {
        &self.config
    }

    /// Replaces engine configuration and invalidates derived retry state when policy changes.
    pub fn set_config(&mut self, config: EngineConfig) -> &mut Self {
        if self.source_error_retry_policy_changed(&config) {
            self.source_error_streak = 0;
        }
        self.config = config;
        self
    }

    /// Validates and replaces engine configuration.
    pub fn set_config_checked(&mut self, config: EngineConfig) -> Result<&mut Self> {
        config.validate()?;
        Ok(self.set_config(config))
    }

    pub fn source_mut(&mut self) -> &mut S {
        &mut self.source
    }

    fn source_error_retry_policy_changed(&self, config: &EngineConfig) -> bool {
        self.config.continue_on_source_error != config.continue_on_source_error
            || self.config.error_delay != config.error_delay
            || self.config.source_error_backoff != config.source_error_backoff
    }

    /// Prepares router runtime state ahead of dispatch.
    pub async fn prepare_router(&self) -> Result<&Self> {
        let _ = self.router.prepare(&self.client).await?;
        Ok(self)
    }

    /// Runs setup bootstrap and prepares router runtime state.
    pub async fn bootstrap(&self, plan: &BootstrapPlan) -> BootstrapOutcome {
        super::bootstrap_router(&self.client, &self.router, plan).await
    }

    /// Runs setup bootstrap with retry/backoff and prepares router state.
    pub async fn bootstrap_with_retry(
        &self,
        plan: &BootstrapPlan,
        policy: BootstrapRetryPolicy,
    ) -> BootstrapOutcome {
        super::bootstrap_router_with_retry(&self.client, &self.router, plan, policy).await
    }

    pub fn on_source_error<F>(mut self, hook: F) -> Self
    where
        F: Fn(&Error) + Send + Sync + 'static,
    {
        self.on_source_error = Some(Arc::new(hook));
        self
    }

    pub fn on_source_error_async<F, Fut>(mut self, hook: F) -> Self
    where
        F: Fn(&Error) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        self.on_source_error_async = Some(Arc::new(move |error| Box::pin(hook(error))));
        self
    }

    pub fn on_handler_error<F>(mut self, hook: F) -> Self
    where
        F: Fn(i64, &Error) + Send + Sync + 'static,
    {
        self.on_handler_error = Some(Arc::new(hook));
        self
    }

    pub fn on_handler_error_async<F, Fut>(mut self, hook: F) -> Self
    where
        F: Fn(i64, &Error) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        self.on_handler_error_async = Some(Arc::new(move |update_id, error| {
            Box::pin(hook(update_id, error))
        }));
        self
    }

    pub fn on_event<F>(mut self, hook: F) -> Self
    where
        F: Fn(&EngineEvent) + Send + Sync + 'static,
    {
        self.on_event = Some(Arc::new(hook));
        self
    }

    pub fn on_event_async<F, Fut>(mut self, hook: F) -> Self
    where
        F: Fn(&EngineEvent) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        self.on_event_async = Some(Arc::new(move |event| Box::pin(hook(event))));
        self
    }

    pub fn on_metric<F>(mut self, hook: F) -> Self
    where
        F: Fn(&EngineMetric) + Send + Sync + 'static,
    {
        self.on_metric = Some(Arc::new(hook));
        self
    }

    pub fn on_metric_async<F, Fut>(mut self, hook: F) -> Self
    where
        F: Fn(&EngineMetric) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        self.on_metric_async = Some(Arc::new(move |metric| Box::pin(hook(metric))));
        self
    }

    /// Runs one poll/prepare/dispatch cycle.
    pub async fn poll_once(&mut self) -> Result<Vec<DispatchOutcome>> {
        self.poll_once_inner()
            .await
            .map_err(|failure| failure.error)
    }

    async fn poll_once_inner(&mut self) -> std::result::Result<Vec<DispatchOutcome>, PollFailure> {
        self.config.validate().map_err(PollFailure::fatal)?;

        let poll_started_at = Instant::now();
        self.notify_event(EngineEvent::PollStarted).await;

        #[cfg(feature = "tracing")]
        let poll_future = self
            .source
            .poll()
            .instrument(tracing::debug_span!("tele.bot.poll"));
        #[cfg(not(feature = "tracing"))]
        let poll_future = self.source.poll();

        let updates = match poll_future.await {
            Ok(updates) => updates,
            Err(error) => {
                self.notify_event(EngineEvent::PollFailed {
                    classification: error.classification(),
                    retryable: error.is_retryable(),
                    status: error.status().map(|status| status.as_u16()),
                    error_code: error.error_code(),
                    request_id: error.request_id().map(ToOwned::to_owned),
                    message: error.to_string(),
                })
                .await;
                return Err(PollFailure::source(error));
            }
        };

        if let Err(error) = self
            .router
            .prepare_for_updates(&self.client, &updates)
            .await
        {
            self.notify_event(EngineEvent::PollFailed {
                classification: error.classification(),
                retryable: error.is_retryable(),
                status: error.status().map(|status| status.as_u16()),
                error_code: error.error_code(),
                request_id: error.request_id().map(ToOwned::to_owned),
                message: error.to_string(),
            })
            .await;
            return Err(PollFailure::source(error));
        }

        self.notify_event(EngineEvent::PollCompleted {
            update_count: updates.len(),
        })
        .await;
        self.notify_metric(EngineMetric::PollLatency {
            update_count: updates.len(),
            latency: poll_started_at.elapsed(),
        })
        .await;

        let outcomes = match self.dispatch_updates(updates).await {
            Ok(outcomes) => outcomes,
            Err(failure) => {
                self.source
                    .commit(&failure.outcomes)
                    .await
                    .map_err(PollFailure::source)?;
                return Err(PollFailure::dispatch(failure.error));
            }
        };
        self.source
            .commit(&outcomes)
            .await
            .map_err(PollFailure::source)?;
        Ok(outcomes)
    }

    pub async fn run(&mut self) -> Result<()> {
        loop {
            let poll_result = self.poll_once_inner().await;
            let delay = self.handle_poll_result(poll_result).await?;
            wait_if_needed(delay).await;
        }
    }

    /// Runs until `shutdown` resolves.
    ///
    /// The returned future is `Send`, so it can be spawned on a multi-threaded Tokio runtime.
    pub async fn run_until<F>(&mut self, shutdown: F) -> Result<()>
    where
        F: Future<Output = ()> + Send,
    {
        tokio::pin!(shutdown);

        loop {
            tokio::select! {
                _ = &mut shutdown => return Ok(()),
                poll_result = self.poll_once_inner() => {
                    let delay = self.handle_poll_result(poll_result).await?;
                    if !delay.is_zero() {
                        tokio::select! {
                            _ = &mut shutdown => return Ok(()),
                            _ = sleep(delay) => {}
                        }
                    }
                }
            }
        }
    }

    async fn dispatch_updates(
        &mut self,
        updates: Vec<Update>,
    ) -> std::result::Result<Vec<DispatchOutcome>, DispatchFailure> {
        if self.config.max_handler_concurrency <= 1 || !self.config.continue_on_handler_error {
            return self.dispatch_updates_sequential(updates).await;
        }
        self.dispatch_updates_concurrent(updates).await
    }

    async fn dispatch_updates_sequential(
        &mut self,
        updates: Vec<Update>,
    ) -> std::result::Result<Vec<DispatchOutcome>, DispatchFailure> {
        let mut outcomes = Vec::with_capacity(updates.len());

        for update in updates {
            let update_id = update.update_id;
            self.notify_unknown_kinds(&update).await;
            let context = BotContext::new(self.client.clone());
            self.notify_event(EngineEvent::DispatchStarted { update_id })
                .await;
            let dispatch_started_at = Instant::now();
            #[cfg(feature = "tracing")]
            let dispatch_future = self
                .router
                .dispatch(context, update)
                .instrument(tracing::debug_span!("tele.bot.dispatch", update_id));
            #[cfg(not(feature = "tracing"))]
            let dispatch_future = self.router.dispatch(context, update);
            match dispatch_future.await {
                Ok(true) => {
                    let outcome = DispatchOutcome::Handled { update_id };
                    self.notify_event(EngineEvent::DispatchCompleted { outcome })
                        .await;
                    self.notify_metric(EngineMetric::DispatchLatency {
                        update_id,
                        outcome: DispatchMetricOutcome::Handled,
                        latency: dispatch_started_at.elapsed(),
                    })
                    .await;
                    outcomes.push(outcome);
                }
                Ok(false) => {
                    let outcome = DispatchOutcome::Ignored { update_id };
                    self.notify_event(EngineEvent::DispatchCompleted { outcome })
                        .await;
                    self.notify_metric(EngineMetric::DispatchLatency {
                        update_id,
                        outcome: DispatchMetricOutcome::Ignored,
                        latency: dispatch_started_at.elapsed(),
                    })
                    .await;
                    outcomes.push(outcome);
                }
                Err(error) => {
                    self.notify_handler_error(update_id, &error).await;
                    self.notify_event(EngineEvent::DispatchFailed {
                        update_id,
                        classification: error.classification(),
                    })
                    .await;
                    self.notify_metric(EngineMetric::DispatchLatency {
                        update_id,
                        outcome: DispatchMetricOutcome::Failed,
                        latency: dispatch_started_at.elapsed(),
                    })
                    .await;
                    let outcome = DispatchOutcome::Failed { update_id };
                    self.notify_event(EngineEvent::DispatchCompleted { outcome })
                        .await;
                    if !self.config.continue_on_handler_error {
                        return Err(DispatchFailure::new(error, outcomes));
                    }
                    outcomes.push(outcome);
                }
            }
        }

        Ok(outcomes)
    }

    async fn dispatch_updates_concurrent(
        &mut self,
        updates: Vec<Update>,
    ) -> std::result::Result<Vec<DispatchOutcome>, DispatchFailure> {
        let max_concurrency = self.config.max_handler_concurrency;
        let semaphore = Arc::new(Semaphore::new(max_concurrency));
        let mut join_set = JoinSet::new();
        let update_count = updates.len();
        let mut outcomes = vec![None; update_count];

        for (index, update) in updates.into_iter().enumerate() {
            let update_id = update.update_id;
            self.notify_unknown_kinds(&update).await;
            self.notify_event(EngineEvent::DispatchStarted { update_id })
                .await;

            let permit = semaphore.clone().acquire_owned().await.map_err(|_| {
                DispatchFailure::new(
                    invalid_request("handler semaphore closed unexpectedly"),
                    successful_source_order_prefix(&outcomes),
                )
            })?;

            let router = self.router.clone();
            let context = BotContext::new(self.client.clone());
            join_set.spawn(async move {
                let _permit = permit;
                let dispatch_started_at = Instant::now();
                #[cfg(feature = "tracing")]
                let dispatch_future = router
                    .dispatch(context, update)
                    .instrument(tracing::debug_span!("tele.bot.dispatch", update_id));
                #[cfg(not(feature = "tracing"))]
                let dispatch_future = router.dispatch(context, update);
                let result = dispatch_future.await;
                (index, update_id, dispatch_started_at.elapsed(), result)
            });
        }

        let mut first_error: Option<Error> = None;

        while let Some(join_result) = join_set.join_next().await {
            match join_result {
                Ok((index, update_id, latency, Ok(true))) => {
                    let outcome = DispatchOutcome::Handled { update_id };
                    self.notify_event(EngineEvent::DispatchCompleted { outcome })
                        .await;
                    self.notify_metric(EngineMetric::DispatchLatency {
                        update_id,
                        outcome: DispatchMetricOutcome::Handled,
                        latency,
                    })
                    .await;
                    outcomes[index] = Some(outcome);
                }
                Ok((index, update_id, latency, Ok(false))) => {
                    let outcome = DispatchOutcome::Ignored { update_id };
                    self.notify_event(EngineEvent::DispatchCompleted { outcome })
                        .await;
                    self.notify_metric(EngineMetric::DispatchLatency {
                        update_id,
                        outcome: DispatchMetricOutcome::Ignored,
                        latency,
                    })
                    .await;
                    outcomes[index] = Some(outcome);
                }
                Ok((index, update_id, latency, Err(error))) => {
                    self.notify_handler_error(update_id, &error).await;
                    self.notify_event(EngineEvent::DispatchFailed {
                        update_id,
                        classification: error.classification(),
                    })
                    .await;
                    self.notify_metric(EngineMetric::DispatchLatency {
                        update_id,
                        outcome: DispatchMetricOutcome::Failed,
                        latency,
                    })
                    .await;
                    let outcome = DispatchOutcome::Failed { update_id };
                    self.notify_event(EngineEvent::DispatchCompleted { outcome })
                        .await;
                    if !self.config.continue_on_handler_error {
                        first_error = Some(error);
                        break;
                    }
                    outcomes[index] = Some(outcome);
                }
                Err(join_error) => {
                    let error = invalid_request(format!("bot handler task failed: {join_error}"));
                    self.notify_handler_error(-1, &error).await;
                    self.notify_event(EngineEvent::DispatchFailed {
                        update_id: -1,
                        classification: error.classification(),
                    })
                    .await;
                    first_error = Some(error);
                    break;
                }
            }
        }

        if let Some(error) = first_error {
            join_set.abort_all();
            while join_set.join_next().await.is_some() {}
            return Err(DispatchFailure::new(
                error,
                successful_source_order_prefix(&outcomes),
            ));
        }

        outcomes
            .into_iter()
            .collect::<Option<Vec<_>>>()
            .ok_or_else(|| {
                DispatchFailure::new(
                    invalid_request("bot dispatch task completed without an outcome"),
                    Vec::new(),
                )
            })
    }

    async fn handle_poll_result(
        &mut self,
        poll_result: std::result::Result<Vec<DispatchOutcome>, PollFailure>,
    ) -> Result<Duration> {
        match poll_result {
            Ok(outcomes) if outcomes.is_empty() => {
                self.source_error_streak = 0;
                Ok(self.config.idle_delay)
            }
            Ok(_) => {
                self.source_error_streak = 0;
                Ok(Duration::ZERO)
            }
            Err(failure) if failure.kind == PollFailureKind::Source => {
                self.notify_source_error(&failure.error).await;
                let streak = self.source_error_streak.saturating_add(1);
                self.notify_metric(EngineMetric::SourceError {
                    classification: failure.error.classification(),
                    retryable: failure.error.is_retryable(),
                    streak,
                })
                .await;
                if !self.config.continue_on_source_error || !failure.error.is_retryable() {
                    return Err(failure.error);
                }
                self.source_error_streak = streak;
                if let Some(backoff) = self.config.source_error_backoff.as_ref() {
                    let delay = exponential_backoff(
                        backoff.base_delay,
                        backoff.max_delay,
                        self.source_error_streak,
                    );
                    let applied_delay =
                        jitter_duration(delay, backoff.jitter_ratio).min(backoff.max_delay);
                    self.notify_metric(EngineMetric::SourceBackoff {
                        streak: self.source_error_streak,
                        delay: applied_delay,
                    })
                    .await;
                    return Ok(applied_delay);
                }
                Ok(self.config.error_delay)
            }
            Err(failure) => Err(failure.error),
        }
    }

    async fn notify_source_error(&mut self, error: &Error) {
        if let Some(hook) = self.on_source_error.as_ref() {
            hook(error);
        }
        if let Some(hook) = self.on_source_error_async.as_ref() {
            hook(error).await;
        }
    }

    async fn notify_handler_error(&mut self, update_id: i64, error: &Error) {
        if let Some(hook) = self.on_handler_error.as_ref() {
            hook(update_id, error);
        }
        if let Some(hook) = self.on_handler_error_async.as_ref() {
            hook(update_id, error).await;
        }
    }

    async fn notify_unknown_kinds(&mut self, update: &Update) {
        let update_kind = update.kind();
        let message_kind = extract_message_kind(update);
        if update_kind != UpdateKind::Unknown && message_kind != Some(MessageKind::Unknown) {
            return;
        }

        self.notify_event(EngineEvent::UnknownKindsDetected {
            update_id: update.update_id,
            update_kind,
            message_kind,
        })
        .await;
    }

    async fn notify_event(&mut self, event: EngineEvent) {
        if let Some(hook) = self.on_event.as_ref() {
            hook(&event);
        }
        if let Some(hook) = self.on_event_async.as_ref() {
            hook(&event).await;
        }
    }

    async fn notify_metric(&mut self, metric: EngineMetric) {
        if let Some(hook) = self.on_metric.as_ref() {
            hook(&metric);
        }
        if let Some(hook) = self.on_metric_async.as_ref() {
            hook(&metric).await;
        }

        #[cfg(feature = "tracing")]
        match &metric {
            EngineMetric::PollLatency {
                update_count,
                latency,
            } => tracing::debug!(
                target: "tele::bot",
                update_count,
                latency_ms = latency.as_millis() as u64,
                "bot poll completed"
            ),
            EngineMetric::DispatchLatency {
                update_id,
                outcome,
                latency,
            } => tracing::debug!(
                target: "tele::bot",
                update_id,
                outcome = ?outcome,
                latency_ms = latency.as_millis() as u64,
                "bot dispatch completed"
            ),
            EngineMetric::SourceError {
                classification,
                retryable,
                streak,
            } => tracing::warn!(
                target: "tele::bot",
                classification = ?classification,
                retryable,
                streak,
                "bot source poll failed"
            ),
            EngineMetric::SourceBackoff { streak, delay } => tracing::warn!(
                target: "tele::bot",
                streak,
                delay_ms = delay.as_millis() as u64,
                "bot source backoff applied"
            ),
        }
    }
}

impl BotEngine<LongPollingSource> {
    /// Builds engine with default long polling source.
    pub fn with_long_polling(client: Client, router: Router) -> Self {
        let source = LongPollingSource::new(client.clone());
        Self::new(client, source, router)
    }
}

impl BotEngine<ChannelUpdateSource> {
    /// Builds engine backed by channel source and returns paired sink.
    pub fn with_channel(
        client: Client,
        router: Router,
        buffer: usize,
    ) -> Result<(UpdateSink, Self)> {
        let (sink, source) = channel_source(buffer)?;
        let engine = Self::new(client, source, router);
        Ok((sink, engine))
    }
}

/// High-level app wrapper that keeps bot runtime setup short for downstream projects.
pub struct BotApp<S>
where
    S: UpdateSource,
{
    engine: BotEngine<S>,
}

impl BotApp<LongPollingSource> {
    pub fn long_polling(client: Client, router: Router) -> Self {
        Self {
            engine: BotEngine::with_long_polling(client, router),
        }
    }
}

impl<S> BotApp<S>
where
    S: UpdateSource,
{
    pub fn from_engine(engine: BotEngine<S>) -> Self {
        Self { engine }
    }

    pub fn engine(&self) -> &BotEngine<S> {
        &self.engine
    }

    pub fn engine_mut(&mut self) -> &mut BotEngine<S> {
        &mut self.engine
    }

    pub fn with_engine_config(mut self, config: EngineConfig) -> Self {
        self.engine = self.engine.with_config(config);
        self
    }

    pub fn with_engine_config_checked(mut self, config: EngineConfig) -> Result<Self> {
        self.engine = self.engine.with_config_checked(config)?;
        Ok(self)
    }

    /// Prepares router runtime state ahead of serving updates.
    pub async fn prepare_router(&self) -> Result<&Self> {
        let _ = self.engine.prepare_router().await?;
        Ok(self)
    }

    /// Runs setup bootstrap and prepares router runtime state.
    pub async fn bootstrap(&self, plan: &BootstrapPlan) -> BootstrapOutcome {
        self.engine.bootstrap(plan).await
    }

    /// Runs setup bootstrap with retry/backoff and prepares router state.
    pub async fn bootstrap_with_retry(
        &self,
        plan: &BootstrapPlan,
        policy: BootstrapRetryPolicy,
    ) -> BootstrapOutcome {
        self.engine.bootstrap_with_retry(plan, policy).await
    }

    pub fn on_source_error<F>(mut self, hook: F) -> Self
    where
        F: Fn(&Error) + Send + Sync + 'static,
    {
        self.engine = self.engine.on_source_error(hook);
        self
    }

    pub fn on_source_error_async<F, Fut>(mut self, hook: F) -> Self
    where
        F: Fn(&Error) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        self.engine = self.engine.on_source_error_async(hook);
        self
    }

    pub fn on_handler_error<F>(mut self, hook: F) -> Self
    where
        F: Fn(i64, &Error) + Send + Sync + 'static,
    {
        self.engine = self.engine.on_handler_error(hook);
        self
    }

    pub fn on_handler_error_async<F, Fut>(mut self, hook: F) -> Self
    where
        F: Fn(i64, &Error) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        self.engine = self.engine.on_handler_error_async(hook);
        self
    }

    pub fn on_event<F>(mut self, hook: F) -> Self
    where
        F: Fn(&EngineEvent) + Send + Sync + 'static,
    {
        self.engine = self.engine.on_event(hook);
        self
    }

    pub fn on_event_async<F, Fut>(mut self, hook: F) -> Self
    where
        F: Fn(&EngineEvent) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        self.engine = self.engine.on_event_async(hook);
        self
    }

    pub fn on_metric<F>(mut self, hook: F) -> Self
    where
        F: Fn(&EngineMetric) + Send + Sync + 'static,
    {
        self.engine = self.engine.on_metric(hook);
        self
    }

    pub fn on_metric_async<F, Fut>(mut self, hook: F) -> Self
    where
        F: Fn(&EngineMetric) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        self.engine = self.engine.on_metric_async(hook);
        self
    }

    pub async fn poll_once(&mut self) -> Result<Vec<DispatchOutcome>> {
        self.engine.poll_once().await
    }

    pub async fn run(&mut self) -> Result<()> {
        self.engine.run().await
    }

    /// Runs until `shutdown` resolves.
    ///
    /// This delegates to `BotEngine::run_until`, so the returned future is also `Send`.
    pub async fn run_until<F>(&mut self, shutdown: F) -> Result<()>
    where
        F: Future<Output = ()> + Send,
    {
        self.engine.run_until(shutdown).await
    }

    pub fn into_engine(self) -> BotEngine<S> {
        self.engine
    }
}

async fn wait_if_needed(duration: Duration) {
    if duration.is_zero() {
        return;
    }

    sleep(duration).await;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_engine() -> Result<BotEngine<ChannelUpdateSource>> {
        let client = Client::builder("http://127.0.0.1:9")?
            .bot_token("123:abc")?
            .build()?;
        let (_sink, source) = channel_source(1)?;
        Ok(BotEngine::new(client, source, Router::new()))
    }

    #[test]
    fn set_config_resets_source_error_streak_when_retry_policy_changes() -> Result<()> {
        let mut engine = test_engine()?;
        engine.source_error_streak = 4;

        engine.set_config(EngineConfig {
            error_delay: Duration::from_secs(2),
            ..EngineConfig::default()
        });

        assert_eq!(engine.source_error_streak, 0);
        Ok(())
    }

    #[test]
    fn set_config_preserves_source_error_streak_for_dispatch_only_changes() -> Result<()> {
        let mut engine = test_engine()?;
        engine.source_error_streak = 4;

        engine.set_config(EngineConfig {
            max_handler_concurrency: 8,
            ..EngineConfig::default()
        });

        assert_eq!(engine.source_error_streak, 4);
        Ok(())
    }

    #[test]
    fn set_config_checked_rejects_invalid_config_without_mutating_engine() -> Result<()> {
        let mut engine = test_engine()?;

        let result = engine.set_config_checked(EngineConfig {
            max_handler_concurrency: 0,
            ..EngineConfig::default()
        });

        assert!(matches!(result, Err(Error::Configuration { .. })));
        assert_eq!(engine.config().max_handler_concurrency, 1);
        Ok(())
    }
}
