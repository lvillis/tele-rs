use super::support::{ReplyContext, callback_query_id, reply_context};
use super::*;

fn text_send_request(
    chat_id: impl Into<ChatId>,
    text: impl Into<String>,
) -> Result<SendMessageRequest> {
    SendMessageRequest::new(chat_id, text)
}

trait ReplyContextRequest {
    fn apply_reply_context(&mut self, context: &ReplyContext);
}

macro_rules! apply_basic_reply_context {
    ($request:ident, $context:ident) => {
        $request.message_thread_id = $context.message_thread_id;
        $request.reply_parameters = $context.reply_parameters.clone();
        $request.business_connection_id = $context.business_connection_id.clone();
    };
}

macro_rules! impl_reply_context_request {
    ($($ty:ty),* $(,)?) => {
        $(
            impl ReplyContextRequest for $ty {
                fn apply_reply_context(&mut self, context: &ReplyContext) {
                    apply_basic_reply_context!(self, context);
                }
            }
        )*
    };
}

macro_rules! impl_direct_messages_reply_context_request {
    ($($ty:ty),* $(,)?) => {
        $(
            impl ReplyContextRequest for $ty {
                fn apply_reply_context(&mut self, context: &ReplyContext) {
                    apply_basic_reply_context!(self, context);
                    self.direct_messages_topic_id = context.direct_messages_topic_id;
                }
            }
        )*
    };
}

impl_direct_messages_reply_context_request!(
    SendMessageRequest,
    SendPhotoRequest,
    SendDocumentRequest,
    SendVideoRequest,
    SendAudioRequest,
    SendAnimationRequest,
    SendVoiceRequest,
    SendVideoNoteRequest,
    SendStickerRequest,
    SendMediaGroupRequest,
    SendLocationRequest,
    SendVenueRequest,
    SendContactRequest,
    SendDiceRequest,
);

impl_reply_context_request!(SendPollRequest);

fn build_reply_request<R>(update: &Update, build: impl FnOnce(i64) -> R) -> Result<R>
where
    R: ReplyContextRequest,
{
    let context = reply_context(update)?;
    let mut request = build(context.chat_id);
    request.apply_reply_context(&context);
    Ok(request)
}

fn try_build_reply_request<R>(update: &Update, build: impl FnOnce(i64) -> Result<R>) -> Result<R>
where
    R: ReplyContextRequest,
{
    let context = reply_context(update)?;
    let mut request = build(context.chat_id)?;
    request.apply_reply_context(&context);
    Ok(request)
}

fn reply_text_request(update: &Update, text: impl Into<String>) -> Result<SendMessageRequest> {
    try_build_reply_request(update, |chat_id| text_send_request(chat_id, text))
}

fn location_send_request(
    chat_id: impl Into<ChatId>,
    latitude: f64,
    longitude: f64,
) -> SendLocationRequest {
    SendLocationRequest::new(chat_id, latitude, longitude)
}

fn reply_location_request(
    update: &Update,
    latitude: f64,
    longitude: f64,
) -> Result<SendLocationRequest> {
    build_reply_request(update, |chat_id| {
        location_send_request(chat_id, latitude, longitude)
    })
}

fn venue_send_request(
    chat_id: impl Into<ChatId>,
    latitude: f64,
    longitude: f64,
    title: impl Into<String>,
    address: impl Into<String>,
) -> SendVenueRequest {
    SendVenueRequest::new(chat_id, latitude, longitude, title, address)
}

fn reply_venue_request(
    update: &Update,
    latitude: f64,
    longitude: f64,
    title: impl Into<String>,
    address: impl Into<String>,
) -> Result<SendVenueRequest> {
    build_reply_request(update, |chat_id| {
        venue_send_request(chat_id, latitude, longitude, title, address)
    })
}

fn contact_send_request(
    chat_id: impl Into<ChatId>,
    phone_number: impl Into<String>,
    first_name: impl Into<String>,
) -> SendContactRequest {
    SendContactRequest::new(chat_id, phone_number, first_name)
}

fn reply_contact_request(
    update: &Update,
    phone_number: impl Into<String>,
    first_name: impl Into<String>,
) -> Result<SendContactRequest> {
    build_reply_request(update, |chat_id| {
        contact_send_request(chat_id, phone_number, first_name)
    })
}

fn poll_send_request(
    chat_id: impl Into<ChatId>,
    question: impl Into<String>,
    options: impl IntoIterator<Item = impl Into<InputPollOption>>,
) -> Result<SendPollRequest> {
    SendPollRequest::new(chat_id, question, options)
}

fn reply_poll_request(
    update: &Update,
    question: impl Into<String>,
    options: impl IntoIterator<Item = impl Into<InputPollOption>>,
) -> Result<SendPollRequest> {
    try_build_reply_request(update, |chat_id| {
        poll_send_request(chat_id, question, options)
    })
}

fn stop_poll_request(chat_id: impl Into<ChatId>, message_id: MessageId) -> StopPollRequest {
    StopPollRequest::new(chat_id, message_id)
}

fn dice_send_request(chat_id: impl Into<ChatId>) -> SendDiceRequest {
    SendDiceRequest::new(chat_id)
}

fn reply_dice_request(update: &Update) -> Result<SendDiceRequest> {
    build_reply_request(update, dice_send_request)
}

fn chat_action_request(chat_id: impl Into<ChatId>, action: ChatAction) -> SendChatActionRequest {
    SendChatActionRequest::new(chat_id, action)
}

fn chat_action_for_update_request(
    update: &Update,
    action: ChatAction,
) -> Result<SendChatActionRequest> {
    let context = reply_context(update)?;
    let mut request = chat_action_request(context.chat_id, action);
    request.business_connection_id = context.business_connection_id;
    request.message_thread_id = context.message_thread_id;
    Ok(request)
}

macro_rules! impl_file_message_request_helpers {
    (
        $send_fn:ident,
        $upload_fn:ident,
        $reply_fn:ident,
        $reply_upload_fn:ident,
        $request_ty:ty,
        $file:ident
    ) => {
        fn $send_fn(chat_id: impl Into<ChatId>, $file: impl Into<String>) -> $request_ty {
            <$request_ty>::new(chat_id, $file)
        }

        fn $upload_fn(chat_id: impl Into<ChatId>) -> $request_ty {
            <$request_ty>::for_upload(chat_id)
        }

        fn $reply_fn(update: &Update, $file: impl Into<String>) -> Result<$request_ty> {
            build_reply_request(update, |chat_id| $send_fn(chat_id, $file))
        }

        fn $reply_upload_fn(update: &Update) -> Result<$request_ty> {
            build_reply_request(update, $upload_fn)
        }
    };
}

impl_file_message_request_helpers!(
    photo_send_request,
    photo_upload_request,
    reply_photo_request,
    reply_photo_upload_request,
    SendPhotoRequest,
    photo
);
impl_file_message_request_helpers!(
    document_send_request,
    document_upload_request,
    reply_document_request,
    reply_document_upload_request,
    SendDocumentRequest,
    document
);
impl_file_message_request_helpers!(
    video_send_request,
    video_upload_request,
    reply_video_request,
    reply_video_upload_request,
    SendVideoRequest,
    video
);
impl_file_message_request_helpers!(
    audio_send_request,
    audio_upload_request,
    reply_audio_request,
    reply_audio_upload_request,
    SendAudioRequest,
    audio
);
impl_file_message_request_helpers!(
    animation_send_request,
    animation_upload_request,
    reply_animation_request,
    reply_animation_upload_request,
    SendAnimationRequest,
    animation
);
impl_file_message_request_helpers!(
    voice_send_request,
    voice_upload_request,
    reply_voice_request,
    reply_voice_upload_request,
    SendVoiceRequest,
    voice
);
impl_file_message_request_helpers!(
    video_note_send_request,
    video_note_upload_request,
    reply_video_note_request,
    reply_video_note_upload_request,
    SendVideoNoteRequest,
    video_note
);
impl_file_message_request_helpers!(
    sticker_send_request,
    sticker_upload_request,
    reply_sticker_request,
    reply_sticker_upload_request,
    SendStickerRequest,
    sticker
);

fn media_group_send_request<I, M>(
    chat_id: impl Into<ChatId>,
    media: I,
) -> Result<SendMediaGroupRequest>
where
    I: IntoIterator<Item = M>,
    M: Into<InputMediaGroupItem>,
{
    SendMediaGroupRequest::new(chat_id, media.into_iter().map(Into::into).collect())
}

fn reply_media_group_request<I, M>(update: &Update, media: I) -> Result<SendMediaGroupRequest>
where
    I: IntoIterator<Item = M>,
    M: Into<InputMediaGroupItem>,
{
    try_build_reply_request(update, |chat_id| media_group_send_request(chat_id, media))
}

fn media_group_upload_request<I, M>(
    chat_id: impl Into<ChatId>,
    media: I,
) -> Result<SendMediaGroupRequest>
where
    I: IntoIterator<Item = M>,
    M: Into<InputMediaGroupItem>,
{
    media_group_send_request(chat_id, media)
}

fn reply_media_group_upload_request<I, M>(
    update: &Update,
    media: I,
) -> Result<SendMediaGroupRequest>
where
    I: IntoIterator<Item = M>,
    M: Into<InputMediaGroupItem>,
{
    reply_media_group_request(update, media)
}

fn callback_answer_request(
    callback_query_id: impl Into<String>,
    text: Option<String>,
) -> AnswerCallbackQueryRequest {
    AnswerCallbackQueryRequest {
        callback_query_id: callback_query_id.into(),
        text,
        show_alert: None,
        url: None,
        cache_time: None,
    }
}

macro_rules! impl_common_callback_answer_builder_methods {
    ($builder:ident, $request_ty:ty) => {
        impl $builder {
            /// Sets the callback answer text shown to the user.
            pub fn text(mut self, text: impl Into<String>) -> Self {
                self.request.text = Some(text.into());
                self
            }

            /// Sets the callback answer text, or clears it when `None`.
            pub fn text_optional(mut self, text: Option<String>) -> Self {
                self.request.text = text;
                self
            }

            /// Shows the callback answer as an alert dialog instead of a toast.
            pub fn show_alert(mut self, enabled: bool) -> Self {
                self.request.show_alert = enabled.then_some(true);
                self
            }

            /// Redirects the user to a URL after the callback answer is acknowledged.
            pub fn url(mut self, url: impl Into<String>) -> Self {
                self.request.url = Some(url.into());
                self
            }

            /// Sets Telegram-side caching for identical callback answers.
            pub fn cache_time(mut self, cache_time: u32) -> Self {
                self.request.cache_time = Some(cache_time);
                self
            }

            /// Returns the typed request for lower-level reuse or inspection.
            pub fn into_request(self) -> $request_ty {
                self.request
            }
        }
    };
}

macro_rules! impl_common_delivery_option_builder_methods {
    ($builder:ident, $request_ty:ty) => {
        impl $builder {
            /// Sets explicit reply parameters.
            pub fn reply_parameters(mut self, reply_parameters: ReplyParameters) -> Self {
                self.request.reply_parameters = Some(reply_parameters);
                self
            }

            /// Replies to a concrete message by id.
            pub fn reply_to_message(mut self, message_id: MessageId) -> Self {
                self.request.reply_parameters = Some(ReplyParameters::new(message_id));
                self
            }

            /// Sends on behalf of a Telegram Business connection.
            pub fn business_connection_id(
                mut self,
                business_connection_id: impl Into<String>,
            ) -> Self {
                self.request.business_connection_id = Some(business_connection_id.into());
                self
            }

            /// Targets a forum topic / message thread when applicable.
            pub fn message_thread_id(mut self, message_thread_id: i64) -> Self {
                self.request.message_thread_id = Some(message_thread_id);
                self
            }

            /// Targets a direct messages topic when sending to a channel direct messages chat.
            pub fn direct_messages_topic_id(mut self, direct_messages_topic_id: i64) -> Self {
                self.request.direct_messages_topic_id = Some(direct_messages_topic_id);
                self
            }

            /// Sends silently when `true`.
            pub fn disable_notification(mut self, enabled: bool) -> Self {
                self.request.disable_notification = enabled.then_some(true);
                self
            }

            /// Protects the sent content from forwarding and saving when `true`.
            pub fn protect_content(mut self, enabled: bool) -> Self {
                self.request.protect_content = enabled.then_some(true);
                self
            }

            /// Allows high-throughput paid broadcast sends when `true`.
            pub fn allow_paid_broadcast(mut self, enabled: bool) -> Self {
                self.request.allow_paid_broadcast = enabled.then_some(true);
                self
            }

            /// Adds a Telegram message effect to the sent content.
            pub fn message_effect_id(mut self, message_effect_id: impl Into<String>) -> Self {
                self.request.message_effect_id = Some(message_effect_id.into());
                self
            }

            /// Returns the typed request for lower-level reuse or inspection.
            pub fn into_request(self) -> $request_ty {
                self.request
            }
        }
    };
}

macro_rules! impl_common_send_option_builder_methods {
    ($builder:ident, $request_ty:ty) => {
        impl $builder {
            /// Attaches reply markup such as an inline keyboard.
            pub fn reply_markup(mut self, reply_markup: impl Into<ReplyMarkup>) -> Self {
                self.request.reply_markup = Some(reply_markup.into());
                self
            }

            /// Sets suggested post parameters for direct messages chats.
            pub fn suggested_post_parameters(
                mut self,
                suggested_post_parameters: SuggestedPostParameters,
            ) -> Self {
                self.request.suggested_post_parameters = Some(suggested_post_parameters);
                self
            }
        }

        impl_common_delivery_option_builder_methods!($builder, $request_ty);
    };
}

macro_rules! impl_common_poll_builder_methods {
    ($builder:ident, $request_ty:ty) => {
        impl $builder {
            /// Attaches reply markup such as an inline keyboard.
            pub fn reply_markup(mut self, reply_markup: impl Into<ReplyMarkup>) -> Self {
                self.request.reply_markup = Some(reply_markup.into());
                self
            }

            /// Sets explicit reply parameters.
            pub fn reply_parameters(mut self, reply_parameters: ReplyParameters) -> Self {
                self.request.reply_parameters = Some(reply_parameters);
                self
            }

            /// Replies to a concrete message by id.
            pub fn reply_to_message(mut self, message_id: MessageId) -> Self {
                self.request.reply_parameters = Some(ReplyParameters::new(message_id));
                self
            }

            /// Sends on behalf of a Telegram Business connection.
            pub fn business_connection_id(
                mut self,
                business_connection_id: impl Into<String>,
            ) -> Self {
                self.request.business_connection_id = Some(business_connection_id.into());
                self
            }

            /// Targets a forum topic / message thread when applicable.
            pub fn message_thread_id(mut self, message_thread_id: i64) -> Self {
                self.request.message_thread_id = Some(message_thread_id);
                self
            }

            /// Sends silently when `true`.
            pub fn disable_notification(mut self, enabled: bool) -> Self {
                self.request.disable_notification = enabled.then_some(true);
                self
            }

            /// Protects the sent content from forwarding and saving when `true`.
            pub fn protect_content(mut self, enabled: bool) -> Self {
                self.request.protect_content = enabled.then_some(true);
                self
            }

            /// Allows high-throughput paid broadcast sends when `true`.
            pub fn allow_paid_broadcast(mut self, enabled: bool) -> Self {
                self.request.allow_paid_broadcast = enabled.then_some(true);
                self
            }

            /// Adds a Telegram message effect to the sent content.
            pub fn message_effect_id(mut self, message_effect_id: impl Into<String>) -> Self {
                self.request.message_effect_id = Some(message_effect_id.into());
                self
            }

            /// Returns the typed request for lower-level reuse or inspection.
            pub fn into_request(self) -> $request_ty {
                self.request
            }
        }
    };
}

macro_rules! impl_chat_action_builder_methods {
    ($builder:ident, $request_ty:ty) => {
        impl $builder {
            /// Sends on behalf of a Telegram Business connection.
            pub fn business_connection_id(
                mut self,
                business_connection_id: impl Into<String>,
            ) -> Self {
                self.request.business_connection_id = Some(business_connection_id.into());
                self
            }

            /// Targets a forum topic / message thread when applicable.
            pub fn message_thread_id(mut self, message_thread_id: i64) -> Self {
                self.request.message_thread_id = Some(message_thread_id);
                self
            }

            /// Returns the typed request for lower-level reuse or inspection.
            pub fn into_request(self) -> $request_ty {
                self.request
            }
        }
    };
}

macro_rules! impl_stop_poll_builder_methods {
    ($builder:ident, $request_ty:ty) => {
        impl $builder {
            /// Sets the business connection used to stop the poll.
            pub fn business_connection_id(
                mut self,
                business_connection_id: impl Into<String>,
            ) -> Self {
                self.request.business_connection_id = Some(business_connection_id.into());
                self
            }

            /// Replaces the stopped poll message inline keyboard.
            pub fn reply_markup(mut self, reply_markup: InlineKeyboardMarkup) -> Self {
                self.request.reply_markup = Some(reply_markup);
                self
            }

            /// Returns the typed request for lower-level reuse or inspection.
            pub fn into_request(self) -> $request_ty {
                self.request
            }
        }
    };
}

macro_rules! impl_common_media_builder_methods {
    ($builder:ident, $request_ty:ty) => {
        impl $builder {
            /// Sets the media caption.
            pub fn caption(mut self, caption: impl Into<String>) -> Self {
                self.request.caption = Some(caption.into());
                self
            }

            /// Sets caption parse mode.
            pub fn parse_mode(mut self, parse_mode: ParseMode) -> Self {
                self.request.parse_mode = Some(parse_mode);
                self
            }

            /// Sets explicit caption entities instead of a parse mode.
            pub fn caption_entities(mut self, entities: Vec<MessageEntity>) -> Self {
                self.request.caption_entities = Some(entities);
                self
            }
        }

        impl_common_send_option_builder_methods!($builder, $request_ty);
    };
}

macro_rules! impl_common_media_group_builder_methods {
    ($builder:ident, $request_ty:ty) => {
        impl $builder {
            /// Appends one more media item to the group.
            pub fn add_media(mut self, media: impl Into<InputMediaGroupItem>) -> Self {
                self.request.media.push(media.into());
                self
            }
        }

        impl_common_delivery_option_builder_methods!($builder, $request_ty);
    };
}

macro_rules! impl_common_sticker_builder_methods {
    ($builder:ident, $request_ty:ty) => {
        impl $builder {
            /// Sets the optional emoji associated with the sticker send.
            pub fn emoji(mut self, emoji: impl Into<String>) -> Self {
                self.request.emoji = Some(emoji.into());
                self
            }
        }

        impl_common_send_option_builder_methods!($builder, $request_ty);
    };
}

/// Stable builder for high-level callback answers on the async app facade.
///
/// Start this from [`AppApi::callback_answer`] or [`AppApi::callback_answer_from_update`] when
/// you need more than the shortcut `answer_callback(...)` helpers expose.
#[cfg(feature = "_async")]
#[derive(Clone)]
#[must_use = "call `.send().await` or `.into_request()` to finish the callback answer"]
pub struct CallbackAnswerBuilder {
    client: Client,
    request: AnswerCallbackQueryRequest,
}

#[cfg(feature = "_async")]
impl CallbackAnswerBuilder {
    fn new(client: Client, request: AnswerCallbackQueryRequest) -> Self {
        Self { client, request }
    }

    /// Sends the callback answer request.
    pub async fn send(self) -> Result<bool> {
        self.client
            .updates()
            .answer_callback_query(&self.request)
            .await
    }
}

#[cfg(feature = "_async")]
impl_common_callback_answer_builder_methods!(CallbackAnswerBuilder, AnswerCallbackQueryRequest);

/// Stable builder for high-level text sends on the async app facade.
///
/// Start this from [`AppApi::text`] or [`AppApi::reply`] for the common message-send path.
#[cfg(feature = "_async")]
#[derive(Clone)]
#[must_use = "call `.send().await` or `.into_request()` to finish the message send"]
pub struct TextSendBuilder {
    client: Client,
    request: SendMessageRequest,
}

#[cfg(feature = "_async")]
impl TextSendBuilder {
    fn new(client: Client, request: SendMessageRequest) -> Self {
        Self { client, request }
    }

    /// Sets text parse mode.
    pub fn parse_mode(mut self, parse_mode: ParseMode) -> Self {
        self.request = self.request.parse_mode(parse_mode);
        self
    }

    /// Sets explicit text entities instead of a parse mode.
    pub fn entities(mut self, entities: Vec<MessageEntity>) -> Self {
        self.request = self.request.entities(entities);
        self
    }

    /// Sets Telegram link preview behavior explicitly.
    pub fn link_preview_options(mut self, link_preview_options: LinkPreviewOptions) -> Self {
        self.request = self.request.link_preview_options(link_preview_options);
        self
    }

    /// Disables link previews for the text message.
    pub fn disable_link_preview(mut self) -> Self {
        self.request = self.request.disable_link_preview();
        self
    }

    /// Sends the message.
    pub async fn send(self) -> Result<Message> {
        self.client.messages().send_message(&self.request).await
    }
}

#[cfg(feature = "_async")]
impl_common_send_option_builder_methods!(TextSendBuilder, SendMessageRequest);

/// Stable builder for high-level location sends on the async app facade.
#[cfg(feature = "_async")]
#[derive(Clone)]
#[must_use = "call `.send().await` or `.into_request()` to finish the send"]
pub struct LocationSendBuilder {
    client: Client,
    request: SendLocationRequest,
}

#[cfg(feature = "_async")]
impl LocationSendBuilder {
    fn new(client: Client, request: SendLocationRequest) -> Self {
        Self { client, request }
    }

    /// Sets horizontal location accuracy in meters.
    pub fn horizontal_accuracy(mut self, horizontal_accuracy: f64) -> Self {
        self.request.horizontal_accuracy = Some(horizontal_accuracy);
        self
    }

    /// Sets live location update period in seconds.
    pub fn live_period(mut self, live_period: u32) -> Self {
        self.request.live_period = Some(live_period);
        self
    }

    /// Sets movement direction in degrees.
    pub fn heading(mut self, heading: u16) -> Self {
        self.request.heading = Some(heading);
        self
    }

    /// Sets proximity alert radius in meters.
    pub fn proximity_alert_radius(mut self, proximity_alert_radius: u32) -> Self {
        self.request.proximity_alert_radius = Some(proximity_alert_radius);
        self
    }

    /// Sends the location.
    pub async fn send(self) -> Result<Message> {
        self.client.messages().send_location(&self.request).await
    }
}

#[cfg(feature = "_async")]
impl_common_send_option_builder_methods!(LocationSendBuilder, SendLocationRequest);

/// Stable builder for high-level venue sends on the async app facade.
#[cfg(feature = "_async")]
#[derive(Clone)]
#[must_use = "call `.send().await` or `.into_request()` to finish the send"]
pub struct VenueSendBuilder {
    client: Client,
    request: SendVenueRequest,
}

#[cfg(feature = "_async")]
impl VenueSendBuilder {
    fn new(client: Client, request: SendVenueRequest) -> Self {
        Self { client, request }
    }

    /// Sets a Foursquare venue id.
    pub fn foursquare_id(mut self, foursquare_id: impl Into<String>) -> Self {
        self.request.foursquare_id = Some(foursquare_id.into());
        self
    }

    /// Sets a Foursquare venue type.
    pub fn foursquare_type(mut self, foursquare_type: impl Into<String>) -> Self {
        self.request.foursquare_type = Some(foursquare_type.into());
        self
    }

    /// Sets a Google Places id.
    pub fn google_place_id(mut self, google_place_id: impl Into<String>) -> Self {
        self.request.google_place_id = Some(google_place_id.into());
        self
    }

    /// Sets a Google Places type.
    pub fn google_place_type(mut self, google_place_type: impl Into<String>) -> Self {
        self.request.google_place_type = Some(google_place_type.into());
        self
    }

    /// Sends the venue.
    pub async fn send(self) -> Result<Message> {
        self.client.messages().send_venue(&self.request).await
    }
}

#[cfg(feature = "_async")]
impl_common_send_option_builder_methods!(VenueSendBuilder, SendVenueRequest);

/// Stable builder for high-level contact sends on the async app facade.
#[cfg(feature = "_async")]
#[derive(Clone)]
#[must_use = "call `.send().await` or `.into_request()` to finish the send"]
pub struct ContactSendBuilder {
    client: Client,
    request: SendContactRequest,
}

#[cfg(feature = "_async")]
impl ContactSendBuilder {
    fn new(client: Client, request: SendContactRequest) -> Self {
        Self { client, request }
    }

    /// Sets the contact last name.
    pub fn last_name(mut self, last_name: impl Into<String>) -> Self {
        self.request.last_name = Some(last_name.into());
        self
    }

    /// Sets the contact vCard payload.
    pub fn vcard(mut self, vcard: impl Into<String>) -> Self {
        self.request.vcard = Some(vcard.into());
        self
    }

    /// Sends the contact.
    pub async fn send(self) -> Result<Message> {
        self.client.messages().send_contact(&self.request).await
    }
}

#[cfg(feature = "_async")]
impl_common_send_option_builder_methods!(ContactSendBuilder, SendContactRequest);

/// Stable builder for high-level poll sends on the async app facade.
#[cfg(feature = "_async")]
#[derive(Clone)]
#[must_use = "call `.send().await` or `.into_request()` to finish the send"]
pub struct PollSendBuilder {
    client: Client,
    request: SendPollRequest,
}

#[cfg(feature = "_async")]
impl PollSendBuilder {
    fn new(client: Client, request: SendPollRequest) -> Self {
        Self { client, request }
    }

    /// Sets question parse mode.
    pub fn question_parse_mode(mut self, parse_mode: ParseMode) -> Self {
        self.request.question_parse_mode = Some(parse_mode);
        self
    }

    /// Sets explicit question entities.
    pub fn question_entities(mut self, entities: Vec<MessageEntity>) -> Self {
        self.request.question_entities = Some(entities);
        self
    }

    /// Sets whether the poll is anonymous.
    pub fn anonymous(mut self, enabled: bool) -> Self {
        self.request.is_anonymous = Some(enabled);
        self
    }

    /// Sets the poll type.
    pub fn kind(mut self, kind: PollKind) -> Self {
        self.request.kind = Some(kind);
        self
    }

    /// Allows selecting multiple answers when `true`.
    pub fn allows_multiple_answers(mut self, enabled: bool) -> Self {
        self.request.allows_multiple_answers = Some(enabled);
        self
    }

    /// Allows voters to change their choice while the poll is open.
    pub fn allows_revoting(mut self, enabled: bool) -> Self {
        self.request.allows_revoting = Some(enabled);
        self
    }

    /// Randomizes the answer order for each voter when `true`.
    pub fn shuffle_options(mut self, enabled: bool) -> Self {
        self.request.shuffle_options = Some(enabled);
        self
    }

    /// Lets users add extra options to a non-anonymous regular poll.
    pub fn allow_adding_options(mut self, enabled: bool) -> Self {
        self.request.allow_adding_options = Some(enabled);
        self
    }

    /// Hides poll results until the poll closes.
    pub fn hide_results_until_closes(mut self, enabled: bool) -> Self {
        self.request.hide_results_until_closes = Some(enabled);
        self
    }

    /// Restricts voting to chat members when Telegram supports it for the target chat.
    pub fn members_only(mut self, enabled: bool) -> Self {
        self.request.members_only = Some(enabled);
        self
    }

    /// Restricts search to the provided country codes for location-based polls.
    pub fn country_codes(mut self, country_codes: Vec<String>) -> Self {
        self.request.country_codes = Some(country_codes);
        self
    }

    /// Sets correct option ids for quiz polls.
    pub fn correct_option_ids(mut self, correct_option_ids: Vec<u8>) -> Self {
        self.request.correct_option_ids = Some(correct_option_ids);
        self
    }

    /// Sets quiz explanation text.
    pub fn explanation(mut self, explanation: impl Into<String>) -> Self {
        self.request.explanation = Some(explanation.into());
        self
    }

    /// Sets quiz explanation parse mode.
    pub fn explanation_parse_mode(mut self, parse_mode: ParseMode) -> Self {
        self.request.explanation_parse_mode = Some(parse_mode);
        self
    }

    /// Sets explicit quiz explanation entities.
    pub fn explanation_entities(mut self, entities: Vec<MessageEntity>) -> Self {
        self.request.explanation_entities = Some(entities);
        self
    }

    /// Adds media to the quiz explanation.
    pub fn explanation_media(mut self, media: impl Into<InputPollMedia>) -> Self {
        self.request.explanation_media = Some(media.into());
        self
    }

    /// Sets how long the poll remains open, in seconds.
    pub fn open_period(mut self, open_period: u32) -> Self {
        self.request.open_period = Some(open_period);
        self
    }

    /// Sets the poll close date as a Unix timestamp.
    pub fn close_date(mut self, close_date: i64) -> Self {
        self.request.close_date = Some(close_date);
        self
    }

    /// Sets poll description text.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.request.description = Some(description.into());
        self
    }

    /// Sets poll description parse mode.
    pub fn description_parse_mode(mut self, parse_mode: ParseMode) -> Self {
        self.request.description_parse_mode = Some(parse_mode);
        self
    }

    /// Sets explicit poll description entities.
    pub fn description_entities(mut self, entities: Vec<MessageEntity>) -> Self {
        self.request.description_entities = Some(entities);
        self
    }

    /// Adds media to the poll description.
    pub fn media(mut self, media: impl Into<InputPollMedia>) -> Self {
        self.request.media = Some(media.into());
        self
    }

    /// Closes the poll immediately when `true`.
    pub fn closed(mut self, enabled: bool) -> Self {
        self.request.is_closed = Some(enabled);
        self
    }

    /// Sends the poll.
    pub async fn send(self) -> Result<Message> {
        self.client.messages().send_poll(&self.request).await
    }
}

#[cfg(feature = "_async")]
impl_common_poll_builder_methods!(PollSendBuilder, SendPollRequest);

/// Stable builder for high-level stop-poll calls on the async app facade.
#[cfg(feature = "_async")]
#[derive(Clone)]
#[must_use = "call `.send().await` or `.into_request()` to finish the stop-poll call"]
pub struct StopPollBuilder {
    client: Client,
    request: StopPollRequest,
}

#[cfg(feature = "_async")]
impl StopPollBuilder {
    fn new(client: Client, request: StopPollRequest) -> Self {
        Self { client, request }
    }

    /// Stops the poll.
    pub async fn send(self) -> Result<Poll> {
        self.client.messages().stop_poll(&self.request).await
    }
}

#[cfg(feature = "_async")]
impl_stop_poll_builder_methods!(StopPollBuilder, StopPollRequest);

/// Stable builder for high-level dice sends on the async app facade.
#[cfg(feature = "_async")]
#[derive(Clone)]
#[must_use = "call `.send().await` or `.into_request()` to finish the send"]
pub struct DiceSendBuilder {
    client: Client,
    request: SendDiceRequest,
}

#[cfg(feature = "_async")]
impl DiceSendBuilder {
    fn new(client: Client, request: SendDiceRequest) -> Self {
        Self { client, request }
    }

    /// Sets the dice animation emoji.
    pub fn emoji(mut self, emoji: DiceEmoji) -> Self {
        self.request.emoji = Some(emoji);
        self
    }

    /// Sends the dice message.
    pub async fn send(self) -> Result<Message> {
        self.client.messages().send_dice(&self.request).await
    }
}

#[cfg(feature = "_async")]
impl_common_send_option_builder_methods!(DiceSendBuilder, SendDiceRequest);

/// Stable builder for high-level chat-action calls on the async app facade.
#[cfg(feature = "_async")]
#[derive(Clone)]
#[must_use = "call `.send().await` or `.into_request()` to finish the chat action"]
pub struct ChatActionBuilder {
    client: Client,
    request: SendChatActionRequest,
}

#[cfg(feature = "_async")]
impl ChatActionBuilder {
    fn new(client: Client, request: SendChatActionRequest) -> Self {
        Self { client, request }
    }

    /// Sends the chat action.
    pub async fn send(self) -> Result<bool> {
        self.client.messages().send_chat_action(&self.request).await
    }
}

#[cfg(feature = "_async")]
impl_chat_action_builder_methods!(ChatActionBuilder, SendChatActionRequest);

/// Stable builder for high-level photo sends on the async app facade.
///
/// Start this from [`AppApi::photo`] or [`AppApi::reply_photo`].
#[cfg(feature = "_async")]
#[derive(Clone)]
#[must_use = "call `.send().await` or `.into_request()` to finish the send"]
pub struct PhotoSendBuilder {
    client: Client,
    request: SendPhotoRequest,
}

#[cfg(feature = "_async")]
impl PhotoSendBuilder {
    fn new(client: Client, request: SendPhotoRequest) -> Self {
        Self { client, request }
    }

    /// Marks the photo as spoiler media when `true`.
    pub fn has_spoiler(mut self, enabled: bool) -> Self {
        self.request.has_spoiler = enabled.then_some(true);
        self
    }

    /// Sends the photo using a Telegram file id or URL.
    pub async fn send(self) -> Result<Message> {
        self.client.messages().send_photo(&self.request).await
    }
}

#[cfg(feature = "_async")]
impl_common_media_builder_methods!(PhotoSendBuilder, SendPhotoRequest);

/// Stable builder for high-level photo uploads on the async app facade.
///
/// Start this from [`AppApi::photo_upload`] or [`AppApi::reply_photo_upload`].
#[cfg(feature = "_async")]
#[derive(Clone)]
#[must_use = "call `.send(&file).await` or `.into_request()` to finish the upload"]
pub struct PhotoUploadBuilder {
    client: Client,
    request: SendPhotoRequest,
}

#[cfg(feature = "_async")]
impl PhotoUploadBuilder {
    fn new(client: Client, request: SendPhotoRequest) -> Self {
        Self { client, request }
    }

    /// Marks the photo as spoiler media when `true`.
    pub fn has_spoiler(mut self, enabled: bool) -> Self {
        self.request.has_spoiler = enabled.then_some(true);
        self
    }

    /// Uploads local bytes as the photo payload.
    pub async fn send(self, file: &UploadFile) -> Result<Message> {
        self.client
            .messages()
            .send_photo_upload(&self.request, file)
            .await
    }
}

#[cfg(feature = "_async")]
impl_common_media_builder_methods!(PhotoUploadBuilder, SendPhotoRequest);

/// Stable builder for high-level document sends on the async app facade.
///
/// Start this from [`AppApi::document`] or [`AppApi::reply_document`].
#[cfg(feature = "_async")]
#[derive(Clone)]
#[must_use = "call `.send().await` or `.into_request()` to finish the send"]
pub struct DocumentSendBuilder {
    client: Client,
    request: SendDocumentRequest,
}

#[cfg(feature = "_async")]
impl DocumentSendBuilder {
    fn new(client: Client, request: SendDocumentRequest) -> Self {
        Self { client, request }
    }

    /// Sets a document thumbnail by file id / URL / attach reference.
    pub fn thumbnail(mut self, thumbnail: impl Into<String>) -> Self {
        self.request.thumbnail = Some(thumbnail.into());
        self
    }

    /// Disables Telegram content-type detection when `true`.
    pub fn disable_content_type_detection(mut self, enabled: bool) -> Self {
        self.request.disable_content_type_detection = enabled.then_some(true);
        self
    }

    /// Sends the document using a Telegram file id or URL.
    pub async fn send(self) -> Result<Message> {
        self.client.messages().send_document(&self.request).await
    }
}

#[cfg(feature = "_async")]
impl_common_media_builder_methods!(DocumentSendBuilder, SendDocumentRequest);

/// Stable builder for high-level document uploads on the async app facade.
///
/// Start this from [`AppApi::document_upload`] or [`AppApi::reply_document_upload`].
#[cfg(feature = "_async")]
#[derive(Clone)]
#[must_use = "call `.send(&file).await`, `.send_parts(...)`, or `.into_request()` to finish the upload"]
pub struct DocumentUploadBuilder {
    client: Client,
    request: SendDocumentRequest,
}

#[cfg(feature = "_async")]
impl DocumentUploadBuilder {
    fn new(client: Client, request: SendDocumentRequest) -> Self {
        Self { client, request }
    }

    /// Sets a document thumbnail by file id / URL / attach reference.
    pub fn thumbnail(mut self, thumbnail: impl Into<String>) -> Self {
        self.request.thumbnail = Some(thumbnail.into());
        self
    }

    /// Disables Telegram content-type detection when `true`.
    pub fn disable_content_type_detection(mut self, enabled: bool) -> Self {
        self.request.disable_content_type_detection = enabled.then_some(true);
        self
    }

    /// Uploads local bytes as the document payload.
    pub async fn send(self, file: &UploadFile) -> Result<Message> {
        self.client
            .messages()
            .send_document_upload(&self.request, file)
            .await
    }

    /// Uploads local bytes as the document payload plus extra `attach://` parts.
    pub async fn send_parts(
        self,
        file: &UploadFile,
        extra_files: &[UploadPart],
    ) -> Result<Message> {
        self.client
            .messages()
            .send_document_upload_parts(&self.request, file, extra_files)
            .await
    }
}

#[cfg(feature = "_async")]
impl_common_media_builder_methods!(DocumentUploadBuilder, SendDocumentRequest);

/// Stable builder for high-level video sends on the async app facade.
///
/// Start this from [`AppApi::video`] or [`AppApi::reply_video`].
#[cfg(feature = "_async")]
#[derive(Clone)]
#[must_use = "call `.send().await` or `.into_request()` to finish the send"]
pub struct VideoSendBuilder {
    client: Client,
    request: SendVideoRequest,
}

#[cfg(feature = "_async")]
impl VideoSendBuilder {
    fn new(client: Client, request: SendVideoRequest) -> Self {
        Self { client, request }
    }

    /// Sets video duration in seconds.
    pub fn duration(mut self, duration: u32) -> Self {
        self.request.duration = Some(duration);
        self
    }

    /// Sets video width in pixels.
    pub fn width(mut self, width: u32) -> Self {
        self.request.width = Some(width);
        self
    }

    /// Sets video height in pixels.
    pub fn height(mut self, height: u32) -> Self {
        self.request.height = Some(height);
        self
    }

    /// Sets a video thumbnail by file id / URL / attach reference.
    pub fn thumbnail(mut self, thumbnail: impl Into<String>) -> Self {
        self.request.thumbnail = Some(thumbnail.into());
        self
    }

    /// Marks the video as streamable when `true`.
    pub fn supports_streaming(mut self, enabled: bool) -> Self {
        self.request.supports_streaming = enabled.then_some(true);
        self
    }

    /// Marks the video as spoiler media when `true`.
    pub fn has_spoiler(mut self, enabled: bool) -> Self {
        self.request.has_spoiler = enabled.then_some(true);
        self
    }

    /// Sends the video using a Telegram file id or URL.
    pub async fn send(self) -> Result<Message> {
        self.client.messages().send_video(&self.request).await
    }
}

#[cfg(feature = "_async")]
impl_common_media_builder_methods!(VideoSendBuilder, SendVideoRequest);

/// Stable builder for high-level video uploads on the async app facade.
///
/// Start this from [`AppApi::video_upload`] or [`AppApi::reply_video_upload`].
#[cfg(feature = "_async")]
#[derive(Clone)]
#[must_use = "call `.send(&file).await`, `.send_parts(...)`, or `.into_request()` to finish the upload"]
pub struct VideoUploadBuilder {
    client: Client,
    request: SendVideoRequest,
}

#[cfg(feature = "_async")]
impl VideoUploadBuilder {
    fn new(client: Client, request: SendVideoRequest) -> Self {
        Self { client, request }
    }

    /// Sets video duration in seconds.
    pub fn duration(mut self, duration: u32) -> Self {
        self.request.duration = Some(duration);
        self
    }

    /// Sets video width in pixels.
    pub fn width(mut self, width: u32) -> Self {
        self.request.width = Some(width);
        self
    }

    /// Sets video height in pixels.
    pub fn height(mut self, height: u32) -> Self {
        self.request.height = Some(height);
        self
    }

    /// Sets a video thumbnail by file id / URL / attach reference.
    pub fn thumbnail(mut self, thumbnail: impl Into<String>) -> Self {
        self.request.thumbnail = Some(thumbnail.into());
        self
    }

    /// Marks the video as streamable when `true`.
    pub fn supports_streaming(mut self, enabled: bool) -> Self {
        self.request.supports_streaming = enabled.then_some(true);
        self
    }

    /// Marks the video as spoiler media when `true`.
    pub fn has_spoiler(mut self, enabled: bool) -> Self {
        self.request.has_spoiler = enabled.then_some(true);
        self
    }

    /// Uploads local bytes as the video payload.
    pub async fn send(self, file: &UploadFile) -> Result<Message> {
        self.client
            .messages()
            .send_video_upload(&self.request, file)
            .await
    }

    /// Uploads local bytes as the video payload plus extra `attach://` parts.
    pub async fn send_parts(
        self,
        file: &UploadFile,
        extra_files: &[UploadPart],
    ) -> Result<Message> {
        self.client
            .messages()
            .send_video_upload_parts(&self.request, file, extra_files)
            .await
    }
}

#[cfg(feature = "_async")]
impl_common_media_builder_methods!(VideoUploadBuilder, SendVideoRequest);

/// Stable builder for high-level audio sends on the async app facade.
///
/// Start this from [`AppApi::audio`] or [`AppApi::reply_audio`].
#[cfg(feature = "_async")]
#[derive(Clone)]
#[must_use = "call `.send().await` or `.into_request()` to finish the send"]
pub struct AudioSendBuilder {
    client: Client,
    request: SendAudioRequest,
}

#[cfg(feature = "_async")]
impl AudioSendBuilder {
    fn new(client: Client, request: SendAudioRequest) -> Self {
        Self { client, request }
    }

    /// Sets audio duration in seconds.
    pub fn duration(mut self, duration: u32) -> Self {
        self.request.duration = Some(duration);
        self
    }

    /// Sets the displayed performer.
    pub fn performer(mut self, performer: impl Into<String>) -> Self {
        self.request.performer = Some(performer.into());
        self
    }

    /// Sets the displayed title.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.request.title = Some(title.into());
        self
    }

    /// Sets an audio thumbnail by file id / URL / attach reference.
    pub fn thumbnail(mut self, thumbnail: impl Into<String>) -> Self {
        self.request.thumbnail = Some(thumbnail.into());
        self
    }

    /// Sends the audio using a Telegram file id or URL.
    pub async fn send(self) -> Result<Message> {
        self.client.messages().send_audio(&self.request).await
    }
}

#[cfg(feature = "_async")]
impl_common_media_builder_methods!(AudioSendBuilder, SendAudioRequest);

/// Stable builder for high-level audio uploads on the async app facade.
///
/// Start this from [`AppApi::audio_upload`] or [`AppApi::reply_audio_upload`].
#[cfg(feature = "_async")]
#[derive(Clone)]
#[must_use = "call `.send(&file).await`, `.send_parts(...)`, or `.into_request()` to finish the upload"]
pub struct AudioUploadBuilder {
    client: Client,
    request: SendAudioRequest,
}

#[cfg(feature = "_async")]
impl AudioUploadBuilder {
    fn new(client: Client, request: SendAudioRequest) -> Self {
        Self { client, request }
    }

    /// Sets audio duration in seconds.
    pub fn duration(mut self, duration: u32) -> Self {
        self.request.duration = Some(duration);
        self
    }

    /// Sets the displayed performer.
    pub fn performer(mut self, performer: impl Into<String>) -> Self {
        self.request.performer = Some(performer.into());
        self
    }

    /// Sets the displayed title.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.request.title = Some(title.into());
        self
    }

    /// Sets an audio thumbnail by file id / URL / attach reference.
    pub fn thumbnail(mut self, thumbnail: impl Into<String>) -> Self {
        self.request.thumbnail = Some(thumbnail.into());
        self
    }

    /// Uploads local bytes as the audio payload.
    pub async fn send(self, file: &UploadFile) -> Result<Message> {
        self.client
            .messages()
            .send_audio_upload(&self.request, file)
            .await
    }

    /// Uploads local bytes as the audio payload plus extra `attach://` parts.
    pub async fn send_parts(
        self,
        file: &UploadFile,
        extra_files: &[UploadPart],
    ) -> Result<Message> {
        self.client
            .messages()
            .send_audio_upload_parts(&self.request, file, extra_files)
            .await
    }
}

#[cfg(feature = "_async")]
impl_common_media_builder_methods!(AudioUploadBuilder, SendAudioRequest);

/// Stable builder for high-level animation sends on the async app facade.
///
/// Start this from [`AppApi::animation`] or [`AppApi::reply_animation`].
#[cfg(feature = "_async")]
#[derive(Clone)]
#[must_use = "call `.send().await` or `.into_request()` to finish the send"]
pub struct AnimationSendBuilder {
    client: Client,
    request: SendAnimationRequest,
}

#[cfg(feature = "_async")]
impl AnimationSendBuilder {
    fn new(client: Client, request: SendAnimationRequest) -> Self {
        Self { client, request }
    }

    /// Sets animation duration in seconds.
    pub fn duration(mut self, duration: u32) -> Self {
        self.request.duration = Some(duration);
        self
    }

    /// Sets animation width in pixels.
    pub fn width(mut self, width: u32) -> Self {
        self.request.width = Some(width);
        self
    }

    /// Sets animation height in pixels.
    pub fn height(mut self, height: u32) -> Self {
        self.request.height = Some(height);
        self
    }

    /// Sets an animation thumbnail by file id / URL / attach reference.
    pub fn thumbnail(mut self, thumbnail: impl Into<String>) -> Self {
        self.request.thumbnail = Some(thumbnail.into());
        self
    }

    /// Marks the animation as spoiler media when `true`.
    pub fn has_spoiler(mut self, enabled: bool) -> Self {
        self.request.has_spoiler = enabled.then_some(true);
        self
    }

    /// Sends the animation using a Telegram file id or URL.
    pub async fn send(self) -> Result<Message> {
        self.client.messages().send_animation(&self.request).await
    }
}

#[cfg(feature = "_async")]
impl_common_media_builder_methods!(AnimationSendBuilder, SendAnimationRequest);

/// Stable builder for high-level animation uploads on the async app facade.
///
/// Start this from [`AppApi::animation_upload`] or [`AppApi::reply_animation_upload`].
#[cfg(feature = "_async")]
#[derive(Clone)]
#[must_use = "call `.send(&file).await`, `.send_parts(...)`, or `.into_request()` to finish the upload"]
pub struct AnimationUploadBuilder {
    client: Client,
    request: SendAnimationRequest,
}

#[cfg(feature = "_async")]
impl AnimationUploadBuilder {
    fn new(client: Client, request: SendAnimationRequest) -> Self {
        Self { client, request }
    }

    /// Sets animation duration in seconds.
    pub fn duration(mut self, duration: u32) -> Self {
        self.request.duration = Some(duration);
        self
    }

    /// Sets animation width in pixels.
    pub fn width(mut self, width: u32) -> Self {
        self.request.width = Some(width);
        self
    }

    /// Sets animation height in pixels.
    pub fn height(mut self, height: u32) -> Self {
        self.request.height = Some(height);
        self
    }

    /// Sets an animation thumbnail by file id / URL / attach reference.
    pub fn thumbnail(mut self, thumbnail: impl Into<String>) -> Self {
        self.request.thumbnail = Some(thumbnail.into());
        self
    }

    /// Marks the animation as spoiler media when `true`.
    pub fn has_spoiler(mut self, enabled: bool) -> Self {
        self.request.has_spoiler = enabled.then_some(true);
        self
    }

    /// Uploads local bytes as the animation payload.
    pub async fn send(self, file: &UploadFile) -> Result<Message> {
        self.client
            .messages()
            .send_animation_upload(&self.request, file)
            .await
    }

    /// Uploads local bytes as the animation payload plus extra `attach://` parts.
    pub async fn send_parts(
        self,
        file: &UploadFile,
        extra_files: &[UploadPart],
    ) -> Result<Message> {
        self.client
            .messages()
            .send_animation_upload_parts(&self.request, file, extra_files)
            .await
    }
}

#[cfg(feature = "_async")]
impl_common_media_builder_methods!(AnimationUploadBuilder, SendAnimationRequest);

/// Stable builder for high-level voice sends on the async app facade.
///
/// Start this from [`AppApi::voice`] or [`AppApi::reply_voice`].
#[cfg(feature = "_async")]
#[derive(Clone)]
#[must_use = "call `.send().await` or `.into_request()` to finish the send"]
pub struct VoiceSendBuilder {
    client: Client,
    request: SendVoiceRequest,
}

#[cfg(feature = "_async")]
impl VoiceSendBuilder {
    fn new(client: Client, request: SendVoiceRequest) -> Self {
        Self { client, request }
    }

    /// Sets voice duration in seconds.
    pub fn duration(mut self, duration: u32) -> Self {
        self.request.duration = Some(duration);
        self
    }

    /// Sends the voice message using a Telegram file id or URL.
    pub async fn send(self) -> Result<Message> {
        self.client.messages().send_voice(&self.request).await
    }
}

#[cfg(feature = "_async")]
impl_common_media_builder_methods!(VoiceSendBuilder, SendVoiceRequest);

/// Stable builder for high-level voice uploads on the async app facade.
///
/// Start this from [`AppApi::voice_upload`] or [`AppApi::reply_voice_upload`].
#[cfg(feature = "_async")]
#[derive(Clone)]
#[must_use = "call `.send(&file).await` or `.into_request()` to finish the upload"]
pub struct VoiceUploadBuilder {
    client: Client,
    request: SendVoiceRequest,
}

#[cfg(feature = "_async")]
impl VoiceUploadBuilder {
    fn new(client: Client, request: SendVoiceRequest) -> Self {
        Self { client, request }
    }

    /// Sets voice duration in seconds.
    pub fn duration(mut self, duration: u32) -> Self {
        self.request.duration = Some(duration);
        self
    }

    /// Uploads local bytes as the voice payload.
    pub async fn send(self, file: &UploadFile) -> Result<Message> {
        self.client
            .messages()
            .send_voice_upload(&self.request, file)
            .await
    }
}

#[cfg(feature = "_async")]
impl_common_media_builder_methods!(VoiceUploadBuilder, SendVoiceRequest);

/// Stable builder for high-level video note sends on the async app facade.
///
/// Start this from [`AppApi::video_note`] or [`AppApi::reply_video_note`].
#[cfg(feature = "_async")]
#[derive(Clone)]
#[must_use = "call `.send().await` or `.into_request()` to finish the send"]
pub struct VideoNoteSendBuilder {
    client: Client,
    request: SendVideoNoteRequest,
}

#[cfg(feature = "_async")]
impl VideoNoteSendBuilder {
    fn new(client: Client, request: SendVideoNoteRequest) -> Self {
        Self { client, request }
    }

    /// Sets video note duration in seconds.
    pub fn duration(mut self, duration: u32) -> Self {
        self.request.duration = Some(duration);
        self
    }

    /// Sets video note diameter in pixels.
    pub fn length(mut self, length: u32) -> Self {
        self.request.length = Some(length);
        self
    }

    /// Sets a video note thumbnail by file id or URL.
    pub fn thumbnail(mut self, thumbnail: impl Into<String>) -> Self {
        self.request.thumbnail = Some(thumbnail.into());
        self
    }

    /// Sends the video note using a Telegram file id or URL.
    pub async fn send(self) -> Result<Message> {
        self.client.messages().send_video_note(&self.request).await
    }
}

#[cfg(feature = "_async")]
impl_common_send_option_builder_methods!(VideoNoteSendBuilder, SendVideoNoteRequest);

/// Stable builder for high-level video note uploads on the async app facade.
///
/// Start this from [`AppApi::video_note_upload`] or [`AppApi::reply_video_note_upload`].
#[cfg(feature = "_async")]
#[derive(Clone)]
#[must_use = "call `.send(&file).await`, `.send_parts(...)`, or `.into_request()` to finish the upload"]
pub struct VideoNoteUploadBuilder {
    client: Client,
    request: SendVideoNoteRequest,
}

#[cfg(feature = "_async")]
impl VideoNoteUploadBuilder {
    fn new(client: Client, request: SendVideoNoteRequest) -> Self {
        Self { client, request }
    }

    /// Sets video note duration in seconds.
    pub fn duration(mut self, duration: u32) -> Self {
        self.request.duration = Some(duration);
        self
    }

    /// Sets video note diameter in pixels.
    pub fn length(mut self, length: u32) -> Self {
        self.request.length = Some(length);
        self
    }

    /// Sets a video note thumbnail by file id / URL / attach reference.
    pub fn thumbnail(mut self, thumbnail: impl Into<String>) -> Self {
        self.request.thumbnail = Some(thumbnail.into());
        self
    }

    /// Uploads local bytes as the video note payload.
    pub async fn send(self, file: &UploadFile) -> Result<Message> {
        self.client
            .messages()
            .send_video_note_upload(&self.request, file)
            .await
    }

    /// Uploads local bytes as the video note payload plus extra `attach://` parts.
    pub async fn send_parts(
        self,
        file: &UploadFile,
        extra_files: &[UploadPart],
    ) -> Result<Message> {
        self.client
            .messages()
            .send_video_note_upload_parts(&self.request, file, extra_files)
            .await
    }
}

#[cfg(feature = "_async")]
impl_common_send_option_builder_methods!(VideoNoteUploadBuilder, SendVideoNoteRequest);

/// Stable builder for high-level sticker sends on the async app facade.
///
/// Start this from [`AppApi::sticker`] or [`AppApi::reply_sticker`].
#[cfg(feature = "_async")]
#[derive(Clone)]
#[must_use = "call `.send().await` or `.into_request()` to finish the send"]
pub struct StickerSendBuilder {
    client: Client,
    request: SendStickerRequest,
}

#[cfg(feature = "_async")]
impl StickerSendBuilder {
    fn new(client: Client, request: SendStickerRequest) -> Self {
        Self { client, request }
    }

    /// Sends the sticker using a Telegram file id or URL.
    pub async fn send(self) -> Result<Message> {
        self.client.stickers().send_sticker(&self.request).await
    }
}

#[cfg(feature = "_async")]
impl_common_sticker_builder_methods!(StickerSendBuilder, SendStickerRequest);

/// Stable builder for high-level sticker uploads on the async app facade.
///
/// Start this from [`AppApi::sticker_upload`] or [`AppApi::reply_sticker_upload`].
#[cfg(feature = "_async")]
#[derive(Clone)]
#[must_use = "call `.send(&file).await` or `.into_request()` to finish the upload"]
pub struct StickerUploadBuilder {
    client: Client,
    request: SendStickerRequest,
}

#[cfg(feature = "_async")]
impl StickerUploadBuilder {
    fn new(client: Client, request: SendStickerRequest) -> Self {
        Self { client, request }
    }

    /// Uploads local bytes as the sticker payload.
    pub async fn send(self, file: &UploadFile) -> Result<Message> {
        self.client
            .stickers()
            .send_sticker_upload(&self.request, file)
            .await
    }
}

#[cfg(feature = "_async")]
impl_common_sticker_builder_methods!(StickerUploadBuilder, SendStickerRequest);

/// Stable builder for high-level media group sends on the async app facade.
///
/// Start this from [`AppApi::media_group`] or [`AppApi::reply_media_group`].
#[cfg(feature = "_async")]
#[derive(Clone)]
#[must_use = "call `.send().await` or `.into_request()` to finish the send"]
pub struct MediaGroupSendBuilder {
    client: Client,
    request: SendMediaGroupRequest,
}

#[cfg(feature = "_async")]
impl MediaGroupSendBuilder {
    fn new(client: Client, request: SendMediaGroupRequest) -> Self {
        Self { client, request }
    }

    /// Sends the media group.
    pub async fn send(self) -> Result<Vec<Message>> {
        self.client.messages().send_media_group(&self.request).await
    }
}

#[cfg(feature = "_async")]
impl_common_media_group_builder_methods!(MediaGroupSendBuilder, SendMediaGroupRequest);

/// Stable builder for high-level media group uploads on the async app facade.
///
/// Start this from [`AppApi::media_group_upload`] or [`AppApi::reply_media_group_upload`].
#[cfg(feature = "_async")]
#[derive(Clone)]
#[must_use = "call `.send(&files).await` or `.into_request()` to finish the upload"]
pub struct MediaGroupUploadBuilder {
    client: Client,
    request: SendMediaGroupRequest,
}

#[cfg(feature = "_async")]
impl MediaGroupUploadBuilder {
    fn new(client: Client, request: SendMediaGroupRequest) -> Self {
        Self { client, request }
    }

    /// Uploads local files referenced by `attach://...` media entries.
    pub async fn send(self, files: &[UploadPart]) -> Result<Vec<Message>> {
        self.client
            .messages()
            .send_media_group_upload(&self.request, files)
            .await
    }
}

#[cfg(feature = "_async")]
impl_common_media_group_builder_methods!(MediaGroupUploadBuilder, SendMediaGroupRequest);

/// Stable app-facing runtime facade for business code.
///
/// Prefer this facade inside handlers through `context.app()`, or directly through
/// `client.app()` in application code that is still part of the runtime plane.
///
/// Use this layer for:
///
/// - message sends via builder-style helpers
/// - callback answers
/// - moderation and governance notices
/// - membership / capability checks
/// - Web App runtime interactions
///
/// Use `client.control()` for startup, bootstrap, router preparation, and other orchestration
/// concerns.
#[cfg(feature = "_async")]
#[derive(Clone)]
pub struct AppApi {
    client: Client,
}

#[cfg(feature = "_async")]
impl AppApi {
    pub(crate) fn new(client: Client) -> Self {
        Self { client }
    }

    /// Returns the governance-oriented moderation facade.
    pub fn moderation(&self) -> ModerationApi {
        ModerationApi::new(self.client.clone())
    }

    /// Returns the membership/capability facade used by install and bind pre-check flows.
    pub fn membership(&self) -> MembershipApi {
        MembershipApi::new(self.client.clone())
    }

    /// Returns the dedicated Web App runtime facade.
    pub fn web_app(&self) -> WebAppApi {
        WebAppApi::new(self.client.clone())
    }

    /// Starts a callback-answer builder.
    ///
    /// Prefer this when you need richer callback options such as `show_alert`, `url`, or
    /// `cache_time`. For simple text-only answers, `answer_callback(...)` remains the short path.
    pub fn callback_answer(&self, callback_query_id: impl Into<String>) -> CallbackAnswerBuilder {
        let request = callback_answer_request(callback_query_id, None);
        CallbackAnswerBuilder::new(self.client.clone(), request)
    }

    /// Starts a callback-answer builder using the callback id extracted from an update.
    pub fn callback_answer_from_update(&self, update: &Update) -> Result<CallbackAnswerBuilder> {
        let Some(callback_query_id) = callback_query_id(update) else {
            return Err(super::support::invalid_request(
                "update does not contain callback query for answerCallbackQuery",
            ));
        };
        Ok(self.callback_answer(callback_query_id))
    }

    /// Starts a text-send builder for a target chat.
    pub fn text(
        &self,
        chat_id: impl Into<ChatId>,
        text: impl Into<String>,
    ) -> Result<TextSendBuilder> {
        let request = text_send_request(chat_id, text)?;
        Ok(TextSendBuilder::new(self.client.clone(), request))
    }

    /// Starts a text-send builder using the update reply target and quoting its source message when present.
    pub fn reply(&self, update: &Update, text: impl Into<String>) -> Result<TextSendBuilder> {
        let request = reply_text_request(update, text)?;
        Ok(TextSendBuilder::new(self.client.clone(), request))
    }

    /// Starts a location-send builder for a target chat.
    pub fn location(
        &self,
        chat_id: impl Into<ChatId>,
        latitude: f64,
        longitude: f64,
    ) -> LocationSendBuilder {
        let request = location_send_request(chat_id, latitude, longitude);
        LocationSendBuilder::new(self.client.clone(), request)
    }

    /// Starts a location-send builder using the update reply target and quoting its source message when present.
    pub fn reply_location(
        &self,
        update: &Update,
        latitude: f64,
        longitude: f64,
    ) -> Result<LocationSendBuilder> {
        let request = reply_location_request(update, latitude, longitude)?;
        Ok(LocationSendBuilder::new(self.client.clone(), request))
    }

    /// Starts a venue-send builder for a target chat.
    pub fn venue(
        &self,
        chat_id: impl Into<ChatId>,
        latitude: f64,
        longitude: f64,
        title: impl Into<String>,
        address: impl Into<String>,
    ) -> VenueSendBuilder {
        let request = venue_send_request(chat_id, latitude, longitude, title, address);
        VenueSendBuilder::new(self.client.clone(), request)
    }

    /// Starts a venue-send builder using the update reply target and quoting its source message when present.
    pub fn reply_venue(
        &self,
        update: &Update,
        latitude: f64,
        longitude: f64,
        title: impl Into<String>,
        address: impl Into<String>,
    ) -> Result<VenueSendBuilder> {
        let request = reply_venue_request(update, latitude, longitude, title, address)?;
        Ok(VenueSendBuilder::new(self.client.clone(), request))
    }

    /// Starts a contact-send builder for a target chat.
    pub fn contact(
        &self,
        chat_id: impl Into<ChatId>,
        phone_number: impl Into<String>,
        first_name: impl Into<String>,
    ) -> ContactSendBuilder {
        let request = contact_send_request(chat_id, phone_number, first_name);
        ContactSendBuilder::new(self.client.clone(), request)
    }

    /// Starts a contact-send builder using the update reply target and quoting its source message when present.
    pub fn reply_contact(
        &self,
        update: &Update,
        phone_number: impl Into<String>,
        first_name: impl Into<String>,
    ) -> Result<ContactSendBuilder> {
        let request = reply_contact_request(update, phone_number, first_name)?;
        Ok(ContactSendBuilder::new(self.client.clone(), request))
    }

    /// Starts a poll-send builder for a target chat.
    pub fn poll(
        &self,
        chat_id: impl Into<ChatId>,
        question: impl Into<String>,
        options: impl IntoIterator<Item = impl Into<InputPollOption>>,
    ) -> Result<PollSendBuilder> {
        let request = poll_send_request(chat_id, question, options)?;
        Ok(PollSendBuilder::new(self.client.clone(), request))
    }

    /// Starts a poll-send builder using the update reply target and quoting its source message when present.
    pub fn reply_poll(
        &self,
        update: &Update,
        question: impl Into<String>,
        options: impl IntoIterator<Item = impl Into<InputPollOption>>,
    ) -> Result<PollSendBuilder> {
        let request = reply_poll_request(update, question, options)?;
        Ok(PollSendBuilder::new(self.client.clone(), request))
    }

    /// Starts a stop-poll builder for a target chat and message.
    pub fn stop_poll(&self, chat_id: impl Into<ChatId>, message_id: MessageId) -> StopPollBuilder {
        let request = stop_poll_request(chat_id, message_id);
        StopPollBuilder::new(self.client.clone(), request)
    }

    /// Starts a dice-send builder for a target chat.
    pub fn dice(&self, chat_id: impl Into<ChatId>) -> DiceSendBuilder {
        let request = dice_send_request(chat_id);
        DiceSendBuilder::new(self.client.clone(), request)
    }

    /// Starts a dice-send builder using the update reply target and quoting its source message when present.
    pub fn reply_dice(&self, update: &Update) -> Result<DiceSendBuilder> {
        let request = reply_dice_request(update)?;
        Ok(DiceSendBuilder::new(self.client.clone(), request))
    }

    /// Starts a chat-action builder for a target chat.
    pub fn chat_action(&self, chat_id: impl Into<ChatId>, action: ChatAction) -> ChatActionBuilder {
        let request = chat_action_request(chat_id, action);
        ChatActionBuilder::new(self.client.clone(), request)
    }

    /// Starts a chat-action builder using the update reply target.
    pub fn chat_action_for_update(
        &self,
        update: &Update,
        action: ChatAction,
    ) -> Result<ChatActionBuilder> {
        let request = chat_action_for_update_request(update, action)?;
        Ok(ChatActionBuilder::new(self.client.clone(), request))
    }

    /// Starts a photo-send builder for a target chat.
    pub fn photo(&self, chat_id: impl Into<ChatId>, photo: impl Into<String>) -> PhotoSendBuilder {
        let request = photo_send_request(chat_id, photo);
        PhotoSendBuilder::new(self.client.clone(), request)
    }

    /// Starts a photo-upload builder for a target chat.
    pub fn photo_upload(&self, chat_id: impl Into<ChatId>) -> PhotoUploadBuilder {
        let request = photo_upload_request(chat_id);
        PhotoUploadBuilder::new(self.client.clone(), request)
    }

    /// Starts a photo-send builder using the update reply target and quoting its source message when present.
    pub fn reply_photo(
        &self,
        update: &Update,
        photo: impl Into<String>,
    ) -> Result<PhotoSendBuilder> {
        let request = reply_photo_request(update, photo)?;
        Ok(PhotoSendBuilder::new(self.client.clone(), request))
    }

    /// Starts a photo-upload builder using the update reply target and quoting its source message when present.
    pub fn reply_photo_upload(&self, update: &Update) -> Result<PhotoUploadBuilder> {
        let request = reply_photo_upload_request(update)?;
        Ok(PhotoUploadBuilder::new(self.client.clone(), request))
    }

    /// Starts a document-send builder for a target chat.
    pub fn document(
        &self,
        chat_id: impl Into<ChatId>,
        document: impl Into<String>,
    ) -> DocumentSendBuilder {
        let request = document_send_request(chat_id, document);
        DocumentSendBuilder::new(self.client.clone(), request)
    }

    /// Starts a document-upload builder for a target chat.
    pub fn document_upload(&self, chat_id: impl Into<ChatId>) -> DocumentUploadBuilder {
        let request = document_upload_request(chat_id);
        DocumentUploadBuilder::new(self.client.clone(), request)
    }

    /// Starts a document-send builder using the update reply target and quoting its source message when present.
    pub fn reply_document(
        &self,
        update: &Update,
        document: impl Into<String>,
    ) -> Result<DocumentSendBuilder> {
        let request = reply_document_request(update, document)?;
        Ok(DocumentSendBuilder::new(self.client.clone(), request))
    }

    /// Starts a document-upload builder using the update reply target and quoting its source message when present.
    pub fn reply_document_upload(&self, update: &Update) -> Result<DocumentUploadBuilder> {
        let request = reply_document_upload_request(update)?;
        Ok(DocumentUploadBuilder::new(self.client.clone(), request))
    }

    /// Starts a video-send builder for a target chat.
    pub fn video(&self, chat_id: impl Into<ChatId>, video: impl Into<String>) -> VideoSendBuilder {
        let request = video_send_request(chat_id, video);
        VideoSendBuilder::new(self.client.clone(), request)
    }

    /// Starts a video-upload builder for a target chat.
    pub fn video_upload(&self, chat_id: impl Into<ChatId>) -> VideoUploadBuilder {
        let request = video_upload_request(chat_id);
        VideoUploadBuilder::new(self.client.clone(), request)
    }

    /// Starts a video-send builder using the update reply target and quoting its source message when present.
    pub fn reply_video(
        &self,
        update: &Update,
        video: impl Into<String>,
    ) -> Result<VideoSendBuilder> {
        let request = reply_video_request(update, video)?;
        Ok(VideoSendBuilder::new(self.client.clone(), request))
    }

    /// Starts a video-upload builder using the update reply target and quoting its source message when present.
    pub fn reply_video_upload(&self, update: &Update) -> Result<VideoUploadBuilder> {
        let request = reply_video_upload_request(update)?;
        Ok(VideoUploadBuilder::new(self.client.clone(), request))
    }

    /// Starts an audio-send builder for a target chat.
    pub fn audio(&self, chat_id: impl Into<ChatId>, audio: impl Into<String>) -> AudioSendBuilder {
        let request = audio_send_request(chat_id, audio);
        AudioSendBuilder::new(self.client.clone(), request)
    }

    /// Starts an audio-upload builder for a target chat.
    pub fn audio_upload(&self, chat_id: impl Into<ChatId>) -> AudioUploadBuilder {
        let request = audio_upload_request(chat_id);
        AudioUploadBuilder::new(self.client.clone(), request)
    }

    /// Starts an audio-send builder using the update reply target and quoting its source message when present.
    pub fn reply_audio(
        &self,
        update: &Update,
        audio: impl Into<String>,
    ) -> Result<AudioSendBuilder> {
        let request = reply_audio_request(update, audio)?;
        Ok(AudioSendBuilder::new(self.client.clone(), request))
    }

    /// Starts an audio-upload builder using the update reply target and quoting its source message when present.
    pub fn reply_audio_upload(&self, update: &Update) -> Result<AudioUploadBuilder> {
        let request = reply_audio_upload_request(update)?;
        Ok(AudioUploadBuilder::new(self.client.clone(), request))
    }

    /// Starts an animation-send builder for a target chat.
    pub fn animation(
        &self,
        chat_id: impl Into<ChatId>,
        animation: impl Into<String>,
    ) -> AnimationSendBuilder {
        let request = animation_send_request(chat_id, animation);
        AnimationSendBuilder::new(self.client.clone(), request)
    }

    /// Starts an animation-upload builder for a target chat.
    pub fn animation_upload(&self, chat_id: impl Into<ChatId>) -> AnimationUploadBuilder {
        let request = animation_upload_request(chat_id);
        AnimationUploadBuilder::new(self.client.clone(), request)
    }

    /// Starts an animation-send builder using the update reply target and quoting its source message when present.
    pub fn reply_animation(
        &self,
        update: &Update,
        animation: impl Into<String>,
    ) -> Result<AnimationSendBuilder> {
        let request = reply_animation_request(update, animation)?;
        Ok(AnimationSendBuilder::new(self.client.clone(), request))
    }

    /// Starts an animation-upload builder using the update reply target and quoting its source message when present.
    pub fn reply_animation_upload(&self, update: &Update) -> Result<AnimationUploadBuilder> {
        let request = reply_animation_upload_request(update)?;
        Ok(AnimationUploadBuilder::new(self.client.clone(), request))
    }

    /// Starts a voice-send builder for a target chat.
    pub fn voice(&self, chat_id: impl Into<ChatId>, voice: impl Into<String>) -> VoiceSendBuilder {
        let request = voice_send_request(chat_id, voice);
        VoiceSendBuilder::new(self.client.clone(), request)
    }

    /// Starts a voice-upload builder for a target chat.
    pub fn voice_upload(&self, chat_id: impl Into<ChatId>) -> VoiceUploadBuilder {
        let request = voice_upload_request(chat_id);
        VoiceUploadBuilder::new(self.client.clone(), request)
    }

    /// Starts a voice-send builder using the update reply target and quoting its source message when present.
    pub fn reply_voice(
        &self,
        update: &Update,
        voice: impl Into<String>,
    ) -> Result<VoiceSendBuilder> {
        let request = reply_voice_request(update, voice)?;
        Ok(VoiceSendBuilder::new(self.client.clone(), request))
    }

    /// Starts a voice-upload builder using the update reply target and quoting its source message when present.
    pub fn reply_voice_upload(&self, update: &Update) -> Result<VoiceUploadBuilder> {
        let request = reply_voice_upload_request(update)?;
        Ok(VoiceUploadBuilder::new(self.client.clone(), request))
    }

    /// Starts a video-note-send builder for a target chat.
    pub fn video_note(
        &self,
        chat_id: impl Into<ChatId>,
        video_note: impl Into<String>,
    ) -> VideoNoteSendBuilder {
        let request = video_note_send_request(chat_id, video_note);
        VideoNoteSendBuilder::new(self.client.clone(), request)
    }

    /// Starts a video-note-upload builder for a target chat.
    pub fn video_note_upload(&self, chat_id: impl Into<ChatId>) -> VideoNoteUploadBuilder {
        let request = video_note_upload_request(chat_id);
        VideoNoteUploadBuilder::new(self.client.clone(), request)
    }

    /// Starts a video-note-send builder using the update reply target and quoting its source message when present.
    pub fn reply_video_note(
        &self,
        update: &Update,
        video_note: impl Into<String>,
    ) -> Result<VideoNoteSendBuilder> {
        let request = reply_video_note_request(update, video_note)?;
        Ok(VideoNoteSendBuilder::new(self.client.clone(), request))
    }

    /// Starts a video-note-upload builder using the update reply target and quoting its source message when present.
    pub fn reply_video_note_upload(&self, update: &Update) -> Result<VideoNoteUploadBuilder> {
        let request = reply_video_note_upload_request(update)?;
        Ok(VideoNoteUploadBuilder::new(self.client.clone(), request))
    }

    /// Starts a sticker-send builder for a target chat.
    pub fn sticker(
        &self,
        chat_id: impl Into<ChatId>,
        sticker: impl Into<String>,
    ) -> StickerSendBuilder {
        let request = sticker_send_request(chat_id, sticker);
        StickerSendBuilder::new(self.client.clone(), request)
    }

    /// Starts a sticker-upload builder for a target chat.
    pub fn sticker_upload(&self, chat_id: impl Into<ChatId>) -> StickerUploadBuilder {
        let request = sticker_upload_request(chat_id);
        StickerUploadBuilder::new(self.client.clone(), request)
    }

    /// Starts a sticker-send builder using the update reply target and quoting its source message when present.
    pub fn reply_sticker(
        &self,
        update: &Update,
        sticker: impl Into<String>,
    ) -> Result<StickerSendBuilder> {
        let request = reply_sticker_request(update, sticker)?;
        Ok(StickerSendBuilder::new(self.client.clone(), request))
    }

    /// Starts a sticker-upload builder using the update reply target and quoting its source message when present.
    pub fn reply_sticker_upload(&self, update: &Update) -> Result<StickerUploadBuilder> {
        let request = reply_sticker_upload_request(update)?;
        Ok(StickerUploadBuilder::new(self.client.clone(), request))
    }

    /// Starts a media-group builder for a target chat.
    ///
    /// `media` must contain 2-10 photo/video/audio/document items.
    pub fn media_group<I, M>(
        &self,
        chat_id: impl Into<ChatId>,
        media: I,
    ) -> Result<MediaGroupSendBuilder>
    where
        I: IntoIterator<Item = M>,
        M: Into<InputMediaGroupItem>,
    {
        let request = media_group_send_request(chat_id, media)?;
        Ok(MediaGroupSendBuilder::new(self.client.clone(), request))
    }

    /// Starts a media-group upload builder for a target chat.
    ///
    /// `media` must contain `attach://...` media or thumbnail references matching uploaded parts.
    pub fn media_group_upload<I, M>(
        &self,
        chat_id: impl Into<ChatId>,
        media: I,
    ) -> Result<MediaGroupUploadBuilder>
    where
        I: IntoIterator<Item = M>,
        M: Into<InputMediaGroupItem>,
    {
        let request = media_group_upload_request(chat_id, media)?;
        Ok(MediaGroupUploadBuilder::new(self.client.clone(), request))
    }

    /// Starts a media-group builder using the update reply target and quoting its source message when present.
    pub fn reply_media_group<I, M>(
        &self,
        update: &Update,
        media: I,
    ) -> Result<MediaGroupSendBuilder>
    where
        I: IntoIterator<Item = M>,
        M: Into<InputMediaGroupItem>,
    {
        let request = reply_media_group_request(update, media)?;
        Ok(MediaGroupSendBuilder::new(self.client.clone(), request))
    }

    /// Starts a media-group upload builder using the update reply target and quoting its source message when present.
    ///
    /// `media` must contain `attach://...` media or thumbnail references matching uploaded parts.
    pub fn reply_media_group_upload<I, M>(
        &self,
        update: &Update,
        media: I,
    ) -> Result<MediaGroupUploadBuilder>
    where
        I: IntoIterator<Item = M>,
        M: Into<InputMediaGroupItem>,
    {
        let request = reply_media_group_upload_request(update, media)?;
        Ok(MediaGroupUploadBuilder::new(self.client.clone(), request))
    }

    /// Shortcut for `text(...).send().await`.
    pub async fn send_text(
        &self,
        chat_id: impl Into<ChatId>,
        text: impl Into<String>,
    ) -> Result<Message> {
        self.text(chat_id, text)?.send().await
    }

    /// Shortcut for `reply(...).send().await`.
    pub async fn reply_text(&self, update: &Update, text: impl Into<String>) -> Result<Message> {
        self.reply(update, text)?.send().await
    }

    /// Shortcut for `callback_answer(...).text_optional(...).send().await`.
    pub async fn answer_callback(
        &self,
        callback_query_id: impl Into<String>,
        text: Option<String>,
    ) -> Result<bool> {
        self.callback_answer(callback_query_id)
            .text_optional(text)
            .send()
            .await
    }

    /// Shortcut for `callback_answer_from_update(...).text_optional(...).send().await`.
    pub async fn answer_callback_from_update(
        &self,
        update: &Update,
        text: Option<String>,
    ) -> Result<bool> {
        self.callback_answer_from_update(update)?
            .text_optional(text)
            .send()
            .await
    }
}

/// Stable builder for high-level callback answers on the blocking app facade.
///
/// Blocking mirror of [`CallbackAnswerBuilder`].
#[cfg(feature = "_blocking")]
#[derive(Clone)]
#[must_use = "call `.send()` or `.into_request()` to finish the callback answer"]
pub struct BlockingCallbackAnswerBuilder {
    client: BlockingClient,
    request: AnswerCallbackQueryRequest,
}

#[cfg(feature = "_blocking")]
impl BlockingCallbackAnswerBuilder {
    fn new(client: BlockingClient, request: AnswerCallbackQueryRequest) -> Self {
        Self { client, request }
    }

    /// Sends the callback answer request.
    pub fn send(self) -> Result<bool> {
        self.client.updates().answer_callback_query(&self.request)
    }
}

#[cfg(feature = "_blocking")]
impl_common_callback_answer_builder_methods!(
    BlockingCallbackAnswerBuilder,
    AnswerCallbackQueryRequest
);

/// Stable builder for high-level text sends on the blocking app facade.
///
/// Blocking mirror of [`TextSendBuilder`].
#[cfg(feature = "_blocking")]
#[derive(Clone)]
#[must_use = "call `.send()` or `.into_request()` to finish the message send"]
pub struct BlockingTextSendBuilder {
    client: BlockingClient,
    request: SendMessageRequest,
}

#[cfg(feature = "_blocking")]
impl BlockingTextSendBuilder {
    fn new(client: BlockingClient, request: SendMessageRequest) -> Self {
        Self { client, request }
    }

    /// Sets text parse mode.
    pub fn parse_mode(mut self, parse_mode: ParseMode) -> Self {
        self.request = self.request.parse_mode(parse_mode);
        self
    }

    /// Sets explicit text entities instead of a parse mode.
    pub fn entities(mut self, entities: Vec<MessageEntity>) -> Self {
        self.request = self.request.entities(entities);
        self
    }

    /// Sets Telegram link preview behavior explicitly.
    pub fn link_preview_options(mut self, link_preview_options: LinkPreviewOptions) -> Self {
        self.request = self.request.link_preview_options(link_preview_options);
        self
    }

    /// Disables link previews for the text message.
    pub fn disable_link_preview(mut self) -> Self {
        self.request = self.request.disable_link_preview();
        self
    }

    /// Sends the message.
    pub fn send(self) -> Result<Message> {
        self.client.messages().send_message(&self.request)
    }
}

#[cfg(feature = "_blocking")]
impl_common_send_option_builder_methods!(BlockingTextSendBuilder, SendMessageRequest);

/// Stable builder for high-level location sends on the blocking app facade.
///
/// Blocking mirror of [`LocationSendBuilder`].
#[cfg(feature = "_blocking")]
#[derive(Clone)]
#[must_use = "call `.send()` or `.into_request()` to finish the send"]
pub struct BlockingLocationSendBuilder {
    client: BlockingClient,
    request: SendLocationRequest,
}

#[cfg(feature = "_blocking")]
impl BlockingLocationSendBuilder {
    fn new(client: BlockingClient, request: SendLocationRequest) -> Self {
        Self { client, request }
    }

    /// Sets horizontal location accuracy in meters.
    pub fn horizontal_accuracy(mut self, horizontal_accuracy: f64) -> Self {
        self.request.horizontal_accuracy = Some(horizontal_accuracy);
        self
    }

    /// Sets live location update period in seconds.
    pub fn live_period(mut self, live_period: u32) -> Self {
        self.request.live_period = Some(live_period);
        self
    }

    /// Sets movement direction in degrees.
    pub fn heading(mut self, heading: u16) -> Self {
        self.request.heading = Some(heading);
        self
    }

    /// Sets proximity alert radius in meters.
    pub fn proximity_alert_radius(mut self, proximity_alert_radius: u32) -> Self {
        self.request.proximity_alert_radius = Some(proximity_alert_radius);
        self
    }

    /// Sends the location.
    pub fn send(self) -> Result<Message> {
        self.client.messages().send_location(&self.request)
    }
}

#[cfg(feature = "_blocking")]
impl_common_send_option_builder_methods!(BlockingLocationSendBuilder, SendLocationRequest);

/// Stable builder for high-level venue sends on the blocking app facade.
///
/// Blocking mirror of [`VenueSendBuilder`].
#[cfg(feature = "_blocking")]
#[derive(Clone)]
#[must_use = "call `.send()` or `.into_request()` to finish the send"]
pub struct BlockingVenueSendBuilder {
    client: BlockingClient,
    request: SendVenueRequest,
}

#[cfg(feature = "_blocking")]
impl BlockingVenueSendBuilder {
    fn new(client: BlockingClient, request: SendVenueRequest) -> Self {
        Self { client, request }
    }

    /// Sets a Foursquare venue id.
    pub fn foursquare_id(mut self, foursquare_id: impl Into<String>) -> Self {
        self.request.foursquare_id = Some(foursquare_id.into());
        self
    }

    /// Sets a Foursquare venue type.
    pub fn foursquare_type(mut self, foursquare_type: impl Into<String>) -> Self {
        self.request.foursquare_type = Some(foursquare_type.into());
        self
    }

    /// Sets a Google Places id.
    pub fn google_place_id(mut self, google_place_id: impl Into<String>) -> Self {
        self.request.google_place_id = Some(google_place_id.into());
        self
    }

    /// Sets a Google Places type.
    pub fn google_place_type(mut self, google_place_type: impl Into<String>) -> Self {
        self.request.google_place_type = Some(google_place_type.into());
        self
    }

    /// Sends the venue.
    pub fn send(self) -> Result<Message> {
        self.client.messages().send_venue(&self.request)
    }
}

#[cfg(feature = "_blocking")]
impl_common_send_option_builder_methods!(BlockingVenueSendBuilder, SendVenueRequest);

/// Stable builder for high-level contact sends on the blocking app facade.
///
/// Blocking mirror of [`ContactSendBuilder`].
#[cfg(feature = "_blocking")]
#[derive(Clone)]
#[must_use = "call `.send()` or `.into_request()` to finish the send"]
pub struct BlockingContactSendBuilder {
    client: BlockingClient,
    request: SendContactRequest,
}

#[cfg(feature = "_blocking")]
impl BlockingContactSendBuilder {
    fn new(client: BlockingClient, request: SendContactRequest) -> Self {
        Self { client, request }
    }

    /// Sets the contact last name.
    pub fn last_name(mut self, last_name: impl Into<String>) -> Self {
        self.request.last_name = Some(last_name.into());
        self
    }

    /// Sets the contact vCard payload.
    pub fn vcard(mut self, vcard: impl Into<String>) -> Self {
        self.request.vcard = Some(vcard.into());
        self
    }

    /// Sends the contact.
    pub fn send(self) -> Result<Message> {
        self.client.messages().send_contact(&self.request)
    }
}

#[cfg(feature = "_blocking")]
impl_common_send_option_builder_methods!(BlockingContactSendBuilder, SendContactRequest);

/// Stable builder for high-level poll sends on the blocking app facade.
///
/// Blocking mirror of [`PollSendBuilder`].
#[cfg(feature = "_blocking")]
#[derive(Clone)]
#[must_use = "call `.send()` or `.into_request()` to finish the send"]
pub struct BlockingPollSendBuilder {
    client: BlockingClient,
    request: SendPollRequest,
}

#[cfg(feature = "_blocking")]
impl BlockingPollSendBuilder {
    fn new(client: BlockingClient, request: SendPollRequest) -> Self {
        Self { client, request }
    }

    /// Sets question parse mode.
    pub fn question_parse_mode(mut self, parse_mode: ParseMode) -> Self {
        self.request.question_parse_mode = Some(parse_mode);
        self
    }

    /// Sets explicit question entities.
    pub fn question_entities(mut self, entities: Vec<MessageEntity>) -> Self {
        self.request.question_entities = Some(entities);
        self
    }

    /// Sets whether the poll is anonymous.
    pub fn anonymous(mut self, enabled: bool) -> Self {
        self.request.is_anonymous = Some(enabled);
        self
    }

    /// Sets the poll type.
    pub fn kind(mut self, kind: PollKind) -> Self {
        self.request.kind = Some(kind);
        self
    }

    /// Allows selecting multiple answers when `true`.
    pub fn allows_multiple_answers(mut self, enabled: bool) -> Self {
        self.request.allows_multiple_answers = Some(enabled);
        self
    }

    /// Allows voters to change their choice while the poll is open.
    pub fn allows_revoting(mut self, enabled: bool) -> Self {
        self.request.allows_revoting = Some(enabled);
        self
    }

    /// Randomizes the answer order for each voter when `true`.
    pub fn shuffle_options(mut self, enabled: bool) -> Self {
        self.request.shuffle_options = Some(enabled);
        self
    }

    /// Lets users add extra options to a non-anonymous regular poll.
    pub fn allow_adding_options(mut self, enabled: bool) -> Self {
        self.request.allow_adding_options = Some(enabled);
        self
    }

    /// Hides poll results until the poll closes.
    pub fn hide_results_until_closes(mut self, enabled: bool) -> Self {
        self.request.hide_results_until_closes = Some(enabled);
        self
    }

    /// Restricts voting to chat members when Telegram supports it for the target chat.
    pub fn members_only(mut self, enabled: bool) -> Self {
        self.request.members_only = Some(enabled);
        self
    }

    /// Restricts search to the provided country codes for location-based polls.
    pub fn country_codes(mut self, country_codes: Vec<String>) -> Self {
        self.request.country_codes = Some(country_codes);
        self
    }

    /// Sets correct option ids for quiz polls.
    pub fn correct_option_ids(mut self, correct_option_ids: Vec<u8>) -> Self {
        self.request.correct_option_ids = Some(correct_option_ids);
        self
    }

    /// Sets quiz explanation text.
    pub fn explanation(mut self, explanation: impl Into<String>) -> Self {
        self.request.explanation = Some(explanation.into());
        self
    }

    /// Sets quiz explanation parse mode.
    pub fn explanation_parse_mode(mut self, parse_mode: ParseMode) -> Self {
        self.request.explanation_parse_mode = Some(parse_mode);
        self
    }

    /// Sets explicit quiz explanation entities.
    pub fn explanation_entities(mut self, entities: Vec<MessageEntity>) -> Self {
        self.request.explanation_entities = Some(entities);
        self
    }

    /// Adds media to the quiz explanation.
    pub fn explanation_media(mut self, media: impl Into<InputPollMedia>) -> Self {
        self.request.explanation_media = Some(media.into());
        self
    }

    /// Sets how long the poll remains open, in seconds.
    pub fn open_period(mut self, open_period: u32) -> Self {
        self.request.open_period = Some(open_period);
        self
    }

    /// Sets the poll close date as a Unix timestamp.
    pub fn close_date(mut self, close_date: i64) -> Self {
        self.request.close_date = Some(close_date);
        self
    }

    /// Sets poll description text.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.request.description = Some(description.into());
        self
    }

    /// Sets poll description parse mode.
    pub fn description_parse_mode(mut self, parse_mode: ParseMode) -> Self {
        self.request.description_parse_mode = Some(parse_mode);
        self
    }

    /// Sets explicit poll description entities.
    pub fn description_entities(mut self, entities: Vec<MessageEntity>) -> Self {
        self.request.description_entities = Some(entities);
        self
    }

    /// Adds media to the poll description.
    pub fn media(mut self, media: impl Into<InputPollMedia>) -> Self {
        self.request.media = Some(media.into());
        self
    }

    /// Closes the poll immediately when `true`.
    pub fn closed(mut self, enabled: bool) -> Self {
        self.request.is_closed = Some(enabled);
        self
    }

    /// Sends the poll.
    pub fn send(self) -> Result<Message> {
        self.client.messages().send_poll(&self.request)
    }
}

#[cfg(feature = "_blocking")]
impl_common_poll_builder_methods!(BlockingPollSendBuilder, SendPollRequest);

/// Stable builder for high-level stop-poll calls on the blocking app facade.
///
/// Blocking mirror of [`StopPollBuilder`].
#[cfg(feature = "_blocking")]
#[derive(Clone)]
#[must_use = "call `.send()` or `.into_request()` to finish the stop-poll call"]
pub struct BlockingStopPollBuilder {
    client: BlockingClient,
    request: StopPollRequest,
}

#[cfg(feature = "_blocking")]
impl BlockingStopPollBuilder {
    fn new(client: BlockingClient, request: StopPollRequest) -> Self {
        Self { client, request }
    }

    /// Stops the poll.
    pub fn send(self) -> Result<Poll> {
        self.client.messages().stop_poll(&self.request)
    }
}

#[cfg(feature = "_blocking")]
impl_stop_poll_builder_methods!(BlockingStopPollBuilder, StopPollRequest);

/// Stable builder for high-level dice sends on the blocking app facade.
///
/// Blocking mirror of [`DiceSendBuilder`].
#[cfg(feature = "_blocking")]
#[derive(Clone)]
#[must_use = "call `.send()` or `.into_request()` to finish the send"]
pub struct BlockingDiceSendBuilder {
    client: BlockingClient,
    request: SendDiceRequest,
}

#[cfg(feature = "_blocking")]
impl BlockingDiceSendBuilder {
    fn new(client: BlockingClient, request: SendDiceRequest) -> Self {
        Self { client, request }
    }

    /// Sets the dice animation emoji.
    pub fn emoji(mut self, emoji: DiceEmoji) -> Self {
        self.request.emoji = Some(emoji);
        self
    }

    /// Sends the dice message.
    pub fn send(self) -> Result<Message> {
        self.client.messages().send_dice(&self.request)
    }
}

#[cfg(feature = "_blocking")]
impl_common_send_option_builder_methods!(BlockingDiceSendBuilder, SendDiceRequest);

/// Stable builder for high-level chat-action calls on the blocking app facade.
///
/// Blocking mirror of [`ChatActionBuilder`].
#[cfg(feature = "_blocking")]
#[derive(Clone)]
#[must_use = "call `.send()` or `.into_request()` to finish the chat action"]
pub struct BlockingChatActionBuilder {
    client: BlockingClient,
    request: SendChatActionRequest,
}

#[cfg(feature = "_blocking")]
impl BlockingChatActionBuilder {
    fn new(client: BlockingClient, request: SendChatActionRequest) -> Self {
        Self { client, request }
    }

    /// Sends the chat action.
    pub fn send(self) -> Result<bool> {
        self.client.messages().send_chat_action(&self.request)
    }
}

#[cfg(feature = "_blocking")]
impl_chat_action_builder_methods!(BlockingChatActionBuilder, SendChatActionRequest);

/// Stable builder for high-level photo sends on the blocking app facade.
///
/// Blocking mirror of [`PhotoSendBuilder`].
#[cfg(feature = "_blocking")]
#[derive(Clone)]
#[must_use = "call `.send()` or `.into_request()` to finish the send"]
pub struct BlockingPhotoSendBuilder {
    client: BlockingClient,
    request: SendPhotoRequest,
}

#[cfg(feature = "_blocking")]
impl BlockingPhotoSendBuilder {
    fn new(client: BlockingClient, request: SendPhotoRequest) -> Self {
        Self { client, request }
    }

    /// Marks the photo as spoiler media when `true`.
    pub fn has_spoiler(mut self, enabled: bool) -> Self {
        self.request.has_spoiler = enabled.then_some(true);
        self
    }

    /// Sends the photo using a Telegram file id or URL.
    pub fn send(self) -> Result<Message> {
        self.client.messages().send_photo(&self.request)
    }
}

#[cfg(feature = "_blocking")]
impl_common_media_builder_methods!(BlockingPhotoSendBuilder, SendPhotoRequest);

/// Stable builder for high-level photo uploads on the blocking app facade.
///
/// Blocking mirror of [`PhotoUploadBuilder`].
#[cfg(feature = "_blocking")]
#[derive(Clone)]
#[must_use = "call `.send(&file)` or `.into_request()` to finish the upload"]
pub struct BlockingPhotoUploadBuilder {
    client: BlockingClient,
    request: SendPhotoRequest,
}

#[cfg(feature = "_blocking")]
impl BlockingPhotoUploadBuilder {
    fn new(client: BlockingClient, request: SendPhotoRequest) -> Self {
        Self { client, request }
    }

    /// Marks the photo as spoiler media when `true`.
    pub fn has_spoiler(mut self, enabled: bool) -> Self {
        self.request.has_spoiler = enabled.then_some(true);
        self
    }

    /// Uploads local bytes as the photo payload.
    pub fn send(self, file: &UploadFile) -> Result<Message> {
        self.client
            .messages()
            .send_photo_upload(&self.request, file)
    }
}

#[cfg(feature = "_blocking")]
impl_common_media_builder_methods!(BlockingPhotoUploadBuilder, SendPhotoRequest);

/// Stable builder for high-level document sends on the blocking app facade.
///
/// Blocking mirror of [`DocumentSendBuilder`].
#[cfg(feature = "_blocking")]
#[derive(Clone)]
#[must_use = "call `.send()` or `.into_request()` to finish the send"]
pub struct BlockingDocumentSendBuilder {
    client: BlockingClient,
    request: SendDocumentRequest,
}

#[cfg(feature = "_blocking")]
impl BlockingDocumentSendBuilder {
    fn new(client: BlockingClient, request: SendDocumentRequest) -> Self {
        Self { client, request }
    }

    /// Sets a document thumbnail by file id / URL / attach reference.
    pub fn thumbnail(mut self, thumbnail: impl Into<String>) -> Self {
        self.request.thumbnail = Some(thumbnail.into());
        self
    }

    /// Disables Telegram content-type detection when `true`.
    pub fn disable_content_type_detection(mut self, enabled: bool) -> Self {
        self.request.disable_content_type_detection = enabled.then_some(true);
        self
    }

    /// Sends the document using a Telegram file id or URL.
    pub fn send(self) -> Result<Message> {
        self.client.messages().send_document(&self.request)
    }
}

#[cfg(feature = "_blocking")]
impl_common_media_builder_methods!(BlockingDocumentSendBuilder, SendDocumentRequest);

/// Stable builder for high-level document uploads on the blocking app facade.
///
/// Blocking mirror of [`DocumentUploadBuilder`].
#[cfg(feature = "_blocking")]
#[derive(Clone)]
#[must_use = "call `.send(&file)`, `.send_parts(...)`, or `.into_request()` to finish the upload"]
pub struct BlockingDocumentUploadBuilder {
    client: BlockingClient,
    request: SendDocumentRequest,
}

#[cfg(feature = "_blocking")]
impl BlockingDocumentUploadBuilder {
    fn new(client: BlockingClient, request: SendDocumentRequest) -> Self {
        Self { client, request }
    }

    /// Sets a document thumbnail by file id / URL / attach reference.
    pub fn thumbnail(mut self, thumbnail: impl Into<String>) -> Self {
        self.request.thumbnail = Some(thumbnail.into());
        self
    }

    /// Disables Telegram content-type detection when `true`.
    pub fn disable_content_type_detection(mut self, enabled: bool) -> Self {
        self.request.disable_content_type_detection = enabled.then_some(true);
        self
    }

    /// Uploads local bytes as the document payload.
    pub fn send(self, file: &UploadFile) -> Result<Message> {
        self.client
            .messages()
            .send_document_upload(&self.request, file)
    }

    /// Uploads local bytes as the document payload plus extra `attach://` parts.
    pub fn send_parts(self, file: &UploadFile, extra_files: &[UploadPart]) -> Result<Message> {
        self.client
            .messages()
            .send_document_upload_parts(&self.request, file, extra_files)
    }
}

#[cfg(feature = "_blocking")]
impl_common_media_builder_methods!(BlockingDocumentUploadBuilder, SendDocumentRequest);

/// Stable builder for high-level video sends on the blocking app facade.
///
/// Blocking mirror of [`VideoSendBuilder`].
#[cfg(feature = "_blocking")]
#[derive(Clone)]
#[must_use = "call `.send()` or `.into_request()` to finish the send"]
pub struct BlockingVideoSendBuilder {
    client: BlockingClient,
    request: SendVideoRequest,
}

#[cfg(feature = "_blocking")]
impl BlockingVideoSendBuilder {
    fn new(client: BlockingClient, request: SendVideoRequest) -> Self {
        Self { client, request }
    }

    /// Sets video duration in seconds.
    pub fn duration(mut self, duration: u32) -> Self {
        self.request.duration = Some(duration);
        self
    }

    /// Sets video width in pixels.
    pub fn width(mut self, width: u32) -> Self {
        self.request.width = Some(width);
        self
    }

    /// Sets video height in pixels.
    pub fn height(mut self, height: u32) -> Self {
        self.request.height = Some(height);
        self
    }

    /// Sets a video thumbnail by file id / URL / attach reference.
    pub fn thumbnail(mut self, thumbnail: impl Into<String>) -> Self {
        self.request.thumbnail = Some(thumbnail.into());
        self
    }

    /// Marks the video as streamable when `true`.
    pub fn supports_streaming(mut self, enabled: bool) -> Self {
        self.request.supports_streaming = enabled.then_some(true);
        self
    }

    /// Marks the video as spoiler media when `true`.
    pub fn has_spoiler(mut self, enabled: bool) -> Self {
        self.request.has_spoiler = enabled.then_some(true);
        self
    }

    /// Sends the video using a Telegram file id or URL.
    pub fn send(self) -> Result<Message> {
        self.client.messages().send_video(&self.request)
    }
}

#[cfg(feature = "_blocking")]
impl_common_media_builder_methods!(BlockingVideoSendBuilder, SendVideoRequest);

/// Stable builder for high-level video uploads on the blocking app facade.
///
/// Blocking mirror of [`VideoUploadBuilder`].
#[cfg(feature = "_blocking")]
#[derive(Clone)]
#[must_use = "call `.send(&file)`, `.send_parts(...)`, or `.into_request()` to finish the upload"]
pub struct BlockingVideoUploadBuilder {
    client: BlockingClient,
    request: SendVideoRequest,
}

#[cfg(feature = "_blocking")]
impl BlockingVideoUploadBuilder {
    fn new(client: BlockingClient, request: SendVideoRequest) -> Self {
        Self { client, request }
    }

    /// Sets video duration in seconds.
    pub fn duration(mut self, duration: u32) -> Self {
        self.request.duration = Some(duration);
        self
    }

    /// Sets video width in pixels.
    pub fn width(mut self, width: u32) -> Self {
        self.request.width = Some(width);
        self
    }

    /// Sets video height in pixels.
    pub fn height(mut self, height: u32) -> Self {
        self.request.height = Some(height);
        self
    }

    /// Sets a video thumbnail by file id / URL / attach reference.
    pub fn thumbnail(mut self, thumbnail: impl Into<String>) -> Self {
        self.request.thumbnail = Some(thumbnail.into());
        self
    }

    /// Marks the video as streamable when `true`.
    pub fn supports_streaming(mut self, enabled: bool) -> Self {
        self.request.supports_streaming = enabled.then_some(true);
        self
    }

    /// Marks the video as spoiler media when `true`.
    pub fn has_spoiler(mut self, enabled: bool) -> Self {
        self.request.has_spoiler = enabled.then_some(true);
        self
    }

    /// Uploads local bytes as the video payload.
    pub fn send(self, file: &UploadFile) -> Result<Message> {
        self.client
            .messages()
            .send_video_upload(&self.request, file)
    }

    /// Uploads local bytes as the video payload plus extra `attach://` parts.
    pub fn send_parts(self, file: &UploadFile, extra_files: &[UploadPart]) -> Result<Message> {
        self.client
            .messages()
            .send_video_upload_parts(&self.request, file, extra_files)
    }
}

#[cfg(feature = "_blocking")]
impl_common_media_builder_methods!(BlockingVideoUploadBuilder, SendVideoRequest);

/// Stable builder for high-level audio sends on the blocking app facade.
///
/// Blocking mirror of [`AudioSendBuilder`].
#[cfg(feature = "_blocking")]
#[derive(Clone)]
#[must_use = "call `.send()` or `.into_request()` to finish the send"]
pub struct BlockingAudioSendBuilder {
    client: BlockingClient,
    request: SendAudioRequest,
}

#[cfg(feature = "_blocking")]
impl BlockingAudioSendBuilder {
    fn new(client: BlockingClient, request: SendAudioRequest) -> Self {
        Self { client, request }
    }

    /// Sets audio duration in seconds.
    pub fn duration(mut self, duration: u32) -> Self {
        self.request.duration = Some(duration);
        self
    }

    /// Sets the displayed performer.
    pub fn performer(mut self, performer: impl Into<String>) -> Self {
        self.request.performer = Some(performer.into());
        self
    }

    /// Sets the displayed title.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.request.title = Some(title.into());
        self
    }

    /// Sets an audio thumbnail by file id / URL / attach reference.
    pub fn thumbnail(mut self, thumbnail: impl Into<String>) -> Self {
        self.request.thumbnail = Some(thumbnail.into());
        self
    }

    /// Sends the audio using a Telegram file id or URL.
    pub fn send(self) -> Result<Message> {
        self.client.messages().send_audio(&self.request)
    }
}

#[cfg(feature = "_blocking")]
impl_common_media_builder_methods!(BlockingAudioSendBuilder, SendAudioRequest);

/// Stable builder for high-level audio uploads on the blocking app facade.
///
/// Blocking mirror of [`AudioUploadBuilder`].
#[cfg(feature = "_blocking")]
#[derive(Clone)]
#[must_use = "call `.send(&file)`, `.send_parts(...)`, or `.into_request()` to finish the upload"]
pub struct BlockingAudioUploadBuilder {
    client: BlockingClient,
    request: SendAudioRequest,
}

#[cfg(feature = "_blocking")]
impl BlockingAudioUploadBuilder {
    fn new(client: BlockingClient, request: SendAudioRequest) -> Self {
        Self { client, request }
    }

    /// Sets audio duration in seconds.
    pub fn duration(mut self, duration: u32) -> Self {
        self.request.duration = Some(duration);
        self
    }

    /// Sets the displayed performer.
    pub fn performer(mut self, performer: impl Into<String>) -> Self {
        self.request.performer = Some(performer.into());
        self
    }

    /// Sets the displayed title.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.request.title = Some(title.into());
        self
    }

    /// Sets an audio thumbnail by file id / URL / attach reference.
    pub fn thumbnail(mut self, thumbnail: impl Into<String>) -> Self {
        self.request.thumbnail = Some(thumbnail.into());
        self
    }

    /// Uploads local bytes as the audio payload.
    pub fn send(self, file: &UploadFile) -> Result<Message> {
        self.client
            .messages()
            .send_audio_upload(&self.request, file)
    }

    /// Uploads local bytes as the audio payload plus extra `attach://` parts.
    pub fn send_parts(self, file: &UploadFile, extra_files: &[UploadPart]) -> Result<Message> {
        self.client
            .messages()
            .send_audio_upload_parts(&self.request, file, extra_files)
    }
}

#[cfg(feature = "_blocking")]
impl_common_media_builder_methods!(BlockingAudioUploadBuilder, SendAudioRequest);

/// Stable builder for high-level animation sends on the blocking app facade.
///
/// Blocking mirror of [`AnimationSendBuilder`].
#[cfg(feature = "_blocking")]
#[derive(Clone)]
#[must_use = "call `.send()` or `.into_request()` to finish the send"]
pub struct BlockingAnimationSendBuilder {
    client: BlockingClient,
    request: SendAnimationRequest,
}

#[cfg(feature = "_blocking")]
impl BlockingAnimationSendBuilder {
    fn new(client: BlockingClient, request: SendAnimationRequest) -> Self {
        Self { client, request }
    }

    /// Sets animation duration in seconds.
    pub fn duration(mut self, duration: u32) -> Self {
        self.request.duration = Some(duration);
        self
    }

    /// Sets animation width in pixels.
    pub fn width(mut self, width: u32) -> Self {
        self.request.width = Some(width);
        self
    }

    /// Sets animation height in pixels.
    pub fn height(mut self, height: u32) -> Self {
        self.request.height = Some(height);
        self
    }

    /// Sets an animation thumbnail by file id / URL / attach reference.
    pub fn thumbnail(mut self, thumbnail: impl Into<String>) -> Self {
        self.request.thumbnail = Some(thumbnail.into());
        self
    }

    /// Marks the animation as spoiler media when `true`.
    pub fn has_spoiler(mut self, enabled: bool) -> Self {
        self.request.has_spoiler = enabled.then_some(true);
        self
    }

    /// Sends the animation using a Telegram file id or URL.
    pub fn send(self) -> Result<Message> {
        self.client.messages().send_animation(&self.request)
    }
}

#[cfg(feature = "_blocking")]
impl_common_media_builder_methods!(BlockingAnimationSendBuilder, SendAnimationRequest);

/// Stable builder for high-level animation uploads on the blocking app facade.
///
/// Blocking mirror of [`AnimationUploadBuilder`].
#[cfg(feature = "_blocking")]
#[derive(Clone)]
#[must_use = "call `.send(&file)`, `.send_parts(...)`, or `.into_request()` to finish the upload"]
pub struct BlockingAnimationUploadBuilder {
    client: BlockingClient,
    request: SendAnimationRequest,
}

#[cfg(feature = "_blocking")]
impl BlockingAnimationUploadBuilder {
    fn new(client: BlockingClient, request: SendAnimationRequest) -> Self {
        Self { client, request }
    }

    /// Sets animation duration in seconds.
    pub fn duration(mut self, duration: u32) -> Self {
        self.request.duration = Some(duration);
        self
    }

    /// Sets animation width in pixels.
    pub fn width(mut self, width: u32) -> Self {
        self.request.width = Some(width);
        self
    }

    /// Sets animation height in pixels.
    pub fn height(mut self, height: u32) -> Self {
        self.request.height = Some(height);
        self
    }

    /// Sets an animation thumbnail by file id / URL / attach reference.
    pub fn thumbnail(mut self, thumbnail: impl Into<String>) -> Self {
        self.request.thumbnail = Some(thumbnail.into());
        self
    }

    /// Marks the animation as spoiler media when `true`.
    pub fn has_spoiler(mut self, enabled: bool) -> Self {
        self.request.has_spoiler = enabled.then_some(true);
        self
    }

    /// Uploads local bytes as the animation payload.
    pub fn send(self, file: &UploadFile) -> Result<Message> {
        self.client
            .messages()
            .send_animation_upload(&self.request, file)
    }

    /// Uploads local bytes as the animation payload plus extra `attach://` parts.
    pub fn send_parts(self, file: &UploadFile, extra_files: &[UploadPart]) -> Result<Message> {
        self.client
            .messages()
            .send_animation_upload_parts(&self.request, file, extra_files)
    }
}

#[cfg(feature = "_blocking")]
impl_common_media_builder_methods!(BlockingAnimationUploadBuilder, SendAnimationRequest);

/// Stable builder for high-level voice sends on the blocking app facade.
///
/// Blocking mirror of [`VoiceSendBuilder`].
#[cfg(feature = "_blocking")]
#[derive(Clone)]
#[must_use = "call `.send()` or `.into_request()` to finish the send"]
pub struct BlockingVoiceSendBuilder {
    client: BlockingClient,
    request: SendVoiceRequest,
}

#[cfg(feature = "_blocking")]
impl BlockingVoiceSendBuilder {
    fn new(client: BlockingClient, request: SendVoiceRequest) -> Self {
        Self { client, request }
    }

    /// Sets voice duration in seconds.
    pub fn duration(mut self, duration: u32) -> Self {
        self.request.duration = Some(duration);
        self
    }

    /// Sends the voice message using a Telegram file id or URL.
    pub fn send(self) -> Result<Message> {
        self.client.messages().send_voice(&self.request)
    }
}

#[cfg(feature = "_blocking")]
impl_common_media_builder_methods!(BlockingVoiceSendBuilder, SendVoiceRequest);

/// Stable builder for high-level voice uploads on the blocking app facade.
///
/// Blocking mirror of [`VoiceUploadBuilder`].
#[cfg(feature = "_blocking")]
#[derive(Clone)]
#[must_use = "call `.send(&file)` or `.into_request()` to finish the upload"]
pub struct BlockingVoiceUploadBuilder {
    client: BlockingClient,
    request: SendVoiceRequest,
}

#[cfg(feature = "_blocking")]
impl BlockingVoiceUploadBuilder {
    fn new(client: BlockingClient, request: SendVoiceRequest) -> Self {
        Self { client, request }
    }

    /// Sets voice duration in seconds.
    pub fn duration(mut self, duration: u32) -> Self {
        self.request.duration = Some(duration);
        self
    }

    /// Uploads local bytes as the voice payload.
    pub fn send(self, file: &UploadFile) -> Result<Message> {
        self.client
            .messages()
            .send_voice_upload(&self.request, file)
    }
}

#[cfg(feature = "_blocking")]
impl_common_media_builder_methods!(BlockingVoiceUploadBuilder, SendVoiceRequest);

/// Stable builder for high-level video note sends on the blocking app facade.
///
/// Blocking mirror of [`VideoNoteSendBuilder`].
#[cfg(feature = "_blocking")]
#[derive(Clone)]
#[must_use = "call `.send()` or `.into_request()` to finish the send"]
pub struct BlockingVideoNoteSendBuilder {
    client: BlockingClient,
    request: SendVideoNoteRequest,
}

#[cfg(feature = "_blocking")]
impl BlockingVideoNoteSendBuilder {
    fn new(client: BlockingClient, request: SendVideoNoteRequest) -> Self {
        Self { client, request }
    }

    /// Sets video note duration in seconds.
    pub fn duration(mut self, duration: u32) -> Self {
        self.request.duration = Some(duration);
        self
    }

    /// Sets video note diameter in pixels.
    pub fn length(mut self, length: u32) -> Self {
        self.request.length = Some(length);
        self
    }

    /// Sets a video note thumbnail by file id or URL.
    pub fn thumbnail(mut self, thumbnail: impl Into<String>) -> Self {
        self.request.thumbnail = Some(thumbnail.into());
        self
    }

    /// Sends the video note using a Telegram file id or URL.
    pub fn send(self) -> Result<Message> {
        self.client.messages().send_video_note(&self.request)
    }
}

#[cfg(feature = "_blocking")]
impl_common_send_option_builder_methods!(BlockingVideoNoteSendBuilder, SendVideoNoteRequest);

/// Stable builder for high-level video note uploads on the blocking app facade.
///
/// Blocking mirror of [`VideoNoteUploadBuilder`].
#[cfg(feature = "_blocking")]
#[derive(Clone)]
#[must_use = "call `.send(&file)`, `.send_parts(...)`, or `.into_request()` to finish the upload"]
pub struct BlockingVideoNoteUploadBuilder {
    client: BlockingClient,
    request: SendVideoNoteRequest,
}

#[cfg(feature = "_blocking")]
impl BlockingVideoNoteUploadBuilder {
    fn new(client: BlockingClient, request: SendVideoNoteRequest) -> Self {
        Self { client, request }
    }

    /// Sets video note duration in seconds.
    pub fn duration(mut self, duration: u32) -> Self {
        self.request.duration = Some(duration);
        self
    }

    /// Sets video note diameter in pixels.
    pub fn length(mut self, length: u32) -> Self {
        self.request.length = Some(length);
        self
    }

    /// Sets a video note thumbnail by file id / URL / attach reference.
    pub fn thumbnail(mut self, thumbnail: impl Into<String>) -> Self {
        self.request.thumbnail = Some(thumbnail.into());
        self
    }

    /// Uploads local bytes as the video note payload.
    pub fn send(self, file: &UploadFile) -> Result<Message> {
        self.client
            .messages()
            .send_video_note_upload(&self.request, file)
    }

    /// Uploads local bytes as the video note payload plus extra `attach://` parts.
    pub fn send_parts(self, file: &UploadFile, extra_files: &[UploadPart]) -> Result<Message> {
        self.client
            .messages()
            .send_video_note_upload_parts(&self.request, file, extra_files)
    }
}

#[cfg(feature = "_blocking")]
impl_common_send_option_builder_methods!(BlockingVideoNoteUploadBuilder, SendVideoNoteRequest);

/// Stable builder for high-level sticker sends on the blocking app facade.
///
/// Blocking mirror of [`StickerSendBuilder`].
#[cfg(feature = "_blocking")]
#[derive(Clone)]
#[must_use = "call `.send()` or `.into_request()` to finish the send"]
pub struct BlockingStickerSendBuilder {
    client: BlockingClient,
    request: SendStickerRequest,
}

#[cfg(feature = "_blocking")]
impl BlockingStickerSendBuilder {
    fn new(client: BlockingClient, request: SendStickerRequest) -> Self {
        Self { client, request }
    }

    /// Sends the sticker using a Telegram file id or URL.
    pub fn send(self) -> Result<Message> {
        self.client.stickers().send_sticker(&self.request)
    }
}

#[cfg(feature = "_blocking")]
impl_common_sticker_builder_methods!(BlockingStickerSendBuilder, SendStickerRequest);

/// Stable builder for high-level sticker uploads on the blocking app facade.
///
/// Blocking mirror of [`StickerUploadBuilder`].
#[cfg(feature = "_blocking")]
#[derive(Clone)]
#[must_use = "call `.send(&file)` or `.into_request()` to finish the upload"]
pub struct BlockingStickerUploadBuilder {
    client: BlockingClient,
    request: SendStickerRequest,
}

#[cfg(feature = "_blocking")]
impl BlockingStickerUploadBuilder {
    fn new(client: BlockingClient, request: SendStickerRequest) -> Self {
        Self { client, request }
    }

    /// Uploads local bytes as the sticker payload.
    pub fn send(self, file: &UploadFile) -> Result<Message> {
        self.client
            .stickers()
            .send_sticker_upload(&self.request, file)
    }
}

#[cfg(feature = "_blocking")]
impl_common_sticker_builder_methods!(BlockingStickerUploadBuilder, SendStickerRequest);

/// Stable builder for high-level media group sends on the blocking app facade.
///
/// Blocking mirror of [`MediaGroupSendBuilder`].
#[cfg(feature = "_blocking")]
#[derive(Clone)]
#[must_use = "call `.send()` or `.into_request()` to finish the send"]
pub struct BlockingMediaGroupSendBuilder {
    client: BlockingClient,
    request: SendMediaGroupRequest,
}

#[cfg(feature = "_blocking")]
impl BlockingMediaGroupSendBuilder {
    fn new(client: BlockingClient, request: SendMediaGroupRequest) -> Self {
        Self { client, request }
    }

    /// Sends the media group.
    pub fn send(self) -> Result<Vec<Message>> {
        self.client.messages().send_media_group(&self.request)
    }
}

#[cfg(feature = "_blocking")]
impl_common_media_group_builder_methods!(BlockingMediaGroupSendBuilder, SendMediaGroupRequest);

/// Stable builder for high-level media group uploads on the blocking app facade.
///
/// Blocking mirror of [`MediaGroupUploadBuilder`].
#[cfg(feature = "_blocking")]
#[derive(Clone)]
#[must_use = "call `.send(&files)` or `.into_request()` to finish the upload"]
pub struct BlockingMediaGroupUploadBuilder {
    client: BlockingClient,
    request: SendMediaGroupRequest,
}

#[cfg(feature = "_blocking")]
impl BlockingMediaGroupUploadBuilder {
    fn new(client: BlockingClient, request: SendMediaGroupRequest) -> Self {
        Self { client, request }
    }

    /// Uploads local files referenced by `attach://...` media entries.
    pub fn send(self, files: &[UploadPart]) -> Result<Vec<Message>> {
        self.client
            .messages()
            .send_media_group_upload(&self.request, files)
    }
}

#[cfg(feature = "_blocking")]
impl_common_media_group_builder_methods!(BlockingMediaGroupUploadBuilder, SendMediaGroupRequest);

/// Stable app-facing runtime facade for blocking workflows.
///
/// Blocking mirror of [`AppApi`]. Prefer this layer for runtime/business code and keep
/// `client.control()` for startup and orchestration concerns.
#[cfg(feature = "_blocking")]
#[derive(Clone)]
pub struct BlockingAppApi {
    client: BlockingClient,
}

#[cfg(feature = "_blocking")]
impl BlockingAppApi {
    pub(crate) fn new(client: BlockingClient) -> Self {
        Self { client }
    }

    /// Returns the governance-oriented moderation facade.
    pub fn moderation(&self) -> BlockingModerationApi {
        BlockingModerationApi::new(self.client.clone())
    }

    /// Returns the membership/capability facade used by install and bind pre-check flows.
    pub fn membership(&self) -> BlockingMembershipApi {
        BlockingMembershipApi::new(self.client.clone())
    }

    /// Returns the dedicated Web App runtime facade.
    pub fn web_app(&self) -> BlockingWebAppApi {
        BlockingWebAppApi::new(self.client.clone())
    }

    /// Starts a callback-answer builder.
    ///
    /// Prefer this when callback replies need options such as `show_alert`, `url`, or
    /// `cache_time`.
    pub fn callback_answer(
        &self,
        callback_query_id: impl Into<String>,
    ) -> BlockingCallbackAnswerBuilder {
        let request = callback_answer_request(callback_query_id, None);
        BlockingCallbackAnswerBuilder::new(self.client.clone(), request)
    }

    /// Starts a callback-answer builder using the callback id extracted from an update.
    pub fn callback_answer_from_update(
        &self,
        update: &Update,
    ) -> Result<BlockingCallbackAnswerBuilder> {
        let Some(callback_query_id) = callback_query_id(update) else {
            return Err(super::support::invalid_request(
                "update does not contain callback query for answerCallbackQuery",
            ));
        };
        Ok(self.callback_answer(callback_query_id))
    }

    /// Starts a text-send builder for a target chat.
    pub fn text(
        &self,
        chat_id: impl Into<ChatId>,
        text: impl Into<String>,
    ) -> Result<BlockingTextSendBuilder> {
        let request = text_send_request(chat_id, text)?;
        Ok(BlockingTextSendBuilder::new(self.client.clone(), request))
    }

    /// Starts a text-send builder using the update reply target and quoting its source message when present.
    pub fn reply(
        &self,
        update: &Update,
        text: impl Into<String>,
    ) -> Result<BlockingTextSendBuilder> {
        let request = reply_text_request(update, text)?;
        Ok(BlockingTextSendBuilder::new(self.client.clone(), request))
    }

    /// Starts a location-send builder for a target chat.
    pub fn location(
        &self,
        chat_id: impl Into<ChatId>,
        latitude: f64,
        longitude: f64,
    ) -> BlockingLocationSendBuilder {
        let request = location_send_request(chat_id, latitude, longitude);
        BlockingLocationSendBuilder::new(self.client.clone(), request)
    }

    /// Starts a location-send builder using the update reply target and quoting its source message when present.
    pub fn reply_location(
        &self,
        update: &Update,
        latitude: f64,
        longitude: f64,
    ) -> Result<BlockingLocationSendBuilder> {
        let request = reply_location_request(update, latitude, longitude)?;
        Ok(BlockingLocationSendBuilder::new(
            self.client.clone(),
            request,
        ))
    }

    /// Starts a venue-send builder for a target chat.
    pub fn venue(
        &self,
        chat_id: impl Into<ChatId>,
        latitude: f64,
        longitude: f64,
        title: impl Into<String>,
        address: impl Into<String>,
    ) -> BlockingVenueSendBuilder {
        let request = venue_send_request(chat_id, latitude, longitude, title, address);
        BlockingVenueSendBuilder::new(self.client.clone(), request)
    }

    /// Starts a venue-send builder using the update reply target and quoting its source message when present.
    pub fn reply_venue(
        &self,
        update: &Update,
        latitude: f64,
        longitude: f64,
        title: impl Into<String>,
        address: impl Into<String>,
    ) -> Result<BlockingVenueSendBuilder> {
        let request = reply_venue_request(update, latitude, longitude, title, address)?;
        Ok(BlockingVenueSendBuilder::new(self.client.clone(), request))
    }

    /// Starts a contact-send builder for a target chat.
    pub fn contact(
        &self,
        chat_id: impl Into<ChatId>,
        phone_number: impl Into<String>,
        first_name: impl Into<String>,
    ) -> BlockingContactSendBuilder {
        let request = contact_send_request(chat_id, phone_number, first_name);
        BlockingContactSendBuilder::new(self.client.clone(), request)
    }

    /// Starts a contact-send builder using the update reply target and quoting its source message when present.
    pub fn reply_contact(
        &self,
        update: &Update,
        phone_number: impl Into<String>,
        first_name: impl Into<String>,
    ) -> Result<BlockingContactSendBuilder> {
        let request = reply_contact_request(update, phone_number, first_name)?;
        Ok(BlockingContactSendBuilder::new(
            self.client.clone(),
            request,
        ))
    }

    /// Starts a poll-send builder for a target chat.
    pub fn poll(
        &self,
        chat_id: impl Into<ChatId>,
        question: impl Into<String>,
        options: impl IntoIterator<Item = impl Into<InputPollOption>>,
    ) -> Result<BlockingPollSendBuilder> {
        let request = poll_send_request(chat_id, question, options)?;
        Ok(BlockingPollSendBuilder::new(self.client.clone(), request))
    }

    /// Starts a poll-send builder using the update reply target and quoting its source message when present.
    pub fn reply_poll(
        &self,
        update: &Update,
        question: impl Into<String>,
        options: impl IntoIterator<Item = impl Into<InputPollOption>>,
    ) -> Result<BlockingPollSendBuilder> {
        let request = reply_poll_request(update, question, options)?;
        Ok(BlockingPollSendBuilder::new(self.client.clone(), request))
    }

    /// Starts a stop-poll builder for a target chat and message.
    pub fn stop_poll(
        &self,
        chat_id: impl Into<ChatId>,
        message_id: MessageId,
    ) -> BlockingStopPollBuilder {
        let request = stop_poll_request(chat_id, message_id);
        BlockingStopPollBuilder::new(self.client.clone(), request)
    }

    /// Starts a dice-send builder for a target chat.
    pub fn dice(&self, chat_id: impl Into<ChatId>) -> BlockingDiceSendBuilder {
        let request = dice_send_request(chat_id);
        BlockingDiceSendBuilder::new(self.client.clone(), request)
    }

    /// Starts a dice-send builder using the update reply target and quoting its source message when present.
    pub fn reply_dice(&self, update: &Update) -> Result<BlockingDiceSendBuilder> {
        let request = reply_dice_request(update)?;
        Ok(BlockingDiceSendBuilder::new(self.client.clone(), request))
    }

    /// Starts a chat-action builder for a target chat.
    pub fn chat_action(
        &self,
        chat_id: impl Into<ChatId>,
        action: ChatAction,
    ) -> BlockingChatActionBuilder {
        let request = chat_action_request(chat_id, action);
        BlockingChatActionBuilder::new(self.client.clone(), request)
    }

    /// Starts a chat-action builder using the update reply target.
    pub fn chat_action_for_update(
        &self,
        update: &Update,
        action: ChatAction,
    ) -> Result<BlockingChatActionBuilder> {
        let request = chat_action_for_update_request(update, action)?;
        Ok(BlockingChatActionBuilder::new(self.client.clone(), request))
    }

    /// Starts a photo-send builder for a target chat.
    pub fn photo(
        &self,
        chat_id: impl Into<ChatId>,
        photo: impl Into<String>,
    ) -> BlockingPhotoSendBuilder {
        let request = photo_send_request(chat_id, photo);
        BlockingPhotoSendBuilder::new(self.client.clone(), request)
    }

    /// Starts a photo-upload builder for a target chat.
    pub fn photo_upload(&self, chat_id: impl Into<ChatId>) -> BlockingPhotoUploadBuilder {
        let request = photo_upload_request(chat_id);
        BlockingPhotoUploadBuilder::new(self.client.clone(), request)
    }

    /// Starts a photo-send builder using the update reply target and quoting its source message when present.
    pub fn reply_photo(
        &self,
        update: &Update,
        photo: impl Into<String>,
    ) -> Result<BlockingPhotoSendBuilder> {
        let request = reply_photo_request(update, photo)?;
        Ok(BlockingPhotoSendBuilder::new(self.client.clone(), request))
    }

    /// Starts a photo-upload builder using the update reply target and quoting its source message when present.
    pub fn reply_photo_upload(&self, update: &Update) -> Result<BlockingPhotoUploadBuilder> {
        let request = reply_photo_upload_request(update)?;
        Ok(BlockingPhotoUploadBuilder::new(
            self.client.clone(),
            request,
        ))
    }

    /// Starts a document-send builder for a target chat.
    pub fn document(
        &self,
        chat_id: impl Into<ChatId>,
        document: impl Into<String>,
    ) -> BlockingDocumentSendBuilder {
        let request = document_send_request(chat_id, document);
        BlockingDocumentSendBuilder::new(self.client.clone(), request)
    }

    /// Starts a document-upload builder for a target chat.
    pub fn document_upload(&self, chat_id: impl Into<ChatId>) -> BlockingDocumentUploadBuilder {
        let request = document_upload_request(chat_id);
        BlockingDocumentUploadBuilder::new(self.client.clone(), request)
    }

    /// Starts a document-send builder using the update reply target and quoting its source message when present.
    pub fn reply_document(
        &self,
        update: &Update,
        document: impl Into<String>,
    ) -> Result<BlockingDocumentSendBuilder> {
        let request = reply_document_request(update, document)?;
        Ok(BlockingDocumentSendBuilder::new(
            self.client.clone(),
            request,
        ))
    }

    /// Starts a document-upload builder using the update reply target and quoting its source message when present.
    pub fn reply_document_upload(&self, update: &Update) -> Result<BlockingDocumentUploadBuilder> {
        let request = reply_document_upload_request(update)?;
        Ok(BlockingDocumentUploadBuilder::new(
            self.client.clone(),
            request,
        ))
    }

    /// Starts a video-send builder for a target chat.
    pub fn video(
        &self,
        chat_id: impl Into<ChatId>,
        video: impl Into<String>,
    ) -> BlockingVideoSendBuilder {
        let request = video_send_request(chat_id, video);
        BlockingVideoSendBuilder::new(self.client.clone(), request)
    }

    /// Starts a video-upload builder for a target chat.
    pub fn video_upload(&self, chat_id: impl Into<ChatId>) -> BlockingVideoUploadBuilder {
        let request = video_upload_request(chat_id);
        BlockingVideoUploadBuilder::new(self.client.clone(), request)
    }

    /// Starts a video-send builder using the update reply target and quoting its source message when present.
    pub fn reply_video(
        &self,
        update: &Update,
        video: impl Into<String>,
    ) -> Result<BlockingVideoSendBuilder> {
        let request = reply_video_request(update, video)?;
        Ok(BlockingVideoSendBuilder::new(self.client.clone(), request))
    }

    /// Starts a video-upload builder using the update reply target and quoting its source message when present.
    pub fn reply_video_upload(&self, update: &Update) -> Result<BlockingVideoUploadBuilder> {
        let request = reply_video_upload_request(update)?;
        Ok(BlockingVideoUploadBuilder::new(
            self.client.clone(),
            request,
        ))
    }

    /// Starts an audio-send builder for a target chat.
    pub fn audio(
        &self,
        chat_id: impl Into<ChatId>,
        audio: impl Into<String>,
    ) -> BlockingAudioSendBuilder {
        let request = audio_send_request(chat_id, audio);
        BlockingAudioSendBuilder::new(self.client.clone(), request)
    }

    /// Starts an audio-upload builder for a target chat.
    pub fn audio_upload(&self, chat_id: impl Into<ChatId>) -> BlockingAudioUploadBuilder {
        let request = audio_upload_request(chat_id);
        BlockingAudioUploadBuilder::new(self.client.clone(), request)
    }

    /// Starts an audio-send builder using the update reply target and quoting its source message when present.
    pub fn reply_audio(
        &self,
        update: &Update,
        audio: impl Into<String>,
    ) -> Result<BlockingAudioSendBuilder> {
        let request = reply_audio_request(update, audio)?;
        Ok(BlockingAudioSendBuilder::new(self.client.clone(), request))
    }

    /// Starts an audio-upload builder using the update reply target and quoting its source message when present.
    pub fn reply_audio_upload(&self, update: &Update) -> Result<BlockingAudioUploadBuilder> {
        let request = reply_audio_upload_request(update)?;
        Ok(BlockingAudioUploadBuilder::new(
            self.client.clone(),
            request,
        ))
    }

    /// Starts an animation-send builder for a target chat.
    pub fn animation(
        &self,
        chat_id: impl Into<ChatId>,
        animation: impl Into<String>,
    ) -> BlockingAnimationSendBuilder {
        let request = animation_send_request(chat_id, animation);
        BlockingAnimationSendBuilder::new(self.client.clone(), request)
    }

    /// Starts an animation-upload builder for a target chat.
    pub fn animation_upload(&self, chat_id: impl Into<ChatId>) -> BlockingAnimationUploadBuilder {
        let request = animation_upload_request(chat_id);
        BlockingAnimationUploadBuilder::new(self.client.clone(), request)
    }

    /// Starts an animation-send builder using the update reply target and quoting its source message when present.
    pub fn reply_animation(
        &self,
        update: &Update,
        animation: impl Into<String>,
    ) -> Result<BlockingAnimationSendBuilder> {
        let request = reply_animation_request(update, animation)?;
        Ok(BlockingAnimationSendBuilder::new(
            self.client.clone(),
            request,
        ))
    }

    /// Starts an animation-upload builder using the update reply target and quoting its source message when present.
    pub fn reply_animation_upload(
        &self,
        update: &Update,
    ) -> Result<BlockingAnimationUploadBuilder> {
        let request = reply_animation_upload_request(update)?;
        Ok(BlockingAnimationUploadBuilder::new(
            self.client.clone(),
            request,
        ))
    }

    /// Starts a voice-send builder for a target chat.
    pub fn voice(
        &self,
        chat_id: impl Into<ChatId>,
        voice: impl Into<String>,
    ) -> BlockingVoiceSendBuilder {
        let request = voice_send_request(chat_id, voice);
        BlockingVoiceSendBuilder::new(self.client.clone(), request)
    }

    /// Starts a voice-upload builder for a target chat.
    pub fn voice_upload(&self, chat_id: impl Into<ChatId>) -> BlockingVoiceUploadBuilder {
        let request = voice_upload_request(chat_id);
        BlockingVoiceUploadBuilder::new(self.client.clone(), request)
    }

    /// Starts a voice-send builder using the update reply target and quoting its source message when present.
    pub fn reply_voice(
        &self,
        update: &Update,
        voice: impl Into<String>,
    ) -> Result<BlockingVoiceSendBuilder> {
        let request = reply_voice_request(update, voice)?;
        Ok(BlockingVoiceSendBuilder::new(self.client.clone(), request))
    }

    /// Starts a voice-upload builder using the update reply target and quoting its source message when present.
    pub fn reply_voice_upload(&self, update: &Update) -> Result<BlockingVoiceUploadBuilder> {
        let request = reply_voice_upload_request(update)?;
        Ok(BlockingVoiceUploadBuilder::new(
            self.client.clone(),
            request,
        ))
    }

    /// Starts a video-note-send builder for a target chat.
    pub fn video_note(
        &self,
        chat_id: impl Into<ChatId>,
        video_note: impl Into<String>,
    ) -> BlockingVideoNoteSendBuilder {
        let request = video_note_send_request(chat_id, video_note);
        BlockingVideoNoteSendBuilder::new(self.client.clone(), request)
    }

    /// Starts a video-note-upload builder for a target chat.
    pub fn video_note_upload(&self, chat_id: impl Into<ChatId>) -> BlockingVideoNoteUploadBuilder {
        let request = video_note_upload_request(chat_id);
        BlockingVideoNoteUploadBuilder::new(self.client.clone(), request)
    }

    /// Starts a video-note-send builder using the update reply target and quoting its source message when present.
    pub fn reply_video_note(
        &self,
        update: &Update,
        video_note: impl Into<String>,
    ) -> Result<BlockingVideoNoteSendBuilder> {
        let request = reply_video_note_request(update, video_note)?;
        Ok(BlockingVideoNoteSendBuilder::new(
            self.client.clone(),
            request,
        ))
    }

    /// Starts a video-note-upload builder using the update reply target and quoting its source message when present.
    pub fn reply_video_note_upload(
        &self,
        update: &Update,
    ) -> Result<BlockingVideoNoteUploadBuilder> {
        let request = reply_video_note_upload_request(update)?;
        Ok(BlockingVideoNoteUploadBuilder::new(
            self.client.clone(),
            request,
        ))
    }

    /// Starts a sticker-send builder for a target chat.
    pub fn sticker(
        &self,
        chat_id: impl Into<ChatId>,
        sticker: impl Into<String>,
    ) -> BlockingStickerSendBuilder {
        let request = sticker_send_request(chat_id, sticker);
        BlockingStickerSendBuilder::new(self.client.clone(), request)
    }

    /// Starts a sticker-upload builder for a target chat.
    pub fn sticker_upload(&self, chat_id: impl Into<ChatId>) -> BlockingStickerUploadBuilder {
        let request = sticker_upload_request(chat_id);
        BlockingStickerUploadBuilder::new(self.client.clone(), request)
    }

    /// Starts a sticker-send builder using the update reply target and quoting its source message when present.
    pub fn reply_sticker(
        &self,
        update: &Update,
        sticker: impl Into<String>,
    ) -> Result<BlockingStickerSendBuilder> {
        let request = reply_sticker_request(update, sticker)?;
        Ok(BlockingStickerSendBuilder::new(
            self.client.clone(),
            request,
        ))
    }

    /// Starts a sticker-upload builder using the update reply target and quoting its source message when present.
    pub fn reply_sticker_upload(&self, update: &Update) -> Result<BlockingStickerUploadBuilder> {
        let request = reply_sticker_upload_request(update)?;
        Ok(BlockingStickerUploadBuilder::new(
            self.client.clone(),
            request,
        ))
    }

    /// Starts a media-group builder for a target chat.
    ///
    /// `media` must contain 2-10 photo/video/audio/document items.
    pub fn media_group<I, M>(
        &self,
        chat_id: impl Into<ChatId>,
        media: I,
    ) -> Result<BlockingMediaGroupSendBuilder>
    where
        I: IntoIterator<Item = M>,
        M: Into<InputMediaGroupItem>,
    {
        let request = media_group_send_request(chat_id, media)?;
        Ok(BlockingMediaGroupSendBuilder::new(
            self.client.clone(),
            request,
        ))
    }

    /// Starts a media-group upload builder for a target chat.
    ///
    /// `media` must contain `attach://...` media or thumbnail references matching uploaded parts.
    pub fn media_group_upload<I, M>(
        &self,
        chat_id: impl Into<ChatId>,
        media: I,
    ) -> Result<BlockingMediaGroupUploadBuilder>
    where
        I: IntoIterator<Item = M>,
        M: Into<InputMediaGroupItem>,
    {
        let request = media_group_upload_request(chat_id, media)?;
        Ok(BlockingMediaGroupUploadBuilder::new(
            self.client.clone(),
            request,
        ))
    }

    /// Starts a media-group builder using the update reply target and quoting its source message when present.
    pub fn reply_media_group<I, M>(
        &self,
        update: &Update,
        media: I,
    ) -> Result<BlockingMediaGroupSendBuilder>
    where
        I: IntoIterator<Item = M>,
        M: Into<InputMediaGroupItem>,
    {
        let request = reply_media_group_request(update, media)?;
        Ok(BlockingMediaGroupSendBuilder::new(
            self.client.clone(),
            request,
        ))
    }

    /// Starts a media-group upload builder using the update reply target and quoting its source message when present.
    ///
    /// `media` must contain `attach://...` media or thumbnail references matching uploaded parts.
    pub fn reply_media_group_upload<I, M>(
        &self,
        update: &Update,
        media: I,
    ) -> Result<BlockingMediaGroupUploadBuilder>
    where
        I: IntoIterator<Item = M>,
        M: Into<InputMediaGroupItem>,
    {
        let request = reply_media_group_upload_request(update, media)?;
        Ok(BlockingMediaGroupUploadBuilder::new(
            self.client.clone(),
            request,
        ))
    }

    /// Shortcut for `text(...).send()`.
    pub fn send_text(
        &self,
        chat_id: impl Into<ChatId>,
        text: impl Into<String>,
    ) -> Result<Message> {
        self.text(chat_id, text)?.send()
    }

    /// Shortcut for `reply(...).send()`.
    pub fn reply_text(&self, update: &Update, text: impl Into<String>) -> Result<Message> {
        self.reply(update, text)?.send()
    }

    /// Shortcut for `callback_answer(...).text_optional(...).send()`.
    pub fn answer_callback(
        &self,
        callback_query_id: impl Into<String>,
        text: Option<String>,
    ) -> Result<bool> {
        self.callback_answer(callback_query_id)
            .text_optional(text)
            .send()
    }

    /// Shortcut for `callback_answer_from_update(...).text_optional(...).send()`.
    pub fn answer_callback_from_update(
        &self,
        update: &Update,
        text: Option<String>,
    ) -> Result<bool> {
        self.callback_answer_from_update(update)?
            .text_optional(text)
            .send()
    }
}
