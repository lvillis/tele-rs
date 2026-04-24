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

pub(crate) fn generate() -> Result<(), Box<dyn std::error::Error>> {
    let paths = paths()?;
    let spec: BotApiSpec = serde_json::from_slice(&fs::read(&paths.spec)?)?;
    spec.validate()
        .map_err(|error| -> Box<dyn std::error::Error> { error.into() })?;

    let mut grouped: HashMap<&'static str, Vec<&MethodSpec>> = DOMAIN_ORDER
        .into_iter()
        .map(|domain| (domain, Vec::new()))
        .collect();
    for method in &spec.advanced_methods {
        let domain = domain_for_method(&method.fn_name);
        let Some(methods) = grouped.get_mut(domain) else {
            return Err(format!("unknown advanced method domain `{domain}`").into());
        };
        methods.push(method);
    }

    write_if_changed(&paths.types_root, &generate_types_root(&grouped))?;

    for domain in DOMAIN_ORDER {
        let path = paths.types_dir.join(format!("advanced_{domain}.rs"));
        let content = generate_domain_module(grouped.get(domain).map_or(&[], Vec::as_slice));
        write_if_changed(&path, &content)?;
    }

    write_if_changed(
        &paths.api_methods,
        &generate_api_methods(&spec.advanced_methods),
    )?;
    Ok(())
}

pub(crate) fn check() -> Result<(), Box<dyn std::error::Error>> {
    let paths = paths()?;
    let generated_paths = generated_paths(&paths);
    let before = read_generated_files(&generated_paths)?;

    generate()?;
    run_cargo_fmt()?;

    let after = read_generated_files(&generated_paths)?;
    let changed = generated_paths
        .iter()
        .zip(before.iter().zip(after.iter()))
        .filter_map(|(path, (before, after))| (before != after).then_some(path))
        .collect::<Vec<_>>();

    if !changed.is_empty() {
        let mut message = String::from("generated advanced API files were out of date:");
        for path in changed {
            let _ = write!(&mut message, "\n- {}", path.display());
        }
        message.push_str("\nrun `cargo run -p tele-codegen -- gen-advanced && cargo fmt --all`");
        return Err(message.into());
    }

    Ok(())
}

fn generated_paths(paths: &Paths) -> Vec<PathBuf> {
    let mut files = Vec::with_capacity(DOMAIN_ORDER.len() + 2);
    files.push(paths.types_root.clone());
    for domain in DOMAIN_ORDER {
        files.push(paths.types_dir.join(format!("advanced_{domain}.rs")));
    }
    files.push(paths.api_methods.clone());
    files
}

fn read_generated_files(
    paths: &[PathBuf],
) -> Result<Vec<Option<String>>, Box<dyn std::error::Error>> {
    paths
        .iter()
        .map(|path| match fs::read_to_string(path) {
            Ok(content) => Ok(Some(content)),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(format!("failed to read {}: {error}", path.display()).into()),
        })
        .collect()
}

fn run_cargo_fmt() -> Result<(), Box<dyn std::error::Error>> {
    let status = Command::new("cargo").args(["fmt", "--all"]).status()?;
    if !status.success() {
        return Err(format!("cargo fmt --all failed with status {status}").into());
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

fn qualify_common_type(field_ty: &str) -> String {
    field_ty
        .replace("ChatId", "crate::types::common::ChatId")
        .replace("MessageId", "crate::types::common::MessageId")
        .replace("UserId", "crate::types::common::UserId")
}

fn ctor_arg_type(field_ty: &str) -> &str {
    match field_ty {
        "String" => "impl Into<String>",
        "crate::types::common::ChatId" => "impl Into<crate::types::common::ChatId>",
        _ => field_ty,
    }
}

fn ctor_assign(field_name: &str, field_ty: &str) -> String {
    match field_ty {
        "String" | "crate::types::common::ChatId" => format!("{field_name}: {field_name}.into()"),
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
        (
            "InlineKeyboardMarkup",
            "crate::types::telegram::InlineKeyboardMarkup",
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
    let _ = writeln!(
        &mut out,
        "/// Typed request marker for advanced API methods."
    );
    let _ = writeln!(&mut out, "pub trait AdvancedRequest: Serialize {{");
    let _ = writeln!(&mut out, "    type Response: DeserializeOwned;");
    let _ = writeln!(&mut out, "    const METHOD: &'static str;");
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
    let _ = writeln!(&mut out, "use super::AdvancedRequest;");
    let _ = writeln!(&mut out);
    if !body.is_empty() {
        let _ = writeln!(&mut out, "{body}");
    }
    out
}

fn generate_api_methods(methods: &[MethodSpec]) -> String {
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
