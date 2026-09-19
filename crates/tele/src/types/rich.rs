//! Structured rich message content supported by Telegram Bot API 10.3.
use crate::types::bot::User;
use crate::types::common::ParseMode;
use crate::types::message::{
    Animation, Audio, Document, InputMediaAnimation, InputMediaAudio, InputMediaDocument,
    InputMediaPhoto, InputMediaVideo, Location, MessageEntity, PhotoSize, Video, Voice,
};
use crate::types::telegram::WebAppInfo;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DisabledButton {}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LoginUrl {
    pub url: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub forward_text: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bot_username: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub request_write_access: Option<bool>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SwitchInlineQueryChosenChat {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub query: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allow_user_chats: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allow_bot_chats: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allow_group_chats: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allow_channel_chats: Option<bool>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CopyTextButton {
    pub text: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RichMessageButton {
    pub text: RichText,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub style: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub callback_data: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub web_app: Option<WebAppInfo>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub login_url: Option<LoginUrl>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub switch_inline_query: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub switch_inline_query_current_chat: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub switch_inline_query_chosen_chat: Option<SwitchInlineQueryChosenChat>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub copy_text: Option<CopyTextButton>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disabled: Option<DisabledButton>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RichBlockCaption {
    pub text: RichText,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub credit: Option<RichText>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RichBlockTableCell {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<RichText>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_header: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub colspan: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rowspan: Option<i64>,
    pub align: String,
    pub valign: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct RichBlockListItem {
    pub label: String,
    pub blocks: Vec<RichBlock>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub has_checkbox: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_checked: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<i64>,
    #[serde(rename = "type")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InputRichBlockListItem {
    pub blocks: Vec<InputRichBlock>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub has_checkbox: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_checked: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<i64>,
    #[serde(rename = "type")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct RichMessage {
    pub blocks: Vec<RichBlock>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_rtl: Option<bool>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct InputRichMessage {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blocks: Option<Vec<InputRichBlock>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub html: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub markdown: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub media: Option<Vec<InputRichMessageMedia>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_rtl: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub skip_entity_detection: Option<bool>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InputRichMessageMedia {
    pub id: String,
    pub media: InputRichMedia,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InputRichMessageContent {
    pub rich_message: InputRichMessage,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InputMediaVoiceNote {
    pub media: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub caption: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parse_mode: Option<ParseMode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub caption_entities: Option<Vec<MessageEntity>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration: Option<i64>,
}

/// Rich text is a string, a concatenation of rich texts, or a formatting entity.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RichText {
    Text(String),
    Concat(Vec<RichText>),
    Entity(Box<RichTextEntity>),
}
impl From<String> for RichText {
    fn from(value: String) -> Self {
        Self::Text(value)
    }
}
impl From<&str> for RichText {
    fn from(value: &str) -> Self {
        Self::Text(value.to_owned())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
#[non_exhaustive]
pub enum RichTextEntity {
    #[serde(rename = "bold")]
    Bold { text: RichText },
    #[serde(rename = "italic")]
    Italic { text: RichText },
    #[serde(rename = "underline")]
    Underline { text: RichText },
    #[serde(rename = "strikethrough")]
    Strikethrough { text: RichText },
    #[serde(rename = "spoiler")]
    Spoiler { text: RichText },
    #[serde(rename = "date_time")]
    DateTime {
        text: RichText,
        unix_time: i64,
        date_time_format: String,
    },
    #[serde(rename = "text_mention")]
    TextMention { text: RichText, user: User },
    #[serde(rename = "subscript")]
    Subscript { text: RichText },
    #[serde(rename = "superscript")]
    Superscript { text: RichText },
    #[serde(rename = "marked")]
    Marked { text: RichText },
    #[serde(rename = "code")]
    Code { text: RichText },
    #[serde(rename = "custom_emoji")]
    CustomEmoji {
        custom_emoji_id: String,
        alternative_text: String,
    },
    #[serde(rename = "mathematical_expression")]
    MathematicalExpression { expression: String },
    #[serde(rename = "url")]
    Url { text: RichText, url: String },
    #[serde(rename = "email_address")]
    EmailAddress {
        text: RichText,
        email_address: String,
    },
    #[serde(rename = "phone_number")]
    PhoneNumber {
        text: RichText,
        phone_number: String,
    },
    #[serde(rename = "bank_card_number")]
    BankCardNumber {
        text: RichText,
        bank_card_number: String,
    },
    #[serde(rename = "mention")]
    Mention { text: RichText, username: String },
    #[serde(rename = "hashtag")]
    Hashtag { text: RichText, hashtag: String },
    #[serde(rename = "cashtag")]
    Cashtag { text: RichText, cashtag: String },
    #[serde(rename = "bot_command")]
    BotCommand { text: RichText, bot_command: String },
    #[serde(rename = "button")]
    Button { button: RichMessageButton },
    #[serde(rename = "anchor")]
    Anchor { name: String },
    #[serde(rename = "anchor_link")]
    AnchorLink { text: RichText, anchor_name: String },
    #[serde(rename = "reference")]
    Reference { text: RichText, name: String },
    #[serde(rename = "reference_link")]
    ReferenceLink {
        text: RichText,
        reference_name: String,
    },
}

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "type")]
#[non_exhaustive]
pub enum RichBlock {
    #[serde(rename = "paragraph")]
    Paragraph { text: RichText },
    #[serde(rename = "heading")]
    SectionHeading { text: RichText, size: i64 },
    #[serde(rename = "pre")]
    Preformatted {
        text: RichText,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        language: Option<String>,
    },
    #[serde(rename = "footer")]
    Footer { text: RichText },
    #[serde(rename = "divider")]
    Divider {},
    #[serde(rename = "mathematical_expression")]
    MathematicalExpression { expression: String },
    #[serde(rename = "anchor")]
    Anchor { name: String },
    #[serde(rename = "list")]
    List { items: Vec<RichBlockListItem> },
    #[serde(rename = "blockquote")]
    BlockQuotation {
        blocks: Vec<RichBlock>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        credit: Option<RichText>,
    },
    #[serde(rename = "expandable_blockquote")]
    ExpandableBlockQuotation {
        text: RichText,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        credit: Option<RichText>,
    },
    #[serde(rename = "pullquote")]
    PullQuotation {
        text: RichText,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        credit: Option<RichText>,
    },
    #[serde(rename = "collage")]
    Collage {
        blocks: Vec<RichBlock>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        caption: Option<RichBlockCaption>,
    },
    #[serde(rename = "slideshow")]
    Slideshow {
        blocks: Vec<RichBlock>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        caption: Option<RichBlockCaption>,
    },
    #[serde(rename = "table")]
    Table {
        cells: Vec<Vec<RichBlockTableCell>>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        is_bordered: Option<bool>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        is_striped: Option<bool>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        is_compact: Option<bool>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        caption: Option<RichText>,
    },
    #[serde(rename = "details")]
    Details {
        summary: RichText,
        blocks: Vec<RichBlock>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        is_open: Option<bool>,
    },
    #[serde(rename = "map")]
    Map {
        location: Location,
        zoom: i64,
        width: i64,
        height: i64,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        caption: Option<RichBlockCaption>,
    },
    #[serde(rename = "buttons")]
    Buttons {
        buttons: Vec<RichMessageButton>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        align: Option<String>,
    },
    #[serde(rename = "animation")]
    Animation {
        animation: Animation,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        has_spoiler: Option<bool>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        caption: Option<RichBlockCaption>,
    },
    #[serde(rename = "audio")]
    Audio {
        audio: Audio,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        caption: Option<RichBlockCaption>,
    },
    #[serde(rename = "document")]
    Document {
        document: Document,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        caption: Option<RichBlockCaption>,
    },
    #[serde(rename = "photo")]
    Photo {
        photo: Vec<PhotoSize>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        has_spoiler: Option<bool>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        caption: Option<RichBlockCaption>,
    },
    #[serde(rename = "video")]
    Video {
        video: Video,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        has_spoiler: Option<bool>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        caption: Option<RichBlockCaption>,
    },
    #[serde(rename = "voice_note")]
    VoiceNote {
        voice_note: Voice,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        caption: Option<RichBlockCaption>,
    },
    #[serde(rename = "thinking")]
    Thinking { text: RichText },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
#[non_exhaustive]
pub enum InputRichBlock {
    #[serde(rename = "paragraph")]
    Paragraph { text: RichText },
    #[serde(rename = "heading")]
    SectionHeading { text: RichText, size: i64 },
    #[serde(rename = "pre")]
    Preformatted {
        text: RichText,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        language: Option<String>,
    },
    #[serde(rename = "footer")]
    Footer { text: RichText },
    #[serde(rename = "divider")]
    Divider {},
    #[serde(rename = "mathematical_expression")]
    MathematicalExpression { expression: String },
    #[serde(rename = "anchor")]
    Anchor { name: String },
    #[serde(rename = "list")]
    List { items: Vec<InputRichBlockListItem> },
    #[serde(rename = "blockquote")]
    BlockQuotation {
        blocks: Vec<InputRichBlock>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        credit: Option<RichText>,
    },
    #[serde(rename = "expandable_blockquote")]
    ExpandableBlockQuotation {
        text: RichText,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        credit: Option<RichText>,
    },
    #[serde(rename = "pullquote")]
    PullQuotation {
        text: RichText,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        credit: Option<RichText>,
    },
    #[serde(rename = "collage")]
    Collage {
        blocks: Vec<InputRichBlock>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        caption: Option<RichBlockCaption>,
    },
    #[serde(rename = "slideshow")]
    Slideshow {
        blocks: Vec<InputRichBlock>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        caption: Option<RichBlockCaption>,
    },
    #[serde(rename = "table")]
    Table {
        cells: Vec<Vec<RichBlockTableCell>>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        is_bordered: Option<bool>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        is_striped: Option<bool>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        is_compact: Option<bool>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        caption: Option<RichText>,
    },
    #[serde(rename = "details")]
    Details {
        summary: RichText,
        blocks: Vec<InputRichBlock>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        is_open: Option<bool>,
    },
    #[serde(rename = "map")]
    Map {
        location: Location,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        zoom: Option<i64>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        width: Option<i64>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        height: Option<i64>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        caption: Option<RichBlockCaption>,
    },
    #[serde(rename = "buttons")]
    Buttons {
        buttons: Vec<RichMessageButton>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        align: Option<String>,
    },
    #[serde(rename = "animation")]
    Animation {
        #[serde(serialize_with = "serialize_animation")]
        animation: InputMediaAnimation,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        caption: Option<RichBlockCaption>,
    },
    #[serde(rename = "audio")]
    Audio {
        #[serde(serialize_with = "serialize_audio")]
        audio: InputMediaAudio,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        caption: Option<RichBlockCaption>,
    },
    #[serde(rename = "document")]
    Document {
        #[serde(serialize_with = "serialize_document")]
        document: InputMediaDocument,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        caption: Option<RichBlockCaption>,
    },
    #[serde(rename = "photo")]
    Photo {
        #[serde(serialize_with = "serialize_photo")]
        photo: InputMediaPhoto,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        caption: Option<RichBlockCaption>,
    },
    #[serde(rename = "video")]
    Video {
        #[serde(serialize_with = "serialize_video")]
        video: InputMediaVideo,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        caption: Option<RichBlockCaption>,
    },
    #[serde(rename = "voice_note")]
    VoiceNote {
        #[serde(serialize_with = "serialize_voice_note")]
        voice_note: InputMediaVoiceNote,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        caption: Option<RichBlockCaption>,
    },
    #[serde(rename = "thinking")]
    Thinking { text: RichText },
}

/// Media accepted by rich messages; captions are ignored in this context.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum InputRichMedia {
    Animation(InputMediaAnimation),
    Audio(InputMediaAudio),
    Document(InputMediaDocument),
    Photo(InputMediaPhoto),
    Video(InputMediaVideo),
    VoiceNote(InputMediaVoiceNote),
}

fn serialize_animation<S: serde::Serializer>(
    value: &InputMediaAnimation,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    #[derive(Serialize)]
    #[serde(tag = "type")]
    enum Tagged<'a> {
        #[serde(rename = "animation")]
        Media(&'a InputMediaAnimation),
    }
    Tagged::Media(value).serialize(serializer)
}

fn serialize_audio<S: serde::Serializer>(
    value: &InputMediaAudio,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    #[derive(Serialize)]
    #[serde(tag = "type")]
    enum Tagged<'a> {
        #[serde(rename = "audio")]
        Media(&'a InputMediaAudio),
    }
    Tagged::Media(value).serialize(serializer)
}

fn serialize_document<S: serde::Serializer>(
    value: &InputMediaDocument,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    #[derive(Serialize)]
    #[serde(tag = "type")]
    enum Tagged<'a> {
        #[serde(rename = "document")]
        Media(&'a InputMediaDocument),
    }
    Tagged::Media(value).serialize(serializer)
}

fn serialize_photo<S: serde::Serializer>(
    value: &InputMediaPhoto,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    #[derive(Serialize)]
    #[serde(tag = "type")]
    enum Tagged<'a> {
        #[serde(rename = "photo")]
        Media(&'a InputMediaPhoto),
    }
    Tagged::Media(value).serialize(serializer)
}

fn serialize_video<S: serde::Serializer>(
    value: &InputMediaVideo,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    #[derive(Serialize)]
    #[serde(tag = "type")]
    enum Tagged<'a> {
        #[serde(rename = "video")]
        Media(&'a InputMediaVideo),
    }
    Tagged::Media(value).serialize(serializer)
}

fn serialize_voice_note<S: serde::Serializer>(
    value: &InputMediaVoiceNote,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    #[derive(Serialize)]
    #[serde(tag = "type")]
    enum Tagged<'a> {
        #[serde(rename = "voice_note")]
        Media(&'a InputMediaVoiceNote),
    }
    Tagged::Media(value).serialize(serializer)
}

fn invalid(reason: &str) -> crate::Error {
    crate::Error::InvalidRequest {
        reason: reason.to_owned(),
    }
}

impl InputRichMessage {
    pub fn html(html: impl Into<String>) -> Self {
        Self {
            html: Some(html.into()),
            ..Self::default()
        }
    }
    pub fn markdown(markdown: impl Into<String>) -> Self {
        Self {
            markdown: Some(markdown.into()),
            ..Self::default()
        }
    }
    pub fn blocks(blocks: Vec<InputRichBlock>) -> Self {
        Self {
            blocks: Some(blocks),
            ..Self::default()
        }
    }

    pub fn validate_for_draft(&self) -> crate::Result<()> {
        self.validate()?;
        let value = serde_json::to_value(self)
            .map_err(|source| crate::Error::SerializeRequest { source })?;
        let mut contains_new_media = false;
        crate::types::upload::visit_file_references(&value, &mut |reference| {
            contains_new_media |= reference.starts_with("attach://")
                || reference
                    .get(..7)
                    .is_some_and(|prefix| prefix.eq_ignore_ascii_case("http://"))
                || reference
                    .get(..8)
                    .is_some_and(|prefix| prefix.eq_ignore_ascii_case("https://"));
        });
        if contains_new_media {
            return Err(invalid(
                "rich message drafts require previously uploaded media",
            ));
        }
        Ok(())
    }

    pub fn validate(&self) -> crate::Result<()> {
        self.validate_with_upload(false)
    }
    pub fn validate_for_upload(&self) -> crate::Result<()> {
        self.validate_with_upload(true)
    }
    fn validate_with_upload(&self, allow_upload: bool) -> crate::Result<()> {
        if usize::from(self.html.is_some())
            + usize::from(self.markdown.is_some())
            + usize::from(self.blocks.is_some())
            != 1
        {
            return Err(invalid(
                "rich_message requires exactly one of html, markdown, or blocks",
            ));
        }
        if let Some(blocks) = &self.blocks {
            for block in blocks {
                block.validate_with_upload(allow_upload)?;
            }
        }
        if let Some(media) = &self.media {
            let mut ids = std::collections::BTreeSet::new();
            for entry in media {
                if entry.id.is_empty()
                    || entry.id.len() > 64
                    || !entry
                        .id
                        .bytes()
                        .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
                {
                    return Err(invalid(
                        "rich message media id must contain 1-64 ASCII letters, digits, underscores or hyphens",
                    ));
                }
                if !ids.insert(&entry.id) {
                    return Err(invalid("rich message media ids must be unique"));
                }
                entry.media.validate_with_upload(allow_upload)?;
            }
        }
        Ok(())
    }
}

impl InputMediaVoiceNote {
    pub fn new(media: impl Into<String>) -> Self {
        Self {
            media: media.into(),
            caption: None,
            parse_mode: None,
            caption_entities: None,
            duration: None,
        }
    }
    pub fn validate_for_upload(&self) -> crate::Result<()> {
        self.validate_content()
    }
    pub fn validate(&self) -> crate::Result<()> {
        if self.media.starts_with("attach://") {
            return Err(invalid("voice note attachments require multipart upload"));
        }
        self.validate_content()
    }
    fn validate_content(&self) -> crate::Result<()> {
        crate::types::validation::required_text("voice_note media", &self.media)?;
        crate::types::validation::optional_caption(self.caption.as_deref())?;
        crate::types::validation::optional_text_formatting(
            "voice_note caption",
            self.caption.as_deref(),
            self.parse_mode,
            self.caption_entities.as_deref(),
        )?;
        if self.duration.is_some_and(|d| d < 0) {
            return Err(invalid("voice note duration must not be negative"));
        }
        Ok(())
    }
}
impl InputRichMedia {
    pub fn validate(&self) -> crate::Result<()> {
        self.validate_with_upload(false)
    }
    pub fn validate_for_upload(&self) -> crate::Result<()> {
        self.validate_with_upload(true)
    }
    fn validate_with_upload(&self, allow_upload: bool) -> crate::Result<()> {
        match self {
            Self::Animation(media) => {
                if allow_upload {
                    media.validate_for_upload()
                } else {
                    media.validate()
                }
            }
            Self::Audio(media) => {
                if allow_upload {
                    media.validate_for_upload()
                } else {
                    media.validate()
                }
            }
            Self::Document(media) => {
                if allow_upload {
                    media.validate_for_upload()
                } else {
                    media.validate()
                }
            }
            Self::Photo(media) => {
                if allow_upload {
                    media.validate_for_upload()
                } else {
                    media.validate()
                }
            }
            Self::Video(media) => {
                if allow_upload {
                    media.validate_for_upload()
                } else {
                    media.validate()
                }
            }
            Self::VoiceNote(media) => {
                if allow_upload {
                    media.validate_for_upload()
                } else {
                    media.validate()
                }
            }
        }
    }
}
impl RichText {
    pub fn validate(&self) -> crate::Result<()> {
        match self {
            Self::Text(_) => Ok(()),
            Self::Concat(parts) => {
                for part in parts {
                    part.validate()?;
                }
                Ok(())
            }
            Self::Entity(entity) => entity.validate(),
        }
    }
}
impl RichBlockCaption {
    pub fn validate(&self) -> crate::Result<()> {
        self.text.validate()?;
        if let Some(credit) = &self.credit {
            credit.validate()?;
        }
        Ok(())
    }
}
impl RichMessageButton {
    pub fn validate(&self) -> crate::Result<()> {
        self.text.validate()?;
        let actions = [
            self.url.is_some(),
            self.callback_data.is_some(),
            self.web_app.is_some(),
            self.login_url.is_some(),
            self.switch_inline_query.is_some(),
            self.switch_inline_query_current_chat.is_some(),
            self.switch_inline_query_chosen_chat.is_some(),
            self.copy_text.is_some(),
            self.disabled.is_some(),
        ];
        if actions.into_iter().filter(|set| *set).count() != 1 {
            return Err(invalid("rich message button requires exactly one action"));
        }
        if let Some(data) = &self.callback_data
            && (data.is_empty() || data.len() > 64)
        {
            return Err(invalid("callback_data must contain 1-64 bytes"));
        }
        if let Some(style) = &self.style {
            if !matches!(style.as_str(), "danger" | "success" | "primary" | "link") {
                return Err(invalid("unsupported rich button style"));
            }
            if style == "link" && self.callback_data.is_none() {
                return Err(invalid("link style requires a callback button"));
            }
        }
        if let Some(copy) = &self.copy_text {
            crate::types::validation::text_length_range("copy_text", &copy.text, 1, 256)?;
        }
        if let Some(web_app) = &self.web_app {
            web_app.validate()?;
        }
        if let Some(login) = &self.login_url {
            crate::types::validation::https_url("login_url", &login.url)?;
        }
        Ok(())
    }
}

impl RichTextEntity {
    pub fn validate(&self) -> crate::Result<()> {
        match self {
            Self::Bold { text, .. } => {
                text.validate()?;
            }
            Self::Italic { text, .. } => {
                text.validate()?;
            }
            Self::Underline { text, .. } => {
                text.validate()?;
            }
            Self::Strikethrough { text, .. } => {
                text.validate()?;
            }
            Self::Spoiler { text, .. } => {
                text.validate()?;
            }
            Self::DateTime { text, .. } => {
                text.validate()?;
            }
            Self::TextMention { text, .. } => {
                text.validate()?;
            }
            Self::Subscript { text, .. } => {
                text.validate()?;
            }
            Self::Superscript { text, .. } => {
                text.validate()?;
            }
            Self::Marked { text, .. } => {
                text.validate()?;
            }
            Self::Code { text, .. } => {
                text.validate()?;
            }
            Self::CustomEmoji { .. } => {}
            Self::MathematicalExpression { .. } => {}
            Self::Url { text, .. } => {
                text.validate()?;
            }
            Self::EmailAddress { text, .. } => {
                text.validate()?;
            }
            Self::PhoneNumber { text, .. } => {
                text.validate()?;
            }
            Self::BankCardNumber { text, .. } => {
                text.validate()?;
            }
            Self::Mention { text, .. } => {
                text.validate()?;
            }
            Self::Hashtag { text, .. } => {
                text.validate()?;
            }
            Self::Cashtag { text, .. } => {
                text.validate()?;
            }
            Self::BotCommand { text, .. } => {
                text.validate()?;
            }
            Self::Button { button, .. } => {
                button.validate()?;
            }
            Self::Anchor { .. } => {}
            Self::AnchorLink { text, .. } => {
                text.validate()?;
            }
            Self::Reference { text, .. } => {
                text.validate()?;
            }
            Self::ReferenceLink { text, .. } => {
                text.validate()?;
            }
        }
        Ok(())
    }
}

impl InputRichBlock {
    pub fn validate(&self) -> crate::Result<()> {
        self.validate_with_upload(false)
    }
    pub fn validate_for_upload(&self) -> crate::Result<()> {
        self.validate_with_upload(true)
    }
    fn validate_with_upload(&self, allow_upload: bool) -> crate::Result<()> {
        match self {
            Self::Paragraph { text, .. } => {
                text.validate()?;
            }
            Self::SectionHeading { text, size, .. } => {
                text.validate()?;
                if !(1..=6).contains(size) {
                    return Err(invalid("rich heading size must be 1-6"));
                }
            }
            Self::Preformatted { text, .. } => {
                text.validate()?;
            }
            Self::Footer { text, .. } => {
                text.validate()?;
            }
            Self::Divider { .. } => {}
            Self::MathematicalExpression { .. } => {}
            Self::Anchor { .. } => {}
            Self::List { items, .. } => {
                for item in items {
                    for block in &item.blocks {
                        block.validate_with_upload(allow_upload)?;
                    }
                }
            }
            Self::BlockQuotation { blocks, credit, .. } => {
                for value in blocks {
                    value.validate_with_upload(allow_upload)?;
                }
                if let Some(credit) = credit {
                    credit.validate()?;
                }
            }
            Self::ExpandableBlockQuotation { text, credit, .. } => {
                text.validate()?;
                if let Some(credit) = credit {
                    credit.validate()?;
                }
            }
            Self::PullQuotation { text, credit, .. } => {
                text.validate()?;
                if let Some(credit) = credit {
                    credit.validate()?;
                }
            }
            Self::Collage {
                blocks, caption, ..
            } => {
                for value in blocks {
                    value.validate_with_upload(allow_upload)?;
                }
                if let Some(caption) = caption {
                    caption.validate()?;
                }
            }
            Self::Slideshow {
                blocks, caption, ..
            } => {
                for value in blocks {
                    value.validate_with_upload(allow_upload)?;
                }
                if let Some(caption) = caption {
                    caption.validate()?;
                }
            }
            Self::Table { cells, caption, .. } => {
                for row in cells {
                    for cell in row {
                        if let Some(text) = &cell.text {
                            text.validate()?;
                        }
                    }
                }
                if let Some(caption) = caption {
                    caption.validate()?;
                }
            }
            Self::Details {
                summary, blocks, ..
            } => {
                summary.validate()?;
                for value in blocks {
                    value.validate_with_upload(allow_upload)?;
                }
            }
            Self::Map {
                caption,
                zoom,
                width,
                height,
                ..
            } => {
                if let Some(caption) = caption {
                    caption.validate()?;
                }
                if zoom.is_some_and(|value| !(0..=24).contains(&value)) {
                    return Err(invalid("rich map zoom is out of range"));
                }
                if width.is_some_and(|value| !(0..=10000).contains(&value)) {
                    return Err(invalid("rich map width is out of range"));
                }
                if height.is_some_and(|value| !(0..=10000).contains(&value)) {
                    return Err(invalid("rich map height is out of range"));
                }
            }
            Self::Buttons { buttons, .. } => {
                for value in buttons {
                    value.validate()?;
                }
                if !(1..=8).contains(&buttons.len()) {
                    return Err(invalid("rich button blocks require 1-8 buttons"));
                }
            }
            Self::Animation {
                animation, caption, ..
            } => {
                if allow_upload {
                    animation.validate_for_upload()?;
                } else {
                    animation.validate()?;
                }
                if let Some(caption) = caption {
                    caption.validate()?;
                }
            }
            Self::Audio { audio, caption, .. } => {
                if allow_upload {
                    audio.validate_for_upload()?;
                } else {
                    audio.validate()?;
                }
                if let Some(caption) = caption {
                    caption.validate()?;
                }
            }
            Self::Document {
                document, caption, ..
            } => {
                if allow_upload {
                    document.validate_for_upload()?;
                } else {
                    document.validate()?;
                }
                if let Some(caption) = caption {
                    caption.validate()?;
                }
            }
            Self::Photo { photo, caption, .. } => {
                if allow_upload {
                    photo.validate_for_upload()?;
                } else {
                    photo.validate()?;
                }
                if let Some(caption) = caption {
                    caption.validate()?;
                }
            }
            Self::Video { video, caption, .. } => {
                if allow_upload {
                    video.validate_for_upload()?;
                } else {
                    video.validate()?;
                }
                if let Some(caption) = caption {
                    caption.validate()?;
                }
            }
            Self::VoiceNote {
                voice_note,
                caption,
                ..
            } => {
                if allow_upload {
                    voice_note.validate_for_upload()?;
                } else {
                    voice_note.validate()?;
                }
                if let Some(caption) = caption {
                    caption.validate()?;
                }
            }
            Self::Thinking { text, .. } => {
                text.validate()?;
            }
        }
        Ok(())
    }
}
