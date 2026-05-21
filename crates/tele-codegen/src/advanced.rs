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
        "getUserProfilePhotos" => "crate::types::bot::UserProfilePhotos",
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
        "getForumTopicIconStickers" => "Vec<crate::types::sticker::Sticker>",
        "getChatMenuButton" => "crate::types::telegram::MenuButton",
        "getMyCommands" => "Vec<crate::types::command::BotCommand>",
        "getMyName" => "crate::types::command::BotName",
        "getMyDescription" => "crate::types::command::BotDescription",
        "getMyShortDescription" => "crate::types::command::BotShortDescription",
        "getMyDefaultAdministratorRights" => "crate::types::chat::ChatAdministratorRights",
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
        "getStickerSet" => "crate::types::sticker::StickerSet",
        "getCustomEmojiStickers" => "Vec<crate::types::sticker::Sticker>",
        "uploadStickerFile" => "crate::types::file::File",
        "sendInvoice" => "crate::types::message::Message",
        "createInvoiceLink" => "String",
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

fn type_has_validate(field_ty: &str) -> bool {
    TYPES_WITH_VALIDATE.contains(&field_ty)
}

fn validated_vec_item_type(field_ty: &str) -> Option<&str> {
    field_ty
        .strip_prefix("Vec<")
        .and_then(|inner| inner.strip_suffix('>'))
        .filter(|inner| type_has_validate(inner))
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

fn validation_rule(param: &ParamSpec) -> Option<String> {
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
            return Some(format!(
                "        validate_required_message_ids(\"{}\", &self.{})?;",
                param.name, param.field_name
            ));
        }

        return Some(format!(
            "        if let Some(values) = self.{}.as_deref() {{\n            validate_message_ids(values)?;\n        }}",
            param.field_name
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
        .filter_map(validation_rule)
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

fn generate_domain_module(methods: &[&MethodSpec]) -> String {
    let body = methods
        .iter()
        .map(|method| render_request(method).trim_end().to_owned())
        .collect::<Vec<_>>()
        .join("\n\n");
    let uses_value = body.contains("Value");
    let uses_required_string_validator = body.contains("validate_required_string(");
    let uses_string_id_validator = body.contains("validate_string_id(");
    let uses_required_vec_validator = body.contains("validate_required_vec(");
    let uses_required_message_ids_validator = body.contains("validate_required_message_ids(");
    let uses_message_ids_validator = body.contains("validate_message_ids(");
    let uses_required_items_validator = body.contains("validate_required_items::<");
    let uses_items_validator = body.contains("validate_items(");
    let uses_positive_i64_validator = body.contains("validate_positive_i64(");
    let uses_non_negative_i64_validator = body.contains("validate_non_negative_i64(");
    let uses_shared_validation = uses_required_string_validator
        || uses_string_id_validator
        || uses_required_vec_validator
        || uses_positive_i64_validator
        || uses_non_negative_i64_validator;
    let uses_error = uses_required_message_ids_validator || uses_required_items_validator;
    let uses_result = body.contains("fn validate(&self) -> Result<()>")
        || uses_required_string_validator
        || uses_string_id_validator
        || uses_required_vec_validator
        || uses_required_message_ids_validator
        || uses_message_ids_validator
        || uses_required_items_validator
        || uses_items_validator
        || uses_positive_i64_validator
        || uses_non_negative_i64_validator;

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
        if uses_non_negative_i64_validator {
            let _ = writeln!(
                &mut out,
                "    non_negative_i64 as validate_non_negative_i64,"
            );
        }
        if uses_positive_i64_validator {
            let _ = writeln!(&mut out, "    positive_i64 as validate_positive_i64,");
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
        let _ = writeln!(&mut out, "}};");
        let _ = writeln!(&mut out);
    }
    let _ = writeln!(&mut out, "use super::AdvancedRequest;");
    let _ = writeln!(&mut out);
    if uses_items_validator || uses_required_items_validator {
        let _ = writeln!(&mut out, "trait GeneratedValidate {{");
        let _ = writeln!(&mut out, "    fn validate_generated(&self) -> Result<()>;");
        let _ = writeln!(&mut out, "}}");
        let _ = writeln!(&mut out);
        for ty in TYPES_WITH_VALIDATE {
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
        let _ = writeln!(
            &mut out,
            "fn validate_message_ids(values: &[crate::types::common::MessageId]) -> Result<()> {{"
        );
        let _ = writeln!(&mut out, "    for value in values {{");
        let _ = writeln!(&mut out, "        value.validate()?;");
        let _ = writeln!(&mut out, "    }}");
        let _ = writeln!(&mut out);
        let _ = writeln!(&mut out, "    Ok(())");
        let _ = writeln!(&mut out, "}}");
        let _ = writeln!(&mut out);
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
        let _ = writeln!(&mut out, "    validate_message_ids(values)");
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
        assert!(generated.contains("if let Some(values) = self.reactions.as_deref()"));
        assert!(generated.contains("if let Some(value) = self.accepted_gift_types.as_ref()"));
        assert!(generated.contains("validate_string_id(\"inline_message_id\""));
        assert!(generated.contains("if let Some(value) = self.business_connection_id.as_deref()"));
        assert!(generated.contains("validate_string_id(\"business_connection_id\", value)?;"));
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
