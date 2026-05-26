use std::collections::HashMap;
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::spec::{BotApiSpec, MethodSpec, ParamSpec};

const SPEC_FILE: &str = "bot_api.json";
const DOMAIN_ORDER: [&str; 7] = [
    "business", "forum", "gifts", "payments", "stickers", "stories", "misc",
];

struct Paths {
    spec: PathBuf,
    types_root: PathBuf,
    api_methods: PathBuf,
    types_dir: PathBuf,
}

struct GeneratedFile {
    path: PathBuf,
    content: String,
}

struct TempDir {
    path: PathBuf,
}

impl TempDir {
    fn create(name: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let mut path = std::env::temp_dir();
        path.push(format!("{name}-{}", std::process::id()));
        if path.exists() {
            fs::remove_dir_all(&path)?;
        }
        fs::create_dir_all(&path)?;
        Ok(Self { path })
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

pub(crate) fn generate() -> Result<(), Box<dyn std::error::Error>> {
    let files = render_generated_files()?;
    write_generated_files(&files)?;
    let paths = files
        .iter()
        .map(|file| file.path.clone())
        .collect::<Vec<_>>();
    run_rustfmt(&paths)?;
    Ok(())
}

pub(crate) fn check() -> Result<(), Box<dyn std::error::Error>> {
    let files = format_generated_files(&render_generated_files()?)?;
    let changed = files
        .iter()
        .filter(|file| {
            fs::read_to_string(&file.path).ok().as_deref() != Some(file.content.as_str())
        })
        .map(|file| file.path.as_path())
        .collect::<Vec<_>>();

    if !changed.is_empty() {
        let mut message = String::from("generated advanced API files are out of date:");
        for path in changed {
            let _ = write!(&mut message, "\n- {}", path.display());
        }
        message.push_str("\nrun `cargo run -p tele-codegen -- gen-advanced`");
        return Err(message.into());
    }

    Ok(())
}

fn render_generated_files() -> Result<Vec<GeneratedFile>, Box<dyn std::error::Error>> {
    let paths = paths()?;
    let spec: BotApiSpec = serde_json::from_slice(&fs::read(&paths.spec)?)?;
    spec.validate()
        .map_err(|error| -> Box<dyn std::error::Error> { error.into() })?;

    let advanced_methods = json_advanced_methods(&spec);

    let mut grouped: HashMap<&'static str, Vec<&MethodSpec>> = DOMAIN_ORDER
        .into_iter()
        .map(|domain| (domain, Vec::new()))
        .collect();
    for &method in &advanced_methods {
        let domain = domain_for_method(&method.fn_name);
        let Some(methods) = grouped.get_mut(domain) else {
            return Err(format!("unknown advanced method domain `{domain}`").into());
        };
        methods.push(method);
    }

    let mut files = Vec::with_capacity(DOMAIN_ORDER.len() + 2);
    files.push(GeneratedFile {
        path: paths.types_root,
        content: generate_types_root(&grouped),
    });

    for domain in DOMAIN_ORDER {
        files.push(GeneratedFile {
            path: paths.types_dir.join(format!("advanced_{domain}.rs")),
            content: generate_domain_module(grouped.get(domain).map_or(&[], Vec::as_slice)),
        });
    }

    files.push(GeneratedFile {
        path: paths.api_methods,
        content: generate_api_methods(&advanced_methods),
    });
    Ok(files)
}

fn write_generated_files(files: &[GeneratedFile]) -> Result<(), Box<dyn std::error::Error>> {
    for file in files {
        write_if_changed(&file.path, &file.content)?;
    }
    Ok(())
}

fn format_generated_files(
    files: &[GeneratedFile],
) -> Result<Vec<GeneratedFile>, Box<dyn std::error::Error>> {
    let temp_dir = TempDir::create("tele-codegen-advanced")?;
    let mut temp_paths = Vec::with_capacity(files.len());
    for file in files {
        let file_name = file.path.file_name().ok_or_else(|| {
            format!(
                "generated file path has no file name: {}",
                file.path.display()
            )
        })?;
        let temp_path = temp_dir.path.join(file_name);
        fs::write(&temp_path, &file.content)?;
        temp_paths.push(temp_path);
    }
    run_rustfmt(&temp_paths)?;
    files
        .iter()
        .zip(temp_paths)
        .map(|(file, temp_path)| {
            Ok(GeneratedFile {
                path: file.path.clone(),
                content: fs::read_to_string(temp_path)?,
            })
        })
        .collect()
}

fn run_rustfmt(paths: &[PathBuf]) -> Result<(), Box<dyn std::error::Error>> {
    if paths.is_empty() {
        return Ok(());
    }
    let output = Command::new("rustfmt")
        .args(["--edition", "2024"])
        .args(paths)
        .output()?;
    if !output.status.success() {
        return Err(format!(
            "rustfmt failed with status {}\n{}{}",
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }

    Ok(())
}

fn paths() -> Result<Paths, Box<dyn std::error::Error>> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let root = workspace_root()?;
    let spec = resolve_spec_path(&manifest_dir)?;
    let types_dir = root.join("crates/tele/src/types");

    Ok(Paths {
        spec,
        types_root: types_dir.join("advanced.rs"),
        api_methods: root.join("crates/tele/src/api/advanced_methods.inc.rs"),
        types_dir,
    })
}

fn workspace_root() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .ok_or_else(|| "failed to resolve workspace root".into())
}

fn resolve_spec_path(codegen_dir: &Path) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let mut candidates = Vec::new();
    if let Some(value) = std::env::var_os("TELE_ADVANCED_SPEC_PATH") {
        candidates.push(PathBuf::from(value));
    }
    candidates.push(codegen_dir.join("spec").join(SPEC_FILE));

    for candidate in &candidates {
        if candidate.exists() {
            return Ok(candidate.clone());
        }
    }

    let mut rendered = String::new();
    for candidate in &candidates {
        let _ = writeln!(&mut rendered, "- {}", candidate.display());
    }
    Err(format!("Could not locate Telegram API spec JSON. Checked:\n{rendered}").into())
}

fn to_pascal_case(snake: &str) -> String {
    snake
        .split('_')
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => {
                    let mut out = String::new();
                    out.extend(first.to_uppercase());
                    out.push_str(chars.as_str());
                    out
                }
                None => String::new(),
            }
        })
        .collect()
}

fn request_type_name(fn_name: &str) -> String {
    let mut base = to_pascal_case(fn_name);
    if !base.ends_with("Request") {
        base.push_str("Request");
    }
    format!("Advanced{base}")
}

fn typed_fn_name(fn_name: &str) -> String {
    format!("{fn_name}_typed")
}

fn requires_multipart_upload(method: &MethodSpec) -> bool {
    method
        .params
        .iter()
        .any(|param| param.required && param.type_raw == "InputFile")
}

fn json_advanced_methods(spec: &BotApiSpec) -> Vec<&MethodSpec> {
    spec.advanced_methods
        .iter()
        .filter(|method| !requires_multipart_upload(method))
        .collect()
}

fn qualify_common_type(field_ty: &str) -> String {
    field_ty
        .replace("NumericChatId", "crate::types::common::NumericChatId")
        .replace("ChatId", "crate::types::common::ChatId")
        .replace("MessageId", "crate::types::common::MessageId")
        .replace("UserId", "crate::types::common::UserId")
}

fn ctor_arg_type(field_ty: &str) -> &str {
    match field_ty {
        "String" => "impl Into<String>",
        "crate::types::common::ChatId" => "impl Into<crate::types::common::ChatId>",
        "crate::types::common::NumericChatId" => "impl Into<crate::types::common::NumericChatId>",
        _ => field_ty,
    }
}

fn ctor_assign(field_name: &str, field_ty: &str) -> String {
    match field_ty {
        "String" | "crate::types::common::ChatId" | "crate::types::common::NumericChatId" => {
            format!("{field_name}: {field_name}.into()")
        }
        _ => field_name.to_owned(),
    }
}

fn map_raw_type(type_raw: &str) -> Option<String> {
    let base_map = [
        ("InputSticker", "crate::types::sticker::InputSticker"),
        ("InputMedia", "crate::types::message::InputMedia"),
        ("LabeledPrice", "crate::types::payment::LabeledPrice"),
        ("ShippingOption", "crate::types::payment::ShippingOption"),
        ("MessageEntity", "crate::types::message::MessageEntity"),
        (
            "InlineQueryResult",
            "crate::types::telegram::InlineQueryResult",
        ),
        ("InputChecklist", "crate::types::telegram::InputChecklist"),
        ("KeyboardButton", "crate::types::telegram::KeyboardButton"),
        (
            "InlineKeyboardMarkup",
            "crate::types::telegram::InlineKeyboardMarkup",
        ),
        (
            "InputProfilePhoto",
            "crate::types::telegram::InputProfilePhoto",
        ),
        (
            "InputStoryContent",
            "crate::types::telegram::InputStoryContent",
        ),
        ("StoryArea", "crate::types::telegram::StoryArea"),
        ("ReplyParameters", "crate::types::telegram::ReplyParameters"),
        (
            "SuggestedPostParameters",
            "crate::types::telegram::SuggestedPostParameters",
        ),
        ("InputPaidMedia", "crate::types::telegram::InputPaidMedia"),
        (
            "AcceptedGiftTypes",
            "crate::types::telegram::AcceptedGiftTypes",
        ),
        ("MenuButton", "crate::types::telegram::MenuButton"),
        ("ReactionType", "crate::types::telegram::ReactionType"),
        (
            "ChatAdministratorRights",
            "crate::types::chat::ChatAdministratorRights",
        ),
        (
            "PassportElementError",
            "crate::types::telegram::PassportElementError",
        ),
        ("MaskPosition", "crate::types::sticker::MaskPosition"),
        (
            "InlineKeyboardMarkup or ReplyKeyboardMarkup or ReplyKeyboardRemove or ForceReply",
            "crate::types::telegram::ReplyMarkup",
        ),
    ];

    let normalized = type_raw.trim();
    if let Some(item_raw) = normalized.strip_prefix("Array of ") {
        let item_ty = base_map
            .iter()
            .find_map(|(raw, ty)| (*raw == item_raw.trim()).then_some(*ty))?;
        return Some(format!("Vec<{item_ty}>"));
    }

    base_map
        .iter()
        .find_map(|(raw, ty)| (*raw == normalized).then_some((*ty).to_owned()))
}

fn resolve_param_type(param: &ParamSpec) -> String {
    match param.field_name.as_str() {
        "parse_mode"
        | "caption_parse_mode"
        | "explanation_parse_mode"
        | "text_parse_mode"
        | "quote_parse_mode"
            if param.type_raw == "String" =>
        {
            return "crate::types::common::ParseMode".to_owned();
        }
        "chat_id" | "from_chat_id" | "new_owner_chat_id"
            if param.type_raw == "Integer" && param.type_rust == "i64" =>
        {
            return "crate::types::common::NumericChatId".to_owned();
        }
        "sticker_format" | "format" if param.type_raw == "String" => {
            return "crate::types::sticker::StickerFormat".to_owned();
        }
        "sticker_type" if param.type_raw == "String" => {
            return "crate::types::sticker::StickerType".to_owned();
        }
        _ => {}
    }

    let current = qualify_common_type(&param.type_rust);
    if !current.contains("Value") {
        return current;
    }

    map_raw_type(&param.type_raw).unwrap_or(current)
}

fn response_type(method: &str, return_desc: &str) -> &'static str {
    match method {
        "getUpdates" => "Vec<crate::types::update::Update>",
        "getWebhookInfo" => "crate::types::webhook::WebhookInfo",
        "getMe" => "crate::types::bot::User",
        "sendMessage" => "crate::types::message::Message",
        "forwardMessage" => "crate::types::message::Message",
        "forwardMessages" => "Vec<crate::types::message::MessageIdObject>",
        "copyMessage" => "crate::types::message::MessageIdObject",
        "copyMessages" => "Vec<crate::types::message::MessageIdObject>",
        "sendPhoto" => "crate::types::message::Message",
        "sendAudio" => "crate::types::message::Message",
        "sendDocument" => "crate::types::message::Message",
        "sendVideo" => "crate::types::message::Message",
        "sendAnimation" => "crate::types::message::Message",
        "sendVoice" => "crate::types::message::Message",
        "sendVideoNote" => "crate::types::message::Message",
        "sendPaidMedia" => "crate::types::message::Message",
        "sendMediaGroup" => "Vec<crate::types::message::Message>",
        "sendLocation" => "crate::types::message::Message",
        "sendVenue" => "crate::types::message::Message",
        "sendContact" => "crate::types::message::Message",
        "sendPoll" => "crate::types::message::Message",
        "sendChecklist" => "crate::types::message::Message",
        "sendDice" => "crate::types::message::Message",
        "sendGame" => "crate::types::message::Message",
        "getUserProfilePhotos" => "crate::types::bot::UserProfilePhotos",
        "getUserProfileAudios" => "crate::types::bot::UserProfileAudios",
        "getFile" => "crate::types::file::File",
        "exportChatInviteLink" => "String",
        "createChatInviteLink" => "crate::types::chat::ChatInviteLink",
        "editChatInviteLink" => "crate::types::chat::ChatInviteLink",
        "createChatSubscriptionInviteLink" => "crate::types::chat::ChatInviteLink",
        "editChatSubscriptionInviteLink" => "crate::types::chat::ChatInviteLink",
        "revokeChatInviteLink" => "crate::types::chat::ChatInviteLink",
        "getChatAdministrators" => "Vec<crate::types::chat::ChatMember>",
        "getChatMemberCount" => "u64",
        "getChatMember" => "crate::types::chat::ChatMember",
        "getUserChatBoosts" => "crate::types::update::UserChatBoosts",
        "getForumTopicIconStickers" => "Vec<crate::types::sticker::Sticker>",
        "createForumTopic" => "crate::types::message::ForumTopic",
        "getChatMenuButton" => "crate::types::telegram::MenuButton",
        "getMyCommands" => "Vec<crate::types::command::BotCommand>",
        "getMyName" => "crate::types::command::BotName",
        "getMyDescription" => "crate::types::command::BotDescription",
        "getMyShortDescription" => "crate::types::command::BotShortDescription",
        "getMyDefaultAdministratorRights" => "crate::types::chat::ChatAdministratorRights",
        "getAvailableGifts" => "crate::types::gift::Gifts",
        "editMessageText" => "crate::types::message::EditMessageResult",
        "editMessageCaption" => "crate::types::message::EditMessageResult",
        "editMessageMedia" => "crate::types::message::EditMessageResult",
        "editMessageLiveLocation" => "crate::types::message::EditMessageResult",
        "stopMessageLiveLocation" => "crate::types::message::EditMessageResult",
        "editMessageChecklist" => "crate::types::message::Message",
        "editMessageReplyMarkup" => "crate::types::message::EditMessageResult",
        "stopPoll" => "crate::types::message::Poll",
        "sendSticker" => "crate::types::message::Message",
        "answerWebAppQuery" => "crate::types::message::SentWebAppMessage",
        "answerShippingQuery" => "bool",
        "answerPreCheckoutQuery" => "bool",
        "getStickerSet" => "crate::types::sticker::StickerSet",
        "getCustomEmojiStickers" => "Vec<crate::types::sticker::Sticker>",
        "uploadStickerFile" => "crate::types::file::File",
        "sendInvoice" => "crate::types::message::Message",
        "createInvoiceLink" => "String",
        "getBusinessConnection" => "crate::types::update::BusinessConnection",
        "getBusinessAccountGifts" | "getUserGifts" | "getChatGifts" => {
            "crate::types::gift::OwnedGifts"
        }
        "getBusinessAccountStarBalance" => "crate::types::message::StarAmount",
        "getMyStarBalance" => "crate::types::message::StarAmount",
        "getStarTransactions" => "crate::types::gift::StarTransactions",
        "savePreparedInlineMessage" => "crate::types::telegram::PreparedInlineMessage",
        "savePreparedKeyboardButton" => "crate::types::telegram::PreparedKeyboardButton",
        "setGameScore" => "crate::types::message::EditMessageResult",
        "getGameHighScores" => "Vec<crate::types::message::GameHighScore>",
        "postStory" | "repostStory" | "editStory" => "crate::types::message::Story",
        _ if return_desc.contains("True") => "bool",
        _ if return_desc.contains("String") => "String",
        _ if return_desc.contains("Array of") && return_desc.contains("Sticker") => {
            "Vec<crate::types::sticker::Sticker>"
        }
        _ if return_desc.contains("Array of") && return_desc.contains("MessageId") => {
            "Vec<crate::types::message::MessageIdObject>"
        }
        _ if return_desc.contains("MessageId") => "crate::types::message::MessageIdObject",
        _ if return_desc.contains("Array of") && return_desc.contains("Message") => {
            "Vec<crate::types::message::Message>"
        }
        _ if return_desc.contains("Message") => "crate::types::message::Message",
        _ if return_desc.contains("Int") => "u64",
        _ => "Value",
    }
}

const TYPES_WITH_VALIDATE: &[&str] = &[
    "crate::types::common::ChatId",
    "crate::types::common::NumericChatId",
    "crate::types::common::UserId",
    "crate::types::common::MessageId",
    "crate::types::message::InputMedia",
    "crate::types::payment::LabeledPrice",
    "crate::types::payment::ShippingOption",
    "crate::types::sticker::InputSticker",
    "crate::types::sticker::MaskPosition",
    "crate::types::telegram::AcceptedGiftTypes",
    "crate::types::telegram::InlineKeyboardMarkup",
    "crate::types::telegram::InlineQueryResult",
    "crate::types::telegram::InputChecklist",
    "crate::types::telegram::InputPaidMedia",
    "crate::types::telegram::InputProfilePhoto",
    "crate::types::telegram::InputStoryContent",
    "crate::types::telegram::KeyboardButton",
    "crate::types::telegram::MenuButton",
    "crate::types::telegram::PassportElementError",
    "crate::types::telegram::ReactionType",
    "crate::types::telegram::ReplyMarkup",
    "crate::types::telegram::ReplyParameters",
    "crate::types::telegram::StoryArea",
    "crate::types::telegram::SuggestedPostParameters",
];

fn type_with_validate(field_ty: &str) -> Option<&'static str> {
    TYPES_WITH_VALIDATE
        .iter()
        .copied()
        .find(|ty| *ty == field_ty)
}

fn type_has_validate(field_ty: &str) -> bool {
    type_with_validate(field_ty).is_some()
}

fn validated_vec_item_type(field_ty: &str) -> Option<&'static str> {
    field_ty
        .strip_prefix("Vec<")
        .and_then(|inner| inner.strip_suffix('>'))
        .and_then(type_with_validate)
}

fn positive_i64_field(field_name: &str) -> bool {
    matches!(
        field_name,
        "active_period"
            | "direct_messages_topic_id"
            | "draft_id"
            | "duration"
            | "emoji_status_expiration_date"
            | "from_story_id"
            | "length"
            | "limit"
            | "max_tip_amount"
            | "message_thread_id"
            | "month_count"
            | "photo_height"
            | "photo_size"
            | "photo_width"
            | "send_date"
            | "star_count"
            | "story_id"
            | "subscription_period"
            | "subscription_price"
    )
}

fn non_negative_i64_field(field_name: &str) -> bool {
    matches!(field_name, "offset" | "position" | "score")
}

fn i64_field_range(method: &MethodSpec, param: &ParamSpec) -> Option<(i64, i64)> {
    match (method.method.as_str(), param.field_name.as_str()) {
        (
            "getUserProfileAudios"
            | "getUserGifts"
            | "getChatGifts"
            | "getBusinessAccountGifts"
            | "getStarTransactions",
            "limit",
        ) => Some((1, 100)),
        _ => None,
    }
}

fn suggested_post_approval_send_date_field(method: &MethodSpec, param: &ParamSpec) -> bool {
    method.method == "approveSuggestedPost" && param.field_name == "send_date"
}

fn string_id_field(field_name: &str) -> bool {
    matches!(
        field_name,
        "business_connection_id"
            | "custom_emoji_id"
            | "inline_message_id"
            | "message_effect_id"
            | "prepared_message_id"
            | "web_app_query_id"
    ) || field_name.ends_with("_id")
}

fn ordered_message_ids_field(method: &MethodSpec, param: &ParamSpec) -> bool {
    matches!(
        (method.method.as_str(), param.field_name.as_str()),
        ("forwardMessages", "message_ids")
    )
}

fn control_free_string_field(method: &MethodSpec, param: &ParamSpec) -> bool {
    matches!(param.field_name.as_str(), "offset")
        || matches!(
            (method.method.as_str(), param.field_name.as_str()),
            ("setUserEmojiStatus", "emoji_status_custom_emoji_id")
        )
}

fn non_empty_control_free_string_field(method: &MethodSpec, param: &ParamSpec) -> bool {
    if matches!(param.field_name.as_str(), "emoji" | "thumbnail") {
        return true;
    }

    matches!(
        (method.method.as_str(), param.field_name.as_str()),
        (
            "getStickerSet"
                | "createNewStickerSet"
                | "addStickerToSet"
                | "replaceStickerInSet"
                | "setStickerSetTitle"
                | "setStickerSetThumbnail"
                | "setCustomEmojiStickerSetThumbnail"
                | "deleteStickerSet",
            "name"
        ) | ("createNewStickerSet" | "setStickerSetTitle", "title")
            | (
                "sendSticker"
                    | "setStickerPositionInSet"
                    | "deleteStickerFromSet"
                    | "setStickerEmojiList"
                    | "setStickerKeywords"
                    | "setStickerMaskPosition",
                "sticker"
            )
            | ("replaceStickerInSet", "old_sticker")
    )
}

fn emoji_free_string_limit(method: &MethodSpec, param: &ParamSpec) -> Option<&'static str> {
    match (method.method.as_str(), param.field_name.as_str()) {
        ("setChatMemberTag", "tag") | ("setChatAdministratorCustomTitle", "custom_title") => {
            Some("16")
        }
        _ => None,
    }
}

fn text_length_range(method: &MethodSpec, param: &ParamSpec) -> Option<(usize, usize)> {
    match (method.method.as_str(), param.field_name.as_str()) {
        ("setBusinessAccountName", "first_name") => Some((1, 64)),
        ("setBusinessAccountName", "last_name") => Some((0, 64)),
        ("verifyUser" | "verifyChat", "custom_description") => Some((0, 70)),
        ("setBusinessAccountBio", "bio") => Some((0, 140)),
        ("declineSuggestedPost", "comment") => Some((0, 128)),
        ("sendGift" | "giftPremiumSubscription", "text") => Some((0, 128)),
        _ => None,
    }
}

fn username_or_empty_field(method: &MethodSpec, param: &ParamSpec) -> bool {
    matches!(
        (method.method.as_str(), param.field_name.as_str()),
        ("setBusinessAccountUsername", "username")
    )
}

fn invoice_string_validator(method: &MethodSpec, param: &ParamSpec) -> Option<&'static str> {
    match (method.method.as_str(), param.field_name.as_str()) {
        ("sendInvoice" | "createInvoiceLink", "title") => Some("validate_invoice_title"),
        ("sendInvoice" | "createInvoiceLink", "description") => {
            Some("validate_invoice_description")
        }
        ("sendInvoice" | "createInvoiceLink", "payload") => Some("validate_invoice_payload"),
        ("sendInvoice" | "createInvoiceLink", "currency") => Some("validate_invoice_currency"),
        _ => None,
    }
}

fn string_items_limit(method: &MethodSpec, param: &ParamSpec) -> Option<&'static str> {
    match (method.method.as_str(), param.field_name.as_str()) {
        ("getCustomEmojiStickers", "custom_emoji_ids") => {
            Some("crate::types::sticker::MAX_CUSTOM_EMOJI_IDS")
        }
        ("setStickerEmojiList", "emoji_list") => Some("crate::types::sticker::MAX_STICKER_EMOJIS"),
        ("setStickerKeywords", "keywords") => Some("crate::types::sticker::MAX_STICKER_KEYWORDS"),
        _ => None,
    }
}

fn validation_rule(method: &MethodSpec, param: &ParamSpec) -> Option<String> {
    let field_ty = resolve_param_type(param);
    if type_has_validate(&field_ty) {
        if param.required {
            return Some(format!("        self.{}.validate()?;", param.field_name));
        }

        return Some(format!(
            "        if let Some(value) = self.{}.as_ref() {{\n            value.validate()?;\n        }}",
            param.field_name
        ));
    }

    if field_ty == "Vec<crate::types::common::MessageId>" {
        if param.required {
            if ordered_message_ids_field(method, param) {
                return Some(format!(
                    "        validate_required_ordered_message_ids(\"{}\", &self.{})?;",
                    param.name, param.field_name
                ));
            }

            return Some(format!(
                "        validate_required_message_ids(\"{}\", &self.{})?;",
                param.name, param.field_name
            ));
        }

        return Some(format!(
            "        if let Some(values) = self.{}.as_deref() {{\n            validate_message_ids(\"{}\", values)?;\n        }}",
            param.field_name, param.name
        ));
    }

    if field_ty == "Vec<String>" {
        if let Some(max_items) = string_items_limit(method, param) {
            if param.required {
                return Some(format!(
                    "        validate_required_limited_string_items(\"{}\", &self.{}, {})?;",
                    param.name, param.field_name, max_items
                ));
            }

            return Some(format!(
                "        if let Some(values) = self.{}.as_deref() {{\n            validate_limited_string_items(\"{}\", values, {})?;\n        }}",
                param.field_name, param.name, max_items
            ));
        }

        if param.required {
            return Some(format!(
                "        validate_required_string_items(\"{}\", &self.{})?;",
                param.name, param.field_name
            ));
        }

        return Some(format!(
            "        if let Some(values) = self.{}.as_deref() {{\n            validate_string_items(\"{}\", values)?;\n        }}",
            param.field_name, param.name
        ));
    }

    if let Some(item_ty) = validated_vec_item_type(&field_ty) {
        if param.required {
            return Some(format!(
                "        validate_required_items::<{}>(\"{}\", &self.{})?;",
                item_ty, param.name, param.field_name
            ));
        }

        return Some(format!(
            "        if let Some(values) = self.{}.as_deref() {{\n            validate_items(values)?;\n        }}",
            param.field_name
        ));
    }

    if field_ty == "i64" {
        if suggested_post_approval_send_date_field(method, param) {
            if param.required {
                return Some(format!(
                    "        validate_suggested_post_approval_send_date(\"{}\", self.{})?;",
                    param.name, param.field_name
                ));
            }

            return Some(format!(
                "        if let Some(value) = self.{} {{\n            validate_suggested_post_approval_send_date(\"{}\", value)?;\n        }}",
                param.field_name, param.name
            ));
        }

        if let Some((min, max)) = i64_field_range(method, param) {
            if param.required {
                return Some(format!(
                    "        validate_i64_range(\"{}\", self.{}, {min}, {max})?;",
                    param.name, param.field_name
                ));
            }

            return Some(format!(
                "        if let Some(value) = self.{} {{\n            validate_i64_range(\"{}\", value, {min}, {max})?;\n        }}",
                param.field_name, param.name
            ));
        }

        if positive_i64_field(&param.field_name) {
            if param.required {
                return Some(format!(
                    "        validate_positive_i64(\"{}\", self.{})?;",
                    param.name, param.field_name
                ));
            }

            return Some(format!(
                "        if let Some(value) = self.{} {{\n            validate_positive_i64(\"{}\", value)?;\n        }}",
                param.field_name, param.name
            ));
        }

        if non_negative_i64_field(&param.field_name) {
            if param.required {
                return Some(format!(
                    "        validate_non_negative_i64(\"{}\", self.{})?;",
                    param.name, param.field_name
                ));
            }

            return Some(format!(
                "        if let Some(value) = self.{} {{\n            validate_non_negative_i64(\"{}\", value)?;\n        }}",
                param.field_name, param.name
            ));
        }
    }

    if field_ty == "String" && control_free_string_field(method, param) {
        if param.required {
            return Some(format!(
                "        validate_control_free_string(\"{}\", &self.{})?;",
                param.name, param.field_name
            ));
        }

        return Some(format!(
            "        if let Some(value) = self.{}.as_deref() {{\n            validate_control_free_string(\"{}\", value)?;\n        }}",
            param.field_name, param.name
        ));
    }

    if field_ty == "String" && string_id_field(&param.field_name) {
        if param.required {
            return Some(format!(
                "        validate_string_id(\"{}\", &self.{})?;",
                param.name, param.field_name
            ));
        }

        return Some(format!(
            "        if let Some(value) = self.{}.as_deref() {{\n            validate_string_id(\"{}\", value)?;\n        }}",
            param.field_name, param.name
        ));
    }

    if field_ty == "String"
        && let Some(validator) = invoice_string_validator(method, param)
    {
        if param.required {
            return Some(format!(
                "        {validator}(\"{}\", &self.{})?;",
                method.method, param.field_name
            ));
        }

        return Some(format!(
            "        if let Some(value) = self.{}.as_deref() {{\n            {validator}(\"{}\", value)?;\n        }}",
            param.field_name, method.method
        ));
    }

    if field_ty == "String"
        && let Some(max_chars) = emoji_free_string_limit(method, param)
    {
        if param.required {
            return Some(format!(
                "        validate_emoji_free_text(\"{}\", &self.{}, {max_chars})?;",
                param.name, param.field_name
            ));
        }

        return Some(format!(
            "        if let Some(value) = self.{}.as_deref() {{\n            validate_emoji_free_text(\"{}\", value, {max_chars})?;\n        }}",
            param.field_name, param.name
        ));
    }

    if field_ty == "String"
        && let Some((min_chars, max_chars)) = text_length_range(method, param)
    {
        if param.required {
            return Some(format!(
                "        validate_text_length_range(\"{}\", &self.{}, {min_chars}, {max_chars})?;",
                param.name, param.field_name
            ));
        }

        return Some(format!(
            "        if let Some(value) = self.{}.as_deref() {{\n            validate_text_length_range(\"{}\", value, {min_chars}, {max_chars})?;\n        }}",
            param.field_name, param.name
        ));
    }

    if field_ty == "String" && username_or_empty_field(method, param) {
        if param.required {
            return Some(format!(
                "        validate_username_or_empty(\"{}\", &self.{})?;",
                param.name, param.field_name
            ));
        }

        return Some(format!(
            "        if let Some(value) = self.{}.as_deref() {{\n            validate_username_or_empty(\"{}\", value)?;\n        }}",
            param.field_name, param.name
        ));
    }

    if field_ty == "String" && non_empty_control_free_string_field(method, param) {
        if param.required {
            return Some(format!(
                "        validate_required_string(\"{}\", &self.{})?;\n        validate_control_free_string(\"{}\", &self.{})?;",
                param.name, param.field_name, param.name, param.field_name
            ));
        }

        return Some(format!(
            "        if let Some(value) = self.{}.as_deref() {{\n            validate_required_string(\"{}\", value)?;\n            validate_control_free_string(\"{}\", value)?;\n        }}",
            param.field_name, param.name, param.name
        ));
    }

    if !param.required {
        return None;
    }

    if field_ty == "String" {
        return Some(format!(
            "        validate_required_string(\"{}\", &self.{})?;",
            param.name, param.field_name
        ));
    }
    if field_ty.starts_with("Vec<") {
        return Some(format!(
            "        validate_required_vec(\"{}\", self.{}.len())?;",
            param.name, param.field_name
        ));
    }

    None
}

fn method_specific_validation_owns_param(method: &MethodSpec, param: &ParamSpec) -> bool {
    matches!(
        (method.method.as_str(), param.field_name.as_str()),
        ("answerShippingQuery", "shipping_options")
            | (
                "sendInvoice" | "createInvoiceLink",
                "prices" | "max_tip_amount" | "suggested_tip_amounts"
            )
            | ("sendInvoice", "reply_markup")
            | (
                "createInvoiceLink",
                "business_connection_id" | "subscription_period"
            )
    )
}

fn method_specific_validation_rules(method: &MethodSpec) -> Vec<String> {
    match method.method.as_str() {
        "sendInvoice" => vec![
            r#"        validate_invoice_prices("sendInvoice", &self.currency, &self.prices)?;"#
                .to_owned(),
            r#"        validate_invoice_tip_configuration(
            "sendInvoice",
            &self.currency,
            self.max_tip_amount,
            self.suggested_tip_amounts.as_deref(),
        )?;"#
                .to_owned(),
            r#"        validate_invoice_reply_markup("sendInvoice", self.reply_markup.as_ref())?;"#
                .to_owned(),
        ],
        "createInvoiceLink" => vec![
            r#"        validate_invoice_business_connection_id(
            "createInvoiceLink",
            &self.currency,
            self.business_connection_id.as_deref(),
        )?;"#
                .to_owned(),
            r#"        validate_invoice_prices("createInvoiceLink", &self.currency, &self.prices)?;"#
                .to_owned(),
            r#"        validate_invoice_tip_configuration(
            "createInvoiceLink",
            &self.currency,
            self.max_tip_amount,
            self.suggested_tip_amounts.as_deref(),
        )?;"#
                .to_owned(),
            r#"        validate_invoice_subscription_period(
            "createInvoiceLink",
            &self.currency,
            self.subscription_period,
            &self.prices,
        )?;"#
                .to_owned(),
        ],
        "answerShippingQuery" => vec![
            r#"        if self.ok {
            if self.error_message.is_some() {
                return Err(Error::InvalidRequest {
                    reason: "answerShippingQuery must omit `error_message` when `ok` is true"
                        .to_owned(),
                });
            }
            let Some(shipping_options) = self.shipping_options.as_deref() else {
                return Err(Error::InvalidRequest {
                    reason: "answerShippingQuery requires `shipping_options` when `ok` is true"
                        .to_owned(),
                });
            };
            validate_required_items::<crate::types::payment::ShippingOption>(
                "shipping_options",
                shipping_options,
            )?;
        } else {
            if self.shipping_options.is_some() {
                return Err(Error::InvalidRequest {
                    reason: "answerShippingQuery must omit `shipping_options` when `ok` is false"
                        .to_owned(),
                });
            }
            if self
                .error_message
                .as_deref()
                .is_none_or(|value| value.trim().is_empty())
            {
                return Err(Error::InvalidRequest {
                    reason:
                        "answerShippingQuery requires non-empty `error_message` when `ok` is false"
                            .to_owned(),
                });
            }
        }"#
            .to_owned(),
        ],
        "answerPreCheckoutQuery" => vec![
            r#"        if self.ok {
            if self.error_message.is_some() {
                return Err(Error::InvalidRequest {
                    reason: "answerPreCheckoutQuery must omit `error_message` when `ok` is true"
                        .to_owned(),
                });
            }
        } else if self
            .error_message
            .as_deref()
            .is_none_or(|value| value.trim().is_empty())
        {
            return Err(Error::InvalidRequest {
                reason:
                    "answerPreCheckoutQuery requires non-empty `error_message` when `ok` is false"
                        .to_owned(),
            });
        }"#
            .to_owned(),
        ],
        "sendGift" => vec![
            r#"        match (self.user_id.is_some(), self.chat_id.is_some()) {
            (true, false) | (false, true) => {}
            (false, false) => {
                return Err(Error::InvalidRequest {
                    reason: "sendGift requires either `user_id` or `chat_id`".to_owned(),
                });
            }
            (true, true) => {
                return Err(Error::InvalidRequest {
                    reason: "sendGift accepts either `user_id` or `chat_id`, not both".to_owned(),
                });
            }
        }"#
            .to_owned(),
        ],
        "giftPremiumSubscription" => vec![
            r#"        match (self.month_count, self.star_count) {
            (3, 1000) | (6, 1500) | (12, 2500) => {}
            _ => {
                return Err(Error::InvalidRequest {
                    reason: "giftPremiumSubscription requires 1000 stars for 3 months, 1500 for 6 months, or 2500 for 12 months"
                        .to_owned(),
                });
            }
        }"#
            .to_owned(),
        ],
        _ => Vec::new(),
    }
}

fn target_arg_expr(
    method: &MethodSpec,
    field: &str,
    required_expr: &str,
    optional_expr: &str,
) -> String {
    match param_by_field(method, field) {
        Some(param) if param.required => required_expr.to_owned(),
        Some(_) => optional_expr.to_owned(),
        None => "None".to_owned(),
    }
}

fn chat_or_inline_message_target_validation_rules(method: &MethodSpec) -> Vec<String> {
    if param_by_field(method, "chat_id").is_none()
        || param_by_field(method, "message_id").is_none()
        || param_by_field(method, "inline_message_id").is_none()
    {
        return Vec::new();
    }

    let chat_id = target_arg_expr(
        method,
        "chat_id",
        "Some(&self.chat_id)",
        "self.chat_id.as_ref()",
    );
    let message_id = target_arg_expr(
        method,
        "message_id",
        "Some(&self.message_id)",
        "self.message_id.as_ref()",
    );
    let inline_message_id = target_arg_expr(
        method,
        "inline_message_id",
        "Some(self.inline_message_id.as_str())",
        "self.inline_message_id.as_deref()",
    );

    vec![format!(
        "        validate_chat_or_inline_message_target(\"{}\", {chat_id}, {message_id}, {inline_message_id})?;",
        method.method
    )]
}

fn param_by_field<'a>(method: &'a MethodSpec, field: &str) -> Option<&'a ParamSpec> {
    method.params.iter().find(|param| param.field_name == field)
}

fn optional_field_expr(param: Option<&ParamSpec>, field: &str) -> String {
    match param {
        Some(param) if param.required => format!("Some(self.{field})"),
        Some(_) => format!("self.{field}"),
        None => "None".to_owned(),
    }
}

fn optional_entities_expr(param: Option<&ParamSpec>, field: &str) -> String {
    match param {
        Some(param) if param.required => format!("Some(self.{field}.as_slice())"),
        Some(_) => format!("self.{field}.as_deref()"),
        None => "None".to_owned(),
    }
}

fn formatting_validation_rules(method: &MethodSpec) -> Vec<String> {
    const FORMATTING_FIELDS: [(&str, &str, &str); 8] = [
        ("text", "parse_mode", "entities"),
        ("text", "text_parse_mode", "text_entities"),
        ("caption", "parse_mode", "caption_entities"),
        ("caption", "caption_parse_mode", "caption_entities"),
        ("question", "question_parse_mode", "question_entities"),
        (
            "explanation",
            "explanation_parse_mode",
            "explanation_entities",
        ),
        (
            "description",
            "description_parse_mode",
            "description_entities",
        ),
        ("quote", "quote_parse_mode", "quote_entities"),
    ];

    let mut rules = Vec::new();
    let mut generated_entity_fields = Vec::new();
    for (text_field, parse_field, entities_field) in FORMATTING_FIELDS {
        let Some(text_param) = param_by_field(method, text_field) else {
            continue;
        };
        let parse_param = param_by_field(method, parse_field);
        let entities_param = param_by_field(method, entities_field);
        if parse_param.is_none() && entities_param.is_none() {
            continue;
        }
        if generated_entity_fields.contains(&(text_field, entities_field)) {
            continue;
        }
        if parse_param.is_none()
            && FORMATTING_FIELDS
                .iter()
                .any(|(other_text, other_parse, other_entities)| {
                    *other_text == text_field
                        && *other_entities == entities_field
                        && *other_parse != parse_field
                        && param_by_field(method, other_parse).is_some()
                })
        {
            continue;
        }
        generated_entity_fields.push((text_field, entities_field));

        let parse_expr = optional_field_expr(parse_param, parse_field);
        let entities_expr = optional_entities_expr(entities_param, entities_field);
        if text_param.required {
            rules.push(format!(
                "        validate_text_formatting(\"{text_field}\", &self.{text_field}, {parse_expr}, {entities_expr})?;"
            ));
        } else {
            rules.push(format!(
                "        validate_optional_text_formatting(\"{text_field}\", self.{text_field}.as_deref(), {parse_expr}, {entities_expr})?;"
            ));
        }
    }

    rules
}

fn domain_for_method(fn_name: &str) -> &'static str {
    if fn_name.contains("business") {
        return "business";
    }
    if fn_name.contains("forum") {
        return "forum";
    }
    if fn_name.contains("gift") || fn_name.contains("star") {
        return "gifts";
    }
    if ["invoice", "shipping", "pre_checkout", "passport"]
        .into_iter()
        .any(|marker| fn_name.contains(marker))
    {
        return "payments";
    }
    if fn_name.contains("sticker") || fn_name.contains("emoji_status") {
        return "stickers";
    }
    if fn_name.contains("story") {
        return "stories";
    }
    "misc"
}

fn render_request(method: &MethodSpec) -> String {
    let req_name = request_type_name(&method.fn_name);
    let required_params: Vec<&ParamSpec> = method
        .params
        .iter()
        .filter(|param| param.required)
        .collect();
    let validation_rules = method
        .params
        .iter()
        .filter(|param| !method_specific_validation_owns_param(method, param))
        .filter_map(|param| validation_rule(method, param))
        .chain(formatting_validation_rules(method))
        .chain(chat_or_inline_message_target_validation_rules(method))
        .chain(method_specific_validation_rules(method))
        .collect::<Vec<_>>();
    let derive = if required_params.is_empty() {
        "#[derive(Clone, Debug, Default, Serialize)]"
    } else {
        "#[derive(Clone, Debug, Serialize)]"
    };

    let mut out = String::new();
    let _ = writeln!(
        &mut out,
        "/// Auto-generated request for `{}`.",
        method.method
    );
    let _ = writeln!(&mut out, "{derive}");
    let _ = writeln!(&mut out, "pub struct {req_name} {{");
    for param in &method.params {
        let cleaned = param.field_name.trim_start_matches("r#");
        let field_ty = resolve_param_type(param);
        if cleaned != param.name {
            let _ = writeln!(&mut out, "    #[serde(rename = \"{}\")]", param.name);
        }
        if param.required {
            let _ = writeln!(&mut out, "    pub {}: {field_ty},", param.field_name);
        } else {
            let _ = writeln!(
                &mut out,
                "    #[serde(default, skip_serializing_if = \"Option::is_none\")]"
            );
            let _ = writeln!(
                &mut out,
                "    pub {}: Option<{field_ty}>,",
                param.field_name
            );
        }
    }
    let _ = writeln!(&mut out, "}}");
    let _ = writeln!(&mut out);

    let _ = writeln!(&mut out, "impl {req_name} {{");
    if method.params.is_empty() {
        let _ = writeln!(&mut out, "    pub fn new() -> Self {{");
        let _ = writeln!(&mut out, "        Self {{}}");
        let _ = writeln!(&mut out, "    }}");
    } else if !required_params.is_empty() {
        let args = required_params
            .iter()
            .map(|param| {
                format!(
                    "{}: {}",
                    param.field_name,
                    ctor_arg_type(&resolve_param_type(param))
                )
            })
            .collect::<Vec<_>>()
            .join(", ");
        let _ = writeln!(&mut out, "    pub fn new({args}) -> Self {{");
        let _ = writeln!(&mut out, "        Self {{");
        for param in &method.params {
            let field_ty = resolve_param_type(param);
            if param.required {
                let _ = writeln!(
                    &mut out,
                    "            {},",
                    ctor_assign(&param.field_name, &field_ty)
                );
            } else {
                let _ = writeln!(&mut out, "            {}: None,", param.field_name);
            }
        }
        let _ = writeln!(&mut out, "        }}");
        let _ = writeln!(&mut out, "    }}");
    } else {
        let _ = writeln!(&mut out, "    pub fn new() -> Self {{");
        let _ = writeln!(&mut out, "        Self::default()");
        let _ = writeln!(&mut out, "    }}");
    }
    let _ = writeln!(&mut out, "}}");
    let _ = writeln!(&mut out);

    let _ = writeln!(&mut out, "impl AdvancedRequest for {req_name} {{");
    let _ = writeln!(
        &mut out,
        "    type Response = {};",
        response_type(&method.method, &method.return_desc)
    );
    let _ = writeln!(
        &mut out,
        "    const METHOD: &'static str = \"{}\";",
        method.method
    );
    if !validation_rules.is_empty() {
        let _ = writeln!(&mut out);
        let _ = writeln!(&mut out, "    fn validate(&self) -> Result<()> {{");
        for rule in validation_rules {
            let _ = writeln!(&mut out, "{rule}");
        }
        let _ = writeln!(&mut out, "        Ok(())");
        let _ = writeln!(&mut out, "    }}");
    }
    let _ = writeln!(&mut out, "}}");
    let _ = writeln!(&mut out);
    out
}

fn generate_types_root(grouped: &HashMap<&'static str, Vec<&MethodSpec>>) -> String {
    let mut out = String::new();
    let _ = writeln!(
        &mut out,
        "// Auto-generated by crates/tele-codegen. Do not edit manually."
    );
    let _ = writeln!(&mut out, "use serde::Serialize;");
    let _ = writeln!(&mut out, "use serde::de::DeserializeOwned;");
    let _ = writeln!(&mut out);
    let _ = writeln!(&mut out, "use crate::Result;");
    let _ = writeln!(&mut out);
    let _ = writeln!(
        &mut out,
        "/// Typed request marker for advanced API methods."
    );
    let _ = writeln!(&mut out, "pub trait AdvancedRequest: Serialize {{");
    let _ = writeln!(&mut out, "    type Response: DeserializeOwned;");
    let _ = writeln!(&mut out, "    const METHOD: &'static str;");
    let _ = writeln!(&mut out);
    let _ = writeln!(&mut out, "    fn validate(&self) -> Result<()> {{");
    let _ = writeln!(&mut out, "        Ok(())");
    let _ = writeln!(&mut out, "    }}");
    let _ = writeln!(&mut out, "}}");
    let _ = writeln!(&mut out);
    for domain in DOMAIN_ORDER {
        let _ = writeln!(&mut out, "#[path = \"advanced_{domain}.rs\"]");
        let _ = writeln!(&mut out, "mod advanced_{domain};");
    }
    let _ = writeln!(&mut out);
    for domain in DOMAIN_ORDER {
        if grouped
            .get(domain)
            .is_some_and(|methods| !methods.is_empty())
        {
            let _ = writeln!(&mut out, "pub use advanced_{domain}::*;");
        }
    }
    out
}

fn generated_validate_types(methods: &[&MethodSpec]) -> Vec<&'static str> {
    let mut used = Vec::new();
    for method in methods {
        for param in &method.params {
            let field_ty = resolve_param_type(param);
            let Some(item_ty) = validated_vec_item_type(&field_ty) else {
                continue;
            };
            if !used.contains(&item_ty) {
                used.push(item_ty);
            }
        }
    }

    used
}

fn generate_domain_module(methods: &[&MethodSpec]) -> String {
    let body = methods
        .iter()
        .map(|method| render_request(method).trim_end().to_owned())
        .collect::<Vec<_>>()
        .join("\n\n");
    let uses_value = body.contains("Value");
    let uses_required_string_validator = body.contains("validate_required_string(");
    let uses_required_string_items_validator = body.contains("validate_required_string_items(");
    let uses_limited_string_items_validator = body.contains("validate_limited_string_items(")
        || body.contains("validate_required_limited_string_items(");
    let uses_string_items_validator = body.contains("validate_string_items(")
        || uses_required_string_items_validator
        || uses_limited_string_items_validator;
    let uses_string_id_validator =
        body.contains("validate_string_id(") || uses_string_items_validator;
    let uses_required_vec_validator = body.contains("validate_required_vec(")
        || uses_required_string_items_validator
        || body.contains("validate_required_limited_string_items(");
    let uses_required_ordered_message_ids_validator =
        body.contains("validate_required_ordered_message_ids(");
    let uses_required_message_ids_validator = body.contains("validate_required_message_ids(")
        || uses_required_ordered_message_ids_validator;
    let uses_message_ids_validator =
        body.contains("validate_message_ids(") || uses_required_message_ids_validator;
    let uses_required_items_validator = body.contains("validate_required_items::<");
    let uses_items_validator = body.contains("validate_items(");
    let uses_i64_range_validator = body.contains("validate_i64_range(");
    let uses_emoji_free_text_validator = body.contains("validate_emoji_free_text(");
    let uses_text_length_range_validator = body.contains("validate_text_length_range(");
    let uses_username_or_empty_validator = body.contains("validate_username_or_empty(");
    let uses_suggested_post_approval_send_date_validator =
        body.contains("validate_suggested_post_approval_send_date(");
    let uses_positive_i64_validator = body.contains("validate_positive_i64(");
    let uses_non_negative_i64_validator = body.contains("validate_non_negative_i64(");
    let uses_control_free_string_validator = body.contains("validate_control_free_string(");
    let uses_invoice_currency_validator = body.contains("validate_invoice_currency(");
    let uses_invoice_description_validator = body.contains("validate_invoice_description(");
    let uses_invoice_payload_validator = body.contains("validate_invoice_payload(");
    let uses_invoice_prices_validator = body.contains("validate_invoice_prices(");
    let uses_invoice_reply_markup_validator = body.contains("validate_invoice_reply_markup(");
    let uses_invoice_business_connection_id_validator =
        body.contains("validate_invoice_business_connection_id(");
    let uses_invoice_subscription_period_validator =
        body.contains("validate_invoice_subscription_period(");
    let uses_invoice_tip_configuration_validator =
        body.contains("validate_invoice_tip_configuration(");
    let uses_invoice_title_validator = body.contains("validate_invoice_title(");
    let uses_payment_validation = uses_invoice_currency_validator
        || uses_invoice_description_validator
        || uses_invoice_payload_validator
        || uses_invoice_prices_validator
        || uses_invoice_reply_markup_validator
        || uses_invoice_business_connection_id_validator
        || uses_invoice_subscription_period_validator
        || uses_invoice_tip_configuration_validator
        || uses_invoice_title_validator;
    let uses_text_formatting_validator = body.contains("validate_text_formatting(")
        || body.contains("validate_optional_text_formatting(");
    let uses_message_target_validator = body.contains("validate_chat_or_inline_message_target(");
    let generated_validate_types = generated_validate_types(methods);
    let uses_shared_validation = uses_required_string_validator
        || uses_string_id_validator
        || uses_required_vec_validator
        || uses_i64_range_validator
        || uses_emoji_free_text_validator
        || uses_text_length_range_validator
        || uses_username_or_empty_validator
        || uses_suggested_post_approval_send_date_validator
        || uses_positive_i64_validator
        || uses_non_negative_i64_validator
        || uses_control_free_string_validator
        || uses_text_formatting_validator;
    let uses_error = uses_message_ids_validator
        || uses_required_items_validator
        || uses_message_target_validator
        || uses_limited_string_items_validator
        || body.contains("Error::InvalidRequest");
    let uses_result = body.contains("fn validate(&self) -> Result<()>")
        || uses_required_string_validator
        || uses_string_id_validator
        || uses_required_vec_validator
        || uses_required_message_ids_validator
        || uses_message_ids_validator
        || uses_required_items_validator
        || uses_items_validator
        || uses_i64_range_validator
        || uses_emoji_free_text_validator
        || uses_text_length_range_validator
        || uses_username_or_empty_validator
        || uses_suggested_post_approval_send_date_validator
        || uses_positive_i64_validator
        || uses_non_negative_i64_validator
        || uses_control_free_string_validator
        || uses_text_formatting_validator;

    let mut out = String::new();
    let _ = writeln!(
        &mut out,
        "// Auto-generated by crates/tele-codegen. Do not edit manually."
    );
    if body.is_empty() {
        return out;
    }

    let _ = writeln!(&mut out, "use serde::Serialize;");
    if uses_value {
        let _ = writeln!(&mut out, "use serde_json::Value;");
    }
    let _ = writeln!(&mut out);
    if uses_error {
        let _ = writeln!(&mut out, "use crate::{{Error, Result}};");
        let _ = writeln!(&mut out);
    } else if uses_result {
        let _ = writeln!(&mut out, "use crate::Result;");
        let _ = writeln!(&mut out);
    }
    if uses_shared_validation {
        let _ = writeln!(&mut out, "use crate::types::validation::{{");
        if uses_i64_range_validator {
            let _ = writeln!(&mut out, "    i64_range as validate_i64_range,");
        }
        if uses_emoji_free_text_validator {
            let _ = writeln!(&mut out, "    emoji_free_text as validate_emoji_free_text,");
        }
        if uses_text_length_range_validator {
            let _ = writeln!(
                &mut out,
                "    text_length_range as validate_text_length_range,"
            );
        }
        if uses_username_or_empty_validator {
            let _ = writeln!(
                &mut out,
                "    username_or_empty as validate_username_or_empty,"
            );
        }
        if uses_suggested_post_approval_send_date_validator {
            let _ = writeln!(
                &mut out,
                "    suggested_post_approval_send_date as validate_suggested_post_approval_send_date,"
            );
        }
        if uses_non_negative_i64_validator {
            let _ = writeln!(
                &mut out,
                "    non_negative_i64 as validate_non_negative_i64,"
            );
        }
        if uses_positive_i64_validator {
            let _ = writeln!(&mut out, "    positive_i64 as validate_positive_i64,");
        }
        if uses_control_free_string_validator {
            let _ = writeln!(
                &mut out,
                "    control_free_string as validate_control_free_string,"
            );
        }
        if body.contains("validate_optional_text_formatting(") {
            let _ = writeln!(
                &mut out,
                "    optional_text_formatting as validate_optional_text_formatting,"
            );
        }
        if uses_required_vec_validator {
            let _ = writeln!(&mut out, "    required_len as validate_required_vec,");
        }
        if uses_required_string_validator {
            let _ = writeln!(&mut out, "    required_string as validate_required_string,");
        }
        if uses_string_id_validator {
            let _ = writeln!(&mut out, "    string_id as validate_string_id,");
        }
        if body.contains("validate_text_formatting(") {
            let _ = writeln!(&mut out, "    text_formatting as validate_text_formatting,");
        }
        let _ = writeln!(&mut out, "}};");
        let _ = writeln!(&mut out);
    }
    if uses_payment_validation {
        let _ = writeln!(&mut out, "use crate::types::payment::{{");
        if uses_invoice_currency_validator {
            let _ = writeln!(&mut out, "    validate_invoice_currency,");
        }
        if uses_invoice_description_validator {
            let _ = writeln!(&mut out, "    validate_invoice_description,");
        }
        if uses_invoice_payload_validator {
            let _ = writeln!(&mut out, "    validate_invoice_payload,");
        }
        if uses_invoice_prices_validator {
            let _ = writeln!(&mut out, "    validate_invoice_prices,");
        }
        if uses_invoice_reply_markup_validator {
            let _ = writeln!(&mut out, "    validate_invoice_reply_markup,");
        }
        if uses_invoice_business_connection_id_validator {
            let _ = writeln!(&mut out, "    validate_invoice_business_connection_id,");
        }
        if uses_invoice_subscription_period_validator {
            let _ = writeln!(&mut out, "    validate_invoice_subscription_period,");
        }
        if uses_invoice_tip_configuration_validator {
            let _ = writeln!(&mut out, "    validate_invoice_tip_configuration,");
        }
        if uses_invoice_title_validator {
            let _ = writeln!(&mut out, "    validate_invoice_title,");
        }
        let _ = writeln!(&mut out, "}};");
        let _ = writeln!(&mut out);
    }
    let _ = writeln!(&mut out, "use super::AdvancedRequest;");
    let _ = writeln!(&mut out);
    if uses_message_target_validator {
        let _ = writeln!(&mut out, "fn validate_chat_or_inline_message_target<T>(");
        let _ = writeln!(&mut out, "    method: &str,");
        let _ = writeln!(&mut out, "    chat_id: Option<&T>,");
        let _ = writeln!(
            &mut out,
            "    message_id: Option<&crate::types::common::MessageId>,"
        );
        let _ = writeln!(&mut out, "    inline_message_id: Option<&str>,");
        let _ = writeln!(&mut out, ") -> Result<()> {{");
        let _ = writeln!(&mut out, "    if inline_message_id.is_some() {{");
        let _ = writeln!(
            &mut out,
            "        if chat_id.is_some() || message_id.is_some() {{"
        );
        let _ = writeln!(&mut out, "            return Err(Error::InvalidRequest {{");
        let _ = writeln!(
            &mut out,
            "                reason: format!(\"{{method}} accepts either `chat_id` with `message_id` or `inline_message_id`, not both\"),"
        );
        let _ = writeln!(&mut out, "            }});");
        let _ = writeln!(&mut out, "        }}");
        let _ = writeln!(&mut out);
        let _ = writeln!(&mut out, "        return Ok(());");
        let _ = writeln!(&mut out, "    }}");
        let _ = writeln!(&mut out);
        let _ = writeln!(
            &mut out,
            "    if chat_id.is_some() && message_id.is_some() {{"
        );
        let _ = writeln!(&mut out, "        return Ok(());");
        let _ = writeln!(&mut out, "    }}");
        let _ = writeln!(&mut out);
        let _ = writeln!(&mut out, "    Err(Error::InvalidRequest {{");
        let _ = writeln!(
            &mut out,
            "        reason: format!(\"{{method}} requires either `chat_id` with `message_id` or `inline_message_id\"),"
        );
        let _ = writeln!(&mut out, "    }})");
        let _ = writeln!(&mut out, "}}");
        let _ = writeln!(&mut out);
    }
    if uses_items_validator || uses_required_items_validator {
        let _ = writeln!(&mut out, "trait GeneratedValidate {{");
        let _ = writeln!(&mut out, "    fn validate_generated(&self) -> Result<()>;");
        let _ = writeln!(&mut out, "}}");
        let _ = writeln!(&mut out);
        for ty in generated_validate_types {
            let _ = writeln!(&mut out, "impl GeneratedValidate for {ty} {{");
            let _ = writeln!(
                &mut out,
                "    fn validate_generated(&self) -> Result<()> {{"
            );
            let _ = writeln!(&mut out, "        self.validate()");
            let _ = writeln!(&mut out, "    }}");
            let _ = writeln!(&mut out, "}}");
            let _ = writeln!(&mut out);
        }
        let _ = writeln!(
            &mut out,
            "fn validate_items<T: GeneratedValidate>(values: &[T]) -> Result<()> {{"
        );
        let _ = writeln!(&mut out, "    for value in values {{");
        let _ = writeln!(&mut out, "        value.validate_generated()?;");
        let _ = writeln!(&mut out, "    }}");
        let _ = writeln!(&mut out);
        let _ = writeln!(&mut out, "    Ok(())");
        let _ = writeln!(&mut out, "}}");
        let _ = writeln!(&mut out);
    }
    if uses_required_items_validator {
        let _ = writeln!(
            &mut out,
            "fn validate_required_items<T: GeneratedValidate>(field: &str, values: &[T]) -> Result<()> {{"
        );
        let _ = writeln!(&mut out, "    if values.is_empty() {{");
        let _ = writeln!(&mut out, "        return Err(Error::InvalidRequest {{");
        let _ = writeln!(
            &mut out,
            "            reason: format!(\"{{field}} cannot be empty\"),"
        );
        let _ = writeln!(&mut out, "        }});");
        let _ = writeln!(&mut out, "    }}");
        let _ = writeln!(&mut out);
        let _ = writeln!(&mut out, "    validate_items(values)");
        let _ = writeln!(&mut out, "}}");
        let _ = writeln!(&mut out);
    }
    if uses_message_ids_validator || uses_required_message_ids_validator {
        let _ = writeln!(&mut out, "const MAX_MESSAGE_IDS: usize = 100;");
        let _ = writeln!(&mut out);
        let _ = writeln!(
            &mut out,
            "fn validate_message_ids(field: &str, values: &[crate::types::common::MessageId]) -> Result<()> {{"
        );
        let _ = writeln!(&mut out, "    if values.len() > MAX_MESSAGE_IDS {{");
        let _ = writeln!(&mut out, "        return Err(Error::InvalidRequest {{");
        let _ = writeln!(
            &mut out,
            "            reason: format!(\"{{field}} accepts at most {{MAX_MESSAGE_IDS}} message ids\"),"
        );
        let _ = writeln!(&mut out, "        }});");
        let _ = writeln!(&mut out, "    }}");
        let _ = writeln!(&mut out);
        let _ = writeln!(
            &mut out,
            "    for (index, value) in values.iter().enumerate() {{"
        );
        let _ = writeln!(&mut out, "        value.validate()?;");
        let _ = writeln!(
            &mut out,
            "        if values[..index].iter().any(|existing| existing == value) {{"
        );
        let _ = writeln!(&mut out, "            return Err(Error::InvalidRequest {{");
        let _ = writeln!(
            &mut out,
            "                reason: format!(\"{{field}} message ids must be unique\"),"
        );
        let _ = writeln!(&mut out, "            }});");
        let _ = writeln!(&mut out, "        }}");
        let _ = writeln!(&mut out, "    }}");
        let _ = writeln!(&mut out);
        let _ = writeln!(&mut out, "    Ok(())");
        let _ = writeln!(&mut out, "}}");
        let _ = writeln!(&mut out);
    }
    if uses_string_items_validator {
        let _ = writeln!(
            &mut out,
            "fn validate_string_items(field: &str, values: &[String]) -> Result<()> {{"
        );
        let _ = writeln!(
            &mut out,
            "    for (index, value) in values.iter().enumerate() {{"
        );
        let _ = writeln!(
            &mut out,
            "        validate_string_id(&format!(\"{{field}}[{{index}}]\"), value)?;"
        );
        let _ = writeln!(&mut out, "    }}");
        let _ = writeln!(&mut out);
        let _ = writeln!(&mut out, "    Ok(())");
        let _ = writeln!(&mut out, "}}");
        let _ = writeln!(&mut out);
        if uses_limited_string_items_validator {
            let _ = writeln!(
                &mut out,
                "fn validate_limited_string_items(field: &str, values: &[String], max_items: usize) -> Result<()> {{"
            );
            let _ = writeln!(&mut out, "    if values.len() > max_items {{");
            let _ = writeln!(&mut out, "        return Err(Error::InvalidRequest {{");
            let _ = writeln!(
                &mut out,
                "            reason: format!(\"{{field}} accepts at most {{max_items}} items\"),"
            );
            let _ = writeln!(&mut out, "        }});");
            let _ = writeln!(&mut out, "    }}");
            let _ = writeln!(&mut out);
            let _ = writeln!(&mut out, "    validate_string_items(field, values)");
            let _ = writeln!(&mut out, "}}");
            let _ = writeln!(&mut out);
            let _ = writeln!(
                &mut out,
                "fn validate_required_limited_string_items(field: &str, values: &[String], max_items: usize) -> Result<()> {{"
            );
            let _ = writeln!(&mut out, "    validate_required_vec(field, values.len())?;");
            let _ = writeln!(
                &mut out,
                "    validate_limited_string_items(field, values, max_items)"
            );
            let _ = writeln!(&mut out, "}}");
            let _ = writeln!(&mut out);
        }
        if uses_required_string_items_validator {
            let _ = writeln!(
                &mut out,
                "fn validate_required_string_items(field: &str, values: &[String]) -> Result<()> {{"
            );
            let _ = writeln!(&mut out, "    validate_required_vec(field, values.len())?;");
            let _ = writeln!(&mut out, "    validate_string_items(field, values)");
            let _ = writeln!(&mut out, "}}");
            let _ = writeln!(&mut out);
        }
    }
    if uses_required_message_ids_validator {
        let _ = writeln!(
            &mut out,
            "fn validate_required_message_ids(field: &str, values: &[crate::types::common::MessageId]) -> Result<()> {{"
        );
        let _ = writeln!(&mut out, "    if values.is_empty() {{");
        let _ = writeln!(&mut out, "        return Err(Error::InvalidRequest {{");
        let _ = writeln!(
            &mut out,
            "            reason: format!(\"{{field}} cannot be empty\"),"
        );
        let _ = writeln!(&mut out, "        }});");
        let _ = writeln!(&mut out, "    }}");
        let _ = writeln!(&mut out);
        let _ = writeln!(&mut out, "    validate_message_ids(field, values)");
        let _ = writeln!(&mut out, "}}");
        let _ = writeln!(&mut out);
    }
    if uses_required_ordered_message_ids_validator {
        let _ = writeln!(
            &mut out,
            "fn validate_required_ordered_message_ids(field: &str, values: &[crate::types::common::MessageId]) -> Result<()> {{"
        );
        let _ = writeln!(
            &mut out,
            "    validate_required_message_ids(field, values)?;"
        );
        let _ = writeln!(&mut out);
        let _ = writeln!(&mut out, "    let mut previous = None;");
        let _ = writeln!(&mut out, "    for value in values {{");
        let _ = writeln!(
            &mut out,
            "        if previous.is_some_and(|previous| value.0 <= previous) {{"
        );
        let _ = writeln!(&mut out, "            return Err(Error::InvalidRequest {{");
        let _ = writeln!(
            &mut out,
            "                reason: format!(\"{{field}} message ids must be strictly increasing\"),"
        );
        let _ = writeln!(&mut out, "            }});");
        let _ = writeln!(&mut out, "        }}");
        let _ = writeln!(&mut out, "        previous = Some(value.0);");
        let _ = writeln!(&mut out, "    }}");
        let _ = writeln!(&mut out);
        let _ = writeln!(&mut out, "    Ok(())");
        let _ = writeln!(&mut out, "}}");
        let _ = writeln!(&mut out);
    }
    if !body.is_empty() {
        let _ = writeln!(&mut out, "{body}");
    }
    out
}

fn generate_api_methods(methods: &[&MethodSpec]) -> String {
    let mut out = String::new();
    let _ = writeln!(
        &mut out,
        "// Auto-generated by crates/tele-codegen. Do not edit manually."
    );
    let _ = writeln!(&mut out, "macro_rules! with_advanced_methods {{");
    let _ = writeln!(&mut out, "    ($macro:ident) => {{");
    let _ = writeln!(&mut out, "        $macro! {{");
    for method in methods {
        let _ = writeln!(
            &mut out,
            "            ({}, {}, \"{}\", {}),",
            method.fn_name,
            typed_fn_name(&method.fn_name),
            method.method,
            request_type_name(&method.fn_name)
        );
    }
    let _ = writeln!(&mut out, "        }}");
    let _ = writeln!(&mut out, "    }};");
    let _ = writeln!(&mut out, "}}");
    out
}

fn write_if_changed(path: &Path, content: &str) -> Result<(), Box<dyn std::error::Error>> {
    if fs::read_to_string(path).ok().as_deref() == Some(content) {
        return Ok(());
    }
    fs::write(path, content)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_names_are_stable() {
        assert_eq!(
            request_type_name("answer_web_app_query"),
            "AdvancedAnswerWebAppQueryRequest"
        );
        assert_eq!(typed_fn_name("get_me"), "get_me_typed");
    }

    #[test]
    fn domains_and_response_types_match_rules() {
        assert_eq!(domain_for_method("get_business_account_gifts"), "business");
        assert_eq!(domain_for_method("refund_star_payment"), "gifts");
        assert_eq!(domain_for_method("create_invoice_link"), "payments");
        assert_eq!(domain_for_method("set_story_privacy"), "stories");
        assert_eq!(
            response_type("getChatMenuButton", ""),
            "crate::types::telegram::MenuButton"
        );
        assert_eq!(response_type("unknown", "Returns True on success"), "bool");
        assert_eq!(
            response_type("getBusinessConnection", ""),
            "crate::types::update::BusinessConnection"
        );
        assert_eq!(
            response_type("getUserChatBoosts", ""),
            "crate::types::update::UserChatBoosts"
        );
        assert_eq!(
            response_type("getUserProfileAudios", ""),
            "crate::types::bot::UserProfileAudios"
        );
        assert_eq!(
            response_type("getAvailableGifts", ""),
            "crate::types::gift::Gifts"
        );
        assert_eq!(
            response_type("getUserGifts", ""),
            "crate::types::gift::OwnedGifts"
        );
        assert_eq!(
            response_type("getBusinessAccountGifts", ""),
            "crate::types::gift::OwnedGifts"
        );
        assert_eq!(
            response_type("getChatGifts", ""),
            "crate::types::gift::OwnedGifts"
        );
        assert_eq!(
            response_type("getMyStarBalance", ""),
            "crate::types::message::StarAmount"
        );
        assert_eq!(
            response_type("getStarTransactions", ""),
            "crate::types::gift::StarTransactions"
        );
        assert_eq!(
            response_type("savePreparedInlineMessage", ""),
            "crate::types::telegram::PreparedInlineMessage"
        );
        assert_eq!(
            response_type("savePreparedKeyboardButton", ""),
            "crate::types::telegram::PreparedKeyboardButton"
        );
        assert_eq!(
            response_type("getBusinessAccountStarBalance", ""),
            "crate::types::message::StarAmount"
        );
        assert_eq!(
            response_type("createForumTopic", ""),
            "crate::types::message::ForumTopic"
        );
        assert_eq!(
            response_type("sendGame", ""),
            "crate::types::message::Message"
        );
        assert_eq!(
            response_type("setGameScore", ""),
            "crate::types::message::EditMessageResult"
        );
        assert_eq!(
            response_type("getGameHighScores", ""),
            "Vec<crate::types::message::GameHighScore>"
        );
        assert_eq!(
            response_type("postStory", ""),
            "crate::types::message::Story"
        );
        assert_eq!(
            response_type("repostStory", ""),
            "crate::types::message::Story"
        );
        assert_eq!(
            response_type("editStory", ""),
            "crate::types::message::Story"
        );
        assert_eq!(response_type("answerShippingQuery", ""), "bool");
        assert_eq!(response_type("answerPreCheckoutQuery", ""), "bool");
    }

    #[test]
    fn upload_only_methods_are_not_generated_for_json_advanced_service() {
        let spec = BotApiSpec {
            version: "test".to_owned(),
            generated_from: "test".to_owned(),
            all_methods: vec!["setChatPhoto".to_owned(), "getMe".to_owned()],
            advanced_methods: vec![
                MethodSpec {
                    fn_name: "set_chat_photo".to_owned(),
                    method: "setChatPhoto".to_owned(),
                    return_desc: "True on success".to_owned(),
                    params: vec![
                        ParamSpec {
                            name: "chat_id".to_owned(),
                            field_name: "chat_id".to_owned(),
                            required: true,
                            type_raw: "Integer or String".to_owned(),
                            type_rust: "ChatId".to_owned(),
                        },
                        ParamSpec {
                            name: "photo".to_owned(),
                            field_name: "photo".to_owned(),
                            required: true,
                            type_raw: "InputFile".to_owned(),
                            type_rust: "Value".to_owned(),
                        },
                    ],
                },
                MethodSpec {
                    fn_name: "get_me".to_owned(),
                    method: "getMe".to_owned(),
                    return_desc: "the bot user".to_owned(),
                    params: Vec::new(),
                },
            ],
        };

        let methods = json_advanced_methods(&spec);
        assert_eq!(methods.len(), 1);
        assert_eq!(methods[0].method, "getMe");
    }

    #[test]
    fn domain_module_imports_value_when_needed() {
        let method = MethodSpec {
            fn_name: "demo".to_owned(),
            method: "demo".to_owned(),
            return_desc: String::new(),
            params: vec![ParamSpec {
                name: "payload".to_owned(),
                field_name: "payload".to_owned(),
                required: false,
                type_raw: "UnknownObject".to_owned(),
                type_rust: "Value".to_owned(),
            }],
        };

        let generated = generate_domain_module(&[&method]);
        assert!(generated.contains("use serde_json::Value;"));
        assert!(generated.contains("pub payload: Option<Value>,"));
    }

    #[test]
    fn generated_validation_uses_common_id_invariants() {
        let method = MethodSpec {
            fn_name: "demo".to_owned(),
            method: "demo".to_owned(),
            return_desc: "Returns True on success".to_owned(),
            params: vec![
                ParamSpec {
                    name: "chat_id".to_owned(),
                    field_name: "chat_id".to_owned(),
                    required: true,
                    type_raw: "Integer or String".to_owned(),
                    type_rust: "ChatId".to_owned(),
                },
                ParamSpec {
                    name: "message_ids".to_owned(),
                    field_name: "message_ids".to_owned(),
                    required: true,
                    type_raw: "Array of Integer".to_owned(),
                    type_rust: "Vec<MessageId>".to_owned(),
                },
                ParamSpec {
                    name: "user_id".to_owned(),
                    field_name: "user_id".to_owned(),
                    required: false,
                    type_raw: "Integer".to_owned(),
                    type_rust: "UserId".to_owned(),
                },
                ParamSpec {
                    name: "result".to_owned(),
                    field_name: "result".to_owned(),
                    required: true,
                    type_raw: "InlineQueryResult".to_owned(),
                    type_rust: "Value".to_owned(),
                },
                ParamSpec {
                    name: "results".to_owned(),
                    field_name: "results".to_owned(),
                    required: true,
                    type_raw: "Array of InlineQueryResult".to_owned(),
                    type_rust: "Vec<Value>".to_owned(),
                },
                ParamSpec {
                    name: "reply_markup".to_owned(),
                    field_name: "reply_markup".to_owned(),
                    required: false,
                    type_raw: "InlineKeyboardMarkup or ReplyKeyboardMarkup or ReplyKeyboardRemove or ForceReply".to_owned(),
                    type_rust: "Value".to_owned(),
                },
                ParamSpec {
                    name: "inline_keyboard".to_owned(),
                    field_name: "inline_keyboard".to_owned(),
                    required: false,
                    type_raw: "InlineKeyboardMarkup".to_owned(),
                    type_rust: "Value".to_owned(),
                },
                ParamSpec {
                    name: "menu_button".to_owned(),
                    field_name: "menu_button".to_owned(),
                    required: false,
                    type_raw: "MenuButton".to_owned(),
                    type_rust: "Value".to_owned(),
                },
                ParamSpec {
                    name: "photo".to_owned(),
                    field_name: "photo".to_owned(),
                    required: true,
                    type_raw: "InputProfilePhoto".to_owned(),
                    type_rust: "Value".to_owned(),
                },
                ParamSpec {
                    name: "button".to_owned(),
                    field_name: "button".to_owned(),
                    required: true,
                    type_raw: "KeyboardButton".to_owned(),
                    type_rust: "Value".to_owned(),
                },
                ParamSpec {
                    name: "media".to_owned(),
                    field_name: "media".to_owned(),
                    required: true,
                    type_raw: "Array of InputPaidMedia".to_owned(),
                    type_rust: "Vec<Value>".to_owned(),
                },
                ParamSpec {
                    name: "edit_media".to_owned(),
                    field_name: "edit_media".to_owned(),
                    required: true,
                    type_raw: "InputMedia".to_owned(),
                    type_rust: "Value".to_owned(),
                },
                ParamSpec {
                    name: "reactions".to_owned(),
                    field_name: "reactions".to_owned(),
                    required: false,
                    type_raw: "Array of ReactionType".to_owned(),
                    type_rust: "Vec<Value>".to_owned(),
                },
                ParamSpec {
                    name: "accepted_gift_types".to_owned(),
                    field_name: "accepted_gift_types".to_owned(),
                    required: false,
                    type_raw: "AcceptedGiftTypes".to_owned(),
                    type_rust: "Value".to_owned(),
                },
                ParamSpec {
                    name: "reply_parameters".to_owned(),
                    field_name: "reply_parameters".to_owned(),
                    required: false,
                    type_raw: "ReplyParameters".to_owned(),
                    type_rust: "Value".to_owned(),
                },
                ParamSpec {
                    name: "prices".to_owned(),
                    field_name: "prices".to_owned(),
                    required: true,
                    type_raw: "Array of LabeledPrice".to_owned(),
                    type_rust: "Vec<Value>".to_owned(),
                },
                ParamSpec {
                    name: "shipping_options".to_owned(),
                    field_name: "shipping_options".to_owned(),
                    required: false,
                    type_raw: "Array of ShippingOption".to_owned(),
                    type_rust: "Vec<Value>".to_owned(),
                },
                ParamSpec {
                    name: "business_connection_id".to_owned(),
                    field_name: "business_connection_id".to_owned(),
                    required: false,
                    type_raw: "String".to_owned(),
                    type_rust: "String".to_owned(),
                },
                ParamSpec {
                    name: "inline_message_id".to_owned(),
                    field_name: "inline_message_id".to_owned(),
                    required: true,
                    type_raw: "String".to_owned(),
                    type_rust: "String".to_owned(),
                },
                ParamSpec {
                    name: "custom_emoji_ids".to_owned(),
                    field_name: "custom_emoji_ids".to_owned(),
                    required: true,
                    type_raw: "Array of String".to_owned(),
                    type_rust: "Vec<String>".to_owned(),
                },
                ParamSpec {
                    name: "keywords".to_owned(),
                    field_name: "keywords".to_owned(),
                    required: false,
                    type_raw: "Array of String".to_owned(),
                    type_rust: "Vec<String>".to_owned(),
                },
            ],
        };

        let generated = generate_domain_module(&[&method]);
        assert!(generated.contains("self.chat_id.validate()?;"));
        assert!(generated.contains("validate_required_message_ids(\"message_ids\""));
        assert!(generated.contains("if let Some(value) = self.user_id.as_ref()"));
        assert!(generated.contains("self.result.validate()?;"));
        assert!(generated.contains("pub photo: crate::types::telegram::InputProfilePhoto,"));
        assert!(generated.contains("self.photo.validate()?;"));
        assert!(generated.contains("pub button: crate::types::telegram::KeyboardButton,"));
        assert!(generated.contains("self.button.validate()?;"));
        assert!(generated.contains("pub edit_media: crate::types::message::InputMedia,"));
        assert!(generated.contains("self.edit_media.validate()?;"));
        assert!(
            generated
                .contains("validate_required_items::<crate::types::telegram::InlineQueryResult>")
        );
        assert!(
            generated
                .contains("impl GeneratedValidate for crate::types::telegram::InlineQueryResult")
        );
        assert!(generated.contains("if let Some(value) = self.reply_parameters.as_ref()"));
        assert!(generated.contains("if let Some(value) = self.reply_markup.as_ref()"));
        assert!(generated.contains("if let Some(value) = self.inline_keyboard.as_ref()"));
        assert!(generated.contains("if let Some(value) = self.menu_button.as_ref()"));
        assert!(
            generated.contains("validate_required_items::<crate::types::telegram::InputPaidMedia>")
        );
        assert!(
            generated.contains("validate_required_items::<crate::types::payment::LabeledPrice>")
        );
        assert!(generated.contains("if let Some(values) = self.shipping_options.as_deref()"));
        assert!(generated.contains("validate_items(values)?;"));
        assert!(generated.contains("if let Some(values) = self.reactions.as_deref()"));
        assert!(generated.contains("if let Some(value) = self.accepted_gift_types.as_ref()"));
        assert!(generated.contains("validate_string_id(\"inline_message_id\""));
        assert!(generated.contains("if let Some(value) = self.business_connection_id.as_deref()"));
        assert!(generated.contains("validate_string_id(\"business_connection_id\", value)?;"));
        assert!(generated.contains("validate_required_string_items(\"custom_emoji_ids\""));
        assert!(generated.contains("if let Some(values) = self.keywords.as_deref()"));
        assert!(generated.contains("validate_string_items(\"keywords\", values)?;"));
    }

    #[test]
    fn generated_forward_messages_requires_ordered_message_ids() {
        let method = MethodSpec {
            fn_name: "forward_messages".to_owned(),
            method: "forwardMessages".to_owned(),
            return_desc: String::new(),
            params: vec![ParamSpec {
                name: "message_ids".to_owned(),
                field_name: "message_ids".to_owned(),
                required: true,
                type_raw: "Array of Integer".to_owned(),
                type_rust: "Vec<MessageId>".to_owned(),
            }],
        };

        let generated = generate_domain_module(&[&method]);
        assert!(generated.contains(
            "validate_required_ordered_message_ids(\"message_ids\", &self.message_ids)?;"
        ));
        assert!(generated.contains("const MAX_MESSAGE_IDS: usize = 100;"));
        assert!(generated.contains("message ids must be strictly increasing"));
    }

    #[test]
    fn generated_validation_requires_chat_or_inline_message_target() {
        let method = MethodSpec {
            fn_name: "edit_message_media".to_owned(),
            method: "editMessageMedia".to_owned(),
            return_desc: "Returns Message or True".to_owned(),
            params: vec![
                ParamSpec {
                    name: "chat_id".to_owned(),
                    field_name: "chat_id".to_owned(),
                    required: false,
                    type_raw: "Integer or String".to_owned(),
                    type_rust: "ChatId".to_owned(),
                },
                ParamSpec {
                    name: "message_id".to_owned(),
                    field_name: "message_id".to_owned(),
                    required: false,
                    type_raw: "Integer".to_owned(),
                    type_rust: "MessageId".to_owned(),
                },
                ParamSpec {
                    name: "inline_message_id".to_owned(),
                    field_name: "inline_message_id".to_owned(),
                    required: false,
                    type_raw: "String".to_owned(),
                    type_rust: "String".to_owned(),
                },
                ParamSpec {
                    name: "media".to_owned(),
                    field_name: "media".to_owned(),
                    required: true,
                    type_raw: "InputMedia".to_owned(),
                    type_rust: "Value".to_owned(),
                },
            ],
        };

        let generated = generate_domain_module(&[&method]);
        assert!(generated.contains("fn validate_chat_or_inline_message_target<T>("));
        assert!(generated.contains(
            "validate_chat_or_inline_message_target(\"editMessageMedia\", self.chat_id.as_ref(), self.message_id.as_ref(), self.inline_message_id.as_deref())?;"
        ));
        assert!(generated.contains("requires either"));
        assert!(generated.contains(
            "accepts either `chat_id` with `message_id` or `inline_message_id`, not both"
        ));
    }

    #[test]
    fn generated_send_gift_requires_exactly_one_target() {
        let method = MethodSpec {
            fn_name: "send_gift".to_owned(),
            method: "sendGift".to_owned(),
            return_desc: "Returns True on success".to_owned(),
            params: vec![
                ParamSpec {
                    name: "user_id".to_owned(),
                    field_name: "user_id".to_owned(),
                    required: false,
                    type_raw: "Integer".to_owned(),
                    type_rust: "UserId".to_owned(),
                },
                ParamSpec {
                    name: "chat_id".to_owned(),
                    field_name: "chat_id".to_owned(),
                    required: false,
                    type_raw: "Integer or String".to_owned(),
                    type_rust: "ChatId".to_owned(),
                },
                ParamSpec {
                    name: "gift_id".to_owned(),
                    field_name: "gift_id".to_owned(),
                    required: true,
                    type_raw: "String".to_owned(),
                    type_rust: "String".to_owned(),
                },
                ParamSpec {
                    name: "text".to_owned(),
                    field_name: "text".to_owned(),
                    required: false,
                    type_raw: "String".to_owned(),
                    type_rust: "String".to_owned(),
                },
            ],
        };

        let generated = generate_domain_module(&[&method]);
        assert!(generated.contains("match (self.user_id.is_some(), self.chat_id.is_some())"));
        assert!(generated.contains("sendGift requires either `user_id` or `chat_id`"));
        assert!(generated.contains("sendGift accepts either `user_id` or `chat_id`, not both"));
        assert!(generated.contains(
            "if let Some(value) = self.text.as_deref() {\n            validate_text_length_range(\"text\", value, 0, 128)?;\n        }"
        ));
    }

    #[test]
    fn generated_premium_gift_subscription_uses_api_price_matrix() {
        let method = MethodSpec {
            fn_name: "gift_premium_subscription".to_owned(),
            method: "giftPremiumSubscription".to_owned(),
            return_desc: "Returns True on success".to_owned(),
            params: vec![
                ParamSpec {
                    name: "month_count".to_owned(),
                    field_name: "month_count".to_owned(),
                    required: true,
                    type_raw: "Integer".to_owned(),
                    type_rust: "i64".to_owned(),
                },
                ParamSpec {
                    name: "star_count".to_owned(),
                    field_name: "star_count".to_owned(),
                    required: true,
                    type_raw: "Integer".to_owned(),
                    type_rust: "i64".to_owned(),
                },
                ParamSpec {
                    name: "text".to_owned(),
                    field_name: "text".to_owned(),
                    required: false,
                    type_raw: "String".to_owned(),
                    type_rust: "String".to_owned(),
                },
            ],
        };

        let generated = generate_domain_module(&[&method]);
        assert!(generated.contains("match (self.month_count, self.star_count)"));
        assert!(generated.contains("(3, 1000) | (6, 1500) | (12, 2500)"));
        assert!(generated.contains(
            "if let Some(value) = self.text.as_deref() {\n            validate_text_length_range(\"text\", value, 0, 128)?;\n        }"
        ));
    }

    #[test]
    fn generated_formatting_fields_use_shared_validation() {
        let draft = MethodSpec {
            fn_name: "send_message_draft".to_owned(),
            method: "sendMessageDraft".to_owned(),
            return_desc: "Returns True on success".to_owned(),
            params: vec![
                ParamSpec {
                    name: "text".to_owned(),
                    field_name: "text".to_owned(),
                    required: true,
                    type_raw: "String".to_owned(),
                    type_rust: "String".to_owned(),
                },
                ParamSpec {
                    name: "parse_mode".to_owned(),
                    field_name: "parse_mode".to_owned(),
                    required: false,
                    type_raw: "String".to_owned(),
                    type_rust: "String".to_owned(),
                },
                ParamSpec {
                    name: "entities".to_owned(),
                    field_name: "entities".to_owned(),
                    required: false,
                    type_raw: "Array of MessageEntity".to_owned(),
                    type_rust: "Vec<MessageEntity>".to_owned(),
                },
            ],
        };
        let gift = MethodSpec {
            fn_name: "send_gift".to_owned(),
            method: "sendGift".to_owned(),
            return_desc: "Returns True on success".to_owned(),
            params: vec![
                ParamSpec {
                    name: "text".to_owned(),
                    field_name: "text".to_owned(),
                    required: false,
                    type_raw: "String".to_owned(),
                    type_rust: "String".to_owned(),
                },
                ParamSpec {
                    name: "text_parse_mode".to_owned(),
                    field_name: "text_parse_mode".to_owned(),
                    required: false,
                    type_raw: "String".to_owned(),
                    type_rust: "String".to_owned(),
                },
                ParamSpec {
                    name: "text_entities".to_owned(),
                    field_name: "text_entities".to_owned(),
                    required: false,
                    type_raw: "Array of MessageEntity".to_owned(),
                    type_rust: "Vec<MessageEntity>".to_owned(),
                },
            ],
        };
        let paid_media = MethodSpec {
            fn_name: "send_paid_media".to_owned(),
            method: "sendPaidMedia".to_owned(),
            return_desc: "Returns Message on success".to_owned(),
            params: vec![
                ParamSpec {
                    name: "caption".to_owned(),
                    field_name: "caption".to_owned(),
                    required: false,
                    type_raw: "String".to_owned(),
                    type_rust: "String".to_owned(),
                },
                ParamSpec {
                    name: "parse_mode".to_owned(),
                    field_name: "parse_mode".to_owned(),
                    required: false,
                    type_raw: "String".to_owned(),
                    type_rust: "String".to_owned(),
                },
                ParamSpec {
                    name: "caption_entities".to_owned(),
                    field_name: "caption_entities".to_owned(),
                    required: false,
                    type_raw: "Array of MessageEntity".to_owned(),
                    type_rust: "Vec<MessageEntity>".to_owned(),
                },
            ],
        };

        let generated = generate_domain_module(&[&draft, &gift, &paid_media]);
        assert!(generated.contains("text_formatting as validate_text_formatting"));
        assert!(
            generated.contains("optional_text_formatting as validate_optional_text_formatting")
        );
        assert!(generated.contains(
            "validate_text_formatting(\"text\", &self.text, self.parse_mode, self.entities.as_deref())?;"
        ));
        assert!(generated.contains(
            "validate_optional_text_formatting(\"text\", self.text.as_deref(), self.text_parse_mode, self.text_entities.as_deref())?;"
        ));
        assert!(generated.contains(
            "validate_optional_text_formatting(\"caption\", self.caption.as_deref(), self.parse_mode, self.caption_entities.as_deref())?;"
        ));
    }

    #[test]
    fn optional_generated_message_ids_import_error_for_bounds_validation() {
        let method = MethodSpec {
            fn_name: "demo".to_owned(),
            method: "demo".to_owned(),
            return_desc: String::new(),
            params: vec![ParamSpec {
                name: "message_ids".to_owned(),
                field_name: "message_ids".to_owned(),
                required: false,
                type_raw: "Array of Integer".to_owned(),
                type_rust: "Vec<MessageId>".to_owned(),
            }],
        };

        let generated = generate_domain_module(&[&method]);
        assert!(generated.contains("use crate::{Error, Result};"));
        assert!(generated.contains("validate_message_ids(\"message_ids\", values)?;"));
    }

    #[test]
    fn generated_payment_callbacks_validate_ok_semantics() {
        let shipping = MethodSpec {
            fn_name: "answer_shipping_query".to_owned(),
            method: "answerShippingQuery".to_owned(),
            return_desc: "Returns True on success".to_owned(),
            params: vec![
                ParamSpec {
                    name: "shipping_query_id".to_owned(),
                    field_name: "shipping_query_id".to_owned(),
                    required: true,
                    type_raw: "String".to_owned(),
                    type_rust: "String".to_owned(),
                },
                ParamSpec {
                    name: "ok".to_owned(),
                    field_name: "ok".to_owned(),
                    required: true,
                    type_raw: "Boolean".to_owned(),
                    type_rust: "bool".to_owned(),
                },
                ParamSpec {
                    name: "shipping_options".to_owned(),
                    field_name: "shipping_options".to_owned(),
                    required: false,
                    type_raw: "Array of ShippingOption".to_owned(),
                    type_rust: "Vec<Value>".to_owned(),
                },
                ParamSpec {
                    name: "error_message".to_owned(),
                    field_name: "error_message".to_owned(),
                    required: false,
                    type_raw: "String".to_owned(),
                    type_rust: "String".to_owned(),
                },
            ],
        };
        let checkout = MethodSpec {
            fn_name: "answer_pre_checkout_query".to_owned(),
            method: "answerPreCheckoutQuery".to_owned(),
            return_desc: "Returns True on success".to_owned(),
            params: vec![
                ParamSpec {
                    name: "pre_checkout_query_id".to_owned(),
                    field_name: "pre_checkout_query_id".to_owned(),
                    required: true,
                    type_raw: "String".to_owned(),
                    type_rust: "String".to_owned(),
                },
                ParamSpec {
                    name: "ok".to_owned(),
                    field_name: "ok".to_owned(),
                    required: true,
                    type_raw: "Boolean".to_owned(),
                    type_rust: "bool".to_owned(),
                },
                ParamSpec {
                    name: "error_message".to_owned(),
                    field_name: "error_message".to_owned(),
                    required: false,
                    type_raw: "String".to_owned(),
                    type_rust: "String".to_owned(),
                },
            ],
        };

        let generated = generate_domain_module(&[&shipping, &checkout]);
        assert!(generated.contains("use crate::{Error, Result};"));
        assert!(
            generated.contains("answerShippingQuery requires `shipping_options` when `ok` is true")
        );
        assert!(
            generated.contains("validate_required_items::<crate::types::payment::ShippingOption>")
        );
        assert!(
            generated.contains(
                "answerShippingQuery requires non-empty `error_message` when `ok` is false"
            )
        );
        assert!(
            generated
                .contains("answerPreCheckoutQuery must omit `error_message` when `ok` is true")
        );
        assert!(generated.contains(
            "answerPreCheckoutQuery requires non-empty `error_message` when `ok` is false"
        ));
    }

    #[test]
    fn generated_invoice_requests_reuse_payment_invariants() {
        let send_invoice = MethodSpec {
            fn_name: "send_invoice".to_owned(),
            method: "sendInvoice".to_owned(),
            return_desc: "Returns Message on success".to_owned(),
            params: vec![
                ParamSpec {
                    name: "title".to_owned(),
                    field_name: "title".to_owned(),
                    required: true,
                    type_raw: "String".to_owned(),
                    type_rust: "String".to_owned(),
                },
                ParamSpec {
                    name: "description".to_owned(),
                    field_name: "description".to_owned(),
                    required: true,
                    type_raw: "String".to_owned(),
                    type_rust: "String".to_owned(),
                },
                ParamSpec {
                    name: "payload".to_owned(),
                    field_name: "payload".to_owned(),
                    required: true,
                    type_raw: "String".to_owned(),
                    type_rust: "String".to_owned(),
                },
                ParamSpec {
                    name: "currency".to_owned(),
                    field_name: "currency".to_owned(),
                    required: true,
                    type_raw: "String".to_owned(),
                    type_rust: "String".to_owned(),
                },
                ParamSpec {
                    name: "prices".to_owned(),
                    field_name: "prices".to_owned(),
                    required: true,
                    type_raw: "Array of LabeledPrice".to_owned(),
                    type_rust: "Vec<Value>".to_owned(),
                },
                ParamSpec {
                    name: "max_tip_amount".to_owned(),
                    field_name: "max_tip_amount".to_owned(),
                    required: false,
                    type_raw: "Integer".to_owned(),
                    type_rust: "i64".to_owned(),
                },
                ParamSpec {
                    name: "suggested_tip_amounts".to_owned(),
                    field_name: "suggested_tip_amounts".to_owned(),
                    required: false,
                    type_raw: "Array of Integer".to_owned(),
                    type_rust: "Vec<i64>".to_owned(),
                },
                ParamSpec {
                    name: "reply_markup".to_owned(),
                    field_name: "reply_markup".to_owned(),
                    required: false,
                    type_raw: "InlineKeyboardMarkup".to_owned(),
                    type_rust: "Value".to_owned(),
                },
            ],
        };
        let create_invoice_link = MethodSpec {
            fn_name: "create_invoice_link".to_owned(),
            method: "createInvoiceLink".to_owned(),
            return_desc: "Returns String on success".to_owned(),
            params: vec![
                ParamSpec {
                    name: "business_connection_id".to_owned(),
                    field_name: "business_connection_id".to_owned(),
                    required: false,
                    type_raw: "String".to_owned(),
                    type_rust: "String".to_owned(),
                },
                ParamSpec {
                    name: "currency".to_owned(),
                    field_name: "currency".to_owned(),
                    required: true,
                    type_raw: "String".to_owned(),
                    type_rust: "String".to_owned(),
                },
                ParamSpec {
                    name: "prices".to_owned(),
                    field_name: "prices".to_owned(),
                    required: true,
                    type_raw: "Array of LabeledPrice".to_owned(),
                    type_rust: "Vec<Value>".to_owned(),
                },
                ParamSpec {
                    name: "subscription_period".to_owned(),
                    field_name: "subscription_period".to_owned(),
                    required: false,
                    type_raw: "Integer".to_owned(),
                    type_rust: "i64".to_owned(),
                },
            ],
        };

        let generated = generate_domain_module(&[&send_invoice, &create_invoice_link]);
        assert!(generated.contains("validate_invoice_title(\"sendInvoice\", &self.title)?;"));
        assert!(
            generated
                .contains("validate_invoice_description(\"sendInvoice\", &self.description)?;")
        );
        assert!(generated.contains("validate_invoice_payload(\"sendInvoice\", &self.payload)?;"));
        assert!(generated.contains("validate_invoice_currency(\"sendInvoice\", &self.currency)?;"));
        assert!(
            generated.contains(
                "validate_invoice_prices(\"sendInvoice\", &self.currency, &self.prices)?;"
            )
        );
        assert!(generated.contains("validate_invoice_business_connection_id("));
        assert!(generated.contains("validate_invoice_tip_configuration("));
        assert!(generated.contains(
            "validate_invoice_reply_markup(\"sendInvoice\", self.reply_markup.as_ref())?;"
        ));
        assert!(generated.contains(
            "validate_invoice_subscription_period(\n            \"createInvoiceLink\",\n            &self.currency,\n            self.subscription_period,\n            &self.prices,\n        )?;"
        ));
        assert!(!generated.contains("validate_positive_i64(\"max_tip_amount\""));
        assert!(
            !generated.contains(
                "validate_required_items::<crate::types::payment::LabeledPrice>(\"prices\""
            )
        );
        assert!(!generated.contains(
            "if let Some(value) = self.reply_markup.as_ref() {\n            value.validate()?;\n        }"
        ));
        assert!(!generated.contains("validate_positive_i64(\"subscription_period\""));
    }

    #[test]
    fn generated_integer_chat_ids_use_numeric_wrapper() {
        let method = MethodSpec {
            fn_name: "demo".to_owned(),
            method: "demo".to_owned(),
            return_desc: "Returns True on success".to_owned(),
            params: vec![
                ParamSpec {
                    name: "chat_id".to_owned(),
                    field_name: "chat_id".to_owned(),
                    required: true,
                    type_raw: "Integer".to_owned(),
                    type_rust: "i64".to_owned(),
                },
                ParamSpec {
                    name: "from_chat_id".to_owned(),
                    field_name: "from_chat_id".to_owned(),
                    required: true,
                    type_raw: "Integer".to_owned(),
                    type_rust: "i64".to_owned(),
                },
                ParamSpec {
                    name: "new_owner_chat_id".to_owned(),
                    field_name: "new_owner_chat_id".to_owned(),
                    required: false,
                    type_raw: "Integer".to_owned(),
                    type_rust: "i64".to_owned(),
                },
            ],
        };

        let generated = generate_domain_module(&[&method]);
        assert!(generated.contains("pub chat_id: crate::types::common::NumericChatId"));
        assert!(generated.contains("pub from_chat_id: crate::types::common::NumericChatId"));
        assert!(
            generated
                .contains("pub new_owner_chat_id: Option<crate::types::common::NumericChatId>")
        );
        assert!(generated.contains("self.chat_id.validate()?;"));
        assert!(generated.contains("self.from_chat_id.validate()?;"));
        assert!(generated.contains("if let Some(value) = self.new_owner_chat_id.as_ref()"));
    }

    #[test]
    fn generated_numeric_fields_use_domain_bounds() {
        let method = MethodSpec {
            fn_name: "demo".to_owned(),
            method: "demo".to_owned(),
            return_desc: "Returns True on success".to_owned(),
            params: vec![
                ParamSpec {
                    name: "star_count".to_owned(),
                    field_name: "star_count".to_owned(),
                    required: true,
                    type_raw: "Integer".to_owned(),
                    type_rust: "i64".to_owned(),
                },
                ParamSpec {
                    name: "limit".to_owned(),
                    field_name: "limit".to_owned(),
                    required: false,
                    type_raw: "Integer".to_owned(),
                    type_rust: "i64".to_owned(),
                },
                ParamSpec {
                    name: "position".to_owned(),
                    field_name: "position".to_owned(),
                    required: true,
                    type_raw: "Integer".to_owned(),
                    type_rust: "i64".to_owned(),
                },
                ParamSpec {
                    name: "offset".to_owned(),
                    field_name: "offset".to_owned(),
                    required: false,
                    type_raw: "Integer".to_owned(),
                    type_rust: "i64".to_owned(),
                },
            ],
        };

        let generated = generate_domain_module(&[&method]);
        assert!(generated.contains("positive_i64 as validate_positive_i64"));
        assert!(generated.contains("non_negative_i64 as validate_non_negative_i64"));
        assert!(generated.contains("validate_positive_i64(\"star_count\", self.star_count)?;"));
        assert!(generated.contains("validate_positive_i64(\"limit\", value)?;"));
        assert!(generated.contains("validate_non_negative_i64(\"position\", self.position)?;"));
        assert!(generated.contains("validate_non_negative_i64(\"offset\", value)?;"));
    }

    #[test]
    fn generated_approve_suggested_post_uses_domain_send_date_validation() {
        let method = MethodSpec {
            fn_name: "approve_suggested_post".to_owned(),
            method: "approveSuggestedPost".to_owned(),
            return_desc: "Returns True on success".to_owned(),
            params: vec![ParamSpec {
                name: "send_date".to_owned(),
                field_name: "send_date".to_owned(),
                required: false,
                type_raw: "Integer".to_owned(),
                type_rust: "i64".to_owned(),
            }],
        };

        let generated = generate_domain_module(&[&method]);
        assert!(generated.contains(
            "suggested_post_approval_send_date as validate_suggested_post_approval_send_date"
        ));
        assert!(
            generated
                .contains("validate_suggested_post_approval_send_date(\"send_date\", value)?;")
        );
        assert!(!generated.contains("validate_positive_i64(\"send_date\", value)?;"));
    }

    #[test]
    fn generated_paginated_limits_use_api_bounds() {
        let method = MethodSpec {
            fn_name: "get_user_profile_audios".to_owned(),
            method: "getUserProfileAudios".to_owned(),
            return_desc: String::new(),
            params: vec![ParamSpec {
                name: "limit".to_owned(),
                field_name: "limit".to_owned(),
                required: false,
                type_raw: "Integer".to_owned(),
                type_rust: "i64".to_owned(),
            }],
        };

        let generated = generate_domain_module(&[&method]);
        assert!(generated.contains("i64_range as validate_i64_range"));
        assert!(generated.contains("validate_i64_range(\"limit\", value, 1, 100)?;"));
        assert!(!generated.contains("validate_positive_i64(\"limit\", value)?;"));
    }

    #[test]
    fn generated_string_offsets_reject_control_characters() {
        let method = MethodSpec {
            fn_name: "get_user_gifts".to_owned(),
            method: "getUserGifts".to_owned(),
            return_desc: String::new(),
            params: vec![ParamSpec {
                name: "offset".to_owned(),
                field_name: "offset".to_owned(),
                required: false,
                type_raw: "String".to_owned(),
                type_rust: "String".to_owned(),
            }],
        };

        let generated = generate_domain_module(&[&method]);
        assert!(generated.contains("control_free_string as validate_control_free_string"));
        assert!(!generated.contains("fn validate_control_free_string("));
        assert!(generated.contains(
            "if let Some(value) = self.offset.as_deref() {\n            validate_control_free_string(\"offset\", value)?;\n        }"
        ));
    }

    #[test]
    fn generated_empty_string_status_reset_uses_control_free_validation() {
        let method = MethodSpec {
            fn_name: "set_user_emoji_status".to_owned(),
            method: "setUserEmojiStatus".to_owned(),
            return_desc: "Returns True on success".to_owned(),
            params: vec![ParamSpec {
                name: "emoji_status_custom_emoji_id".to_owned(),
                field_name: "emoji_status_custom_emoji_id".to_owned(),
                required: false,
                type_raw: "String".to_owned(),
                type_rust: "String".to_owned(),
            }],
        };

        let generated = generate_domain_module(&[&method]);
        assert!(generated.contains("control_free_string as validate_control_free_string"));
        assert!(generated.contains(
            "if let Some(value) = self.emoji_status_custom_emoji_id.as_deref() {\n            validate_control_free_string(\"emoji_status_custom_emoji_id\", value)?;\n        }"
        ));
        assert!(!generated.contains("validate_string_id(\"emoji_status_custom_emoji_id\""));
    }

    #[test]
    fn generated_member_tag_uses_api_text_bounds() {
        let method = MethodSpec {
            fn_name: "set_chat_member_tag".to_owned(),
            method: "setChatMemberTag".to_owned(),
            return_desc: "Returns True on success".to_owned(),
            params: vec![ParamSpec {
                name: "tag".to_owned(),
                field_name: "tag".to_owned(),
                required: false,
                type_raw: "String".to_owned(),
                type_rust: "String".to_owned(),
            }],
        };

        let generated = generate_domain_module(&[&method]);
        assert!(generated.contains("emoji_free_text as validate_emoji_free_text"));
        assert!(generated.contains(
            "if let Some(value) = self.tag.as_deref() {\n            validate_emoji_free_text(\"tag\", value, 16)?;\n        }"
        ));
    }

    #[test]
    fn generated_short_profile_text_uses_api_bounds() {
        let business_name = MethodSpec {
            fn_name: "set_business_account_name".to_owned(),
            method: "setBusinessAccountName".to_owned(),
            return_desc: "Returns True on success".to_owned(),
            params: vec![
                ParamSpec {
                    name: "first_name".to_owned(),
                    field_name: "first_name".to_owned(),
                    required: true,
                    type_raw: "String".to_owned(),
                    type_rust: "String".to_owned(),
                },
                ParamSpec {
                    name: "last_name".to_owned(),
                    field_name: "last_name".to_owned(),
                    required: false,
                    type_raw: "String".to_owned(),
                    type_rust: "String".to_owned(),
                },
            ],
        };
        let username = MethodSpec {
            fn_name: "set_business_account_username".to_owned(),
            method: "setBusinessAccountUsername".to_owned(),
            return_desc: "Returns True on success".to_owned(),
            params: vec![ParamSpec {
                name: "username".to_owned(),
                field_name: "username".to_owned(),
                required: false,
                type_raw: "String".to_owned(),
                type_rust: "String".to_owned(),
            }],
        };
        let bio = MethodSpec {
            fn_name: "set_business_account_bio".to_owned(),
            method: "setBusinessAccountBio".to_owned(),
            return_desc: "Returns True on success".to_owned(),
            params: vec![ParamSpec {
                name: "bio".to_owned(),
                field_name: "bio".to_owned(),
                required: false,
                type_raw: "String".to_owned(),
                type_rust: "String".to_owned(),
            }],
        };
        let verify_user = MethodSpec {
            fn_name: "verify_user".to_owned(),
            method: "verifyUser".to_owned(),
            return_desc: "Returns True on success".to_owned(),
            params: vec![ParamSpec {
                name: "custom_description".to_owned(),
                field_name: "custom_description".to_owned(),
                required: false,
                type_raw: "String".to_owned(),
                type_rust: "String".to_owned(),
            }],
        };
        let decline_suggested_post = MethodSpec {
            fn_name: "decline_suggested_post".to_owned(),
            method: "declineSuggestedPost".to_owned(),
            return_desc: "Returns True on success".to_owned(),
            params: vec![ParamSpec {
                name: "comment".to_owned(),
                field_name: "comment".to_owned(),
                required: false,
                type_raw: "String".to_owned(),
                type_rust: "String".to_owned(),
            }],
        };

        let generated = generate_domain_module(&[
            &business_name,
            &username,
            &bio,
            &verify_user,
            &decline_suggested_post,
        ]);
        assert!(generated.contains("text_length_range as validate_text_length_range"));
        assert!(generated.contains("username_or_empty as validate_username_or_empty"));
        assert!(
            generated
                .contains("validate_text_length_range(\"first_name\", &self.first_name, 1, 64)?;")
        );
        assert!(generated.contains(
            "if let Some(value) = self.last_name.as_deref() {\n            validate_text_length_range(\"last_name\", value, 0, 64)?;\n        }"
        ));
        assert!(generated.contains(
            "if let Some(value) = self.username.as_deref() {\n            validate_username_or_empty(\"username\", value)?;\n        }"
        ));
        assert!(generated.contains(
            "if let Some(value) = self.bio.as_deref() {\n            validate_text_length_range(\"bio\", value, 0, 140)?;\n        }"
        ));
        assert!(generated.contains(
            "if let Some(value) = self.custom_description.as_deref() {\n            validate_text_length_range(\"custom_description\", value, 0, 70)?;\n        }"
        ));
        assert!(generated.contains(
            "if let Some(value) = self.comment.as_deref() {\n            validate_text_length_range(\"comment\", value, 0, 128)?;\n        }"
        ));
    }

    #[test]
    fn generated_optional_plain_strings_reject_empty_and_control_characters() {
        let method = MethodSpec {
            fn_name: "send_sticker".to_owned(),
            method: "sendSticker".to_owned(),
            return_desc: "Returns Message".to_owned(),
            params: vec![ParamSpec {
                name: "emoji".to_owned(),
                field_name: "emoji".to_owned(),
                required: false,
                type_raw: "String".to_owned(),
                type_rust: "String".to_owned(),
            }],
        };

        let generated = generate_domain_module(&[&method]);
        assert!(generated.contains("required_string as validate_required_string"));
        assert!(generated.contains("control_free_string as validate_control_free_string"));
        assert!(generated.contains(
            "if let Some(value) = self.emoji.as_deref() {\n            validate_required_string(\"emoji\", value)?;\n            validate_control_free_string(\"emoji\", value)?;\n        }"
        ));
    }

    #[test]
    fn generated_sticker_strings_reject_control_characters() {
        let get_sticker_set = MethodSpec {
            fn_name: "get_sticker_set".to_owned(),
            method: "getStickerSet".to_owned(),
            return_desc: "Returns StickerSet".to_owned(),
            params: vec![ParamSpec {
                name: "name".to_owned(),
                field_name: "name".to_owned(),
                required: true,
                type_raw: "String".to_owned(),
                type_rust: "String".to_owned(),
            }],
        };
        let set_title = MethodSpec {
            fn_name: "set_sticker_set_title".to_owned(),
            method: "setStickerSetTitle".to_owned(),
            return_desc: "Returns True on success".to_owned(),
            params: vec![
                ParamSpec {
                    name: "name".to_owned(),
                    field_name: "name".to_owned(),
                    required: true,
                    type_raw: "String".to_owned(),
                    type_rust: "String".to_owned(),
                },
                ParamSpec {
                    name: "title".to_owned(),
                    field_name: "title".to_owned(),
                    required: true,
                    type_raw: "String".to_owned(),
                    type_rust: "String".to_owned(),
                },
            ],
        };
        let replace = MethodSpec {
            fn_name: "replace_sticker_in_set".to_owned(),
            method: "replaceStickerInSet".to_owned(),
            return_desc: "Returns True on success".to_owned(),
            params: vec![ParamSpec {
                name: "old_sticker".to_owned(),
                field_name: "old_sticker".to_owned(),
                required: true,
                type_raw: "String".to_owned(),
                type_rust: "String".to_owned(),
            }],
        };

        let generated = generate_domain_module(&[&get_sticker_set, &set_title, &replace]);
        assert!(generated.contains("control_free_string as validate_control_free_string"));
        assert!(generated.contains(
            "validate_required_string(\"name\", &self.name)?;\n        validate_control_free_string(\"name\", &self.name)?;"
        ));
        assert!(generated.contains(
            "validate_required_string(\"title\", &self.title)?;\n        validate_control_free_string(\"title\", &self.title)?;"
        ));
        assert!(generated.contains(
            "validate_required_string(\"old_sticker\", &self.old_sticker)?;\n        validate_control_free_string(\"old_sticker\", &self.old_sticker)?;"
        ));
    }

    #[test]
    fn generated_sticker_string_lists_use_api_limits() {
        let custom_emoji = MethodSpec {
            fn_name: "get_custom_emoji_stickers".to_owned(),
            method: "getCustomEmojiStickers".to_owned(),
            return_desc: "Returns Array of Sticker".to_owned(),
            params: vec![ParamSpec {
                name: "custom_emoji_ids".to_owned(),
                field_name: "custom_emoji_ids".to_owned(),
                required: true,
                type_raw: "Array of String".to_owned(),
                type_rust: "Vec<String>".to_owned(),
            }],
        };
        let emoji_list = MethodSpec {
            fn_name: "set_sticker_emoji_list".to_owned(),
            method: "setStickerEmojiList".to_owned(),
            return_desc: "Returns True on success".to_owned(),
            params: vec![ParamSpec {
                name: "emoji_list".to_owned(),
                field_name: "emoji_list".to_owned(),
                required: true,
                type_raw: "Array of String".to_owned(),
                type_rust: "Vec<String>".to_owned(),
            }],
        };
        let keywords = MethodSpec {
            fn_name: "set_sticker_keywords".to_owned(),
            method: "setStickerKeywords".to_owned(),
            return_desc: "Returns True on success".to_owned(),
            params: vec![ParamSpec {
                name: "keywords".to_owned(),
                field_name: "keywords".to_owned(),
                required: false,
                type_raw: "Array of String".to_owned(),
                type_rust: "Vec<String>".to_owned(),
            }],
        };

        let generated = generate_domain_module(&[&custom_emoji, &emoji_list, &keywords]);
        assert!(generated.contains("fn validate_limited_string_items("));
        assert!(generated.contains(
            "validate_required_limited_string_items(\"custom_emoji_ids\", &self.custom_emoji_ids, crate::types::sticker::MAX_CUSTOM_EMOJI_IDS)?;"
        ));
        assert!(generated.contains(
            "validate_required_limited_string_items(\"emoji_list\", &self.emoji_list, crate::types::sticker::MAX_STICKER_EMOJIS)?;"
        ));
        assert!(generated.contains(
            "validate_limited_string_items(\"keywords\", values, crate::types::sticker::MAX_STICKER_KEYWORDS)?;"
        ));
        assert!(!generated.contains("fn validate_required_string_items("));
        assert!(generated.contains("accepts at most {max_items} items"));
    }

    #[test]
    fn generated_sticker_requests_use_typed_enums_and_nested_validation() {
        let method = MethodSpec {
            fn_name: "create_new_sticker_set".to_owned(),
            method: "createNewStickerSet".to_owned(),
            return_desc: "Returns True on success".to_owned(),
            params: vec![
                ParamSpec {
                    name: "stickers".to_owned(),
                    field_name: "stickers".to_owned(),
                    required: true,
                    type_raw: "Array of InputSticker".to_owned(),
                    type_rust: "Value".to_owned(),
                },
                ParamSpec {
                    name: "sticker_type".to_owned(),
                    field_name: "sticker_type".to_owned(),
                    required: false,
                    type_raw: "String".to_owned(),
                    type_rust: "String".to_owned(),
                },
                ParamSpec {
                    name: "sticker_format".to_owned(),
                    field_name: "sticker_format".to_owned(),
                    required: true,
                    type_raw: "String".to_owned(),
                    type_rust: "String".to_owned(),
                },
            ],
        };

        let generated = generate_domain_module(&[&method]);
        assert!(generated.contains("Vec<crate::types::sticker::InputSticker>"));
        assert!(generated.contains("Option<crate::types::sticker::StickerType>"));
        assert!(generated.contains("pub sticker_format: crate::types::sticker::StickerFormat"));
        assert!(
            generated.contains("validate_required_items::<crate::types::sticker::InputSticker>")
        );
    }

    #[test]
    fn generated_parse_modes_use_shared_enum() {
        let method = MethodSpec {
            fn_name: "demo".to_owned(),
            method: "demo".to_owned(),
            return_desc: "Returns True on success".to_owned(),
            params: vec![
                ParamSpec {
                    name: "parse_mode".to_owned(),
                    field_name: "parse_mode".to_owned(),
                    required: false,
                    type_raw: "String".to_owned(),
                    type_rust: "String".to_owned(),
                },
                ParamSpec {
                    name: "text_parse_mode".to_owned(),
                    field_name: "text_parse_mode".to_owned(),
                    required: false,
                    type_raw: "String".to_owned(),
                    type_rust: "String".to_owned(),
                },
            ],
        };

        let generated = generate_domain_module(&[&method]);
        assert!(generated.contains("pub parse_mode: Option<crate::types::common::ParseMode>"));
        assert!(generated.contains("pub text_parse_mode: Option<crate::types::common::ParseMode>"));
    }

    #[test]
    fn bundled_spec_is_self_describing() {
        let spec_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("spec")
            .join(SPEC_FILE);
        let bytes = fs::read(spec_path);
        assert!(bytes.is_ok());
        let bytes = match bytes {
            Ok(bytes) => bytes,
            Err(_) => return,
        };

        let spec = serde_json::from_slice::<BotApiSpec>(&bytes);
        assert!(spec.is_ok());
        let spec = match spec {
            Ok(spec) => spec,
            Err(_) => return,
        };

        assert!(spec.validate().is_ok());
        assert!(spec.all_methods.len() >= spec.advanced_methods.len());
    }
}
