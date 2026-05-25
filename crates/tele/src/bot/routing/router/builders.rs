use super::*;

macro_rules! impl_route_dsl_methods {
    () => {
        pub fn group_only(mut self) -> Self {
            self.config.group_only();
            self
        }

        pub fn supergroup_only(mut self) -> Self {
            self.config.supergroup_only();
            self
        }

        pub fn admin_only(mut self) -> Self {
            self.config.admin_only();
            self
        }

        pub fn owner_only(mut self) -> Self {
            self.config.owner_only();
            self
        }

        pub fn require_capabilities(
            mut self,
            capabilities: &[ChatAdministratorCapability],
        ) -> Self {
            self.config.require_capabilities(capabilities.to_vec());
            self
        }

        pub fn bot_can(mut self, capabilities: &[ChatAdministratorCapability]) -> Self {
            self.config.bot_can(capabilities.to_vec());
            self
        }

        pub fn throttle(mut self, scope: ThrottleScope, limit: u32, window: Duration) -> Self {
            self.config.throttle(scope, limit, window);
            self
        }

        pub fn throttle_actor(self, window: Duration) -> Self {
            self.throttle(ThrottleScope::Actor, 1, window)
        }

        pub fn throttle_subject(self, window: Duration) -> Self {
            self.throttle(ThrottleScope::Subject, 1, window)
        }

        pub fn throttle_chat(self, window: Duration) -> Self {
            self.throttle(ThrottleScope::Chat, 1, window)
        }

        pub fn throttle_command(self, window: Duration) -> Self {
            self.throttle(ThrottleScope::Command, 1, window)
        }
    };
}

/// Chainable DSL for non-extracting update routes.
pub struct UpdateRouteBuilder<'a> {
    router: &'a mut Router,
    filter: RouteFilterFn,
    config: RouteDslConfig,
}

impl<'a> UpdateRouteBuilder<'a> {
    pub(super) fn new(
        router: &'a mut Router,
        route_label: impl Into<String>,
        filter: RouteFilterFn,
    ) -> Self {
        Self {
            router,
            filter,
            config: RouteDslConfig::new(route_label),
        }
    }

    impl_route_dsl_methods!();

    pub fn handle<H, Fut>(self, handler: H) -> &'a mut Router
    where
        H: Fn(BotContext, Update) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = HandlerResult> + Send + 'static,
    {
        let filter = Arc::clone(&self.filter);
        let guards = Arc::new(self.config.guards);
        let handler = Arc::new(handler);
        self.router.route_fallible_with_state(
            move |update, state| filter(update, state),
            move |context, update, _state| {
                let guards = Arc::clone(&guards);
                let handler = Arc::clone(&handler);
                async move {
                    evaluate_route_pipeline(
                        context,
                        update,
                        guards.as_ref(),
                        |_update| Ok(Some(())),
                        move |context, update, ()| handler(context, update),
                    )
                    .await
                }
            },
        )
    }

    pub fn handle_fallible<H, Fut>(self, handler: H) -> &'a mut Router
    where
        H: Fn(BotContext, Update) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = HandlerResult> + Send + 'static,
    {
        self.handle(handler)
    }

    pub fn handle_with_policy<H, Fut>(self, policy: ErrorPolicy, handler: H) -> &'a mut Router
    where
        H: Fn(BotContext, Update) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = HandlerResult> + Send + 'static,
    {
        let filter = Arc::clone(&self.filter);
        let guards = Arc::new(self.config.guards);
        let handler = Arc::new(handler);
        self.router.route_with_policy_state(
            move |update, state| filter(update, state),
            policy,
            move |context, update, _state| {
                let guards = Arc::clone(&guards);
                let handler = Arc::clone(&handler);
                async move {
                    evaluate_route_pipeline(
                        context,
                        update,
                        guards.as_ref(),
                        |_update| Ok(Some(())),
                        move |context, update, ()| handler(context, update),
                    )
                    .await
                }
            },
        )
    }
}

/// Chainable DSL for extractor-backed routes.
pub struct ExtractedRouteBuilder<'a, E> {
    router: &'a mut Router,
    config: RouteDslConfig,
    filters: Vec<ExtractedFilterFn<E>>,
    extracted_guards: Vec<ExtractedGuardFn<E>>,
    _extracted: std::marker::PhantomData<E>,
}

impl<'a, E> ExtractedRouteBuilder<'a, E>
where
    E: UpdateExtractor + Send + 'static,
{
    pub(super) fn new(router: &'a mut Router, route_label: impl Into<String>) -> Self {
        Self {
            router,
            config: RouteDslConfig::new(route_label),
            filters: Vec::new(),
            extracted_guards: Vec::new(),
            _extracted: std::marker::PhantomData,
        }
    }

    impl_route_dsl_methods!();

    pub fn filter<P>(mut self, predicate: P) -> Self
    where
        P: Fn(&E, &Update) -> bool + Send + Sync + 'static,
    {
        self.filters.push(Arc::new(predicate));
        self
    }

    pub fn guard<G>(mut self, guard: G) -> Self
    where
        G: Fn(&E, &Update) -> HandlerResult + Send + Sync + 'static,
    {
        self.extracted_guards.push(Arc::new(guard));
        self
    }

    pub fn map<T, M>(self, mapper: M) -> MappedExtractedRouteBuilder<'a, E, T>
    where
        T: Send + 'static,
        M: Fn(&E, &Update) -> Option<T> + Send + Sync + 'static,
    {
        MappedExtractedRouteBuilder {
            router: self.router,
            config: self.config,
            filters: self.filters,
            extracted_guards: self.extracted_guards,
            mapper: Arc::new(mapper),
            _extracted: std::marker::PhantomData,
        }
    }

    pub fn handle<H, Fut>(self, handler: H) -> &'a mut Router
    where
        H: Fn(BotContext, Update, E) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = HandlerResult> + Send + 'static,
    {
        let filters = Arc::new(self.filters);
        let extracted_guards = Arc::new(self.extracted_guards);
        let guards = Arc::new(self.config.guards);
        let handler = Arc::new(handler);
        self.router.route_prepared_handler_with_state(
            {
                let filters = Arc::clone(&filters);
                move |update, _state| {
                    let extracted = E::extract(update)?;
                    filters
                        .iter()
                        .all(|filter| filter(&extracted, update))
                        .then_some(extracted)
                }
            },
            RouteResolution::Default,
            move |context, update, _state, extracted| {
                let extracted_guards = Arc::clone(&extracted_guards);
                let guards = Arc::clone(&guards);
                let handler = Arc::clone(&handler);
                async move {
                    run_route_guards(guards.as_ref(), &context, &update).await?;
                    run_extracted_guards(extracted_guards.as_ref(), &extracted, &update)?;
                    handler(context, update, extracted).await
                }
            },
        )
    }

    pub fn handle_fallible<H, Fut>(self, handler: H) -> &'a mut Router
    where
        H: Fn(BotContext, Update, E) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = HandlerResult> + Send + 'static,
    {
        self.handle(handler)
    }

    pub fn handle_with_policy<H, Fut>(self, policy: ErrorPolicy, handler: H) -> &'a mut Router
    where
        H: Fn(BotContext, Update, E) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = HandlerResult> + Send + 'static,
    {
        let filters = Arc::new(self.filters);
        let extracted_guards = Arc::new(self.extracted_guards);
        let guards = Arc::new(self.config.guards);
        let handler = Arc::new(handler);
        self.router.route_prepared_handler_with_state(
            {
                let filters = Arc::clone(&filters);
                move |update, _state| {
                    let extracted = E::extract(update)?;
                    filters
                        .iter()
                        .all(|filter| filter(&extracted, update))
                        .then_some(extracted)
                }
            },
            RouteResolution::Policy(policy),
            move |context, update, _state, extracted| {
                let extracted_guards = Arc::clone(&extracted_guards);
                let guards = Arc::clone(&guards);
                let handler = Arc::clone(&handler);
                async move {
                    run_route_guards(guards.as_ref(), &context, &update).await?;
                    run_extracted_guards(extracted_guards.as_ref(), &extracted, &update)?;
                    handler(context, update, extracted).await
                }
            },
        )
    }
}

/// Chainable DSL for extractor routes with a mapping step before the handler.
pub struct MappedExtractedRouteBuilder<'a, E, T> {
    router: &'a mut Router,
    config: RouteDslConfig,
    filters: Vec<ExtractedFilterFn<E>>,
    extracted_guards: Vec<ExtractedGuardFn<E>>,
    mapper: ExtractedMapFn<E, T>,
    _extracted: std::marker::PhantomData<E>,
}

impl<'a, E, T> MappedExtractedRouteBuilder<'a, E, T>
where
    E: UpdateExtractor + Send + 'static,
    T: Send + 'static,
{
    impl_route_dsl_methods!();

    pub fn handle<H, Fut>(self, handler: H) -> &'a mut Router
    where
        H: Fn(BotContext, Update, T) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = HandlerResult> + Send + 'static,
    {
        let filters = Arc::new(self.filters);
        let extracted_guards = Arc::new(self.extracted_guards);
        let guards = Arc::new(self.config.guards);
        let mapper = Arc::clone(&self.mapper);
        let handler = Arc::new(handler);
        self.router.route_prepared_handler_with_state(
            {
                let filters = Arc::clone(&filters);
                let mapper = Arc::clone(&mapper);
                move |update, _state| {
                    let extracted = E::extract(update)?;
                    if !filters.iter().all(|filter| filter(&extracted, update)) {
                        return None;
                    }
                    let mapped = mapper(&extracted, update)?;
                    Some((extracted, mapped))
                }
            },
            RouteResolution::Default,
            move |context, update, _state, (extracted, mapped)| {
                let extracted_guards = Arc::clone(&extracted_guards);
                let guards = Arc::clone(&guards);
                let handler = Arc::clone(&handler);
                async move {
                    run_route_guards(guards.as_ref(), &context, &update).await?;
                    run_extracted_guards(extracted_guards.as_ref(), &extracted, &update)?;
                    handler(context, update, mapped).await
                }
            },
        )
    }

    pub fn handle_fallible<H, Fut>(self, handler: H) -> &'a mut Router
    where
        H: Fn(BotContext, Update, T) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = HandlerResult> + Send + 'static,
    {
        self.handle(handler)
    }

    pub fn handle_with_policy<H, Fut>(self, policy: ErrorPolicy, handler: H) -> &'a mut Router
    where
        H: Fn(BotContext, Update, T) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = HandlerResult> + Send + 'static,
    {
        let filters = Arc::new(self.filters);
        let extracted_guards = Arc::new(self.extracted_guards);
        let guards = Arc::new(self.config.guards);
        let mapper = Arc::clone(&self.mapper);
        let handler = Arc::new(handler);
        self.router.route_prepared_handler_with_state(
            {
                let filters = Arc::clone(&filters);
                let mapper = Arc::clone(&mapper);
                move |update, _state| {
                    let extracted = E::extract(update)?;
                    if !filters.iter().all(|filter| filter(&extracted, update)) {
                        return None;
                    }
                    let mapped = mapper(&extracted, update)?;
                    Some((extracted, mapped))
                }
            },
            RouteResolution::Policy(policy),
            move |context, update, _state, (extracted, mapped)| {
                let extracted_guards = Arc::clone(&extracted_guards);
                let guards = Arc::clone(&guards);
                let handler = Arc::clone(&handler);
                async move {
                    run_route_guards(guards.as_ref(), &context, &update).await?;
                    run_extracted_guards(extracted_guards.as_ref(), &extracted, &update)?;
                    handler(context, update, mapped).await
                }
            },
        )
    }
}

/// Chainable DSL for raw slash-command routes.
pub struct CommandInputRouteBuilder<'a> {
    router: &'a mut Router,
    config: RouteDslConfig,
}

impl<'a> CommandInputRouteBuilder<'a> {
    pub(super) fn new(router: &'a mut Router) -> Self {
        Self {
            router,
            config: RouteDslConfig::new("command_input"),
        }
    }

    impl_route_dsl_methods!();

    pub fn handle<H, Fut>(self, handler: H) -> &'a mut Router
    where
        H: Fn(BotContext, Update, CommandData) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = HandlerResult> + Send + 'static,
    {
        let guards = Arc::new(self.config.guards);
        let handler = Arc::new(handler);
        self.router.has_command_routes = true;
        self.router.route_prepared_handler_with_state(
            move |update, state| {
                extract_command_data_for_bot(update, state.command_target.as_deref())
            },
            RouteResolution::Default,
            move |context, update, _state, command| {
                let guards = Arc::clone(&guards);
                let handler = Arc::clone(&handler);
                async move {
                    run_route_guards(guards.as_ref(), &context, &update).await?;
                    handler(context, update, command).await
                }
            },
        )
    }

    pub fn handle_fallible<H, Fut>(self, handler: H) -> &'a mut Router
    where
        H: Fn(BotContext, Update, CommandData) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = HandlerResult> + Send + 'static,
    {
        self.handle(handler)
    }

    pub fn handle_with_policy<H, Fut>(self, policy: ErrorPolicy, handler: H) -> &'a mut Router
    where
        H: Fn(BotContext, Update, CommandData) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = HandlerResult> + Send + 'static,
    {
        let guards = Arc::new(self.config.guards);
        let handler = Arc::new(handler);
        self.router.has_command_routes = true;
        self.router.route_prepared_handler_with_state(
            move |update, state| {
                extract_command_data_for_bot(update, state.command_target.as_deref())
            },
            RouteResolution::Policy(policy),
            move |context, update, _state, command| {
                let guards = Arc::clone(&guards);
                let handler = Arc::clone(&handler);
                async move {
                    run_route_guards(guards.as_ref(), &context, &update).await?;
                    handler(context, update, command).await
                }
            },
        )
    }
}

/// Chainable DSL for command-scoped route configuration.
pub struct CommandRouteBuilder<'a> {
    router: &'a mut Router,
    command: String,
    config: RouteDslConfig,
}

impl<'a> CommandRouteBuilder<'a> {
    pub(super) fn new(router: &'a mut Router, command: String) -> Result<Self> {
        let command = validate_route_command_name(&command)?;
        let route_label = format!("command:{command}");
        Ok(Self {
            router,
            command,
            config: RouteDslConfig::new(route_label),
        })
    }

    impl_route_dsl_methods!();

    pub fn parse<T>(self) -> ParsedCommandRouteBuilder<'a, T>
    where
        T: CommandArgs + Send + Sync + 'static,
    {
        ParsedCommandRouteBuilder {
            router: self.router,
            command: self.command,
            config: self.config,
            _parsed: std::marker::PhantomData,
        }
    }

    pub fn handle<H, Fut>(self, handler: H) -> &'a mut Router
    where
        H: Fn(BotContext, Update) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = HandlerResult> + Send + 'static,
    {
        let expected = self.command;
        let guards = Arc::new(self.config.guards);
        let handler = Arc::new(handler);
        self.router.has_command_routes = true;
        self.router.route_fallible_with_state(
            move |update, state| {
                extract_command_for_bot(update, state.command_target.as_deref())
                    .is_some_and(|command| command == expected)
            },
            move |context, update, _state| {
                let guards = Arc::clone(&guards);
                let handler = Arc::clone(&handler);
                async move {
                    evaluate_route_pipeline(
                        context,
                        update,
                        guards.as_ref(),
                        |_update| Ok(Some(())),
                        move |context, update, ()| handler(context, update),
                    )
                    .await
                }
            },
        )
    }

    pub fn handle_fallible<H, Fut>(self, handler: H) -> &'a mut Router
    where
        H: Fn(BotContext, Update) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = HandlerResult> + Send + 'static,
    {
        self.handle(handler)
    }

    pub fn handle_with_policy<H, Fut>(self, policy: ErrorPolicy, handler: H) -> &'a mut Router
    where
        H: Fn(BotContext, Update) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = HandlerResult> + Send + 'static,
    {
        let expected = self.command;
        let guards = Arc::new(self.config.guards);
        let handler = Arc::new(handler);
        self.router.has_command_routes = true;
        self.router.route_with_policy_state(
            move |update, state| {
                extract_command_for_bot(update, state.command_target.as_deref())
                    .is_some_and(|command| command == expected)
            },
            policy,
            move |context, update, _state| {
                let guards = Arc::clone(&guards);
                let handler = Arc::clone(&handler);
                async move {
                    evaluate_route_pipeline(
                        context,
                        update,
                        guards.as_ref(),
                        |_update| Ok(Some(())),
                        move |context, update, ()| handler(context, update),
                    )
                    .await
                }
            },
        )
    }
}

/// Chainable DSL for command routes that parse typed trailing arguments.
pub struct ParsedCommandRouteBuilder<'a, T> {
    router: &'a mut Router,
    command: String,
    config: RouteDslConfig,
    _parsed: std::marker::PhantomData<T>,
}

impl<'a, T> ParsedCommandRouteBuilder<'a, T>
where
    T: CommandArgs + Send + Sync + 'static,
{
    impl_route_dsl_methods!();

    pub fn handle<H, Fut>(self, handler: H) -> &'a mut Router
    where
        H: Fn(BotContext, Update, T) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = HandlerResult> + Send + 'static,
    {
        let expected = self.command;
        let guards = Arc::new(self.config.guards);
        let handler = Arc::new(handler);
        self.router.has_command_routes = true;
        self.router.route_prepared_handler_with_state(
            move |update, state| {
                let command =
                    extract_command_data_for_bot(update, state.command_target.as_deref())?;
                (command.name == expected).then_some(command)
            },
            RouteResolution::Default,
            move |context, update, _state, command| {
                let guards = Arc::clone(&guards);
                let handler = Arc::clone(&handler);
                async move {
                    run_route_guards(guards.as_ref(), &context, &update).await?;
                    let parsed = T::parse(command.args_trimmed()).map_err(HandlerError::user)?;
                    handler(context, update, parsed).await
                }
            },
        )
    }

    pub fn handle_fallible<H, Fut>(self, handler: H) -> &'a mut Router
    where
        H: Fn(BotContext, Update, T) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = HandlerResult> + Send + 'static,
    {
        self.handle(handler)
    }

    pub fn handle_with_policy<H, Fut>(self, policy: ErrorPolicy, handler: H) -> &'a mut Router
    where
        H: Fn(BotContext, Update, T) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = HandlerResult> + Send + 'static,
    {
        let expected = self.command;
        let guards = Arc::new(self.config.guards);
        let handler = Arc::new(handler);
        self.router.has_command_routes = true;
        self.router.route_prepared_handler_with_state(
            move |update, state| {
                let command =
                    extract_command_data_for_bot(update, state.command_target.as_deref())?;
                (command.name == expected).then_some(command)
            },
            RouteResolution::Policy(policy),
            move |context, update, _state, command| {
                let guards = Arc::clone(&guards);
                let handler = Arc::clone(&handler);
                async move {
                    run_route_guards(guards.as_ref(), &context, &update).await?;
                    let parsed = T::parse(command.args_trimmed()).map_err(HandlerError::user)?;
                    handler(context, update, parsed).await
                }
            },
        )
    }
}

/// Chainable DSL for typed slash-command routes.
pub struct TypedCommandRouteBuilder<'a, C> {
    router: &'a mut Router,
    config: RouteDslConfig,
    _command: std::marker::PhantomData<C>,
}

impl<'a, C> TypedCommandRouteBuilder<'a, C>
where
    C: BotCommands + Send + Sync + 'static,
{
    pub(super) fn new(router: &'a mut Router) -> Self {
        Self {
            router,
            config: RouteDslConfig::new(format!("typed_command:{}", std::any::type_name::<C>())),
            _command: std::marker::PhantomData,
        }
    }

    impl_route_dsl_methods!();

    pub fn handle<H, Fut>(self, handler: H) -> &'a mut Router
    where
        H: Fn(BotContext, Update, C) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = HandlerResult> + Send + 'static,
    {
        let guards = Arc::new(self.config.guards);
        let handler = Arc::new(handler);
        self.router.has_command_routes = true;
        self.router.route_prepared_handler_with_state(
            move |update, state| {
                parse_typed_command_for_bot::<C>(update, state.command_target.as_deref())
            },
            RouteResolution::Default,
            move |context, update, _state, command| {
                let guards = Arc::clone(&guards);
                let handler = Arc::clone(&handler);
                async move {
                    run_route_guards(guards.as_ref(), &context, &update).await?;
                    handler(context, update, command).await
                }
            },
        )
    }

    pub fn handle_input<H, Fut>(self, handler: H) -> &'a mut Router
    where
        H: Fn(BotContext, Update, TypedCommandInput<C>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = HandlerResult> + Send + 'static,
    {
        let guards = Arc::new(self.config.guards);
        let handler = Arc::new(handler);
        self.router.has_command_routes = true;
        self.router.route_prepared_handler_with_state(
            move |update, state| {
                let command =
                    parse_typed_command_for_bot::<C>(update, state.command_target.as_deref())?;
                let raw = extract_command_data_for_bot(update, state.command_target.as_deref())?;
                Some(TypedCommandInput { command, raw })
            },
            RouteResolution::Default,
            move |context, update, _state, input| {
                let guards = Arc::clone(&guards);
                let handler = Arc::clone(&handler);
                async move {
                    run_route_guards(guards.as_ref(), &context, &update).await?;
                    handler(context, update, input).await
                }
            },
        )
    }

    pub fn handle_input_fallible<H, Fut>(self, handler: H) -> &'a mut Router
    where
        H: Fn(BotContext, Update, TypedCommandInput<C>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = HandlerResult> + Send + 'static,
    {
        self.handle_input(handler)
    }

    pub fn handle_fallible<H, Fut>(self, handler: H) -> &'a mut Router
    where
        H: Fn(BotContext, Update, C) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = HandlerResult> + Send + 'static,
    {
        self.handle(handler)
    }

    pub fn handle_with_policy<H, Fut>(self, policy: ErrorPolicy, handler: H) -> &'a mut Router
    where
        H: Fn(BotContext, Update, C) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = HandlerResult> + Send + 'static,
    {
        let guards = Arc::new(self.config.guards);
        let handler = Arc::new(handler);
        self.router.has_command_routes = true;
        self.router.route_prepared_handler_with_state(
            move |update, state| {
                parse_typed_command_for_bot::<C>(update, state.command_target.as_deref())
            },
            RouteResolution::Policy(policy),
            move |context, update, _state, command| {
                let guards = Arc::clone(&guards);
                let handler = Arc::clone(&handler);
                async move {
                    run_route_guards(guards.as_ref(), &context, &update).await?;
                    handler(context, update, command).await
                }
            },
        )
    }

    pub fn handle_input_with_policy<H, Fut>(self, policy: ErrorPolicy, handler: H) -> &'a mut Router
    where
        H: Fn(BotContext, Update, TypedCommandInput<C>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = HandlerResult> + Send + 'static,
    {
        let guards = Arc::new(self.config.guards);
        let handler = Arc::new(handler);
        self.router.has_command_routes = true;
        self.router.route_prepared_handler_with_state(
            move |update, state| {
                let command =
                    parse_typed_command_for_bot::<C>(update, state.command_target.as_deref())?;
                let raw = extract_command_data_for_bot(update, state.command_target.as_deref())?;
                Some(TypedCommandInput { command, raw })
            },
            RouteResolution::Policy(policy),
            move |context, update, _state, input| {
                let guards = Arc::clone(&guards);
                let handler = Arc::clone(&handler);
                async move {
                    run_route_guards(guards.as_ref(), &context, &update).await?;
                    handler(context, update, input).await
                }
            },
        )
    }
}

/// Chainable DSL for codec-aware callback routes.
pub struct CallbackRouteBuilder<'a, T, C = CallbackPayloadCodec> {
    router: &'a mut Router,
    config: RouteDslConfig,
    _payload: std::marker::PhantomData<T>,
    _codec: std::marker::PhantomData<C>,
}

pub type TypedCallbackRouteBuilder<'a, T> = CallbackRouteBuilder<'a, T, CallbackPayloadCodec>;
pub type CompactCallbackRouteBuilder<'a, T> = CallbackRouteBuilder<'a, T, CompactCallbackCodec>;

impl<'a, T, C> CallbackRouteBuilder<'a, T, C>
where
    T: Send + Sync + 'static,
    C: CallbackCodec<T>,
{
    pub(super) fn new(router: &'a mut Router) -> Self {
        Self {
            router,
            config: RouteDslConfig::new(format!("callback:{}", std::any::type_name::<T>())),
            _payload: std::marker::PhantomData,
            _codec: std::marker::PhantomData,
        }
    }

    impl_route_dsl_methods!();

    pub fn handle<H, Fut>(self, handler: H) -> &'a mut Router
    where
        H: Fn(BotContext, Update, CodedCallbackInput<T, C>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = HandlerResult> + Send + 'static,
    {
        let guards = Arc::new(self.config.guards);
        let handler = Arc::new(handler);
        self.router.route_prepared_handler_with_state(
            |update, _state| CodedCallbackInput::<T, C>::extract(update),
            RouteResolution::Default,
            move |context, update, _state, payload| {
                let guards = Arc::clone(&guards);
                let handler = Arc::clone(&handler);
                async move {
                    run_route_guards(guards.as_ref(), &context, &update).await?;
                    handler(context, update, payload).await
                }
            },
        )
    }

    pub fn handle_fallible<H, Fut>(self, handler: H) -> &'a mut Router
    where
        H: Fn(BotContext, Update, CodedCallbackInput<T, C>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = HandlerResult> + Send + 'static,
    {
        self.handle(handler)
    }

    pub fn handle_with_policy<H, Fut>(self, policy: ErrorPolicy, handler: H) -> &'a mut Router
    where
        H: Fn(BotContext, Update, CodedCallbackInput<T, C>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = HandlerResult> + Send + 'static,
    {
        let guards = Arc::new(self.config.guards);
        let handler = Arc::new(handler);
        self.router.route_prepared_handler_with_state(
            |update, _state| CodedCallbackInput::<T, C>::extract(update),
            RouteResolution::Policy(policy),
            move |context, update, _state, payload| {
                let guards = Arc::clone(&guards);
                let handler = Arc::clone(&handler);
                async move {
                    run_route_guards(guards.as_ref(), &context, &update).await?;
                    handler(context, update, payload).await
                }
            },
        )
    }
}
