use serde_json::json;
use tele::types::advanced::*;
use tele::types::*;

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn rich_message_wire_format_preserves_nested_entities_and_media_tags() -> TestResult {
    let value = json!({"blocks":[
        {"type":"paragraph","text":["hello ",{"type":"bold","text":"world"}]},
        {"type":"voice_note","voice_note":{"type":"voice_note","media":"voice-id"}},
        {"type":"document","document":{"type":"document","media":"document-id"}},
        {"type":"expandable_blockquote","text":"details"},
        {"type":"table","cells":[[{"text":"cell","align":"left","valign":"top"}]],"is_compact":true},
        {"type":"buttons","buttons":[{"text":"disabled","disabled":{}}]}
    ]});
    let message: InputRichMessage = serde_json::from_value(value.clone())?;
    message.validate()?;
    assert_eq!(serde_json::to_value(&message)?, value);
    let request = AdvancedSendRichMessageRequest::new(1_i64, message);
    request.validate()?;
    assert_eq!(serde_json::to_value(request)?["rich_message"], value);
    Ok(())
}

#[test]
fn rich_content_requires_one_representation_and_valid_nested_blocks() -> TestResult {
    assert!(InputRichMessage::default().validate().is_err());
    let mut message = InputRichMessage::html("<p>hello</p>");
    message.validate()?;
    message.markdown = Some("hello".into());
    assert!(message.validate().is_err());
    assert!(
        InputRichMessage::blocks(vec![InputRichBlock::SectionHeading {
            text: "heading".into(),
            size: 7,
        }])
        .validate()
        .is_err()
    );
    assert!(
        InputRichMessage::blocks(vec![InputRichBlock::Buttons {
            buttons: vec![],
            align: None,
        }])
        .validate()
        .is_err()
    );
    Ok(())
}

#[test]
fn ephemeral_send_and_reply_omit_regular_message_id() -> TestResult {
    let mut request = SendMessageRequest::new(-100_i64, "private reply")?;
    request.ephemeral_message_parameters = Some(EphemeralMessageParameters::new(UserId(42)));
    request.reply_parameters = Some(ReplyParameters::ephemeral(15));
    request.validate()?;
    let value = serde_json::to_value(&request)?;
    assert_eq!(
        value["ephemeral_message_parameters"]["receiver_user_id"],
        42
    );
    assert_eq!(value["reply_parameters"]["ephemeral_message_id"], 15);
    assert!(value["reply_parameters"].get("message_id").is_none());
    let mut reply = ReplyParameters::ephemeral(15);
    reply.message_id = Some(MessageId(1));
    assert!(reply.validate().is_err());
    Ok(())
}

#[test]
fn new_update_events_and_message_content_are_classified() -> TestResult {
    let subscription: Update = serde_json::from_value(json!({"update_id":1,"subscription":{
        "user":{"id":42,"is_bot":false,"first_name":"user"},"invoice_payload":"plan","state":"active"
    }}))?;
    assert_eq!(subscription.kind(), UpdateKind::Subscription);
    assert!(!subscription.has_kind(UpdateKind::Unknown));
    let stopped: Update =
        serde_json::from_value(json!({"update_id":2,"stopped_message_generation":{
            "chat":{"id":42,"type":"private"},"draft_id":-1
        }}))?;
    assert_eq!(stopped.kind(), UpdateKind::StoppedMessageGeneration);
    assert_eq!(
        AllowedUpdate::from_kind(UpdateKind::StoppedMessageGeneration)?.as_str(),
        "stopped_message_generation"
    );
    let message: Message = serde_json::from_value(
        json!({"message_id":0,"date":0,"chat":{"id":42,"type":"private"},
            "ephemeral_message_id":15,"rich_message":{"blocks":[{"type":"paragraph","text":"hello"}]}
        }),
    )?;
    assert_eq!(message.kind(), MessageKind::RichMessage);
    assert!(!message.has_kind(MessageKind::Unknown));
    Ok(())
}

#[test]
fn rich_edits_inline_content_and_signed_draft_ids_are_supported() -> TestResult {
    let mut edit = EditMessageTextRequest::for_chat_message(42_i64, MessageId(1), "old")?;
    edit.text = None;
    edit.rich_message = Some(InputRichMessage::markdown("**new**"));
    edit.validate()?;
    assert!(serde_json::to_value(&edit)?.get("text").is_none());
    let mut article = InlineQueryResultArticle::new("result", "Title", "unused");
    article.input_message_content = InputRichMessageContent {
        rich_message: InputRichMessage::html("<p>rich</p>"),
    }
    .into();
    article.validate()?;
    let value = serde_json::to_value(article)?;
    assert!(value["input_message_content"].get("message_text").is_none());
    assert!(value["input_message_content"].get("rich_message").is_some());
    AdvancedSendMessageDraftRequest::new(42_i64, -1).validate()?;
    assert!(
        AdvancedSendMessageDraftRequest::new(42_i64, 0)
            .validate()
            .is_err()
    );
    Ok(())
}

#[test]
fn new_methods_have_concrete_response_types() {
    fn response<Q: AdvancedRequest<Response = R>, R: serde::de::DeserializeOwned>() {}
    response::<AdvancedSendLivePhotoRequest, Message>();
    response::<AdvancedSendRichMessageRequest, Message>();
    response::<AdvancedAnswerGuestQueryRequest, SentGuestMessage>();
    response::<AdvancedGetManagedBotAccessSettingsRequest, BotAccessSettings>();
    response::<AdvancedGetUserPersonalChatMessagesRequest, Vec<Message>>();
    response::<AdvancedEditEphemeralMessageTextRequest, bool>();
    response::<AdvancedSendRichMessageDraftRequest, bool>();
    response::<AdvancedAnswerChatJoinRequestQueryRequest, bool>();
    response::<AdvancedSendChatJoinRequestWebAppRequest, bool>();
    response::<AdvancedSetManagedBotAccessSettingsRequest, bool>();
    response::<AdvancedEditEphemeralMessageMediaRequest, bool>();
    response::<AdvancedEditEphemeralMessageCaptionRequest, bool>();
    response::<AdvancedEditEphemeralMessageReplyMarkupRequest, bool>();
    response::<AdvancedDeleteEphemeralMessageRequest, bool>();
    response::<AdvancedDeleteMessageReactionRequest, bool>();
    response::<AdvancedDeleteAllMessageReactionsRequest, bool>();
}

#[test]
fn new_keyboard_options_and_permissions_are_serialized() -> TestResult {
    let mut button = InlineKeyboardButton::new("Unavailable");
    button.disabled = Some(DisabledButton {});
    button.validate()?;
    let mut inline = InlineKeyboardMarkup::single_row(vec![button]);
    inline.force_reply = Some(true);
    let value = serde_json::to_value(inline)?;
    assert_eq!(value["force_reply"], true);
    assert_eq!(value["inline_keyboard"][0][0]["disabled"], json!({}));
    let permissions = ChatPermissions::allow_all();
    let value = serde_json::to_value(permissions)?;
    assert_eq!(value["can_react_to_messages"], true);
    assert_eq!(value["can_edit_tag"], true);
    let mut command = BotCommand::new("private", "Private reply")?;
    command.is_ephemeral = Some(true);
    assert_eq!(serde_json::to_value(command)?["is_ephemeral"], true);
    let link = InputPollOptionMedia::link("https://example.com");
    link.validate()?;
    assert_eq!(
        serde_json::to_value(link)?,
        json!({"type":"link","url":"https://example.com"})
    );
    Ok(())
}

#[test]
fn every_official_rich_text_and_block_variant_is_decoded() -> TestResult {
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/rich_message_variants.json"))?;
    let text: Vec<RichText> = serde_json::from_value(fixture["text"].clone())?;
    let input: Vec<InputRichBlock> = serde_json::from_value(fixture["input_blocks"].clone())?;
    let blocks: Vec<RichBlock> = serde_json::from_value(fixture["blocks"].clone())?;
    assert_eq!(text.len(), 26);
    assert_eq!(input.len(), 24);
    assert_eq!(blocks.len(), 24);
    for value in &text {
        value.validate()?;
    }
    for value in &input {
        value.validate()?;
    }
    let encoded = serde_json::to_value(&input)?;
    let decoded: Vec<InputRichBlock> = serde_json::from_value(encoded.clone())?;
    assert_eq!(serde_json::to_value(decoded)?, encoded);
    Ok(())
}

#[test]
fn rich_drafts_reject_new_media_but_allow_ordinary_links() -> TestResult {
    let media = InputRichMessage::blocks(vec![InputRichBlock::Photo {
        photo: InputMediaPhoto::new("https://example.com/photo.jpg"),
        caption: None,
    }]);
    assert!(
        AdvancedSendRichMessageDraftRequest::new(1_i64, -1, media)
            .validate()
            .is_err()
    );
    let text = InputRichMessage::blocks(vec![InputRichBlock::Paragraph {
        text: RichText::Entity(Box::new(RichTextEntity::Url {
            text: "link".into(),
            url: "https://example.com".into(),
        })),
    }]);
    AdvancedSendRichMessageDraftRequest::new(1_i64, -1, text).validate()?;
    Ok(())
}
