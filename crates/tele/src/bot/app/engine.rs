use super::*;

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
            source_error_streak: 0,
        }
    }

    pub fn with_config(mut self, config: EngineConfig) -> Self {
        self.config = config;
        self
    }

    pub fn config_mut(&mut self) -> &mut EngineConfig {
        &mut self.config
    }

    pub fn source_mut(&mut self) -> &mut S {
        &mut self.source
    }

    /// Prepares router runtime state ahead of dispatch.
    pub async fn prepare_router(&self) -> Result<&Self> {
        let _ = self.router.prepare(&self.client).await?;
        Ok(self)
    }

    /// Runs startup bootstrap and prepares router runtime state.
    pub async fn bootstrap(&self, plan: &BootstrapPlan) -> Result<BootstrapReport> {
        super::bootstrap_router(&self.client, &self.router, plan).await
    }

    /// Runs startup bootstrap with retry/backoff and prepares router state.
    pub async fn bootstrap_with_retry(
        &self,
        plan: &BootstrapPlan,
        policy: BootstrapRetryPolicy,
    ) -> Result<BootstrapReport> {
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

    pub async fn poll_once(&mut self) -> Result<Vec<DispatchOutcome>> {
        self.notify_event(EngineEvent::PollStarted).await;

        let updates = match self.source.poll().await {
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
                return Err(error);
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
            return Err(error);
        }

        self.notify_event(EngineEvent::PollCompleted {
            update_count: updates.len(),
        })
        .await;

        self.dispatch_updates(updates).await
    }

    pub async fn run(&mut self) -> Result<()> {
        loop {
            let poll_result = self.poll_once().await;
            let delay = self.handle_poll_result(poll_result).await?;
            wait_if_needed(delay).await;
        }
    }

    pub async fn run_until<F>(&mut self, shutdown: F) -> Result<()>
    where
        F: Future<Output = ()> + Send,
    {
        tokio::pin!(shutdown);

        loop {
            tokio::select! {
                _ = &mut shutdown => return Ok(()),
                poll_result = self.poll_once() => {
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

    async fn dispatch_updates(&self, updates: Vec<Update>) -> Result<Vec<DispatchOutcome>> {
        if self.config.max_handler_concurrency <= 1 {
            return self.dispatch_updates_sequential(updates).await;
        }
        self.dispatch_updates_concurrent(updates).await
    }

    async fn dispatch_updates_sequential(
        &self,
        updates: Vec<Update>,
    ) -> Result<Vec<DispatchOutcome>> {
        let mut outcomes = Vec::with_capacity(updates.len());

        for update in updates {
            let update_id = update.update_id;
            self.notify_unknown_kinds(&update).await;
            let context = BotContext::new(self.client.clone());
            self.notify_event(EngineEvent::DispatchStarted { update_id })
                .await;
            match self.router.dispatch(context, update).await {
                Ok(true) => {
                    let outcome = DispatchOutcome::Handled { update_id };
                    self.notify_event(EngineEvent::DispatchCompleted { outcome })
                        .await;
                    outcomes.push(outcome);
                }
                Ok(false) => {
                    let outcome = DispatchOutcome::Ignored { update_id };
                    self.notify_event(EngineEvent::DispatchCompleted { outcome })
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
                    if !self.config.continue_on_handler_error {
                        return Err(error);
                    }
                    let outcome = DispatchOutcome::Ignored { update_id };
                    self.notify_event(EngineEvent::DispatchCompleted { outcome })
                        .await;
                    outcomes.push(outcome);
                }
            }
        }

        Ok(outcomes)
    }

    async fn dispatch_updates_concurrent(
        &self,
        updates: Vec<Update>,
    ) -> Result<Vec<DispatchOutcome>> {
        let max_concurrency = self.config.max_handler_concurrency.max(1);
        let semaphore = Arc::new(Semaphore::new(max_concurrency));
        let mut join_set = JoinSet::new();

        for update in updates {
            let update_id = update.update_id;
            self.notify_unknown_kinds(&update).await;
            self.notify_event(EngineEvent::DispatchStarted { update_id })
                .await;

            let permit = semaphore
                .clone()
                .acquire_owned()
                .await
                .map_err(|_| invalid_request("handler semaphore closed unexpectedly"))?;

            let router = self.router.clone();
            let context = BotContext::new(self.client.clone());
            join_set.spawn(async move {
                let _permit = permit;
                let result = router.dispatch(context, update).await;
                (update_id, result)
            });
        }

        let mut outcomes = Vec::new();
        let mut first_error: Option<Error> = None;

        while let Some(join_result) = join_set.join_next().await {
            match join_result {
                Ok((update_id, Ok(true))) => {
                    let outcome = DispatchOutcome::Handled { update_id };
                    self.notify_event(EngineEvent::DispatchCompleted { outcome })
                        .await;
                    outcomes.push(outcome);
                }
                Ok((update_id, Ok(false))) => {
                    let outcome = DispatchOutcome::Ignored { update_id };
                    self.notify_event(EngineEvent::DispatchCompleted { outcome })
                        .await;
                    outcomes.push(outcome);
                }
                Ok((update_id, Err(error))) => {
                    self.notify_handler_error(update_id, &error).await;
                    self.notify_event(EngineEvent::DispatchFailed {
                        update_id,
                        classification: error.classification(),
                    })
                    .await;
                    if !self.config.continue_on_handler_error {
                        first_error = Some(error);
                        break;
                    }
                    let outcome = DispatchOutcome::Ignored { update_id };
                    self.notify_event(EngineEvent::DispatchCompleted { outcome })
                        .await;
                    outcomes.push(outcome);
                }
                Err(join_error) => {
                    let error = invalid_request(format!("bot handler task failed: {join_error}"));
                    self.notify_handler_error(-1, &error).await;
                    self.notify_event(EngineEvent::DispatchFailed {
                        update_id: -1,
                        classification: error.classification(),
                    })
                    .await;
                    if !self.config.continue_on_handler_error {
                        first_error = Some(error);
                        break;
                    }
                }
            }
        }

        if let Some(error) = first_error {
            join_set.abort_all();
            while join_set.join_next().await.is_some() {}
            return Err(error);
        }

        Ok(outcomes)
    }

    async fn handle_poll_result(
        &mut self,
        poll_result: Result<Vec<DispatchOutcome>>,
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
            Err(error) => {
                self.notify_source_error(&error).await;
                if !self.config.continue_on_source_error {
                    return Err(error);
                }
                self.source_error_streak = self.source_error_streak.saturating_add(1);
                if let Some(backoff) = self.config.source_error_backoff.as_ref() {
                    let delay = exponential_backoff(
                        backoff.base_delay,
                        backoff.max_delay,
                        self.source_error_streak,
                    );
                    return Ok(jitter_duration(delay, backoff.jitter_ratio).min(backoff.max_delay));
                }
                Ok(self.config.error_delay)
            }
        }
    }

    async fn notify_source_error(&self, error: &Error) {
        if let Some(hook) = self.on_source_error.as_ref() {
            hook(error);
        }
        if let Some(hook) = self.on_source_error_async.as_ref() {
            hook(error).await;
        }
    }

    async fn notify_handler_error(&self, update_id: i64, error: &Error) {
        if let Some(hook) = self.on_handler_error.as_ref() {
            hook(update_id, error);
        }
        if let Some(hook) = self.on_handler_error_async.as_ref() {
            hook(update_id, error).await;
        }
    }

    async fn notify_unknown_kinds(&self, update: &Update) {
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

    async fn notify_event(&self, event: EngineEvent) {
        if let Some(hook) = self.on_event.as_ref() {
            hook(&event);
        }
        if let Some(hook) = self.on_event_async.as_ref() {
            hook(&event).await;
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
    pub fn with_channel(client: Client, router: Router, buffer: usize) -> (UpdateSink, Self) {
        let (sink, source) = channel_source(buffer);
        let engine = Self::new(client, source, router);
        (sink, engine)
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

    /// Prepares router runtime state ahead of serving updates.
    pub async fn prepare_router(&self) -> Result<&Self> {
        let _ = self.engine.prepare_router().await?;
        Ok(self)
    }

    /// Runs startup bootstrap and prepares router runtime state.
    pub async fn bootstrap(&self, plan: &BootstrapPlan) -> Result<BootstrapReport> {
        self.engine.bootstrap(plan).await
    }

    /// Runs startup bootstrap with retry/backoff and prepares router state.
    pub async fn bootstrap_with_retry(
        &self,
        plan: &BootstrapPlan,
        policy: BootstrapRetryPolicy,
    ) -> Result<BootstrapReport> {
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

    pub async fn poll_once(&mut self) -> Result<Vec<DispatchOutcome>> {
        self.engine.poll_once().await
    }

    pub async fn run(&mut self) -> Result<()> {
        self.engine.run().await
    }

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
