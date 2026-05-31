#![cfg(feature = "_async")]

use std::sync::{Arc, Mutex};
use std::time::Duration;

use tele::testing::{FakeTelegramServer, RequestExpectation};
use tele::types::advanced::{
    AdvancedAnswerPreCheckoutQueryRequest, AdvancedAnswerShippingQueryRequest,
    AdvancedAnswerWebAppQueryRequest, AdvancedApproveSuggestedPostRequest,
    AdvancedCreateForumTopicRequest, AdvancedCreateInvoiceLinkRequest,
    AdvancedDeclineSuggestedPostRequest, AdvancedEditMessageMediaRequest,
    AdvancedForwardMessagesRequest, AdvancedGetAvailableGiftsRequest,
    AdvancedGetBusinessAccountGiftsRequest, AdvancedGetBusinessAccountStarBalanceRequest,
    AdvancedGetChatGiftsRequest, AdvancedGetCustomEmojiStickersRequest,
    AdvancedGetGameHighScoresRequest, AdvancedGetStarTransactionsRequest,
    AdvancedGetStickerSetRequest, AdvancedGetUserChatBoostsRequest, AdvancedGetUserGiftsRequest,
    AdvancedGetUserProfileAudiosRequest, AdvancedGiftPremiumSubscriptionRequest,
    AdvancedPostStoryRequest, AdvancedRepostStoryRequest, AdvancedSavePreparedInlineMessageRequest,
    AdvancedSavePreparedKeyboardButtonRequest, AdvancedSendGameRequest, AdvancedSendGiftRequest,
    AdvancedSendInvoiceRequest, AdvancedSendMessageDraftRequest, AdvancedSendPaidMediaRequest,
    AdvancedSendStickerRequest, AdvancedSetBusinessAccountBioRequest,
    AdvancedSetBusinessAccountNameRequest, AdvancedSetBusinessAccountUsernameRequest,
    AdvancedSetChatMemberTagRequest, AdvancedSetStickerEmojiListRequest,
    AdvancedSetStickerKeywordsRequest, AdvancedSetStickerSetTitleRequest,
    AdvancedSetUserEmojiStatusRequest, AdvancedVerifyUserRequest,
};
use tele::types::{
    AnswerInlineQueryRequest, BotCommand, ChatAction, ChatAdministratorCapability, ChatId,
    CreateInvoiceLinkRequest, DiceEmoji, GetFileRequest, GetMyCommandsRequest,
    InlineKeyboardButton, InlineKeyboardMarkup, InlineQueryResult, InlineQueryResultsButton,
    InputMediaGroupItem, InputMediaPhoto, InputMediaVideo, InputPaidMedia, InputPollMedia,
    InputStoryContent, KeyboardButton, KeyboardButtonRequestUsers, LabeledPrice, MessageEntity,
    MessageId, ParseMode, PollKind, SendDocumentRequest, SendMediaGroupRequest, SendMessageRequest,
    SendPhotoRequest, SendStickerRequest, SetChatPhotoRequest, SetMyCommandsRequest,
    ShippingOption, StickerFormat, StoryArea, StoryAreaPosition, SuggestedPostParameters, Update,
    UploadStickerFileRequest, WebAppData,
};
use tele::{
    BanMemberOptions, BootstrapPlan, BootstrapRetryPolicy, BootstrapStepPhase, BootstrapStepStatus,
    Client, ClientMetric, Error, ErrorClass, MenuButtonConfig, RestrictMemberOptions, RetryConfig,
    UploadFile, UploadPart,
};

#[cfg(feature = "bot")]
use tele::types::BotCommandScope;

type DynError = Box<dyn std::error::Error + Send + Sync>;
type TestServer = FakeTelegramServer;

fn valid_suggested_post_send_date() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(600, |duration| duration.as_secs() as i64 + 600)
}

fn message_entity(kind: &str, length: u32) -> serde_json::Result<MessageEntity> {
    serde_json::from_value(serde_json::json!({
        "type": kind,
        "offset": 0,
        "length": length
    }))
}

fn spawn_server(
    expected_path: &'static str,
    response_status: u16,
    response_body: &'static str,
) -> Result<(String, TestServer), DynError> {
    let server = FakeTelegramServer::single(
        RequestExpectation::post(expected_path).respond_json(response_status, response_body),
    )?;
    Ok((server.base_url().to_owned(), server))
}

fn spawn_server_with_checks(
    expected_path: &'static str,
    response_status: u16,
    response_body: &'static str,
    required_substrings: &'static [&'static str],
) -> Result<(String, TestServer), DynError> {
    let mut expectation =
        RequestExpectation::post(expected_path).respond_json(response_status, response_body);
    for required in required_substrings {
        expectation = expectation.contains_case_insensitive(*required);
    }
    let server = FakeTelegramServer::single(expectation)?;
    Ok((server.base_url().to_owned(), server))
}

fn spawn_server_script(
    script: Vec<(&'static str, u16, &'static str)>,
) -> Result<(String, TestServer), DynError> {
    let expectations = script
        .into_iter()
        .map(|(expected_path, response_status, response_body)| {
            RequestExpectation::post(expected_path).respond_json(response_status, response_body)
        })
        .collect();
    let server = FakeTelegramServer::start(expectations)?;
    Ok((server.base_url().to_owned(), server))
}

fn fast_retry(max_attempts: usize, allow_non_idempotent_retries: bool) -> RetryConfig {
    let mut retry = RetryConfig::default();
    retry.max_attempts = max_attempts;
    retry.base_backoff = Duration::from_millis(1);
    retry.max_backoff = Duration::from_millis(1);
    retry.jitter_ratio = 0.0;
    retry.allow_non_idempotent_retries = allow_non_idempotent_retries;
    retry
}

fn fast_bootstrap_retry(max_attempts: usize) -> BootstrapRetryPolicy {
    BootstrapRetryPolicy {
        max_attempts,
        base_backoff: Duration::from_millis(1),
        max_backoff: Duration::from_millis(1),
        jitter_ratio: 0.0,
        continue_on_failure: false,
    }
}

fn join_server(server: TestServer) -> Result<(), DynError> {
    let _ = server.finish()?;
    Ok(())
}

#[tokio::test]
async fn client_metric_hook_records_method_latency() -> Result<(), DynError> {
    let response =
        r#"{"ok":true,"result":{"id":42,"is_bot":true,"first_name":"tele","username":"tele_bot"}}"#;
    let (base_url, handle) = spawn_server("/bot123:abc/getMe", 200, response)?;
    let metrics = Arc::new(Mutex::new(Vec::<ClientMetric>::new()));

    let client = Client::builder(base_url)?
        .bot_token("123:abc")?
        .on_metric({
            let metrics = Arc::clone(&metrics);
            move |metric| {
                if let Ok(mut captured) = metrics.lock() {
                    captured.push(metric.clone());
                }
            }
        })
        .build()?;

    let _ = client.bot().get_me().await?;
    join_server(handle)?;

    let captured = metrics.lock().map_err(|_| "client metric mutex poisoned")?;
    assert_eq!(captured.len(), 1);
    assert_eq!(captured[0].method, "getMe");
    assert!(captured[0].success);
    assert!(captured[0].latency >= Duration::ZERO);
    assert_eq!(captured[0].classification, None);

    Ok(())
}

#[tokio::test]
async fn get_me_success() -> Result<(), DynError> {
    let response =
        r#"{"ok":true,"result":{"id":42,"is_bot":true,"first_name":"tele","username":"tele_bot"}}"#;
    let (base_url, handle) = spawn_server("/bot123:abc/getMe", 200, response)?;

    let client = Client::builder(base_url)?.bot_token("123:abc")?.build()?;

    let me = client.bot().get_me().await?;
    assert_eq!(me.username.as_deref(), Some("tele_bot"));

    join_server(handle)?;
    Ok(())
}

#[tokio::test]
async fn raw_layer_call_no_params_success() -> Result<(), DynError> {
    let response =
        r#"{"ok":true,"result":{"id":42,"is_bot":true,"first_name":"tele","username":"tele_bot"}}"#;
    let (base_url, handle) = spawn_server("/bot123:abc/getMe", 200, response)?;

    let client = Client::builder(base_url)?.bot_token("123:abc")?.build()?;
    let me: tele::types::User = client.raw().call_no_params("getMe").await?;
    assert_eq!(me.username.as_deref(), Some("tele_bot"));

    join_server(handle)?;
    Ok(())
}

#[tokio::test]
async fn typed_layer_advanced_request_success() -> Result<(), DynError> {
    let response = r#"{"ok":true,"result":{"gifts":[]}}"#;
    let (base_url, handle) = spawn_server("/bot123:abc/getAvailableGifts", 200, response)?;

    let client = Client::builder(base_url)?.bot_token("123:abc")?.build()?;
    let request = AdvancedGetAvailableGiftsRequest::new();
    let gifts = client.typed().call(&request).await?;
    assert!(gifts.gifts.is_empty());

    join_server(handle)?;
    Ok(())
}

#[tokio::test]
async fn typed_layer_validates_advanced_request_before_transport() -> Result<(), DynError> {
    let client = Client::builder("http://127.0.0.1:1")?
        .bot_token("123:abc")?
        .build()?;
    let request = AdvancedForwardMessagesRequest::new(1_i64, 2_i64, Vec::new());

    let error = match client.typed().call(&request).await {
        Ok(_) => return Err("empty required vector must be rejected before transport".into()),
        Err(error) => error,
    };
    assert!(matches!(error, Error::InvalidRequest { .. }));

    let request =
        AdvancedForwardMessagesRequest::new(1_i64, 2_i64, vec![MessageId(10), MessageId(9)]);
    let error = match client.typed().call(&request).await {
        Ok(_) => return Err("unordered message ids must be rejected before transport".into()),
        Err(error) => error,
    };
    assert!(matches!(error, Error::InvalidRequest { .. }));

    let request =
        AdvancedForwardMessagesRequest::new(1_i64, 2_i64, vec![MessageId(9), MessageId(9)]);
    let error = match client.typed().call(&request).await {
        Ok(_) => return Err("duplicate message ids must be rejected before transport".into()),
        Err(error) => error,
    };
    assert!(matches!(error, Error::InvalidRequest { .. }));

    let request = AdvancedSendGiftRequest::new("gift-id");
    let error = match client
        .advanced()
        .send_gift::<serde_json::Value>(&request)
        .await
    {
        Ok(_) => return Err("generated sendGift requires a user or chat target".into()),
        Err(error) => error,
    };
    assert!(matches!(error, Error::InvalidRequest { .. }));

    let mut request = AdvancedSendGiftRequest::new("gift-id");
    request.user_id = Some(tele::types::UserId(1));
    request.chat_id = Some(1_i64.into());
    let error = match client
        .advanced()
        .send_gift::<serde_json::Value>(&request)
        .await
    {
        Ok(_) => return Err("generated sendGift must reject ambiguous targets".into()),
        Err(error) => error,
    };
    assert!(matches!(error, Error::InvalidRequest { .. }));

    let mut request = AdvancedSendGiftRequest::new("gift-id");
    request.user_id = Some(tele::types::UserId(1));
    request.text = Some("a".repeat(129));
    let error = match client
        .advanced()
        .send_gift::<serde_json::Value>(&request)
        .await
    {
        Ok(_) => return Err("generated sendGift text must enforce API length".into()),
        Err(error) => error,
    };
    assert!(matches!(error, Error::InvalidRequest { .. }));

    let mut request = AdvancedSendGiftRequest::new("gift-id");
    request.user_id = Some(tele::types::UserId(1));
    request.text = Some("https://example.com".to_owned());
    request.text_entities = Some(vec![message_entity("url", 19)?]);
    let error = match client
        .advanced()
        .send_gift::<serde_json::Value>(&request)
        .await
    {
        Ok(_) => return Err("generated sendGift must reject entities Telegram ignores".into()),
        Err(error) => error,
    };
    assert!(matches!(error, Error::InvalidRequest { .. }));

    let request = AdvancedGiftPremiumSubscriptionRequest::new(tele::types::UserId(1), 3, 1500);
    let error = match client
        .advanced()
        .gift_premium_subscription::<serde_json::Value>(&request)
        .await
    {
        Ok(_) => {
            return Err("generated premium gift subscriptions must enforce star pricing".into());
        }
        Err(error) => error,
    };
    assert!(matches!(error, Error::InvalidRequest { .. }));

    Ok(())
}

#[tokio::test]
async fn advanced_service_validates_generated_request_before_transport() -> Result<(), DynError> {
    let client = Client::builder("http://127.0.0.1:1")?
        .bot_token("123:abc")?
        .build()?;
    let request = AdvancedForwardMessagesRequest::new(1_i64, 2_i64, Vec::new());

    let error = match client
        .advanced()
        .forward_messages::<serde_json::Value>(&request)
        .await
    {
        Ok(_) => return Err("empty required vector must be rejected before transport".into()),
        Err(error) => error,
    };
    assert!(matches!(error, Error::InvalidRequest { .. }));

    let request =
        AdvancedForwardMessagesRequest::new(1_i64, 2_i64, vec![MessageId(10), MessageId(9)]);
    let error = match client
        .advanced()
        .forward_messages::<serde_json::Value>(&request)
        .await
    {
        Ok(_) => return Err("unordered message ids must be rejected before transport".into()),
        Err(error) => error,
    };
    assert!(matches!(error, Error::InvalidRequest { .. }));

    let paid_media = InputPaidMedia::photo("file-id");

    let request = AdvancedSendPaidMediaRequest::new(1_i64, 25_001, vec![paid_media.clone()]);
    let error = match client
        .advanced()
        .send_paid_media::<serde_json::Value>(&request)
        .await
    {
        Ok(_) => return Err("paid media star_count must enforce the Bot API maximum".into()),
        Err(error) => error,
    };
    assert!(matches!(error, Error::InvalidRequest { .. }));

    let request = AdvancedSendPaidMediaRequest::new(1_i64, 1, vec![paid_media.clone(); 11]);
    let error = match client
        .advanced()
        .send_paid_media::<serde_json::Value>(&request)
        .await
    {
        Ok(_) => return Err("paid media must enforce the Bot API media item limit".into()),
        Err(error) => error,
    };
    assert!(matches!(error, Error::InvalidRequest { .. }));

    let mut request = AdvancedSendPaidMediaRequest::new(1_i64, 1, vec![paid_media]);
    request.payload = Some("x".repeat(129));
    let error = match client
        .advanced()
        .send_paid_media::<serde_json::Value>(&request)
        .await
    {
        Ok(_) => return Err("paid media payload must enforce the Bot API byte limit".into()),
        Err(error) => error,
    };
    assert!(matches!(error, Error::InvalidRequest { .. }));

    let mut request =
        AdvancedSendPaidMediaRequest::new(1_i64, 1, vec![InputPaidMedia::photo("file-id")]);
    request.caption = Some("x".repeat(1025));
    let error = match client
        .advanced()
        .send_paid_media::<serde_json::Value>(&request)
        .await
    {
        Ok(_) => return Err("paid media captions must enforce the Bot API length".into()),
        Err(error) => error,
    };
    assert!(matches!(error, Error::InvalidRequest { .. }));

    let invalid_paid_media = InputPaidMedia::new(serde_json::json!({"type": "photo"}));
    assert!(matches!(
        invalid_paid_media,
        Err(Error::InvalidRequest { .. })
    ));

    let mut request = AdvancedSendMessageDraftRequest::new(1_i64, 1);
    request.text = Some("x".repeat(4097));
    let error = match client
        .advanced()
        .send_message_draft::<serde_json::Value>(&request)
        .await
    {
        Ok(_) => return Err("message drafts must enforce text length when present".into()),
        Err(error) => error,
    };
    assert!(matches!(error, Error::InvalidRequest { .. }));

    let story_content = InputStoryContent::photo("file-id");
    let request = AdvancedPostStoryRequest::new("business-id", story_content.clone(), 86_401);
    let error = match client
        .advanced()
        .post_story::<serde_json::Value>(&request)
        .await
    {
        Ok(_) => return Err("postStory active_period must use Bot API enum values".into()),
        Err(error) => error,
    };
    assert!(matches!(error, Error::InvalidRequest { .. }));

    let position = StoryAreaPosition::new(50.0, 50.0, 20.0, 10.0, 0.0, 4.0);
    let mut request = AdvancedPostStoryRequest::new("business-id", story_content, 86_400);
    request.areas = Some(vec![StoryArea::location(position, 1.0, 2.0); 11]);
    let error = match client
        .advanced()
        .post_story::<serde_json::Value>(&request)
        .await
    {
        Ok(_) => return Err("postStory areas must enforce StoryArea type limits".into()),
        Err(error) => error,
    };
    assert!(matches!(error, Error::InvalidRequest { .. }));

    let request = AdvancedRepostStoryRequest::new("business-id", 1_i64, 1, 86_401);
    let error = match client
        .advanced()
        .repost_story::<serde_json::Value>(&request)
        .await
    {
        Ok(_) => return Err("repostStory active_period must use Bot API enum values".into()),
        Err(error) => error,
    };
    assert!(matches!(error, Error::InvalidRequest { .. }));

    let mut request = AdvancedSendStickerRequest::new(1_i64, "sticker-file-id");
    request.business_connection_id = Some(" \n ".to_owned());
    let error = match client
        .advanced()
        .send_sticker::<serde_json::Value>(&request)
        .await
    {
        Ok(_) => return Err("empty generated string identifiers must be rejected".into()),
        Err(error) => error,
    };
    assert!(matches!(error, Error::InvalidRequest { .. }));

    let mut request = AdvancedSetUserEmojiStatusRequest::new(tele::types::UserId(1));
    request.emoji_status_custom_emoji_id = Some("bad\nstatus".to_owned());
    let error = match client
        .advanced()
        .set_user_emoji_status::<serde_json::Value>(&request)
        .await
    {
        Ok(_) => return Err("generated emoji status ids must reject control characters".into()),
        Err(error) => error,
    };
    assert!(matches!(error, Error::InvalidRequest { .. }));

    let mut request = AdvancedSetChatMemberTagRequest::new(1_i64, tele::types::UserId(1));
    request.tag = Some("a".repeat(17));
    let error = match client
        .advanced()
        .set_chat_member_tag::<serde_json::Value>(&request)
        .await
    {
        Ok(_) => return Err("generated member tags must enforce API length".into()),
        Err(error) => error,
    };
    assert!(matches!(error, Error::InvalidRequest { .. }));

    let mut request = AdvancedSetChatMemberTagRequest::new(1_i64, tele::types::UserId(1));
    request.tag = Some("ops🚀".to_owned());
    let error = match client
        .advanced()
        .set_chat_member_tag::<serde_json::Value>(&request)
        .await
    {
        Ok(_) => return Err("generated member tags must reject emoji".into()),
        Err(error) => error,
    };
    assert!(matches!(error, Error::InvalidRequest { .. }));

    let request = AdvancedSetBusinessAccountNameRequest::new("business-connection", "");
    let error = match client
        .advanced()
        .set_business_account_name::<serde_json::Value>(&request)
        .await
    {
        Ok(_) => return Err("generated business first names must not be empty".into()),
        Err(error) => error,
    };
    assert!(matches!(error, Error::InvalidRequest { .. }));

    let mut request = AdvancedSetBusinessAccountNameRequest::new("business-connection", "first");
    request.last_name = Some("a".repeat(65));
    let error = match client
        .advanced()
        .set_business_account_name::<serde_json::Value>(&request)
        .await
    {
        Ok(_) => return Err("generated business last names must enforce API length".into()),
        Err(error) => error,
    };
    assert!(matches!(error, Error::InvalidRequest { .. }));

    let mut request = AdvancedSetBusinessAccountUsernameRequest::new("business-connection");
    request.username = Some("bad name".to_owned());
    let error = match client
        .advanced()
        .set_business_account_username::<serde_json::Value>(&request)
        .await
    {
        Ok(_) => return Err("generated business usernames must enforce username syntax".into()),
        Err(error) => error,
    };
    assert!(matches!(error, Error::InvalidRequest { .. }));

    let mut request = AdvancedSetBusinessAccountBioRequest::new("business-connection");
    request.bio = Some("a".repeat(141));
    let error = match client
        .advanced()
        .set_business_account_bio::<serde_json::Value>(&request)
        .await
    {
        Ok(_) => return Err("generated business bios must enforce API length".into()),
        Err(error) => error,
    };
    assert!(matches!(error, Error::InvalidRequest { .. }));

    let mut request = AdvancedVerifyUserRequest::new(tele::types::UserId(1));
    request.custom_description = Some("a".repeat(71));
    let error = match client
        .advanced()
        .verify_user::<serde_json::Value>(&request)
        .await
    {
        Ok(_) => return Err("generated verification descriptions must enforce API length".into()),
        Err(error) => error,
    };
    assert!(matches!(error, Error::InvalidRequest { .. }));

    let mut request = AdvancedDeclineSuggestedPostRequest::new(1_i64, MessageId(1));
    request.comment = Some("a".repeat(129));
    let error = match client
        .advanced()
        .decline_suggested_post::<serde_json::Value>(&request)
        .await
    {
        Ok(_) => return Err("generated suggested post comments must enforce API length".into()),
        Err(error) => error,
    };
    assert!(matches!(error, Error::InvalidRequest { .. }));

    let mut request = AdvancedApproveSuggestedPostRequest::new(1_i64, MessageId(1));
    request.send_date = Some(1);
    let error = match client
        .advanced()
        .approve_suggested_post::<serde_json::Value>(&request)
        .await
    {
        Ok(_) => {
            return Err("generated suggested post approval dates must be in the future".into());
        }
        Err(error) => error,
    };
    assert!(matches!(error, Error::InvalidRequest { .. }));

    let mut request = AdvancedSendStickerRequest::new(1_i64, "sticker-file-id");
    request.emoji = Some("bad\nemoji".to_owned());
    let error = match client
        .advanced()
        .send_sticker::<serde_json::Value>(&request)
        .await
    {
        Ok(_) => return Err("invalid generated sticker emoji must be rejected".into()),
        Err(error) => error,
    };
    assert!(matches!(error, Error::InvalidRequest { .. }));

    let request = AdvancedGetStickerSetRequest::new("bad\nset-name");
    let error = match client
        .advanced()
        .get_sticker_set::<serde_json::Value>(&request)
        .await
    {
        Ok(_) => return Err("invalid generated sticker set names must be rejected".into()),
        Err(error) => error,
    };
    assert!(matches!(error, Error::InvalidRequest { .. }));

    let request = AdvancedSetStickerSetTitleRequest::new("set-name", "bad\ntitle");
    let error = match client
        .advanced()
        .set_sticker_set_title::<serde_json::Value>(&request)
        .await
    {
        Ok(_) => return Err("invalid generated sticker set titles must be rejected".into()),
        Err(error) => error,
    };
    assert!(matches!(error, Error::InvalidRequest { .. }));

    let mut request = AdvancedSendInvoiceRequest::new(
        1_i64,
        "title",
        "description",
        "payload",
        "USD",
        vec![LabeledPrice::new("item", 100)],
    );
    request.reply_markup = Some(InlineKeyboardMarkup::single_row(vec![
        InlineKeyboardButton::callback("Open", "invoice")?,
    ]));
    let error = match client
        .advanced()
        .send_invoice::<serde_json::Value>(&request)
        .await
    {
        Ok(_) => return Err("generated invoice markup must start with a Pay button".into()),
        Err(error) => error,
    };
    assert!(matches!(error, Error::InvalidRequest { .. }));

    let request = AdvancedCreateInvoiceLinkRequest::new(
        "title",
        "description",
        "payload",
        "USD",
        vec![LabeledPrice::new("", 100)],
    );
    let error = match client
        .advanced()
        .create_invoice_link::<serde_json::Value>(&request)
        .await
    {
        Ok(_) => return Err("invalid generated payment price entries must be rejected".into()),
        Err(error) => error,
    };
    assert!(matches!(error, Error::InvalidRequest { .. }));

    let request = AdvancedSendInvoiceRequest::new(
        1_i64,
        "t".repeat(33),
        "description",
        "payload",
        "USD",
        vec![LabeledPrice::new("item", 100)],
    );
    let error = match client
        .advanced()
        .send_invoice::<serde_json::Value>(&request)
        .await
    {
        Ok(_) => return Err("generated invoice titles must enforce API limits".into()),
        Err(error) => error,
    };
    assert!(matches!(error, Error::InvalidRequest { .. }));

    let request = AdvancedCreateInvoiceLinkRequest::new(
        "title",
        "description",
        "payload",
        "usd",
        vec![LabeledPrice::new("item", 100)],
    );
    let error = match client
        .advanced()
        .create_invoice_link::<serde_json::Value>(&request)
        .await
    {
        Ok(_) => return Err("generated invoice currency must use ISO uppercase format".into()),
        Err(error) => error,
    };
    assert!(matches!(error, Error::InvalidRequest { .. }));

    let request = AdvancedCreateInvoiceLinkRequest::new(
        "title",
        "description",
        "payload",
        "XTR",
        vec![
            LabeledPrice::new("item", 100),
            LabeledPrice::new("shipping", 10),
        ],
    );
    let error = match client
        .advanced()
        .create_invoice_link::<serde_json::Value>(&request)
        .await
    {
        Ok(_) => return Err("generated Stars invoice prices must contain one item".into()),
        Err(error) => error,
    };
    assert!(matches!(error, Error::InvalidRequest { .. }));

    let mut request = AdvancedCreateInvoiceLinkRequest::new(
        "title",
        "description",
        "payload",
        "USD",
        vec![LabeledPrice::new("item", 100)],
    );
    request.suggested_tip_amounts = Some(vec![1]);
    let error = match client
        .advanced()
        .create_invoice_link::<serde_json::Value>(&request)
        .await
    {
        Ok(_) => return Err("generated invoice tips must require max_tip_amount".into()),
        Err(error) => error,
    };
    assert!(matches!(error, Error::InvalidRequest { .. }));

    let mut request = AdvancedCreateInvoiceLinkRequest::new(
        "title",
        "description",
        "payload",
        "USD",
        vec![LabeledPrice::new("item", 100)],
    );
    request.business_connection_id = Some("business-1".to_owned());
    let error = match client
        .advanced()
        .create_invoice_link::<serde_json::Value>(&request)
        .await
    {
        Ok(_) => {
            return Err(
                "generated invoice business_connection_id must require Stars currency".into(),
            );
        }
        Err(error) => error,
    };
    assert!(matches!(error, Error::InvalidRequest { .. }));

    let mut request = AdvancedCreateInvoiceLinkRequest::new(
        "title",
        "description",
        "payload",
        "USD",
        vec![LabeledPrice::new("item", 100)],
    );
    request.subscription_period = Some(2_592_000);
    let error = match client
        .advanced()
        .create_invoice_link::<serde_json::Value>(&request)
        .await
    {
        Ok(_) => return Err("generated invoice subscriptions must require Stars currency".into()),
        Err(error) => error,
    };
    assert!(matches!(error, Error::InvalidRequest { .. }));

    let mut request = AdvancedCreateInvoiceLinkRequest::new(
        "title",
        "description",
        "payload",
        "XTR",
        vec![LabeledPrice::new("item", 100)],
    );
    request.subscription_period = Some(1);
    let error = match client
        .advanced()
        .create_invoice_link::<serde_json::Value>(&request)
        .await
    {
        Ok(_) => return Err("generated invoice subscriptions must enforce Telegram period".into()),
        Err(error) => error,
    };
    assert!(matches!(error, Error::InvalidRequest { .. }));

    let mut request = AdvancedAnswerShippingQueryRequest::new("shipping-query", true);
    request.shipping_options = Some(vec![ShippingOption::new(
        "",
        "shipping",
        vec![LabeledPrice::new("shipping", 100)],
    )]);
    let error = match client
        .advanced()
        .answer_shipping_query::<serde_json::Value>(&request)
        .await
    {
        Ok(_) => return Err("invalid generated shipping option entries must be rejected".into()),
        Err(error) => error,
    };
    assert!(matches!(error, Error::InvalidRequest { .. }));

    let request = AdvancedAnswerShippingQueryRequest::new("shipping-query", true);
    let error = match client
        .advanced()
        .answer_shipping_query::<serde_json::Value>(&request)
        .await
    {
        Ok(_) => return Err("generated shipping success requires shipping options".into()),
        Err(error) => error,
    };
    assert!(matches!(error, Error::InvalidRequest { .. }));

    let mut request = AdvancedAnswerShippingQueryRequest::new("shipping-query", false);
    request.shipping_options = Some(vec![ShippingOption::new(
        "standard",
        "shipping",
        vec![LabeledPrice::new("shipping", 100)],
    )]);
    request.error_message = Some("unavailable".to_owned());
    let error = match client
        .advanced()
        .answer_shipping_query::<serde_json::Value>(&request)
        .await
    {
        Ok(_) => return Err("generated shipping failure must omit shipping options".into()),
        Err(error) => error,
    };
    assert!(matches!(error, Error::InvalidRequest { .. }));

    let request = AdvancedAnswerPreCheckoutQueryRequest::new("checkout-query", false);
    let error = match client
        .advanced()
        .answer_pre_checkout_query::<serde_json::Value>(&request)
        .await
    {
        Ok(_) => return Err("generated checkout failure requires an error message".into()),
        Err(error) => error,
    };
    assert!(matches!(error, Error::InvalidRequest { .. }));

    let mut request = AdvancedAnswerPreCheckoutQueryRequest::new("checkout-query", true);
    request.error_message = Some("declined".to_owned());
    let error = match client
        .advanced()
        .answer_pre_checkout_query::<serde_json::Value>(&request)
        .await
    {
        Ok(_) => return Err("generated checkout success must omit error_message".into()),
        Err(error) => error,
    };
    assert!(matches!(error, Error::InvalidRequest { .. }));

    let request = AdvancedGetCustomEmojiStickersRequest::new(vec!["".to_owned()]);
    let error = match client
        .advanced()
        .get_custom_emoji_stickers::<serde_json::Value>(&request)
        .await
    {
        Ok(_) => return Err("empty generated string array entries must be rejected".into()),
        Err(error) => error,
    };
    assert!(matches!(error, Error::InvalidRequest { .. }));

    let request = AdvancedGetCustomEmojiStickersRequest::new(vec!["emoji-id".to_owned(); 201]);
    let error = match client
        .advanced()
        .get_custom_emoji_stickers::<serde_json::Value>(&request)
        .await
    {
        Ok(_) => return Err("generated custom emoji id lists must enforce API limits".into()),
        Err(error) => error,
    };
    assert!(matches!(error, Error::InvalidRequest { .. }));

    let mut request = AdvancedGetUserGiftsRequest::new(tele::types::UserId(1));
    request.offset = Some("bad\noffset".to_owned());
    let error = match client
        .advanced()
        .get_user_gifts::<serde_json::Value>(&request)
        .await
    {
        Ok(_) => return Err("generated string offsets must reject control characters".into()),
        Err(error) => error,
    };
    assert!(matches!(error, Error::InvalidRequest { .. }));

    let mut request = AdvancedGetUserProfileAudiosRequest::new(tele::types::UserId(1));
    request.limit = Some(101);
    let error = match client
        .advanced()
        .get_user_profile_audios::<serde_json::Value>(&request)
        .await
    {
        Ok(_) => {
            return Err("generated user profile audio limits must enforce API bounds".into());
        }
        Err(error) => error,
    };
    assert!(matches!(error, Error::InvalidRequest { .. }));

    let mut request = AdvancedGetUserGiftsRequest::new(tele::types::UserId(1));
    request.limit = Some(101);
    let error = match client
        .advanced()
        .get_user_gifts::<serde_json::Value>(&request)
        .await
    {
        Ok(_) => return Err("generated user gift limits must enforce API bounds".into()),
        Err(error) => error,
    };
    assert!(matches!(error, Error::InvalidRequest { .. }));

    let mut request = AdvancedGetChatGiftsRequest::new(1_i64);
    request.limit = Some(101);
    let error = match client
        .advanced()
        .get_chat_gifts::<serde_json::Value>(&request)
        .await
    {
        Ok(_) => return Err("generated chat gift limits must enforce API bounds".into()),
        Err(error) => error,
    };
    assert!(matches!(error, Error::InvalidRequest { .. }));

    let mut request = AdvancedGetBusinessAccountGiftsRequest::new("business-connection");
    request.limit = Some(101);
    let error = match client
        .advanced()
        .get_business_account_gifts::<serde_json::Value>(&request)
        .await
    {
        Ok(_) => return Err("generated business gift limits must enforce API bounds".into()),
        Err(error) => error,
    };
    assert!(matches!(error, Error::InvalidRequest { .. }));

    let mut request = AdvancedGetStarTransactionsRequest::new();
    request.limit = Some(101);
    let error = match client
        .advanced()
        .get_star_transactions::<serde_json::Value>(&request)
        .await
    {
        Ok(_) => return Err("generated star transaction limits must enforce API bounds".into()),
        Err(error) => error,
    };
    assert!(matches!(error, Error::InvalidRequest { .. }));

    let request =
        AdvancedSetStickerEmojiListRequest::new("sticker-file-id", vec!["😀".to_owned(); 21]);
    let error = match client
        .advanced()
        .set_sticker_emoji_list::<serde_json::Value>(&request)
        .await
    {
        Ok(_) => return Err("generated sticker emoji lists must enforce API limits".into()),
        Err(error) => error,
    };
    assert!(matches!(error, Error::InvalidRequest { .. }));

    let mut request = AdvancedSetStickerKeywordsRequest::new("sticker-file-id");
    request.keywords = Some(vec!["ok".to_owned(), "\n".to_owned()]);
    let error = match client
        .advanced()
        .set_sticker_keywords::<serde_json::Value>(&request)
        .await
    {
        Ok(_) => {
            return Err("invalid optional generated string array entries must be rejected".into());
        }
        Err(error) => error,
    };
    assert!(matches!(error, Error::InvalidRequest { .. }));

    let mut request = AdvancedSetStickerKeywordsRequest::new("sticker-file-id");
    request.keywords = Some(vec!["tag".to_owned(); 21]);
    let error = match client
        .advanced()
        .set_sticker_keywords::<serde_json::Value>(&request)
        .await
    {
        Ok(_) => return Err("generated sticker keyword lists must enforce API limits".into()),
        Err(error) => error,
    };
    assert!(matches!(error, Error::InvalidRequest { .. }));

    let request =
        AdvancedEditMessageMediaRequest::new(InputMediaPhoto::new("photo-file-id").into());
    let error = match client
        .advanced()
        .edit_message_media::<serde_json::Value>(&request)
        .await
    {
        Ok(_) => return Err("generated editMessageMedia requires a message target".into()),
        Err(error) => error,
    };
    assert!(matches!(error, Error::InvalidRequest { .. }));

    let mut request =
        AdvancedEditMessageMediaRequest::new(InputMediaPhoto::new("photo-file-id").into());
    request.chat_id = Some(1_i64.into());
    request.message_id = Some(MessageId(1));
    request.inline_message_id = Some("inline-message".to_owned());
    let error = match client
        .advanced()
        .edit_message_media::<serde_json::Value>(&request)
        .await
    {
        Ok(_) => return Err("generated editMessageMedia must reject ambiguous targets".into()),
        Err(error) => error,
    };
    assert!(matches!(error, Error::InvalidRequest { .. }));

    Ok(())
}

#[tokio::test]
async fn raw_retry_rejects_invalid_policy_before_request() -> Result<(), DynError> {
    let client = Client::builder("http://127.0.0.1:9")?
        .bot_token("123:abc")?
        .build()?;
    let mut retry = RetryConfig::default();
    retry.base_backoff = Duration::ZERO;

    let error = match client
        .raw()
        .call_no_params_with_retry::<tele::types::User>("getMe", retry)
        .await
    {
        Ok(_) => return Err("invalid retry policy unexpectedly succeeded".into()),
        Err(error) => error,
    };
    assert!(matches!(error, Error::Configuration { .. }));

    Ok(())
}

#[tokio::test]
async fn raw_retry_does_not_multiply_transport_retries() -> Result<(), DynError> {
    let failure = r#"{"ok":false,"error_code":502,"description":"Bad Gateway"}"#;
    let (base_url, server) = spawn_server_script(vec![
        ("/bot123:abc/getMe", 502, failure),
        ("/bot123:abc/getMe", 502, failure),
    ])?;

    let client = Client::builder(base_url)?
        .bot_token("123:abc")?
        .retry_config(fast_retry(3, true))?
        .build()?;

    let error = match client
        .raw()
        .call_no_params_with_retry::<tele::types::User>("getMe", fast_retry(2, false))
        .await
    {
        Ok(_) => return Err("retry unexpectedly succeeded after method retry budget".into()),
        Err(error) => error,
    };

    assert!(matches!(error, Error::Api { .. }));
    assert_eq!(error.status().map(|status| status.as_u16()), Some(502));
    assert_eq!(server.finish()?.len(), 2);

    Ok(())
}

#[tokio::test]
async fn client_default_retry_applies_method_policy_to_read_only_methods() -> Result<(), DynError> {
    let failure = r#"{"ok":false,"error_code":502,"description":"Bad Gateway"}"#;
    let success =
        r#"{"ok":true,"result":{"id":7,"is_bot":true,"first_name":"tele","username":"retry_bot"}}"#;
    let (base_url, server) = spawn_server_script(vec![
        ("/bot123:abc/getMe", 502, failure),
        ("/bot123:abc/getMe", 200, success),
    ])?;

    let client = Client::builder(base_url)?
        .bot_token("123:abc")?
        .retry_config(fast_retry(2, false))?
        .build()?;

    let me = client.bot().get_me().await?;

    assert_eq!(me.username.as_deref(), Some("retry_bot"));
    assert_eq!(server.finish()?.len(), 2);
    Ok(())
}

#[tokio::test]
async fn client_default_retry_respects_total_timeout_across_attempts() -> Result<(), DynError> {
    let failure = r#"{"ok":false,"error_code":502,"description":"Bad Gateway"}"#;
    let (base_url, server) = spawn_server("/bot123:abc/getMe", 502, failure)?;
    let mut retry = fast_retry(2, false);
    retry.base_backoff = Duration::from_secs(1);
    retry.max_backoff = Duration::from_secs(1);

    let client = Client::builder(base_url)?
        .bot_token("123:abc")?
        .total_timeout(Some(Duration::from_millis(500)))?
        .retry_config(retry)?
        .build()?;

    let error = match client.bot().get_me().await {
        Ok(_) => return Err("retry must not start after total timeout budget is exhausted".into()),
        Err(error) => error,
    };

    assert!(matches!(error, Error::Api { .. }));
    assert_eq!(server.finish()?.len(), 1);
    Ok(())
}

#[tokio::test]
async fn client_default_retry_keeps_mutating_methods_single_attempt_without_opt_in()
-> Result<(), DynError> {
    let server = FakeTelegramServer::single(
        RequestExpectation::post("/bot123:abc/sendMessage").respond_json(
            503,
            r#"{"ok":false,"error_code":503,"description":"Service Unavailable"}"#,
        ),
    )?;

    let client = Client::builder(server.base_url())?
        .bot_token("123:abc")?
        .retry_config(fast_retry(2, false))?
        .build()?;
    let request = SendMessageRequest::new(12_i64, "hello")?;

    let error = match client.messages().send_message(&request).await {
        Ok(_) => return Err("sendMessage must not retry ambiguous 503 without opt-in".into()),
        Err(error) => error,
    };

    assert!(matches!(error, Error::Api { .. }));
    assert_eq!(server.finish()?.len(), 1);
    Ok(())
}

#[tokio::test]
async fn raw_retry_retries_non_json_rate_limit_for_mutating_method() -> Result<(), DynError> {
    let rate_limited = "<html>too many requests</html>";
    let ok_response = r#"{"ok":true,"result":{"message_id":91,"date":1710000013,"chat":{"id":12,"type":"private"},"text":"hello"}}"#;
    let (base_url, server) = spawn_server_script(vec![
        ("/bot123:abc/sendMessage", 429, rate_limited),
        ("/bot123:abc/sendMessage", 200, ok_response),
    ])?;

    let client = Client::builder(base_url)?
        .bot_token("123:abc")?
        .retry_config(fast_retry(1, false))?
        .build()?;
    let payload = serde_json::json!({
        "chat_id": 12,
        "text": "hello"
    });

    let message = client
        .raw()
        .call_json_with_retry::<tele::types::Message, _>(
            "sendMessage",
            &payload,
            fast_retry(2, false),
        )
        .await?;

    assert_eq!(message.message_id, MessageId(91));
    assert_eq!(server.finish()?.len(), 2);
    Ok(())
}

#[tokio::test]
async fn raw_retry_keeps_retry_after_503_non_json_non_idempotent_safe() -> Result<(), DynError> {
    let server = FakeTelegramServer::single(
        RequestExpectation::post("/bot123:abc/sendMessage")
            .respond_json(503, "<html>service unavailable</html>")
            .response_header("Retry-After", "0"),
    )?;

    let client = Client::builder(server.base_url())?
        .bot_token("123:abc")?
        .retry_config(fast_retry(1, false))?
        .build()?;
    let payload = serde_json::json!({
        "chat_id": 12,
        "text": "hello"
    });

    let error = match client
        .raw()
        .call_json_with_retry::<tele::types::Message, _>(
            "sendMessage",
            &payload,
            fast_retry(2, false),
        )
        .await
    {
        Ok(_) => {
            return Err("Retry-After on 503 must not bypass non-idempotent retry policy".into());
        }
        Err(error) => error,
    };

    assert!(matches!(error, Error::Transport { .. }));
    assert_eq!(error.status().map(|status| status.as_u16()), Some(503));
    assert!(!error.is_rate_limited());
    assert_eq!(error.retry_after(), Some(Duration::ZERO));
    assert_eq!(server.finish()?.len(), 1);
    Ok(())
}

#[tokio::test]
async fn raw_retry_keeps_retry_after_503_api_error_non_idempotent_safe() -> Result<(), DynError> {
    let server = FakeTelegramServer::single(
        RequestExpectation::post("/bot123:abc/sendMessage")
            .respond_json(
                503,
                r#"{"ok":false,"error_code":503,"description":"Service Unavailable"}"#,
            )
            .response_header("Retry-After", "0"),
    )?;

    let client = Client::builder(server.base_url())?
        .bot_token("123:abc")?
        .retry_config(fast_retry(1, false))?
        .build()?;
    let payload = serde_json::json!({
        "chat_id": 12,
        "text": "hello"
    });

    let error = match client
        .raw()
        .call_json_with_retry::<tele::types::Message, _>(
            "sendMessage",
            &payload,
            fast_retry(2, false),
        )
        .await
    {
        Ok(_) => {
            return Err(
                "Retry-After on API 503 must not bypass non-idempotent retry policy".into(),
            );
        }
        Err(error) => error,
    };

    assert!(matches!(error, Error::Api { .. }));
    assert_eq!(error.status().map(|status| status.as_u16()), Some(503));
    assert!(!error.is_rate_limited());
    assert_eq!(error.retry_after(), Some(Duration::ZERO));
    assert_eq!(server.finish()?.len(), 1);
    Ok(())
}

#[tokio::test]
async fn setup_bootstrap_retry_does_not_multiply_transport_retries() -> Result<(), DynError> {
    let failure = r#"{"ok":false,"error_code":502,"description":"Bad Gateway"}"#;
    let (base_url, server) = spawn_server_script(vec![
        ("/bot123:abc/getMe", 502, failure),
        ("/bot123:abc/getMe", 502, failure),
    ])?;

    let client = Client::builder(base_url)?
        .bot_token("123:abc")?
        .retry_config(fast_retry(3, true))?
        .build()?;

    let outcome = client
        .control()
        .setup()
        .bootstrap_with_retry(
            &BootstrapPlan::new().fail_fast_get_me(),
            fast_bootstrap_retry(2),
        )
        .await;
    let error = outcome
        .error()
        .ok_or("bootstrap retry unexpectedly succeeded")?;

    assert!(matches!(error, Error::Api { .. }));
    assert_eq!(error.status().map(|status| status.as_u16()), Some(502));
    assert_eq!(outcome.report.me.diagnostics.attempt_count, 2);
    assert_eq!(server.finish()?.len(), 2);

    Ok(())
}

#[tokio::test]
async fn bootstrap_retry_rejects_invalid_policy_before_work() -> Result<(), DynError> {
    let client = Client::builder("http://127.0.0.1:9")?
        .bot_token("123:abc")?
        .build()?;
    let policy = BootstrapRetryPolicy {
        max_attempts: 0,
        ..BootstrapRetryPolicy::default()
    };

    let outcome = client
        .control()
        .setup()
        .bootstrap_with_retry(&BootstrapPlan::default(), policy)
        .await;
    assert!(matches!(outcome.error(), Some(Error::Configuration { .. })));

    Ok(())
}

#[tokio::test]
async fn api_error_exposes_retry_after() -> Result<(), DynError> {
    let response = r#"{"ok":false,"error_code":429,"description":"Too Many Requests","parameters":{"retry_after":3}}"#;
    let (base_url, handle) = spawn_server("/bot123:abc/getMe", 200, response)?;

    let client = Client::builder(base_url)?
        .bot_token("123:abc")?
        .retry_config(fast_retry(1, false))?
        .build()?;

    let err = match client.bot().get_me().await {
        Ok(_) => {
            return Err("expected Telegram API error".into());
        }
        Err(err) => err,
    };

    assert!(matches!(err, Error::Api { .. }));
    assert_eq!(err.retry_after(), Some(Duration::from_secs(3)));

    join_server(handle)?;
    Ok(())
}

#[tokio::test]
async fn app_send_text_success() -> Result<(), DynError> {
    let response = r#"{"ok":true,"result":{"message_id":7,"date":1710000000,"chat":{"id":1,"type":"private"},"text":"hello"}}"#;
    let (base_url, handle) = spawn_server("/bot123:abc/sendMessage", 200, response)?;

    let client = Client::builder(base_url)?.bot_token("123:abc")?.build()?;
    let sent = client.app().send_text(1_i64, "hello").await?;
    assert_eq!(sent.message_id.0, 7);

    join_server(handle)?;
    Ok(())
}

#[tokio::test]
async fn app_text_builder_supports_markup_and_common_options() -> Result<(), DynError> {
    let response = r#"{"ok":true,"result":{"message_id":9,"date":1710000001,"chat":{"id":1,"type":"private"},"text":"hello builder"}}"#;
    let expectations = vec![
        RequestExpectation::post("/bot123:abc/sendMessage")
            .contains_case_insensitive("\"chat_id\":1")
            .contains_case_insensitive("\"text\":\"hello builder\"")
            .contains_case_insensitive("\"parse_mode\":\"MarkdownV2\"")
            .contains_case_insensitive("\"disable_notification\":true")
            .contains_case_insensitive("\"protect_content\":true")
            .contains_case_insensitive("\"message_thread_id\":99")
            .contains_case_insensitive("\"direct_messages_topic_id\":7")
            .contains_case_insensitive("\"allow_paid_broadcast\":true")
            .contains_case_insensitive("\"message_effect_id\":\"effect-1\"")
            .contains_case_insensitive("\"suggested_post_parameters\":{")
            .contains_case_insensitive("\"reply_parameters\":{\"message_id\":55")
            .contains_case_insensitive("\"link_preview_options\":{\"is_disabled\":true")
            .contains_case_insensitive("\"reply_markup\":{\"inline_keyboard\":[[{\"text\":\"Open\"")
            .contains_case_insensitive("\"callback_data\":\"open:1\"")
            .respond_json(200, response),
    ];
    let server = FakeTelegramServer::start(expectations)?;

    let client = Client::builder(server.base_url())?
        .bot_token("123:abc")?
        .build()?;
    let markup =
        InlineKeyboardMarkup::single_row(vec![InlineKeyboardButton::callback("Open", "open:1")?]);

    let sent = client
        .app()
        .text(1_i64, "hello builder")?
        .parse_mode(ParseMode::MarkdownV2)
        .reply_to_message(MessageId(55))
        .message_thread_id(99)
        .direct_messages_topic_id(7)
        .disable_notification(true)
        .protect_content(true)
        .allow_paid_broadcast(true)
        .message_effect_id("effect-1")
        .suggested_post_parameters(SuggestedPostParameters::new(serde_json::json!({
            "send_date": valid_suggested_post_send_date()
        }))?)
        .disable_link_preview()
        .reply_markup(markup)
        .send()
        .await?;
    assert_eq!(sent.message_id.0, 9);

    join_server(server)?;
    Ok(())
}

#[tokio::test]
async fn app_non_file_message_builders_support_common_options() -> Result<(), DynError> {
    let expectations = vec![
        RequestExpectation::post("/bot123:abc/sendLocation")
            .contains_case_insensitive("\"chat_id\":1")
            .contains_case_insensitive("\"latitude\":37.5")
            .contains_case_insensitive("\"longitude\":-122.25")
            .contains_case_insensitive("\"horizontal_accuracy\":12.5")
            .contains_case_insensitive("\"live_period\":60")
            .contains_case_insensitive("\"heading\":90")
            .contains_case_insensitive("\"proximity_alert_radius\":100")
            .contains_case_insensitive("\"disable_notification\":true")
            .contains_case_insensitive("\"protect_content\":true")
            .contains_case_insensitive("\"message_thread_id\":31")
            .contains_case_insensitive("\"direct_messages_topic_id\":8")
            .contains_case_insensitive("\"allow_paid_broadcast\":true")
            .contains_case_insensitive("\"message_effect_id\":\"location-effect\"")
            .contains_case_insensitive("\"suggested_post_parameters\":{")
            .contains_case_insensitive("\"reply_parameters\":{\"message_id\":80")
            .contains_case_insensitive("\"reply_markup\":{\"inline_keyboard\":[[{\"text\":\"Open map\"")
            .respond_json(
                200,
                r#"{"ok":true,"result":{"message_id":30,"date":1710000020,"chat":{"id":1,"type":"private"}}}"#,
            ),
        RequestExpectation::post("/bot123:abc/sendVenue")
            .contains_case_insensitive("\"chat_id\":1")
            .contains_case_insensitive("\"latitude\":37.5")
            .contains_case_insensitive("\"longitude\":-122.25")
            .contains_case_insensitive("\"title\":\"Tele HQ\"")
            .contains_case_insensitive("\"address\":\"1 Bot Street\"")
            .contains_case_insensitive("\"google_place_id\":\"place-1\"")
            .contains_case_insensitive("\"google_place_type\":\"office\"")
            .contains_case_insensitive("\"message_thread_id\":32")
            .contains_case_insensitive("\"reply_parameters\":{\"message_id\":81")
            .respond_json(
                200,
                r#"{"ok":true,"result":{"message_id":31,"date":1710000021,"chat":{"id":1,"type":"private"}}}"#,
            ),
        RequestExpectation::post("/bot123:abc/sendContact")
            .contains_case_insensitive("\"chat_id\":1")
            .contains_case_insensitive("\"phone_number\":\"+15550000000\"")
            .contains_case_insensitive("\"first_name\":\"Tele\"")
            .contains_case_insensitive("\"last_name\":\"Bot\"")
            .contains_case_insensitive("\"vcard\":\"BEGIN:VCARD")
            .contains_case_insensitive("\"message_thread_id\":33")
            .contains_case_insensitive("\"reply_parameters\":{\"message_id\":82")
            .respond_json(
                200,
                r#"{"ok":true,"result":{"message_id":32,"date":1710000022,"chat":{"id":1,"type":"private"}}}"#,
            ),
        RequestExpectation::post("/bot123:abc/sendPoll")
            .contains_case_insensitive("\"chat_id\":1")
            .contains_case_insensitive("\"question\":\"Pick one\"")
            .contains_case_insensitive("\"options\":[{\"text\":\"yes\"},{\"text\":\"no\"}]")
            .contains_case_insensitive("\"is_anonymous\":false")
            .contains_case_insensitive("\"type\":\"regular\"")
            .contains_case_insensitive("\"allows_multiple_answers\":true")
            .contains_case_insensitive("\"allows_revoting\":true")
            .contains_case_insensitive("\"shuffle_options\":true")
            .contains_case_insensitive("\"allow_adding_options\":true")
            .contains_case_insensitive("\"hide_results_until_closes\":true")
            .contains_case_insensitive("\"members_only\":true")
            .contains_case_insensitive("\"open_period\":60")
            .contains_case_insensitive("\"description\":\"runtime poll\"")
            .contains_case_insensitive("\"media\":{\"type\":\"location\"")
            .contains_case_insensitive("\"explanation_media\":{\"type\":\"photo\"")
            .contains_case_insensitive("\"message_thread_id\":34")
            .contains_case_insensitive("\"reply_parameters\":{\"message_id\":83")
            .contains_case_insensitive("\"reply_markup\":{\"inline_keyboard\":[[{\"text\":\"Vote\"")
            .respond_json(
                200,
                r#"{"ok":true,"result":{"message_id":33,"date":1710000023,"chat":{"id":1,"type":"private"}}}"#,
            ),
        RequestExpectation::post("/bot123:abc/sendDice")
            .contains_case_insensitive("\"chat_id\":1")
            .contains_case_insensitive("\"message_thread_id\":35")
            .contains_case_insensitive("\"reply_parameters\":{\"message_id\":84")
            .respond_json(
                200,
                r#"{"ok":true,"result":{"message_id":34,"date":1710000024,"chat":{"id":1,"type":"private"}}}"#,
            ),
        RequestExpectation::post("/bot123:abc/sendChatAction")
            .contains_case_insensitive("\"chat_id\":1")
            .contains_case_insensitive("\"action\":\"typing\"")
            .contains_case_insensitive("\"message_thread_id\":36")
            .contains_case_insensitive("\"business_connection_id\":\"business-1\"")
            .respond_json(200, r#"{"ok":true,"result":true}"#),
        RequestExpectation::post("/bot123:abc/stopPoll")
            .contains_case_insensitive("\"chat_id\":1")
            .contains_case_insensitive("\"message_id\":85")
            .contains_case_insensitive("\"business_connection_id\":\"business-stop-poll\"")
            .contains_case_insensitive("\"reply_markup\":{\"inline_keyboard\":[[{\"text\":\"Closed\"")
            .respond_json(
                200,
                r#"{"ok":true,"result":{"id":"poll-1","question":"Pick one","options":[{"text":"yes","voter_count":0},{"text":"no","voter_count":0}],"total_voter_count":0,"is_closed":true,"is_anonymous":false,"type":"regular","allows_multiple_answers":false}}"#,
            ),
    ];
    let server = FakeTelegramServer::start(expectations)?;

    let client = Client::builder(server.base_url())?
        .bot_token("123:abc")?
        .build()?;

    let location_markup = InlineKeyboardMarkup::single_row(vec![InlineKeyboardButton::callback(
        "Open map",
        "location:1",
    )?]);
    let location = client
        .app()
        .location(1_i64, 37.5, -122.25)
        .horizontal_accuracy(12.5)
        .live_period(60)
        .heading(90)
        .proximity_alert_radius(100)
        .reply_to_message(MessageId(80))
        .message_thread_id(31)
        .direct_messages_topic_id(8)
        .disable_notification(true)
        .protect_content(true)
        .allow_paid_broadcast(true)
        .message_effect_id("location-effect")
        .suggested_post_parameters(SuggestedPostParameters::new(serde_json::json!({
            "send_date": valid_suggested_post_send_date()
        }))?)
        .reply_markup(location_markup)
        .send()
        .await?;
    assert_eq!(location.message_id.0, 30);

    let venue = client
        .app()
        .venue(1_i64, 37.5, -122.25, "Tele HQ", "1 Bot Street")
        .google_place_id("place-1")
        .google_place_type("office")
        .reply_to_message(MessageId(81))
        .message_thread_id(32)
        .send()
        .await?;
    assert_eq!(venue.message_id.0, 31);

    let contact = client
        .app()
        .contact(1_i64, "+15550000000", "Tele")
        .last_name("Bot")
        .vcard("BEGIN:VCARD\nEND:VCARD")
        .reply_to_message(MessageId(82))
        .message_thread_id(33)
        .send()
        .await?;
    assert_eq!(contact.message_id.0, 32);

    let poll_markup =
        InlineKeyboardMarkup::single_row(vec![InlineKeyboardButton::callback("Vote", "poll:1")?]);
    let poll = client
        .app()
        .poll(1_i64, "Pick one", ["yes", "no"])?
        .anonymous(false)
        .kind(PollKind::Regular)
        .allows_multiple_answers(true)
        .allows_revoting(true)
        .shuffle_options(true)
        .allow_adding_options(true)
        .hide_results_until_closes(true)
        .members_only(true)
        .open_period(60)
        .description("runtime poll")
        .media(InputPollMedia::location(37.5, -122.25))
        .explanation_media(InputPollMedia::photo("poll-explanation-photo"))
        .reply_to_message(MessageId(83))
        .message_thread_id(34)
        .reply_markup(poll_markup)
        .send()
        .await?;
    assert_eq!(poll.message_id.0, 33);

    let dice = client
        .app()
        .dice(1_i64)
        .reply_to_message(MessageId(84))
        .message_thread_id(35)
        .send()
        .await?;
    assert_eq!(dice.message_id.0, 34);

    let dice_request = client
        .app()
        .dice(1_i64)
        .emoji(DiceEmoji::Darts)
        .into_request();
    assert!(matches!(dice_request.emoji, Some(DiceEmoji::Darts)));

    let action_ok = client
        .app()
        .chat_action(1_i64, ChatAction::Typing)
        .message_thread_id(36)
        .business_connection_id("business-1")
        .send()
        .await?;
    assert!(action_ok);

    let closed_markup = InlineKeyboardMarkup::single_row(vec![InlineKeyboardButton::callback(
        "Closed",
        "poll:closed",
    )?]);
    let stopped = client
        .app()
        .stop_poll(1_i64, MessageId(85))
        .business_connection_id("business-stop-poll")
        .reply_markup(closed_markup)
        .send()
        .await?;
    assert_eq!(stopped.id, "poll-1");

    join_server(server)?;
    Ok(())
}

#[tokio::test]
async fn app_callback_answer_builder_supports_common_options() -> Result<(), DynError> {
    let response = r#"{"ok":true,"result":true}"#;
    const CHECKS: [&str; 5] = [
        "\"callback_query_id\":\"callback-42\"",
        "\"text\":\"Updated\"",
        "\"show_alert\":true",
        "\"url\":\"https://example.com/callback\"",
        "\"cache_time\":30",
    ];
    let (base_url, handle) =
        spawn_server_with_checks("/bot123:abc/answerCallbackQuery", 200, response, &CHECKS)?;

    let client = Client::builder(base_url)?.bot_token("123:abc")?.build()?;
    let ok = client
        .app()
        .callback_answer("callback-42")
        .text("Updated")
        .show_alert(true)
        .url("https://example.com/callback")
        .cache_time(30)
        .send()
        .await?;
    assert!(ok);

    join_server(handle)?;
    Ok(())
}

#[tokio::test]
async fn app_media_builders_support_common_send_options() -> Result<(), DynError> {
    let expectations = vec![
        RequestExpectation::post("/bot123:abc/sendPhoto")
            .contains_case_insensitive("\"chat_id\":1")
            .contains_case_insensitive("\"photo\":\"photo-file-id\"")
            .contains_case_insensitive("\"caption\":\"photo caption\"")
            .contains_case_insensitive("\"parse_mode\":\"MarkdownV2\"")
            .contains_case_insensitive("\"has_spoiler\":true")
            .contains_case_insensitive("\"disable_notification\":true")
            .contains_case_insensitive("\"protect_content\":true")
            .contains_case_insensitive("\"message_thread_id\":11")
            .contains_case_insensitive("\"direct_messages_topic_id\":17")
            .contains_case_insensitive("\"allow_paid_broadcast\":true")
            .contains_case_insensitive("\"message_effect_id\":\"photo-effect\"")
            .contains_case_insensitive("\"suggested_post_parameters\":{")
            .contains_case_insensitive("\"reply_parameters\":{\"message_id\":55")
            .contains_case_insensitive("\"reply_markup\":{\"inline_keyboard\":[[{\"text\":\"View photo\"")
            .contains_case_insensitive("\"callback_data\":\"photo:1\"")
            .respond_json(
                200,
                r#"{"ok":true,"result":{"message_id":10,"date":1710000002,"chat":{"id":1,"type":"private"}}}"#,
            ),
        RequestExpectation::post("/bot123:abc/sendDocument")
            .contains_case_insensitive("\"chat_id\":1")
            .contains_case_insensitive("\"document\":\"document-file-id\"")
            .contains_case_insensitive("\"thumbnail\":\"document-thumb-id\"")
            .contains_case_insensitive("\"caption\":\"document caption\"")
            .contains_case_insensitive("\"parse_mode\":\"MarkdownV2\"")
            .contains_case_insensitive("\"disable_content_type_detection\":true")
            .contains_case_insensitive("\"disable_notification\":true")
            .contains_case_insensitive("\"protect_content\":true")
            .contains_case_insensitive("\"message_thread_id\":12")
            .contains_case_insensitive("\"reply_parameters\":{\"message_id\":56")
            .contains_case_insensitive("\"reply_markup\":{\"inline_keyboard\":[[{\"text\":\"View document\"")
            .contains_case_insensitive("\"callback_data\":\"document:1\"")
            .respond_json(
                200,
                r#"{"ok":true,"result":{"message_id":11,"date":1710000003,"chat":{"id":1,"type":"private"}}}"#,
            ),
        RequestExpectation::post("/bot123:abc/sendVideo")
            .contains_case_insensitive("\"chat_id\":1")
            .contains_case_insensitive("\"video\":\"video-file-id\"")
            .contains_case_insensitive("\"duration\":30")
            .contains_case_insensitive("\"width\":1920")
            .contains_case_insensitive("\"height\":1080")
            .contains_case_insensitive("\"thumbnail\":\"video-thumb-id\"")
            .contains_case_insensitive("\"caption\":\"video caption\"")
            .contains_case_insensitive("\"parse_mode\":\"MarkdownV2\"")
            .contains_case_insensitive("\"supports_streaming\":true")
            .contains_case_insensitive("\"has_spoiler\":true")
            .contains_case_insensitive("\"disable_notification\":true")
            .contains_case_insensitive("\"protect_content\":true")
            .contains_case_insensitive("\"message_thread_id\":13")
            .contains_case_insensitive("\"reply_parameters\":{\"message_id\":57")
            .contains_case_insensitive("\"reply_markup\":{\"inline_keyboard\":[[{\"text\":\"View video\"")
            .contains_case_insensitive("\"callback_data\":\"video:1\"")
            .respond_json(
                200,
                r#"{"ok":true,"result":{"message_id":12,"date":1710000004,"chat":{"id":1,"type":"private"}}}"#,
            ),
    ];
    let server = FakeTelegramServer::start(expectations)?;

    let client = Client::builder(server.base_url())?
        .bot_token("123:abc")?
        .build()?;

    let photo_markup = InlineKeyboardMarkup::single_row(vec![InlineKeyboardButton::callback(
        "View photo",
        "photo:1",
    )?]);
    let photo = client
        .app()
        .photo(1_i64, "photo-file-id")
        .caption("photo caption")
        .parse_mode(ParseMode::MarkdownV2)
        .has_spoiler(true)
        .reply_to_message(MessageId(55))
        .message_thread_id(11)
        .direct_messages_topic_id(17)
        .disable_notification(true)
        .protect_content(true)
        .allow_paid_broadcast(true)
        .message_effect_id("photo-effect")
        .suggested_post_parameters(SuggestedPostParameters::new(serde_json::json!({
            "send_date": valid_suggested_post_send_date()
        }))?)
        .reply_markup(photo_markup)
        .send()
        .await?;
    assert_eq!(photo.message_id.0, 10);

    let document_markup = InlineKeyboardMarkup::single_row(vec![InlineKeyboardButton::callback(
        "View document",
        "document:1",
    )?]);
    let document = client
        .app()
        .document(1_i64, "document-file-id")
        .thumbnail("document-thumb-id")
        .caption("document caption")
        .parse_mode(ParseMode::MarkdownV2)
        .disable_content_type_detection(true)
        .reply_to_message(MessageId(56))
        .message_thread_id(12)
        .disable_notification(true)
        .protect_content(true)
        .reply_markup(document_markup)
        .send()
        .await?;
    assert_eq!(document.message_id.0, 11);

    let video_markup = InlineKeyboardMarkup::single_row(vec![InlineKeyboardButton::callback(
        "View video",
        "video:1",
    )?]);
    let video = client
        .app()
        .video(1_i64, "video-file-id")
        .duration(30)
        .width(1920)
        .height(1080)
        .thumbnail("video-thumb-id")
        .caption("video caption")
        .parse_mode(ParseMode::MarkdownV2)
        .supports_streaming(true)
        .has_spoiler(true)
        .reply_to_message(MessageId(57))
        .message_thread_id(13)
        .disable_notification(true)
        .protect_content(true)
        .reply_markup(video_markup)
        .send()
        .await?;
    assert_eq!(video.message_id.0, 12);

    join_server(server)?;
    Ok(())
}

#[tokio::test]
async fn app_richer_media_builders_support_common_send_options() -> Result<(), DynError> {
    let expectations = vec![
        RequestExpectation::post("/bot123:abc/sendAudio")
            .contains_case_insensitive("\"chat_id\":1")
            .contains_case_insensitive("\"audio\":\"audio-file-id\"")
            .contains_case_insensitive("\"caption\":\"audio caption\"")
            .contains_case_insensitive("\"parse_mode\":\"MarkdownV2\"")
            .contains_case_insensitive("\"duration\":120")
            .contains_case_insensitive("\"performer\":\"tele band\"")
            .contains_case_insensitive("\"title\":\"tele song\"")
            .contains_case_insensitive("\"thumbnail\":\"audio-thumb-id\"")
            .contains_case_insensitive("\"message_thread_id\":14")
            .contains_case_insensitive("\"reply_parameters\":{\"message_id\":58")
            .contains_case_insensitive("\"reply_markup\":{\"inline_keyboard\":[[{\"text\":\"Play audio\"")
            .contains_case_insensitive("\"callback_data\":\"audio:1\"")
            .respond_json(
                200,
                r#"{"ok":true,"result":{"message_id":13,"date":1710000005,"chat":{"id":1,"type":"private"}}}"#,
            ),
        RequestExpectation::post("/bot123:abc/sendAnimation")
            .contains_case_insensitive("\"chat_id\":1")
            .contains_case_insensitive("\"animation\":\"animation-file-id\"")
            .contains_case_insensitive("\"caption\":\"animation caption\"")
            .contains_case_insensitive("\"parse_mode\":\"MarkdownV2\"")
            .contains_case_insensitive("\"duration\":7")
            .contains_case_insensitive("\"width\":480")
            .contains_case_insensitive("\"height\":320")
            .contains_case_insensitive("\"thumbnail\":\"animation-thumb-id\"")
            .contains_case_insensitive("\"has_spoiler\":true")
            .contains_case_insensitive("\"message_thread_id\":15")
            .contains_case_insensitive("\"reply_parameters\":{\"message_id\":59")
            .contains_case_insensitive("\"reply_markup\":{\"inline_keyboard\":[[{\"text\":\"Play animation\"")
            .contains_case_insensitive("\"callback_data\":\"animation:1\"")
            .respond_json(
                200,
                r#"{"ok":true,"result":{"message_id":14,"date":1710000006,"chat":{"id":1,"type":"private"}}}"#,
            ),
        RequestExpectation::post("/bot123:abc/sendVoice")
            .contains_case_insensitive("\"chat_id\":1")
            .contains_case_insensitive("\"voice\":\"voice-file-id\"")
            .contains_case_insensitive("\"caption\":\"voice caption\"")
            .contains_case_insensitive("\"parse_mode\":\"MarkdownV2\"")
            .contains_case_insensitive("\"duration\":25")
            .contains_case_insensitive("\"message_thread_id\":16")
            .contains_case_insensitive("\"reply_parameters\":{\"message_id\":60")
            .contains_case_insensitive("\"reply_markup\":{\"inline_keyboard\":[[{\"text\":\"Play voice\"")
            .contains_case_insensitive("\"callback_data\":\"voice:1\"")
            .respond_json(
                200,
                r#"{"ok":true,"result":{"message_id":15,"date":1710000007,"chat":{"id":1,"type":"private"}}}"#,
            ),
        RequestExpectation::post("/bot123:abc/sendVideoNote")
            .contains_case_insensitive("\"chat_id\":1")
            .contains_case_insensitive("\"video_note\":\"video-note-file-id\"")
            .contains_case_insensitive("\"duration\":12")
            .contains_case_insensitive("\"length\":240")
            .contains_case_insensitive("\"thumbnail\":\"video-note-thumb-id\"")
            .contains_case_insensitive("\"message_thread_id\":19")
            .contains_case_insensitive("\"reply_parameters\":{\"message_id\":63")
            .contains_case_insensitive("\"reply_markup\":{\"inline_keyboard\":[[{\"text\":\"Play video note\"")
            .contains_case_insensitive("\"callback_data\":\"video-note:1\"")
            .respond_json(
                200,
                r#"{"ok":true,"result":{"message_id":19,"date":1710000011,"chat":{"id":1,"type":"private"}}}"#,
            ),
        RequestExpectation::post("/bot123:abc/sendSticker")
            .contains_case_insensitive("\"chat_id\":1")
            .contains_case_insensitive("\"sticker\":\"sticker-file-id\"")
            .contains_case_insensitive("\"emoji\":\":fire:\"")
            .contains_case_insensitive("\"message_thread_id\":17")
            .respond_json(
                200,
                r#"{"ok":true,"result":{"message_id":16,"date":1710000008,"chat":{"id":1,"type":"private"}}}"#,
            ),
        RequestExpectation::post("/bot123:abc/sendMediaGroup")
            .contains_case_insensitive("\"chat_id\":1")
            .contains_case_insensitive("\"media\":[{\"type\":\"photo\",\"media\":\"group-photo-file-id\",\"caption\":\"group photo caption\"")
            .contains_case_insensitive("\"type\":\"video\",\"media\":\"group-video-file-id\",\"caption\":\"group video caption\"")
            .contains_case_insensitive("\"supports_streaming\":true")
            .contains_case_insensitive("\"message_thread_id\":18")
            .contains_case_insensitive("\"reply_parameters\":{\"message_id\":62")
            .respond_json(
                200,
                r#"{"ok":true,"result":[{"message_id":17,"date":1710000009,"chat":{"id":1,"type":"private"}},{"message_id":18,"date":1710000010,"chat":{"id":1,"type":"private"}}]}"#,
            ),
    ];
    let server = FakeTelegramServer::start(expectations)?;

    let client = Client::builder(server.base_url())?
        .bot_token("123:abc")?
        .build()?;

    let audio_markup = InlineKeyboardMarkup::single_row(vec![InlineKeyboardButton::callback(
        "Play audio",
        "audio:1",
    )?]);
    let audio = client
        .app()
        .audio(1_i64, "audio-file-id")
        .caption("audio caption")
        .parse_mode(ParseMode::MarkdownV2)
        .duration(120)
        .performer("tele band")
        .title("tele song")
        .thumbnail("audio-thumb-id")
        .reply_to_message(MessageId(58))
        .message_thread_id(14)
        .reply_markup(audio_markup)
        .send()
        .await?;
    assert_eq!(audio.message_id.0, 13);

    let animation_markup = InlineKeyboardMarkup::single_row(vec![InlineKeyboardButton::callback(
        "Play animation",
        "animation:1",
    )?]);
    let animation = client
        .app()
        .animation(1_i64, "animation-file-id")
        .caption("animation caption")
        .parse_mode(ParseMode::MarkdownV2)
        .duration(7)
        .width(480)
        .height(320)
        .thumbnail("animation-thumb-id")
        .has_spoiler(true)
        .reply_to_message(MessageId(59))
        .message_thread_id(15)
        .reply_markup(animation_markup)
        .send()
        .await?;
    assert_eq!(animation.message_id.0, 14);

    let voice_markup = InlineKeyboardMarkup::single_row(vec![InlineKeyboardButton::callback(
        "Play voice",
        "voice:1",
    )?]);
    let voice = client
        .app()
        .voice(1_i64, "voice-file-id")
        .caption("voice caption")
        .parse_mode(ParseMode::MarkdownV2)
        .duration(25)
        .reply_to_message(MessageId(60))
        .message_thread_id(16)
        .reply_markup(voice_markup)
        .send()
        .await?;
    assert_eq!(voice.message_id.0, 15);

    let video_note_markup = InlineKeyboardMarkup::single_row(vec![InlineKeyboardButton::callback(
        "Play video note",
        "video-note:1",
    )?]);
    let video_note = client
        .app()
        .video_note(1_i64, "video-note-file-id")
        .duration(12)
        .length(240)
        .thumbnail("video-note-thumb-id")
        .reply_to_message(MessageId(63))
        .message_thread_id(19)
        .reply_markup(video_note_markup)
        .send()
        .await?;
    assert_eq!(video_note.message_id.0, 19);

    let sticker_markup = InlineKeyboardMarkup::single_row(vec![InlineKeyboardButton::callback(
        "Review sticker",
        "sticker:1",
    )?]);
    let sticker = client
        .app()
        .sticker(1_i64, "sticker-file-id")
        .emoji(":fire:")
        .reply_to_message(MessageId(61))
        .message_thread_id(17)
        .reply_markup(sticker_markup)
        .send()
        .await?;
    assert_eq!(sticker.message_id.0, 16);

    let group = client
        .app()
        .media_group(
            1_i64,
            vec![
                serde_json::from_value::<InputMediaGroupItem>(serde_json::json!({
                    "type": "photo",
                    "media": "group-photo-file-id",
                    "caption": "group photo caption",
                    "parse_mode": "MarkdownV2"
                }))?,
                serde_json::from_value::<InputMediaGroupItem>(serde_json::json!({
                    "type": "video",
                    "media": "group-video-file-id",
                    "caption": "group video caption",
                    "parse_mode": "MarkdownV2",
                    "width": 1920,
                    "height": 1080,
                    "duration": 30,
                    "supports_streaming": true
                }))?,
            ],
        )?
        .reply_to_message(MessageId(62))
        .message_thread_id(18)
        .send()
        .await?;
    assert_eq!(group.len(), 2);
    assert_eq!(group[0].message_id.0, 17);

    join_server(server)?;
    Ok(())
}

#[tokio::test]
async fn app_sticker_builder_supports_common_send_options() -> Result<(), DynError> {
    let client = Client::builder("https://api.telegram.org")?
        .bot_token("123:abc")?
        .build()?;
    let markup = InlineKeyboardMarkup::single_row(vec![InlineKeyboardButton::callback(
        "Review sticker",
        "sticker:1",
    )?]);

    let request = client
        .app()
        .sticker(1_i64, "sticker-file-id")
        .emoji(":fire:")
        .reply_to_message(MessageId(61))
        .message_thread_id(17)
        .reply_markup(markup)
        .into_request();

    assert_eq!(request.emoji.as_deref(), Some(":fire:"));
    assert_eq!(request.message_thread_id, Some(17));
    assert_eq!(
        serde_json::to_value(request.reply_parameters.as_ref())?,
        serde_json::json!({"message_id":61})
    );
    assert_eq!(
        serde_json::to_value(request.reply_markup.as_ref())?,
        serde_json::json!({
            "inline_keyboard": [[{"text":"Review sticker","callback_data":"sticker:1"}]]
        })
    );

    Ok(())
}

#[tokio::test]
async fn app_reply_text_uses_join_request_user_chat_id() -> Result<(), DynError> {
    let response = r#"{"ok":true,"result":{"message_id":8,"date":1710000001,"chat":{"id":7001,"type":"private"},"text":"hello"}}"#;
    let (base_url, handle) = spawn_server_with_checks(
        "/bot123:abc/sendMessage",
        200,
        response,
        &[
            "\"chat_id\":7001",
            "\"text\":\"hello\"",
            "\"disable_notification\":true",
        ],
    )?;

    let client = Client::builder(base_url)?.bot_token("123:abc")?.build()?;
    let update: Update = serde_json::from_value(serde_json::json!({
        "update_id": 43,
        "chat_join_request": {
            "chat": {"id": -10010, "type": "supergroup", "title": "mods"},
            "from": {"id": 701, "is_bot": false, "first_name": "candidate"},
            "user_chat_id": 7001,
            "date": 1710000001
        }
    }))?;

    let sent = client
        .app()
        .reply(&update, "hello")?
        .disable_notification(true)
        .send()
        .await?;
    assert_eq!(sent.message_id.0, 8);

    join_server(handle)?;
    Ok(())
}

#[tokio::test]
async fn app_reply_text_quotes_source_message() -> Result<(), DynError> {
    let response = r#"{"ok":true,"result":{"message_id":9,"date":1710000002,"chat":{"id":-10010,"type":"supergroup","title":"mods"},"text":"quoted"}}"#;
    let (base_url, handle) = spawn_server_with_checks(
        "/bot123:abc/sendMessage",
        200,
        response,
        &[
            "\"chat_id\":-10010",
            "\"text\":\"quoted\"",
            "\"reply_parameters\":{\"message_id\":55",
        ],
    )?;

    let client = Client::builder(base_url)?.bot_token("123:abc")?.build()?;
    let update: Update = serde_json::from_value(serde_json::json!({
        "update_id": 44,
        "message": {
            "message_id": 55,
            "date": 1710000001,
            "chat": {"id": -10010, "type": "supergroup", "title": "mods"},
            "from": {"id": 701, "is_bot": false, "first_name": "candidate"},
            "text": "/start"
        }
    }))?;

    let sent = client.app().reply(&update, "quoted")?.send().await?;
    assert_eq!(sent.message_id.0, 9);

    join_server(handle)?;
    Ok(())
}

#[test]
fn app_reply_text_preserves_source_message_thread() -> Result<(), DynError> {
    let client = Client::builder("http://127.0.0.1:9")?
        .bot_token("123:abc")?
        .build()?;
    let update: Update = serde_json::from_value(serde_json::json!({
        "update_id": 4401,
        "message": {
            "message_id": 55,
            "message_thread_id": 88,
            "date": 1710000001,
            "chat": {"id": -10010, "type": "supergroup", "title": "mods"},
            "from": {"id": 701, "is_bot": false, "first_name": "candidate"},
            "text": "/start"
        }
    }))?;

    let request = client.app().reply(&update, "quoted")?.into_request();

    assert_eq!(request.chat_id, ChatId::Id(-10010));
    assert_eq!(request.message_thread_id, Some(88));
    assert_eq!(
        request
            .reply_parameters
            .as_ref()
            .map(|parameters| parameters.message_id),
        Some(MessageId(55))
    );
    Ok(())
}

#[tokio::test]
async fn app_reply_text_targets_inaccessible_callback_message() -> Result<(), DynError> {
    let response = r#"{"ok":true,"result":{"message_id":9,"date":1710000002,"chat":{"id":-10010,"type":"supergroup","title":"mods"},"text":"quoted"}}"#;
    let (base_url, handle) = spawn_server_with_checks(
        "/bot123:abc/sendMessage",
        200,
        response,
        &[
            "\"chat_id\":-10010",
            "\"text\":\"quoted\"",
            "\"reply_parameters\":{\"message_id\":55",
        ],
    )?;

    let client = Client::builder(base_url)?.bot_token("123:abc")?.build()?;
    let update: Update = serde_json::from_value(serde_json::json!({
        "update_id": 45,
        "callback_query": {
            "id": "cb-inaccessible",
            "from": {"id": 701, "is_bot": false, "first_name": "candidate"},
            "message": {
                "message_id": 55,
                "date": 0,
                "chat": {"id": -10010, "type": "supergroup", "title": "mods"}
            },
            "chat_instance": "ci",
            "data": "payload"
        }
    }))?;

    let sent = client.app().reply(&update, "quoted")?.send().await?;
    assert_eq!(sent.message_id.0, 9);

    join_server(handle)?;
    Ok(())
}

#[tokio::test]
async fn app_reply_text_supports_business_message_updates() -> Result<(), DynError> {
    let response = r#"{"ok":true,"result":{"message_id":10,"date":1710000003,"chat":{"id":7001,"type":"private","first_name":"customer"},"text":"business reply"}}"#;
    let (base_url, handle) = spawn_server_with_checks(
        "/bot123:abc/sendMessage",
        200,
        response,
        &[
            "\"business_connection_id\":\"business-1\"",
            "\"chat_id\":7001",
            "\"text\":\"business reply\"",
            "\"reply_parameters\":{\"message_id\":77",
        ],
    )?;

    let client = Client::builder(base_url)?.bot_token("123:abc")?.build()?;
    let update: Update = serde_json::from_value(serde_json::json!({
        "update_id": 46,
        "business_message": {
            "message_id": 77,
            "business_connection_id": "business-1",
            "date": 1710000002,
            "chat": {"id": 7001, "type": "private", "first_name": "customer"},
            "from": {"id": 7001, "is_bot": false, "first_name": "customer"},
            "text": "business hello"
        }
    }))?;

    let sent = client
        .app()
        .reply(&update, "business reply")?
        .send()
        .await?;
    assert_eq!(sent.message_id.0, 10);

    join_server(handle)?;
    Ok(())
}

#[tokio::test]
async fn app_reply_text_preserves_deleted_business_message_context() -> Result<(), DynError> {
    let response = r#"{"ok":true,"result":{"message_id":11,"date":1710000004,"chat":{"id":7001,"type":"private","first_name":"customer"},"text":"business cleanup"}}"#;
    let (base_url, handle) = spawn_server_with_checks(
        "/bot123:abc/sendMessage",
        200,
        response,
        &[
            "\"business_connection_id\":\"business-1\"",
            "\"chat_id\":7001",
            "\"text\":\"business cleanup\"",
        ],
    )?;

    let client = Client::builder(base_url)?.bot_token("123:abc")?.build()?;
    let update: Update = serde_json::from_value(serde_json::json!({
        "update_id": 49,
        "deleted_business_messages": {
            "business_connection_id": "business-1",
            "chat": {"id": 7001, "type": "private", "first_name": "customer"},
            "message_ids": [77, 78]
        }
    }))?;

    let sent = client
        .app()
        .reply(&update, "business cleanup")?
        .send()
        .await?;
    assert_eq!(sent.message_id.0, 11);

    join_server(handle)?;
    Ok(())
}

#[tokio::test]
async fn app_reply_text_supports_business_connection_updates() -> Result<(), DynError> {
    let response = r#"{"ok":true,"result":{"message_id":12,"date":1710000005,"chat":{"id":7101,"type":"private","first_name":"customer"},"text":"business connected"}}"#;
    let (base_url, handle) = spawn_server_with_checks(
        "/bot123:abc/sendMessage",
        200,
        response,
        &[
            "\"business_connection_id\":\"business-2\"",
            "\"chat_id\":7101",
            "\"text\":\"business connected\"",
        ],
    )?;

    let client = Client::builder(base_url)?.bot_token("123:abc")?.build()?;
    let update: Update = serde_json::from_value(serde_json::json!({
        "update_id": 50,
        "business_connection": {
            "id": "business-2",
            "user": {"id": 7101, "is_bot": false, "first_name": "customer"},
            "user_chat_id": 7101,
            "date": 1710000005,
            "is_enabled": true
        }
    }))?;

    let sent = client
        .app()
        .reply(&update, "business connected")?
        .send()
        .await?;
    assert_eq!(sent.message_id.0, 12);

    join_server(handle)?;
    Ok(())
}

#[tokio::test]
async fn app_reply_text_quotes_reaction_source_message() -> Result<(), DynError> {
    let response = r#"{"ok":true,"result":{"message_id":13,"date":1710000006,"chat":{"id":-10012,"type":"supergroup","title":"mods"},"text":"reaction reply"}}"#;
    let (base_url, handle) = spawn_server_with_checks(
        "/bot123:abc/sendMessage",
        200,
        response,
        &[
            "\"chat_id\":-10012",
            "\"text\":\"reaction reply\"",
            "\"reply_parameters\":{\"message_id\":88",
        ],
    )?;

    let client = Client::builder(base_url)?.bot_token("123:abc")?.build()?;
    let update: Update = serde_json::from_value(serde_json::json!({
        "update_id": 51,
        "message_reaction": {
            "chat": {"id": -10012, "type": "supergroup", "title": "mods"},
            "message_id": 88,
            "user": {"id": 7102, "is_bot": false, "first_name": "reactor"},
            "date": 1710000006,
            "old_reaction": [],
            "new_reaction": [{"type": "emoji", "emoji": "👍"}]
        }
    }))?;

    let sent = client
        .app()
        .reply(&update, "reaction reply")?
        .send()
        .await?;
    assert_eq!(sent.message_id.0, 13);

    join_server(handle)?;
    Ok(())
}

#[tokio::test]
async fn app_reply_text_supports_chat_boost_updates() -> Result<(), DynError> {
    let response = r#"{"ok":true,"result":{"message_id":14,"date":1710000007,"chat":{"id":-10013,"type":"supergroup","title":"mods"},"text":"boost reply"}}"#;
    let (base_url, handle) = spawn_server_with_checks(
        "/bot123:abc/sendMessage",
        200,
        response,
        &["\"chat_id\":-10013", "\"text\":\"boost reply\""],
    )?;

    let client = Client::builder(base_url)?.bot_token("123:abc")?.build()?;
    let update: Update = serde_json::from_value(serde_json::json!({
        "update_id": 52,
        "chat_boost": {
            "chat": {"id": -10013, "type": "supergroup", "title": "mods"},
            "boost": {
                "boost_id": "boost-2",
                "add_date": 1710000007,
                "expiration_date": 1710086407,
                "source": {
                    "source": "premium",
                    "user": {"id": 7103, "is_bot": false, "first_name": "booster"}
                }
            }
        }
    }))?;

    let sent = client.app().reply(&update, "boost reply")?.send().await?;
    assert_eq!(sent.message_id.0, 14);

    join_server(handle)?;
    Ok(())
}

#[tokio::test]
async fn app_business_media_replies_preserve_reply_context() -> Result<(), DynError> {
    let expectations = vec![
        RequestExpectation::post("/bot123:abc/sendPhoto")
            .contains_case_insensitive("\"business_connection_id\":\"business-1\"")
            .contains_case_insensitive("\"chat_id\":7001")
            .contains_case_insensitive("\"reply_parameters\":{\"message_id\":77")
            .contains_case_insensitive("\"photo\":\"photo-file-id\"")
            .respond_json(
                200,
                r#"{"ok":true,"result":{"message_id":11,"date":1710000004,"chat":{"id":7001,"type":"private","first_name":"customer"},"photo":[{"file_id":"photo_1","file_unique_id":"photo_unique_1","width":10,"height":10}]}}"#,
            ),
        RequestExpectation::post("/bot123:abc/sendSticker")
            .contains_case_insensitive("\"business_connection_id\":\"business-1\"")
            .contains_case_insensitive("\"chat_id\":7001")
            .contains_case_insensitive("\"reply_parameters\":{\"message_id\":77")
            .contains_case_insensitive("\"sticker\":\"sticker-file-id\"")
            .respond_json(
                200,
                r#"{"ok":true,"result":{"message_id":12,"date":1710000005,"chat":{"id":7001,"type":"private","first_name":"customer"}}}"#,
            ),
        RequestExpectation::post("/bot123:abc/sendMediaGroup")
            .contains_case_insensitive("\"business_connection_id\":\"business-1\"")
            .contains_case_insensitive("\"chat_id\":7001")
            .contains_case_insensitive("\"reply_parameters\":{\"message_id\":77")
            .contains_case_insensitive("\"media\"")
            .respond_json(
                200,
                r#"{"ok":true,"result":[{"message_id":13,"date":1710000006,"chat":{"id":7001,"type":"private","first_name":"customer"}},{"message_id":14,"date":1710000006,"chat":{"id":7001,"type":"private","first_name":"customer"}}]}"#,
            ),
    ];
    let server = FakeTelegramServer::start(expectations)?;
    let base_url = server.base_url().to_owned();

    let client = Client::builder(base_url)?.bot_token("123:abc")?.build()?;
    let update: Update = serde_json::from_value(serde_json::json!({
        "update_id": 48,
        "business_message": {
            "message_id": 77,
            "business_connection_id": "business-1",
            "date": 1710000002,
            "chat": {"id": 7001, "type": "private", "first_name": "customer"},
            "from": {"id": 7001, "is_bot": false, "first_name": "customer"},
            "text": "business hello"
        }
    }))?;

    let photo = client
        .app()
        .reply_photo(&update, "photo-file-id")?
        .send()
        .await?;
    assert_eq!(photo.message_id.0, 11);

    let sticker = client
        .app()
        .reply_sticker(&update, "sticker-file-id")?
        .send()
        .await?;
    assert_eq!(sticker.message_id.0, 12);

    let album = client
        .app()
        .reply_media_group(
            &update,
            [
                InputMediaPhoto::new("media-photo-1"),
                InputMediaPhoto::new("media-photo-2"),
            ],
        )?
        .send()
        .await?;
    assert_eq!(album.len(), 2);
    assert_eq!(album[0].message_id.0, 13);

    join_server(server)?;
    Ok(())
}

#[tokio::test]
async fn app_reply_text_rejects_guest_message_updates() -> Result<(), DynError> {
    let client = Client::builder("http://127.0.0.1:9")?
        .bot_token("123:abc")?
        .build()?;
    let update: Update = serde_json::from_value(serde_json::json!({
        "update_id": 47,
        "guest_message": {
            "message_id": 78,
            "guest_query_id": "guest-1",
            "date": 1710000003,
            "chat": {"id": 8001, "type": "private", "first_name": "guest"},
            "from": {"id": 8001, "is_bot": false, "first_name": "guest"},
            "text": "guest hello"
        }
    }))?;

    assert!(matches!(
        client.app().reply(&update, "guest reply"),
        Err(Error::InvalidRequest { reason }) if reason.contains("answerGuestQuery")
    ));

    Ok(())
}

#[tokio::test]
async fn transport_error_redacts_token() -> Result<(), DynError> {
    let client = Client::builder("http://127.0.0.1:9")?
        .bot_token("123:abc")?
        .request_timeout(Duration::from_millis(100))?
        .total_timeout(Some(Duration::from_millis(300)))?
        .build()?;

    let err = match client.bot().get_me().await {
        Ok(_) => {
            return Err("expected transport error".into());
        }
        Err(err) => err,
    };

    let text = err.to_string();
    assert!(!text.contains("123:abc"));
    assert!(err.request_id().is_some());
    Ok(())
}

#[tokio::test]
async fn build_configuration_error_is_not_mapped_as_transport() -> Result<(), DynError> {
    let error = match Client::builder("https://api.telegram.org")?
        .bot_token("123:abc")?
        .no_proxy(["example.com", "[::1]not-a-port"])
    {
        Ok(_) => return Err("expected configuration failure".into()),
        Err(error) => error,
    };

    assert!(matches!(error, Error::Configuration { .. }));
    assert_eq!(error.classification(), ErrorClass::Configuration);
    assert!(!error.is_retryable());
    Ok(())
}

#[tokio::test]
async fn set_and_get_my_commands_success() -> Result<(), DynError> {
    let set_response = r#"{"ok":true,"result":true}"#;
    let (set_base_url, set_handle) = spawn_server("/bot123:abc/setMyCommands", 200, set_response)?;

    let set_client = Client::builder(set_base_url)?
        .bot_token("123:abc")?
        .build()?;
    let set_request = SetMyCommandsRequest::new(vec![BotCommand::new("start", "start the bot")?])?;
    let set_result = set_client.bot().set_my_commands(&set_request).await?;
    assert!(set_result);
    join_server(set_handle)?;

    let get_response =
        r#"{"ok":true,"result":[{"command":"start","description":"start the bot"}]}"#;
    let (get_base_url, get_handle) = spawn_server("/bot123:abc/getMyCommands", 200, get_response)?;

    let get_client = Client::builder(get_base_url)?
        .bot_token("123:abc")?
        .build()?;
    let get_request = GetMyCommandsRequest::default();
    let commands = get_client.bot().get_my_commands(&get_request).await?;
    assert_eq!(commands.len(), 1);
    assert_eq!(commands[0].command, "start");
    join_server(get_handle)?;

    Ok(())
}

#[tokio::test]
async fn bootstrap_skips_unchanged_commands_and_menu_button() -> Result<(), DynError> {
    let script = vec![
        (
            "/bot123:abc/getMyCommands",
            200,
            r#"{"ok":true,"result":[{"command":"start","description":"start the bot","future_command_field":"kept"}]}"#,
        ),
        (
            "/bot123:abc/getChatMenuButton",
            200,
            r#"{"ok":true,"result":{"type":"commands","future_menu_button_field":"kept"}}"#,
        ),
    ];
    let (base_url, handle) = spawn_server_script(script)?;

    let client = Client::builder(base_url)?.bot_token("123:abc")?.build()?;
    let plan = BootstrapPlan::new()
        .commands(vec![BotCommand::new("start", "start the bot")?])?
        .menu_button(MenuButtonConfig::commands());

    let outcome = client.control().setup().bootstrap(&plan).await;
    assert!(outcome.is_success());
    let Some(commands) = outcome.report.commands.as_ref() else {
        return Err("expected commands step report".into());
    };
    assert_eq!(commands.applied, Some(false));
    assert_eq!(commands.synced, Some(true));
    let Some(menu_button) = outcome.report.menu_button.as_ref() else {
        return Err("expected menu button step report".into());
    };
    assert_eq!(menu_button.applied, Some(false));
    assert_eq!(menu_button.synced, Some(true));

    join_server(handle)?;
    Ok(())
}

#[cfg(feature = "bot")]
#[derive(Clone, Debug)]
enum DemoCommand {
    Start,
}

#[cfg(feature = "bot")]
impl tele::bot::BotCommands for DemoCommand {
    fn parse(command: &str, _args: &str) -> Option<Self> {
        if command == "start" {
            Some(Self::Start)
        } else {
            None
        }
    }

    fn descriptions() -> &'static [tele::bot::CommandDescription] {
        &[tele::bot::CommandDescription {
            command: "start",
            description: "start command",
        }]
    }
}

#[cfg(feature = "bot")]
#[tokio::test]
async fn setup_set_typed_commands_with_scope_and_language() -> Result<(), DynError> {
    let response = r#"{"ok":true,"result":true}"#;
    const CHECKS: [&str; 4] = [
        "\"commands\":[{\"command\":\"start\",\"description\":\"start command\"}]",
        "\"scope\":{\"type\":\"all_private_chats\"}",
        "\"language_code\":\"zh-hans\"",
        "POST /bot123:abc/setMyCommands HTTP/1.1",
    ];
    let (base_url, handle) =
        spawn_server_with_checks("/bot123:abc/setMyCommands", 200, response, &CHECKS)?;

    let client = Client::builder(base_url)?.bot_token("123:abc")?.build()?;
    let applied = client
        .control()
        .setup()
        .set_typed_commands_with_options::<DemoCommand>(
            Some(BotCommandScope::AllPrivateChats),
            Some("zh-hans".to_owned()),
        )
        .await?;
    assert!(applied);

    join_server(handle)?;
    Ok(())
}

#[cfg(feature = "bot")]
#[tokio::test]
async fn bootstrap_plan_typed_commands_with_scope_and_language() -> Result<(), DynError> {
    let expectations = vec![
        RequestExpectation::post("/bot123:abc/getMyCommands")
            .contains_case_insensitive("\"scope\":{\"type\":\"all_private_chats\"}")
            .contains_case_insensitive("\"language_code\":\"zh-hans\"")
            .respond_json(200, r#"{"ok":true,"result":[]}"#),
        RequestExpectation::post("/bot123:abc/setMyCommands")
            .contains_case_insensitive(
                "\"commands\":[{\"command\":\"start\",\"description\":\"start command\"}]",
            )
            .contains_case_insensitive("\"scope\":{\"type\":\"all_private_chats\"}")
            .contains_case_insensitive("\"language_code\":\"zh-hans\"")
            .respond_json(200, r#"{"ok":true,"result":true}"#),
    ];
    let server = FakeTelegramServer::start(expectations)?;

    let client = Client::builder(server.base_url())?
        .bot_token("123:abc")?
        .build()?;
    let plan = BootstrapPlan::new().typed_commands_with_options::<DemoCommand>(
        Some(BotCommandScope::AllPrivateChats),
        Some("zh-hans".to_owned()),
    )?;

    let outcome = client.control().setup().bootstrap(&plan).await;
    assert!(outcome.is_success());
    let Some(commands) = outcome.report.commands.as_ref() else {
        return Err("expected commands step report".into());
    };
    assert_eq!(commands.applied, Some(true));
    assert_eq!(commands.synced, Some(true));

    join_server(server)?;
    Ok(())
}

#[tokio::test]
async fn bootstrap_retry_can_continue_on_failure() -> Result<(), DynError> {
    let client = Client::builder("http://127.0.0.1:9")?
        .bot_token("123:abc")?
        .request_timeout(Duration::from_millis(100))?
        .total_timeout(Some(Duration::from_millis(300)))?
        .build()?;

    let plan = BootstrapPlan::new().commands(vec![BotCommand::new("start", "start bot")?])?;
    let outcome = client
        .control()
        .setup()
        .bootstrap_with_retry(
            &plan,
            BootstrapRetryPolicy {
                max_attempts: 1,
                continue_on_failure: true,
                ..BootstrapRetryPolicy::default()
            },
        )
        .await;
    assert!(outcome.is_success());
    let Some(commands) = outcome.report.commands.as_ref() else {
        return Err("expected commands step report".into());
    };
    assert_eq!(commands.applied, Some(false));
    assert_eq!(commands.diagnostics.status, BootstrapStepStatus::Warned);

    Ok(())
}

#[tokio::test]
async fn setup_bootstrap_warns_on_retryable_get_me_after_retries() -> Result<(), DynError> {
    let client = Client::builder("http://127.0.0.1:9")?
        .bot_token("123:abc")?
        .request_timeout(Duration::from_millis(40))?
        .total_timeout(Some(Duration::from_millis(120)))?
        .build()?;
    let plan = BootstrapPlan::new().warn_and_continue_on_retryable_get_me();
    let outcome = client
        .control()
        .setup()
        .bootstrap_with_retry(
            &plan,
            BootstrapRetryPolicy {
                max_attempts: 2,
                base_backoff: Duration::from_millis(1),
                max_backoff: Duration::from_millis(5),
                continue_on_failure: false,
                ..BootstrapRetryPolicy::default()
            },
        )
        .await;

    assert!(outcome.is_success());
    assert!(outcome.error.is_none());
    assert!(outcome.report.me.value.is_none());
    assert_eq!(
        outcome.report.me.diagnostics.status,
        BootstrapStepStatus::Warned
    );
    assert_eq!(
        outcome.report.me.diagnostics.phase,
        Some(BootstrapStepPhase::Fetch)
    );
    assert_eq!(
        outcome.report.me.diagnostics.classification,
        Some(ErrorClass::Transport)
    );
    assert!(outcome.report.me.diagnostics.retryable);
    assert_eq!(outcome.report.me.diagnostics.attempt_count, 2);
    assert!(outcome.report.me.diagnostics.request_id.is_some());

    Ok(())
}

#[tokio::test]
async fn setup_bootstrap_reports_unchanged_steps() -> Result<(), DynError> {
    let script = vec![
        (
            "/bot123:abc/getMyCommands",
            200,
            r#"{"ok":true,"result":[{"command":"start","description":"start the bot"}]}"#,
        ),
        (
            "/bot123:abc/getChatMenuButton",
            200,
            r#"{"ok":true,"result":{"type":"commands","future_menu_button_field":"kept"}}"#,
        ),
    ];
    let (base_url, handle) = spawn_server_script(script)?;

    let client = Client::builder(base_url)?.bot_token("123:abc")?.build()?;
    let plan = BootstrapPlan::new()
        .commands(vec![BotCommand::new("start", "start the bot")?])?
        .menu_button(MenuButtonConfig::commands());

    let outcome = client
        .control()
        .setup()
        .bootstrap_with_retry(&plan, BootstrapRetryPolicy::default())
        .await;

    assert!(outcome.is_success());
    let Some(commands) = outcome.report.commands.as_ref() else {
        return Err("expected commands step report".into());
    };
    assert_eq!(commands.applied, Some(false));
    assert_eq!(commands.synced, Some(true));
    assert_eq!(commands.diagnostics.status, BootstrapStepStatus::Unchanged);
    assert_eq!(commands.diagnostics.phase, Some(BootstrapStepPhase::Check));
    assert_eq!(commands.diagnostics.attempt_count, 1);

    let Some(menu_button) = outcome.report.menu_button.as_ref() else {
        return Err("expected menu button step report".into());
    };
    assert_eq!(menu_button.applied, Some(false));
    assert_eq!(menu_button.synced, Some(true));
    assert_eq!(
        menu_button.diagnostics.status,
        BootstrapStepStatus::Unchanged
    );
    assert_eq!(
        menu_button.diagnostics.phase,
        Some(BootstrapStepPhase::Check)
    );
    assert_eq!(menu_button.diagnostics.attempt_count, 1);

    join_server(handle)?;
    Ok(())
}

#[tokio::test]
async fn web_app_answer_query_from_payload() -> Result<(), DynError> {
    let response = r#"{"ok":true,"result":{"inline_message_id":"inline-42"}}"#;
    const CHECKS: [&str; 3] = [
        "\"web_app_query_id\":\"query-42\"",
        "\"type\":\"article\"",
        "\"title\":\"From Payload\"",
    ];
    let (base_url, handle) =
        spawn_server_with_checks("/bot123:abc/answerWebAppQuery", 200, response, &CHECKS)?;

    let client = Client::builder(base_url)?.bot_token("123:abc")?.build()?;
    let web_app_data = WebAppData::new("{\"query_id\":\"query-42\",\"item\":\"coffee\"}", "Open");
    let result = InlineQueryResult::article("r-42", "From Payload", "ok")?;
    let sent = client
        .app()
        .web_app()
        .answer_query_from_payload::<serde_json::Value, _>(&web_app_data, result)
        .await?;
    assert_eq!(sent.inline_message_id, "inline-42");

    join_server(handle)?;
    Ok(())
}

#[tokio::test]
async fn setup_and_web_app_facades_handle_menu_button_and_query_answer() -> Result<(), DynError> {
    let response = r#"{"ok":true,"result":true}"#;
    let answer_response = r#"{"ok":true,"result":{"inline_message_id":"inline-99"}}"#;
    let expectations = vec![
        RequestExpectation::post("/bot123:abc/setChatMenuButton")
            .contains_case_insensitive("\"chat_id\":42")
            .contains_case_insensitive("\"menu_button\":{\"type\":\"web_app\"")
            .contains_case_insensitive("\"url\":\"https://example.com/mini-app\"")
            .respond_json(200, response),
        RequestExpectation::post("/bot123:abc/answerWebAppQuery")
            .contains_case_insensitive("\"web_app_query_id\":\"query-99\"")
            .contains_case_insensitive("\"title\":\"Facade Answer\"")
            .respond_json(200, answer_response),
    ];
    let server = FakeTelegramServer::start(expectations)?;

    let client = Client::builder(server.base_url())?
        .bot_token("123:abc")?
        .build()?;
    let applied = client
        .control()
        .setup()
        .set_menu_button(MenuButtonConfig::for_chat_web_app(
            42,
            "Open Mini App",
            "https://example.com/mini-app",
        ))
        .await?;
    assert!(applied);

    let web_app_data = WebAppData::new("{\"query_id\":\"query-99\",\"item\":\"tea\"}", "Open");
    let result = InlineQueryResult::article("article-99", "Facade Answer", "done")?;
    let sent = client
        .app()
        .web_app()
        .answer_query_from_payload::<serde_json::Value, _>(&web_app_data, result)
        .await?;
    assert_eq!(sent.inline_message_id, "inline-99");

    join_server(server)?;
    Ok(())
}

#[tokio::test]
async fn app_membership_facade_handles_bot_member_and_capabilities() -> Result<(), DynError> {
    let expectations = vec![
        RequestExpectation::post("/bot123:abc/getMe").respond_json(
            200,
            r#"{"ok":true,"result":{"id":999,"is_bot":true,"first_name":"tele","username":"tele_bot"}}"#,
        ),
        RequestExpectation::post("/bot123:abc/getChatMember")
            .contains_case_insensitive("\"chat_id\":-10010")
            .contains_case_insensitive("\"user_id\":999")
            .respond_json(
                200,
                r#"{"ok":true,"result":{"status":"administrator","user":{"id":999,"is_bot":true,"first_name":"tele"},"can_manage_chat":true,"can_delete_messages":true}}"#,
            ),
        RequestExpectation::post("/bot123:abc/getMe").respond_json(
            200,
            r#"{"ok":true,"result":{"id":999,"is_bot":true,"first_name":"tele","username":"tele_bot"}}"#,
        ),
        RequestExpectation::post("/bot123:abc/getChatMember")
            .contains_case_insensitive("\"chat_id\":-10010")
            .contains_case_insensitive("\"user_id\":999")
            .respond_json(
                200,
                r#"{"ok":true,"result":{"status":"administrator","user":{"id":999,"is_bot":true,"first_name":"tele"},"can_manage_chat":true,"can_delete_messages":true}}"#,
            ),
        RequestExpectation::post("/bot123:abc/getChatAdministrators")
            .contains_case_insensitive("\"chat_id\":-10010")
            .respond_json(
                200,
                r#"{"ok":true,"result":[{"status":"administrator","user":{"id":999,"is_bot":true,"first_name":"tele"},"can_manage_chat":true,"can_delete_messages":true},{"status":"administrator","user":{"id":701,"is_bot":false,"first_name":"owner"},"can_manage_chat":true,"can_restrict_members":true}]}"#,
            ),
    ];
    let server = FakeTelegramServer::start(expectations)?;

    let client = Client::builder(server.base_url())?
        .bot_token("123:abc")?
        .build()?;
    let membership = client.app().membership();

    let bot_member = membership.bot_member(-10010_i64).await?;
    assert_eq!(bot_member.user().map(|user| user.id.0), Some(999));
    assert!(bot_member.has_capability(ChatAdministratorCapability::ManageChat));

    let missing = membership
        .bot_missing_capabilities(
            -10010_i64,
            &[
                ChatAdministratorCapability::ManageChat,
                ChatAdministratorCapability::RestrictMembers,
            ],
        )
        .await?;
    assert_eq!(missing, vec![ChatAdministratorCapability::RestrictMembers]);

    let administrators = membership.administrators(-10010_i64).await?;
    assert_eq!(administrators.len(), 2);

    join_server(server)?;
    Ok(())
}

#[tokio::test]
async fn moderation_facade_handles_join_actions_and_member_controls() -> Result<(), DynError> {
    let expectations = vec![
        RequestExpectation::post("/bot123:abc/approveChatJoinRequest")
            .contains_case_insensitive("\"chat_id\":-10010")
            .contains_case_insensitive("\"user_id\":701")
            .respond_json(200, r#"{"ok":true,"result":true}"#),
        RequestExpectation::post("/bot123:abc/declineChatJoinRequest")
            .contains_case_insensitive("\"chat_id\":-10010")
            .contains_case_insensitive("\"user_id\":701")
            .respond_json(200, r#"{"ok":true,"result":true}"#),
        RequestExpectation::post("/bot123:abc/banChatMember")
            .contains_case_insensitive("\"chat_id\":-10010")
            .contains_case_insensitive("\"user_id\":701")
            .contains_case_insensitive("\"until_date\":1710009999")
            .contains_case_insensitive("\"revoke_messages\":true")
            .respond_json(200, r#"{"ok":true,"result":true}"#),
        RequestExpectation::post("/bot123:abc/restrictChatMember")
            .contains_case_insensitive("\"chat_id\":-10010")
            .contains_case_insensitive("\"user_id\":701")
            .contains_case_insensitive("\"can_send_messages\":false")
            .contains_case_insensitive("\"can_manage_topics\":false")
            .contains_case_insensitive("\"use_independent_chat_permissions\":true")
            .contains_case_insensitive("\"until_date\":1710011111")
            .respond_json(200, r#"{"ok":true,"result":true}"#),
        RequestExpectation::post("/bot123:abc/deleteMessage")
            .contains_case_insensitive("\"chat_id\":-10010")
            .contains_case_insensitive("\"message_id\":55")
            .respond_json(200, r#"{"ok":true,"result":true}"#),
    ];
    let server = FakeTelegramServer::start(expectations)?;

    let client = Client::builder(server.base_url())?
        .bot_token("123:abc")?
        .build()?;
    let join_update: Update = serde_json::from_value(serde_json::json!({
        "update_id": 43,
        "chat_join_request": {
            "chat": {"id": -10010, "type": "supergroup", "title": "mods"},
            "from": {"id": 701, "is_bot": false, "first_name": "candidate"},
            "user_chat_id": 7001,
            "date": 1710000001
        }
    }))?;
    let message_update: Update = serde_json::from_value(serde_json::json!({
        "update_id": 44,
        "message": {
            "message_id": 55,
            "date": 1710000002,
            "chat": {"id": -10010, "type": "supergroup", "title": "mods"},
            "from": {"id": 701, "is_bot": false, "first_name": "candidate"},
            "text": "spam"
        }
    }))?;
    let message = message_update
        .message
        .as_deref()
        .ok_or("missing test message")?;

    assert!(
        client
            .app()
            .moderation()
            .approve_join_request_from_update(&join_update)
            .await?
    );
    assert!(
        client
            .app()
            .moderation()
            .decline_join_request_from_update(&join_update)
            .await?
    );
    assert!(
        client
            .app()
            .moderation()
            .ban_author_with(
                message,
                BanMemberOptions::new()
                    .until_date(1710009999)
                    .revoke_messages(true),
            )
            .await?
    );
    assert!(
        client
            .app()
            .moderation()
            .mute_author_with(
                message,
                RestrictMemberOptions::new()
                    .use_independent_chat_permissions(true)
                    .until_date(1710011111),
            )
            .await?
    );
    assert!(
        client
            .app()
            .moderation()
            .delete_from_update(&message_update)
            .await?
    );

    join_server(server)?;
    Ok(())
}

#[tokio::test]
async fn moderation_notice_facade_reuses_text_builder() -> Result<(), DynError> {
    let response = r#"{"ok":true,"result":{"message_id":56,"date":1710000003,"chat":{"id":-10010,"type":"supergroup","title":"mods"},"message_thread_id":88,"text":"Message removed"}}"#;
    let expectations = vec![
        RequestExpectation::post("/bot123:abc/sendMessage")
            .contains_case_insensitive("\"chat_id\":-10010")
            .contains_case_insensitive("\"text\":\"Message removed\"")
            .contains_case_insensitive("\"reply_parameters\":{\"message_id\":55")
            .contains_case_insensitive("\"message_thread_id\":88")
            .contains_case_insensitive("\"disable_notification\":true")
            .contains_case_insensitive(
                "\"reply_markup\":{\"inline_keyboard\":[[{\"text\":\"Review\"",
            )
            .contains_case_insensitive("\"callback_data\":\"review:55\"")
            .respond_json(200, response),
    ];
    let server = FakeTelegramServer::start(expectations)?;

    let client = Client::builder(server.base_url())?
        .bot_token("123:abc")?
        .build()?;
    let update: Update = serde_json::from_value(serde_json::json!({
        "update_id": 45,
        "message": {
            "message_id": 55,
            "message_thread_id": 88,
            "date": 1710000002,
            "chat": {"id": -10010, "type": "supergroup", "title": "mods"},
            "from": {"id": 701, "is_bot": false, "first_name": "candidate"},
            "text": "spam"
        }
    }))?;
    let message = update.message.as_deref().ok_or("missing test message")?;
    let markup = InlineKeyboardMarkup::single_row(vec![InlineKeyboardButton::callback(
        "Review",
        "review:55",
    )?]);

    let sent = client
        .app()
        .moderation()
        .notice()
        .for_message(message, "Message removed")?
        .disable_notification(true)
        .reply_markup(markup)
        .send()
        .await?;
    assert_eq!(sent.message_id.0, 56);

    join_server(server)?;
    Ok(())
}

#[tokio::test]
async fn setup_set_chat_menu_button_uses_high_level_helper() -> Result<(), DynError> {
    let response = r#"{"ok":true,"result":true}"#;
    const CHECKS: [&str; 4] = [
        "\"chat_id\":42",
        "\"menu_button\":{\"type\":\"web_app\"",
        "\"text\":\"Open Mini App\"",
        "\"url\":\"https://example.com/mini-app\"",
    ];
    let (base_url, handle) =
        spawn_server_with_checks("/bot123:abc/setChatMenuButton", 200, response, &CHECKS)?;

    let client = Client::builder(base_url)?.bot_token("123:abc")?.build()?;
    let applied = client
        .control()
        .setup()
        .set_menu_button(MenuButtonConfig::for_chat_web_app(
            42,
            "Open Mini App",
            "https://example.com/mini-app",
        ))
        .await?;
    assert!(applied);

    join_server(handle)?;
    Ok(())
}

#[tokio::test]
async fn get_file_success() -> Result<(), DynError> {
    let response = r#"{"ok":true,"result":{"file_id":"file_1","file_unique_id":"uniq_1","file_size":128,"file_path":"photos/pic.jpg"}}"#;
    let (base_url, handle) = spawn_server("/bot123:abc/getFile", 200, response)?;

    let client = Client::builder(base_url)?.bot_token("123:abc")?.build()?;
    let file = client
        .files()
        .get_file(&GetFileRequest::new("file_1"))
        .await?;
    assert_eq!(file.file_id, "file_1");
    assert_eq!(file.file_path.as_deref(), Some("photos/pic.jpg"));

    join_server(handle)?;
    Ok(())
}

#[tokio::test]
async fn send_photo_upload_multipart_success() -> Result<(), DynError> {
    let response = r#"{"ok":true,"result":{"message_id":100,"date":1710000000,"chat":{"id":1,"type":"private"},"photo":[{"file_id":"file_1","file_unique_id":"uniq_1","width":10,"height":10}]}}"#;
    const CHECKS: [&str; 4] = [
        "Content-Type: multipart/form-data; boundary=",
        "name=\"chat_id\"",
        "name=\"photo\"; filename=\"image.jpg\"",
        "binary-photo-data",
    ];
    let (base_url, handle) =
        spawn_server_with_checks("/bot123:abc/sendPhoto", 200, response, &CHECKS)?;

    let client = Client::builder(base_url)?.bot_token("123:abc")?.build()?;
    let file = UploadFile::from_bytes("image.jpg", b"binary-photo-data".to_vec())?;
    let request = SendPhotoRequest::for_upload(1_i64);
    let message = client.messages().send_photo_upload(&request, &file).await?;
    assert_eq!(message.message_id.0, 100);

    join_server(handle)?;
    Ok(())
}

#[tokio::test]
async fn set_chat_photo_upload_multipart_success() -> Result<(), DynError> {
    let response = r#"{"ok":true,"result":true}"#;
    const CHECKS: [&str; 4] = [
        "Content-Type: multipart/form-data; boundary=",
        "name=\"chat_id\"",
        "name=\"photo\"; filename=\"chat-photo.jpg\"",
        "binary-chat-photo-data",
    ];
    let (base_url, handle) =
        spawn_server_with_checks("/bot123:abc/setChatPhoto", 200, response, &CHECKS)?;

    let client = Client::builder(base_url)?.bot_token("123:abc")?.build()?;
    let request = SetChatPhotoRequest::new(1_i64);
    let file = UploadFile::from_bytes("chat-photo.jpg", b"binary-chat-photo-data".to_vec())?;
    let ok = client
        .chats()
        .set_chat_photo_upload(&request, &file)
        .await?;
    assert!(ok);

    join_server(handle)?;
    Ok(())
}

#[tokio::test]
async fn app_photo_builder_send_upload_success() -> Result<(), DynError> {
    let response = r#"{"ok":true,"result":{"message_id":102,"date":1710000005,"chat":{"id":1,"type":"private"},"photo":[{"file_id":"file_2","file_unique_id":"uniq_2","width":10,"height":10}]}}"#;
    const CHECKS: [&str; 5] = [
        "Content-Type: multipart/form-data; boundary=",
        "name=\"chat_id\"",
        "name=\"caption\"",
        "name=\"photo\"; filename=\"builder-image.jpg\"",
        "binary-builder-photo-data",
    ];
    let (base_url, handle) =
        spawn_server_with_checks("/bot123:abc/sendPhoto", 200, response, &CHECKS)?;

    let client = Client::builder(base_url)?.bot_token("123:abc")?.build()?;
    let file = UploadFile::from_bytes("builder-image.jpg", b"binary-builder-photo-data".to_vec())?;
    let message = client
        .app()
        .photo_upload(1_i64)
        .caption("builder upload")
        .send(&file)
        .await?;
    assert_eq!(message.message_id.0, 102);

    join_server(handle)?;
    Ok(())
}

#[tokio::test]
async fn app_audio_builder_send_upload_success() -> Result<(), DynError> {
    let response = r#"{"ok":true,"result":{"message_id":103,"date":1710000011,"chat":{"id":1,"type":"private"}}}"#;
    const CHECKS: [&str; 5] = [
        "Content-Type: multipart/form-data; boundary=",
        "name=\"chat_id\"",
        "name=\"caption\"",
        "name=\"audio\"; filename=\"builder-audio.mp3\"",
        "binary-builder-audio-data",
    ];
    let (base_url, handle) =
        spawn_server_with_checks("/bot123:abc/sendAudio", 200, response, &CHECKS)?;

    let client = Client::builder(base_url)?.bot_token("123:abc")?.build()?;
    let file = UploadFile::from_bytes("builder-audio.mp3", b"binary-builder-audio-data".to_vec())?;
    let message = client
        .app()
        .audio_upload(1_i64)
        .caption("builder audio upload")
        .send(&file)
        .await?;
    assert_eq!(message.message_id.0, 103);

    join_server(handle)?;
    Ok(())
}

#[tokio::test]
async fn app_video_note_upload_builder_success() -> Result<(), DynError> {
    let response = r#"{"ok":true,"result":{"message_id":108,"date":1710000016,"chat":{"id":1,"type":"private"}}}"#;
    const CHECKS: [&str; 8] = [
        "Content-Type: multipart/form-data; boundary=",
        "name=\"chat_id\"",
        "name=\"duration\"",
        "name=\"length\"",
        "attach://video_note_thumb0",
        "name=\"video_note\"; filename=\"builder-video-note.mp4\"",
        "name=\"video_note_thumb0\"; filename=\"builder-video-note-thumb.jpg\"",
        "binary-builder-video-note-data",
    ];
    let (base_url, handle) =
        spawn_server_with_checks("/bot123:abc/sendVideoNote", 200, response, &CHECKS)?;

    let client = Client::builder(base_url)?.bot_token("123:abc")?.build()?;
    let file = UploadFile::from_bytes(
        "builder-video-note.mp4",
        b"binary-builder-video-note-data".to_vec(),
    )?;
    let thumbnail = UploadPart::from_bytes(
        "video_note_thumb0",
        "builder-video-note-thumb.jpg",
        b"binary-builder-video-note-thumb-data".to_vec(),
    )?;
    let message = client
        .app()
        .video_note_upload(1_i64)
        .duration(3)
        .length(240)
        .thumbnail(thumbnail.attach_uri())
        .send_parts(&file, &[thumbnail])
        .await?;
    assert_eq!(message.message_id.0, 108);

    join_server(handle)?;
    Ok(())
}

#[tokio::test]
async fn send_document_upload_with_thumbnail_multipart_success() -> Result<(), DynError> {
    let response = r#"{"ok":true,"result":{"message_id":104,"date":1710000012,"chat":{"id":1,"type":"private"},"document":{"file_id":"doc_1","file_unique_id":"doc_unique_1"}}}"#;
    const CHECKS: [&str; 8] = [
        "Content-Type: multipart/form-data; boundary=",
        "name=\"chat_id\"",
        "name=\"thumbnail\"",
        "attach://thumb0",
        "name=\"document\"; filename=\"report.pdf\"",
        "binary-document-data",
        "name=\"thumb0\"; filename=\"report-thumb.jpg\"",
        "binary-thumbnail-data",
    ];
    let (base_url, handle) =
        spawn_server_with_checks("/bot123:abc/sendDocument", 200, response, &CHECKS)?;

    let client = Client::builder(base_url)?.bot_token("123:abc")?.build()?;
    let document = UploadFile::from_bytes("report.pdf", b"binary-document-data".to_vec())?;
    let thumbnail = UploadPart::from_bytes(
        "thumb0",
        "report-thumb.jpg",
        b"binary-thumbnail-data".to_vec(),
    )?;
    let mut request = SendDocumentRequest::for_upload(1_i64);
    request.thumbnail = Some(thumbnail.attach_uri());
    let message = client
        .messages()
        .send_document_upload_parts(&request, &document, &[thumbnail])
        .await?;
    assert_eq!(message.message_id.0, 104);

    join_server(handle)?;
    Ok(())
}

#[tokio::test]
async fn send_media_group_upload_multipart_success() -> Result<(), DynError> {
    let response = r#"{"ok":true,"result":[{"message_id":104,"date":1710000012,"chat":{"id":1,"type":"private"},"photo":[{"file_id":"photo_1","file_unique_id":"photo_unique_1","width":10,"height":10}]},{"message_id":105,"date":1710000013,"chat":{"id":1,"type":"private"},"video":{"file_id":"video_1","file_unique_id":"video_unique_1","width":640,"height":480,"duration":5}}]}"#;
    const CHECKS: [&str; 8] = [
        "Content-Type: multipart/form-data; boundary=",
        "name=\"chat_id\"",
        "name=\"media\"",
        "attach://photo0",
        "attach://video0",
        "name=\"photo0\"; filename=\"album-photo.jpg\"",
        "name=\"video0\"; filename=\"album-video.mp4\"",
        "binary-album-video-data",
    ];
    let (base_url, handle) =
        spawn_server_with_checks("/bot123:abc/sendMediaGroup", 200, response, &CHECKS)?;

    let client = Client::builder(base_url)?.bot_token("123:abc")?.build()?;
    let photo_part = UploadPart::from_bytes(
        "photo0",
        "album-photo.jpg",
        b"binary-album-photo-data".to_vec(),
    )?;
    let video_part = UploadPart::from_bytes(
        "video0",
        "album-video.mp4",
        b"binary-album-video-data".to_vec(),
    )?;
    let request = SendMediaGroupRequest::new(
        1_i64,
        vec![
            InputMediaPhoto::new(photo_part.attach_uri())
                .caption("photo item")
                .into(),
            InputMediaVideo::new(video_part.attach_uri()).into(),
        ],
    )?;
    let messages = client
        .messages()
        .send_media_group_upload(&request, &[photo_part, video_part])
        .await?;
    assert_eq!(messages.len(), 2);
    assert_eq!(messages[0].message_id.0, 104);

    join_server(handle)?;
    Ok(())
}

#[tokio::test]
async fn app_media_group_upload_builder_success() -> Result<(), DynError> {
    let response = r#"{"ok":true,"result":[{"message_id":106,"date":1710000014,"chat":{"id":1,"type":"private"},"photo":[{"file_id":"photo_2","file_unique_id":"photo_unique_2","width":10,"height":10}]},{"message_id":107,"date":1710000015,"chat":{"id":1,"type":"private"},"video":{"file_id":"video_2","file_unique_id":"video_unique_2","width":640,"height":480,"duration":5}}]}"#;
    const CHECKS: [&str; 7] = [
        "Content-Type: multipart/form-data; boundary=",
        "name=\"chat_id\"",
        "name=\"media\"",
        "attach://builder_photo0",
        "attach://builder_video0",
        "name=\"builder_photo0\"; filename=\"builder-album-photo.jpg\"",
        "name=\"builder_video0\"; filename=\"builder-album-video.mp4\"",
    ];
    let (base_url, handle) =
        spawn_server_with_checks("/bot123:abc/sendMediaGroup", 200, response, &CHECKS)?;

    let client = Client::builder(base_url)?.bot_token("123:abc")?.build()?;
    let photo_part = UploadPart::from_bytes(
        "builder_photo0",
        "builder-album-photo.jpg",
        b"builder-photo-data".to_vec(),
    )?;
    let video_part = UploadPart::from_bytes(
        "builder_video0",
        "builder-album-video.mp4",
        b"builder-video-data".to_vec(),
    )?;
    let messages = client
        .app()
        .media_group_upload(
            1_i64,
            vec![
                InputMediaGroupItem::from(InputMediaPhoto::new(photo_part.attach_uri())),
                InputMediaGroupItem::from(InputMediaVideo::new(video_part.attach_uri())),
            ],
        )?
        .send(&[photo_part, video_part])
        .await?;
    assert_eq!(messages.len(), 2);
    assert_eq!(messages[0].message_id.0, 106);

    join_server(handle)?;
    Ok(())
}

#[tokio::test]
async fn advanced_get_available_gifts_success() -> Result<(), DynError> {
    let response = r#"{"ok":true,"result":{"gifts":[{"id":"gift-1","sticker":{"file_id":"sticker","file_unique_id":"sticker-unique","type":"regular","width":64,"height":64,"is_animated":false,"is_video":false},"star_count":15,"is_premium":true,"background":{"center_color":1,"edge_color":2,"text_color":3}}],"catalog":"main"}}"#;
    let (base_url, handle) = spawn_server("/bot123:abc/getAvailableGifts", 200, response)?;

    let client = Client::builder(base_url)?.bot_token("123:abc")?.build()?;
    let request = AdvancedGetAvailableGiftsRequest::new();
    let gifts = client
        .advanced()
        .get_available_gifts_typed(&request)
        .await?;
    assert_eq!(gifts.gifts.len(), 1);
    assert_eq!(gifts.gifts[0].id, "gift-1");
    assert_eq!(gifts.gifts[0].star_count, 15);
    assert!(gifts.gifts[0].is_premium);
    assert_eq!(
        gifts.gifts[0].background.as_ref().map(|bg| bg.text_color),
        Some(3)
    );
    assert_eq!(gifts.extra["catalog"], "main");

    join_server(handle)?;
    Ok(())
}

#[tokio::test]
async fn advanced_get_user_gifts_typed_returns_owned_gifts() -> Result<(), DynError> {
    let response = r#"{"ok":true,"result":{"total_count":1,"gifts":[{"type":"regular","gift":{"id":"gift-1","sticker":{"file_id":"sticker","file_unique_id":"sticker-unique","type":"regular","width":64,"height":64,"is_animated":false,"is_video":false},"star_count":15},"send_date":1710000000}],"next_offset":"next","scope":"user"}}"#;
    let (base_url, handle) = spawn_server_with_checks(
        "/bot123:abc/getUserGifts",
        200,
        response,
        &["\"user_id\":42", "\"limit\":1"],
    )?;

    let client = Client::builder(base_url)?.bot_token("123:abc")?.build()?;
    let mut request = AdvancedGetUserGiftsRequest::new(tele::types::UserId(42));
    request.limit = Some(1);
    let gifts = client.advanced().get_user_gifts_typed(&request).await?;
    assert_eq!(gifts.total_count, 1);
    assert_eq!(gifts.next_offset.as_deref(), Some("next"));
    let gift = gifts.gifts[0]
        .as_regular()
        .ok_or("expected regular owned gift")?;
    assert_eq!(gifts.gifts[0].kind(), Some("regular"));
    assert_eq!(gift.gift.id, "gift-1");
    assert_eq!(gift.gift.star_count, 15);
    assert_eq!(gifts.extra["scope"], "user");

    join_server(handle)?;
    Ok(())
}

#[tokio::test]
async fn advanced_get_star_transactions_typed_returns_transactions() -> Result<(), DynError> {
    let response = r#"{"ok":true,"result":{"transactions":[{"id":"tx-1","amount":5,"nanostar_amount":7,"date":1710000000,"source":{"type":"user","transaction_type":"gift_purchase","user":{"id":7,"is_bot":false,"first_name":"Ada"},"gift":{"id":"gift-1","sticker":{"file_id":"sticker","file_unique_id":"sticker-unique","type":"regular","width":64,"height":64,"is_animated":false,"is_video":false},"star_count":15}},"note":"kept"}]}}"#;
    let (base_url, handle) = spawn_server_with_checks(
        "/bot123:abc/getStarTransactions",
        200,
        response,
        &["\"offset\":0", "\"limit\":1"],
    )?;

    let client = Client::builder(base_url)?.bot_token("123:abc")?.build()?;
    let mut request = AdvancedGetStarTransactionsRequest::new();
    request.offset = Some(0);
    request.limit = Some(1);
    let transactions = client
        .advanced()
        .get_star_transactions_typed(&request)
        .await?;
    assert_eq!(transactions.transactions.len(), 1);
    assert_eq!(transactions.transactions[0].id, "tx-1");
    assert_eq!(transactions.transactions[0].nanostar_amount, Some(7));
    let source = transactions.transactions[0]
        .source
        .as_ref()
        .ok_or("expected transaction source")?;
    assert_eq!(source.kind(), Some("user"));
    let user = source
        .as_user()
        .ok_or("expected user transaction partner")?;
    assert_eq!(user.transaction_type.as_str(), "gift_purchase");
    assert_eq!(user.user.id.0, 7);
    assert_eq!(
        user.gift.as_ref().map(|gift| gift.id.as_str()),
        Some("gift-1")
    );
    assert_eq!(transactions.transactions[0].extra["note"], "kept");

    join_server(handle)?;
    Ok(())
}

#[tokio::test]
async fn answer_web_app_query_typed_success() -> Result<(), DynError> {
    let response = r#"{"ok":true,"result":{"inline_message_id":"inline-msg-1"}}"#;
    const CHECKS: [&str; 3] = [
        "\"web_app_query_id\":\"query-1\"",
        "\"type\":\"article\"",
        "\"id\":\"result-1\"",
    ];
    let (base_url, handle) =
        spawn_server_with_checks("/bot123:abc/answerWebAppQuery", 200, response, &CHECKS)?;

    let client = Client::builder(base_url)?.bot_token("123:abc")?.build()?;
    let result = InlineQueryResult::new(serde_json::json!({
        "type": "article",
        "id": "result-1",
        "title": "Mini App result",
        "input_message_content": {
            "message_text": "Mini App accepted"
        }
    }))?;
    let request = AdvancedAnswerWebAppQueryRequest::new("query-1", result);
    let sent = client
        .advanced()
        .answer_web_app_query_typed(&request)
        .await?;
    assert_eq!(sent.inline_message_id, "inline-msg-1");

    join_server(handle)?;
    Ok(())
}

#[tokio::test]
async fn advanced_send_game_typed_returns_message() -> Result<(), DynError> {
    let response = r#"{"ok":true,"result":{"message_id":51,"date":1710000000,"chat":{"id":1,"type":"private"},"game":{"title":"Space","description":"Arcade","photo":[{"file_id":"photo","file_unique_id":"photo-unique","width":64,"height":64}]}}}"#;
    let (base_url, handle) = spawn_server_with_checks(
        "/bot123:abc/sendGame",
        200,
        response,
        &["\"game_short_name\":\"space\""],
    )?;

    let client = Client::builder(base_url)?.bot_token("123:abc")?.build()?;
    let request = AdvancedSendGameRequest::new(1_i64, "space");
    let message = client.advanced().send_game_typed(&request).await?;
    assert_eq!(message.message_id.0, 51);
    assert!(message.game.is_some());

    join_server(handle)?;
    Ok(())
}

#[tokio::test]
async fn advanced_get_game_high_scores_typed_returns_scores() -> Result<(), DynError> {
    let response = r#"{"ok":true,"result":[{"position":1,"user":{"id":42,"is_bot":false,"first_name":"Ada"},"score":9001,"tier":"gold"}]}"#;
    let (base_url, handle) = spawn_server_with_checks(
        "/bot123:abc/getGameHighScores",
        200,
        response,
        &["\"user_id\":42", "\"chat_id\":1", "\"message_id\":99"],
    )?;

    let client = Client::builder(base_url)?.bot_token("123:abc")?.build()?;
    let mut request = AdvancedGetGameHighScoresRequest::new(tele::types::UserId(42));
    request.chat_id = Some(1_i64.into());
    request.message_id = Some(MessageId(99));
    let scores = client
        .advanced()
        .get_game_high_scores_typed(&request)
        .await?;
    assert_eq!(scores.len(), 1);
    assert_eq!(scores[0].position, 1);
    assert_eq!(scores[0].user.id.0, 42);
    assert_eq!(scores[0].score, 9_001);
    assert_eq!(scores[0].extra["tier"], "gold");

    join_server(handle)?;
    Ok(())
}

#[tokio::test]
async fn advanced_get_user_profile_audios_typed_returns_audios() -> Result<(), DynError> {
    let response = r#"{"ok":true,"result":{"total_count":1,"audios":[{"file_id":"audio","file_unique_id":"audio-unique","duration":7,"title":"Intro"}],"page":"first"}}"#;
    let (base_url, handle) = spawn_server_with_checks(
        "/bot123:abc/getUserProfileAudios",
        200,
        response,
        &["\"user_id\":42", "\"limit\":1"],
    )?;

    let client = Client::builder(base_url)?.bot_token("123:abc")?.build()?;
    let mut request = AdvancedGetUserProfileAudiosRequest::new(tele::types::UserId(42));
    request.limit = Some(1);
    let profile_audios = client
        .advanced()
        .get_user_profile_audios_typed(&request)
        .await?;
    assert_eq!(profile_audios.total_count, 1);
    assert_eq!(profile_audios.audios[0].file_id, "audio");
    assert_eq!(profile_audios.audios[0].title.as_deref(), Some("Intro"));
    assert_eq!(profile_audios.extra["page"], "first");

    join_server(handle)?;
    Ok(())
}

#[tokio::test]
async fn advanced_set_user_emoji_status_allows_empty_status_id() -> Result<(), DynError> {
    let (base_url, handle) = spawn_server_with_checks(
        "/bot123:abc/setUserEmojiStatus",
        200,
        r#"{"ok":true,"result":true}"#,
        &["\"user_id\":42", "\"emoji_status_custom_emoji_id\":\"\""],
    )?;

    let client = Client::builder(base_url)?.bot_token("123:abc")?.build()?;
    let mut request = AdvancedSetUserEmojiStatusRequest::new(tele::types::UserId(42));
    request.emoji_status_custom_emoji_id = Some(String::new());
    let result = client
        .advanced()
        .set_user_emoji_status::<bool>(&request)
        .await?;
    assert!(result);

    join_server(handle)?;
    Ok(())
}

#[tokio::test]
async fn advanced_set_chat_member_tag_allows_empty_tag() -> Result<(), DynError> {
    let (base_url, handle) = spawn_server_with_checks(
        "/bot123:abc/setChatMemberTag",
        200,
        r#"{"ok":true,"result":true}"#,
        &["\"chat_id\":1", "\"user_id\":42", "\"tag\":\"\""],
    )?;

    let client = Client::builder(base_url)?.bot_token("123:abc")?.build()?;
    let mut request = AdvancedSetChatMemberTagRequest::new(1_i64, tele::types::UserId(42));
    request.tag = Some(String::new());
    let result = client
        .advanced()
        .set_chat_member_tag::<bool>(&request)
        .await?;
    assert!(result);

    join_server(handle)?;
    Ok(())
}

#[tokio::test]
async fn advanced_set_business_account_username_allows_empty_username() -> Result<(), DynError> {
    let (base_url, handle) = spawn_server_with_checks(
        "/bot123:abc/setBusinessAccountUsername",
        200,
        r#"{"ok":true,"result":true}"#,
        &[
            "\"business_connection_id\":\"business-connection\"",
            "\"username\":\"\"",
        ],
    )?;

    let client = Client::builder(base_url)?.bot_token("123:abc")?.build()?;
    let mut request = AdvancedSetBusinessAccountUsernameRequest::new("business-connection");
    request.username = Some(String::new());
    let result = client
        .advanced()
        .set_business_account_username::<bool>(&request)
        .await?;
    assert!(result);

    join_server(handle)?;
    Ok(())
}

#[tokio::test]
async fn advanced_prepared_mini_app_outputs_are_typed() -> Result<(), DynError> {
    let inline_response =
        r#"{"ok":true,"result":{"id":"prepared-inline","expiration_date":1710086400}}"#;
    let keyboard_response = r#"{"ok":true,"result":{"id":"prepared-keyboard"}}"#;
    let expectations = vec![
        RequestExpectation::post("/bot123:abc/savePreparedInlineMessage")
            .contains_case_insensitive("\"user_id\":42")
            .contains_case_insensitive("\"type\":\"article\"")
            .respond_json(200, inline_response),
        RequestExpectation::post("/bot123:abc/savePreparedKeyboardButton")
            .contains_case_insensitive("\"request_users\"")
            .respond_json(200, keyboard_response),
    ];
    let server = FakeTelegramServer::start(expectations)?;
    let base_url = server.base_url().to_owned();

    let client = Client::builder(base_url)?.bot_token("123:abc")?.build()?;
    let result = InlineQueryResult::article("article-1", "Prepared", "message")?;
    let inline_request =
        AdvancedSavePreparedInlineMessageRequest::new(tele::types::UserId(42), result);
    let prepared_inline = client
        .advanced()
        .save_prepared_inline_message_typed(&inline_request)
        .await?;
    assert_eq!(prepared_inline.id, "prepared-inline");
    assert_eq!(prepared_inline.expiration_date, 1_710_086_400);

    let button = KeyboardButton::new("Pick user").request_users(KeyboardButtonRequestUsers::new(1));
    let keyboard_request =
        AdvancedSavePreparedKeyboardButtonRequest::new(tele::types::UserId(42), button);
    let prepared_keyboard = client
        .advanced()
        .save_prepared_keyboard_button_typed(&keyboard_request)
        .await?;
    assert_eq!(prepared_keyboard.id, "prepared-keyboard");

    join_server(server)?;
    Ok(())
}

#[tokio::test]
async fn advanced_create_forum_topic_typed_returns_forum_topic() -> Result<(), DynError> {
    let response = r#"{"ok":true,"result":{"message_thread_id":61,"name":"topic","icon_color":7322096,"icon_custom_emoji_id":"emoji-id"}}"#;
    let (base_url, handle) = spawn_server_with_checks(
        "/bot123:abc/createForumTopic",
        200,
        response,
        &["\"chat_id\":1", "\"name\":\"topic\""],
    )?;

    let client = Client::builder(base_url)?.bot_token("123:abc")?.build()?;
    let request = AdvancedCreateForumTopicRequest::new(1_i64, "topic");
    let topic = client.advanced().create_forum_topic_typed(&request).await?;
    assert_eq!(topic.message_thread_id, 61);
    assert_eq!(topic.name, "topic");
    assert_eq!(topic.icon_color, 7_322_096);
    assert_eq!(topic.icon_custom_emoji_id.as_deref(), Some("emoji-id"));

    join_server(handle)?;
    Ok(())
}

#[tokio::test]
async fn advanced_business_star_balance_typed_returns_star_amount() -> Result<(), DynError> {
    let response = r#"{"ok":true,"result":{"amount":42,"nanostar_amount":7}}"#;
    let (base_url, handle) = spawn_server_with_checks(
        "/bot123:abc/getBusinessAccountStarBalance",
        200,
        response,
        &["\"business_connection_id\":\"business-id\""],
    )?;

    let client = Client::builder(base_url)?.bot_token("123:abc")?.build()?;
    let request = AdvancedGetBusinessAccountStarBalanceRequest::new("business-id");
    let balance = client
        .advanced()
        .get_business_account_star_balance_typed(&request)
        .await?;
    assert_eq!(balance.amount, 42);
    assert_eq!(balance.nanostar_amount, Some(7));

    join_server(handle)?;
    Ok(())
}

#[tokio::test]
async fn advanced_user_chat_boosts_typed_returns_boosts() -> Result<(), DynError> {
    let response = r#"{"ok":true,"result":{"boosts":[{"boost_id":"boost-1","add_date":1710000000,"expiration_date":1710086400,"source":{"source":"premium"}}]}}"#;
    let (base_url, handle) = spawn_server_with_checks(
        "/bot123:abc/getUserChatBoosts",
        200,
        response,
        &["\"chat_id\":1", "\"user_id\":42"],
    )?;

    let client = Client::builder(base_url)?.bot_token("123:abc")?.build()?;
    let request = AdvancedGetUserChatBoostsRequest::new(1_i64, tele::types::UserId(42));
    let boosts = client
        .advanced()
        .get_user_chat_boosts_typed(&request)
        .await?;
    assert_eq!(boosts.boosts.len(), 1);
    assert_eq!(boosts.boosts[0].boost_id, "boost-1");
    assert_eq!(boosts.boosts[0].source.source, "premium");

    join_server(handle)?;
    Ok(())
}

#[tokio::test]
async fn advanced_post_story_typed_returns_story() -> Result<(), DynError> {
    let response =
        r#"{"ok":true,"result":{"chat":{"id":1,"type":"private"},"id":77,"extra":"kept"}}"#;
    let (base_url, handle) = spawn_server_with_checks(
        "/bot123:abc/postStory",
        200,
        response,
        &[
            "\"business_connection_id\":\"business-id\"",
            "\"active_period\":86400",
            "\"type\":\"photo\"",
            "\"photo\":\"file-id\"",
        ],
    )?;

    let client = Client::builder(base_url)?.bot_token("123:abc")?.build()?;
    let content = InputStoryContent::photo("file-id");
    let request = AdvancedPostStoryRequest::new("business-id", content, 86_400);
    let story = client.advanced().post_story_typed(&request).await?;
    assert_eq!(story.id, 77);
    assert_eq!(story.chat.id, 1);
    assert_eq!(story.extra["extra"], "kept");

    join_server(handle)?;
    Ok(())
}

#[tokio::test]
async fn answer_inline_query_with_typed_button_success() -> Result<(), DynError> {
    let response = r#"{"ok":true,"result":true}"#;
    const CHECKS: [&str; 5] = [
        "\"inline_query_id\":\"inline-q-1\"",
        "\"type\":\"article\"",
        "\"id\":\"result-inline-1\"",
        "\"button\":{\"text\":\"Open Mini App\"",
        "\"web_app\":{\"url\":\"https://example.com/mini-app\"}",
    ];
    let (base_url, handle) =
        spawn_server_with_checks("/bot123:abc/answerInlineQuery", 200, response, &CHECKS)?;

    let client = Client::builder(base_url)?.bot_token("123:abc")?.build()?;
    let inline_result =
        InlineQueryResult::article("result-inline-1", "Inline title", "Inline message text")?;
    let request = AnswerInlineQueryRequest::new("inline-q-1", vec![inline_result]).button(
        InlineQueryResultsButton::web_app("Open Mini App", "https://example.com/mini-app"),
    );
    let ok = client.updates().answer_inline_query(&request).await?;
    assert!(ok);

    join_server(handle)?;
    Ok(())
}

#[tokio::test]
async fn create_invoice_link_success() -> Result<(), DynError> {
    let response = r#"{"ok":true,"result":"https://t.me/$1234"}"#;
    let (base_url, handle) = spawn_server("/bot123:abc/createInvoiceLink", 200, response)?;

    let client = Client::builder(base_url)?.bot_token("123:abc")?.build()?;
    let request = CreateInvoiceLinkRequest::new(
        "Pro Plan",
        "Monthly subscription",
        "order-1",
        "USD",
        vec![LabeledPrice::new("Pro Plan", 499)],
    )?;
    let link = client.payments().create_invoice_link(&request).await?;
    assert_eq!(link, "https://t.me/$1234");

    join_server(handle)?;
    Ok(())
}

#[tokio::test]
async fn send_sticker_upload_multipart_success() -> Result<(), DynError> {
    let response = r#"{"ok":true,"result":{"message_id":101,"date":1710000001,"chat":{"id":1,"type":"private"}}}"#;
    const CHECKS: [&str; 4] = [
        "Content-Type: multipart/form-data; boundary=",
        "name=\"chat_id\"",
        "name=\"sticker\"; filename=\"sticker.webp\"",
        "binary-sticker-data",
    ];
    let (base_url, handle) =
        spawn_server_with_checks("/bot123:abc/sendSticker", 200, response, &CHECKS)?;

    let client = Client::builder(base_url)?.bot_token("123:abc")?.build()?;
    let request = SendStickerRequest::for_upload(1_i64);
    let file = UploadFile::from_bytes("sticker.webp", b"binary-sticker-data".to_vec())?;
    let message = client
        .stickers()
        .send_sticker_upload(&request, &file)
        .await?;
    assert_eq!(message.message_id.0, 101);

    join_server(handle)?;
    Ok(())
}

#[tokio::test]
async fn upload_sticker_file_upload_multipart_success() -> Result<(), DynError> {
    let response =
        r#"{"ok":true,"result":{"file_id":"sticker_file","file_unique_id":"sticker_unique"}}"#;
    const CHECKS: [&str; 5] = [
        "Content-Type: multipart/form-data; boundary=",
        "name=\"user_id\"",
        "name=\"sticker_format\"",
        "name=\"sticker\"; filename=\"upload-sticker.webp\"",
        "binary-upload-sticker-data",
    ];
    let (base_url, handle) =
        spawn_server_with_checks("/bot123:abc/uploadStickerFile", 200, response, &CHECKS)?;

    let client = Client::builder(base_url)?.bot_token("123:abc")?.build()?;
    let request = UploadStickerFileRequest::new(1.into(), StickerFormat::Static);
    let file = UploadFile::from_bytes(
        "upload-sticker.webp",
        b"binary-upload-sticker-data".to_vec(),
    )?;
    let uploaded = client
        .stickers()
        .upload_sticker_file_upload(&request, &file)
        .await?;
    assert_eq!(uploaded.file_id, "sticker_file");

    join_server(handle)?;
    Ok(())
}
