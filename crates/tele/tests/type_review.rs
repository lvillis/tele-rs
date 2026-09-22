use serde_json::json;
use tele::types::advanced::{AdvancedEditEphemeralMessageTextRequest, AdvancedRequest};
use tele::types::{
    InlineQueryResultArticle, InputMessageContent, InputRichMessageContent, LinkPreviewOptions,
    MessageId, UserId,
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn ephemeral_text_edits_validate_nested_link_preview_options() -> TestResult {
    let mut request =
        AdvancedEditEphemeralMessageTextRequest::new(-100_i64, UserId(42), MessageId(1));
    request.text = Some("Website: https://example.com".to_owned());
    let mut preview = LinkPreviewOptions::new();
    preview.prefer_small_media = Some(true);
    preview.prefer_large_media = Some(true);
    request.link_preview_options = Some(preview);
    assert!(request.validate().is_err());

    let mut preview = LinkPreviewOptions::new();
    preview.url = Some("not a URL".to_owned());
    request.link_preview_options = Some(preview);
    assert!(request.validate().is_err());

    request.link_preview_options = Some(LinkPreviewOptions::disabled());
    request.validate()?;
    Ok(())
}

#[test]
fn rich_query_results_reject_new_media_at_every_validation_entry_point() -> TestResult {
    for rich_message in [
        json!({"blocks": [{"type": "photo", "photo": {
            "type": "photo", "media": "https://example.com/photo.jpg"
        }}]}),
        json!({"blocks": [{"type": "details", "summary": "Details", "blocks": [
            {"type": "photo", "photo": {"type": "photo", "media": "attach://photo"}}
        ]}]}),
        json!({"html": "<img src=\"tg://photo?id=photo\">", "media": [
            {"id": "photo", "media": {"type": "photo", "media": "HTTP://example.com/photo.jpg"}}
        ]}),
        json!({"blocks": [{"type": "video", "video": {
            "type": "video", "media": "video-file-id", "cover": "https://example.com/cover.jpg"
        }}]}),
    ] {
        let content = InputRichMessageContent {
            rich_message: serde_json::from_value(rich_message)?,
        };
        assert!(content.validate().is_err());
        let content = InputMessageContent::from(content);
        assert!(content.validate().is_err());
        let mut article = InlineQueryResultArticle::new("result", "Title", "placeholder");
        article.input_message_content = content;
        assert!(article.validate().is_err());
    }
    Ok(())
}

#[test]
fn rich_query_results_allow_existing_media_and_ordinary_links() -> TestResult {
    for rich_message in [
        json!({"blocks": [
            {"type": "paragraph", "text": {"type": "url", "text": "Website", "url": "https://example.com"}},
            {"type": "photo", "photo": {"type": "photo", "media": "photo-file-id"},
             "caption": {"text": "attach://this-is-caption-text"}}
        ]}),
        json!({"html": "<a href=\"https://example.com\">Website</a>"}),
        json!({"markdown": "[Website](https://example.com)"}),
        json!({"html": "<img src=\"tg://photo?id=photo\">", "media": [
            {"id": "photo", "media": {"type": "photo", "media": "photo-file-id"}}
        ]}),
    ] {
        let content = InputRichMessageContent {
            rich_message: serde_json::from_value(rich_message)?,
        };
        content.validate()?;
        let mut article = InlineQueryResultArticle::new("result", "Title", "placeholder");
        article.input_message_content = content.into();
        article.validate()?;
    }
    Ok(())
}
