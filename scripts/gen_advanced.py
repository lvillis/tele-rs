#!/usr/bin/env python3
"""Generate advanced Telegram API request models and service methods.

Spec lookup order:
1. `TELE_ADVANCED_SPEC_PATH` environment variable
2. `.docs/spec/telegram_bot_api_9_4_methods.json`
3. `scripts/spec/telegram_bot_api_9_4_methods.json`

Outputs:
- `crates/tele/src/types/advanced.rs`
- `crates/tele/src/api/advanced.rs`
"""

from __future__ import annotations

import json
import os
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parent.parent
SPEC_FILE = "telegram_bot_api_9_4_methods.json"
DEFAULT_SPEC_PATHS = [
    ROOT / ".docs/spec" / SPEC_FILE,
    ROOT / "scripts/spec" / SPEC_FILE,
]
TYPES_OUT = ROOT / "crates/tele/src/types/advanced.rs"
API_OUT = ROOT / "crates/tele/src/api/advanced.rs"


def resolve_spec_path() -> Path:
    env_value = os.environ.get("TELE_ADVANCED_SPEC_PATH")
    candidates: list[Path] = []
    if env_value:
        candidates.append(Path(env_value).expanduser())
    candidates.extend(DEFAULT_SPEC_PATHS)

    for candidate in candidates:
        if candidate.exists():
            return candidate

    rendered = "\n".join(f"- {path}" for path in candidates)
    raise FileNotFoundError(
        "Could not locate Telegram API spec JSON. Checked:\n" f"{rendered}"
    )


def to_pascal_case(snake: str) -> str:
    return "".join(part.capitalize() for part in snake.split("_"))


def request_type_name(fn_name: str) -> str:
    base = to_pascal_case(fn_name)
    if not base.endswith("Request"):
        base += "Request"
    return f"Advanced{base}"


def typed_fn_name(fn_name: str) -> str:
    return f"{fn_name}_typed"


def ctor_arg_type(field_ty: str) -> str:
    if field_ty == "String":
        return "impl Into<String>"
    if field_ty == "ChatId":
        return "impl Into<ChatId>"
    return field_ty


def ctor_assign(field_name: str, field_ty: str) -> str:
    if field_ty in {"String", "ChatId"}:
        return f"{field_name}: {field_name}.into()"
    return field_name


def map_raw_type(type_raw: str) -> str | None:
    base_map: dict[str, str] = {
        "InputSticker": "crate::types::sticker::InputSticker",
        "InputMedia": "crate::types::message::InputMedia",
        "LabeledPrice": "crate::types::payment::LabeledPrice",
        "ShippingOption": "crate::types::payment::ShippingOption",
        "MessageEntity": "crate::types::message::MessageEntity",
        "InlineQueryResult": "crate::types::telegram::InlineQueryResult",
        "InputChecklist": "crate::types::telegram::InputChecklist",
        "InlineKeyboardMarkup": "crate::types::telegram::InlineKeyboardMarkup",
        "InputStoryContent": "crate::types::telegram::InputStoryContent",
        "StoryArea": "crate::types::telegram::StoryArea",
        "ReplyParameters": "crate::types::telegram::ReplyParameters",
        "SuggestedPostParameters": "crate::types::telegram::SuggestedPostParameters",
        "InputPaidMedia": "crate::types::telegram::InputPaidMedia",
        "AcceptedGiftTypes": "crate::types::telegram::AcceptedGiftTypes",
        "MenuButton": "crate::types::telegram::MenuButton",
        "ReactionType": "crate::types::telegram::ReactionType",
        "ChatAdministratorRights": "crate::types::chat::ChatAdministratorRights",
        "PassportElementError": "crate::types::telegram::PassportElementError",
        "MaskPosition": "crate::types::sticker::MaskPosition",
        "InlineKeyboardMarkup or ReplyKeyboardMarkup or ReplyKeyboardRemove or ForceReply": "crate::types::telegram::ReplyMarkup",
    }

    normalized = type_raw.strip()
    if normalized.startswith("Array of "):
        item_raw = normalized.removeprefix("Array of ").strip()
        item_ty = base_map.get(item_raw)
        if item_ty is not None:
            return f"Vec<{item_ty}>"
        return None

    return base_map.get(normalized)


def resolve_param_type(param: dict[str, Any]) -> str:
    current = param["type_rust"]
    if "Value" not in current:
        return current

    mapped = map_raw_type(param["type_raw"])
    if mapped is not None:
        return mapped

    return current


def response_type(method: str, return_desc: str) -> str:
    manual: dict[str, str] = {
        "getUpdates": "Vec<crate::types::update::Update>",
        "getWebhookInfo": "crate::types::webhook::WebhookInfo",
        "getMe": "crate::types::bot::User",
        "sendMessage": "crate::types::message::Message",
        "forwardMessage": "crate::types::message::Message",
        "forwardMessages": "Vec<crate::types::message::MessageIdObject>",
        "copyMessage": "crate::types::message::MessageIdObject",
        "copyMessages": "Vec<crate::types::message::MessageIdObject>",
        "sendPhoto": "crate::types::message::Message",
        "sendAudio": "crate::types::message::Message",
        "sendDocument": "crate::types::message::Message",
        "sendVideo": "crate::types::message::Message",
        "sendAnimation": "crate::types::message::Message",
        "sendVoice": "crate::types::message::Message",
        "sendVideoNote": "crate::types::message::Message",
        "sendPaidMedia": "crate::types::message::Message",
        "sendMediaGroup": "Vec<crate::types::message::Message>",
        "sendLocation": "crate::types::message::Message",
        "sendVenue": "crate::types::message::Message",
        "sendContact": "crate::types::message::Message",
        "sendPoll": "crate::types::message::Message",
        "sendChecklist": "crate::types::message::Message",
        "sendDice": "crate::types::message::Message",
        "getUserProfilePhotos": "crate::types::bot::UserProfilePhotos",
        "getFile": "crate::types::file::File",
        "exportChatInviteLink": "String",
        "createChatInviteLink": "crate::types::chat::ChatInviteLink",
        "editChatInviteLink": "crate::types::chat::ChatInviteLink",
        "createChatSubscriptionInviteLink": "crate::types::chat::ChatInviteLink",
        "editChatSubscriptionInviteLink": "crate::types::chat::ChatInviteLink",
        "revokeChatInviteLink": "crate::types::chat::ChatInviteLink",
        "getChatAdministrators": "Vec<crate::types::chat::ChatMember>",
        "getChatMemberCount": "u64",
        "getChatMember": "crate::types::chat::ChatMember",
        "getForumTopicIconStickers": "Vec<crate::types::sticker::Sticker>",
        "getChatMenuButton": "crate::types::telegram::MenuButton",
        "getMyCommands": "Vec<crate::types::command::BotCommand>",
        "getMyName": "crate::types::command::BotName",
        "getMyDescription": "crate::types::command::BotDescription",
        "getMyShortDescription": "crate::types::command::BotShortDescription",
        "getMyDefaultAdministratorRights": "crate::types::chat::ChatAdministratorRights",
        "editMessageText": "crate::types::message::EditMessageResult",
        "editMessageCaption": "crate::types::message::EditMessageResult",
        "editMessageMedia": "crate::types::message::EditMessageResult",
        "editMessageLiveLocation": "crate::types::message::EditMessageResult",
        "stopMessageLiveLocation": "crate::types::message::EditMessageResult",
        "editMessageChecklist": "crate::types::message::Message",
        "editMessageReplyMarkup": "crate::types::message::EditMessageResult",
        "stopPoll": "crate::types::message::Poll",
        "sendSticker": "crate::types::message::Message",
        "answerWebAppQuery": "crate::types::message::SentWebAppMessage",
        "getStickerSet": "crate::types::sticker::StickerSet",
        "getCustomEmojiStickers": "Vec<crate::types::sticker::Sticker>",
        "uploadStickerFile": "crate::types::file::File",
        "sendInvoice": "crate::types::message::Message",
        "createInvoiceLink": "String",
    }
    if method in manual:
        return manual[method]

    desc = return_desc or ""
    if "True" in desc:
        return "bool"
    if "String" in desc:
        return "String"
    if "Array of" in desc and "Sticker" in desc:
        return "Vec<crate::types::sticker::Sticker>"
    if "Array of" in desc and "MessageId" in desc:
        return "Vec<crate::types::message::MessageIdObject>"
    if "MessageId" in desc:
        return "crate::types::message::MessageIdObject"
    if "Array of" in desc and "Message" in desc:
        return "Vec<crate::types::message::Message>"
    if "Message" in desc:
        return "crate::types::message::Message"
    if "Int" in desc:
        return "u64"
    return "Value"


def generate_types(spec: dict[str, Any]) -> str:
    methods: list[dict[str, Any]] = spec["methods"]
    out: list[str] = []
    out.append("use serde::de::DeserializeOwned;")
    out.append("use serde::Serialize;")
    out.append("use serde_json::Value;")
    out.append("")
    out.append("use crate::types::common::{ChatId, MessageId, UserId};")
    out.append("")
    out.append("/// Typed request marker for advanced API methods.")
    out.append("pub trait AdvancedRequest: Serialize {")
    out.append("    type Response: DeserializeOwned;")
    out.append("    const METHOD: &'static str;")
    out.append("}")
    out.append("")

    for method in methods:
        fn_name = method["fn_name"]
        method_name = method["method"]
        params: list[dict[str, Any]] = method["params"]
        req_name = request_type_name(fn_name)
        req_params = [p for p in params if p["required"]]
        derive = "#[derive(Clone, Debug, Serialize)]"
        if not req_params:
            derive = "#[derive(Clone, Debug, Default, Serialize)]"

        out.append(f"/// Auto-generated request for `{method_name}`.")
        out.append(derive)
        out.append(f"pub struct {req_name} {{")
        if params:
            for p in params:
                original_name = p["name"]
                field_name = p["field_name"]
                cleaned = field_name.removeprefix("r#")
                field_ty = resolve_param_type(p)
                if cleaned != original_name:
                    out.append(f"    #[serde(rename = \"{original_name}\")]")
                if p["required"]:
                    out.append(f"    pub {field_name}: {field_ty},")
                else:
                    out.append("    #[serde(default, skip_serializing_if = \"Option::is_none\")]")
                    out.append(f"    pub {field_name}: Option<{field_ty}>,")
        out.append("}")
        out.append("")

        out.append(f"impl {req_name} {{")
        if not params:
            out.append("    pub fn new() -> Self {")
            out.append("        Self {}")
            out.append("    }")
        elif req_params:
            args = ", ".join(
                f"{p['field_name']}: {ctor_arg_type(resolve_param_type(p))}" for p in req_params
            )
            out.append(f"    pub fn new({args}) -> Self {{")
            out.append("        Self {")
            req_names = {p["field_name"] for p in req_params}
            for p in params:
                field_name = p["field_name"]
                field_ty = resolve_param_type(p)
                if field_name in req_names:
                    out.append(f"            {ctor_assign(field_name, field_ty)},")
                else:
                    out.append(f"            {field_name}: None,")
            out.append("        }")
            out.append("    }")
        else:
            out.append("    pub fn new() -> Self {")
            out.append("        Self::default()")
            out.append("    }")
        out.append("}")
        out.append("")

        ret_ty = response_type(method_name, method.get("return_desc", ""))
        out.append(f"impl AdvancedRequest for {req_name} {{")
        out.append(f"    type Response = {ret_ty};")
        out.append(f"    const METHOD: &'static str = \"{method_name}\";")
        out.append("}")
        out.append("")

    return "\n".join(out).rstrip() + "\n"


def generate_api(spec: dict[str, Any]) -> str:
    methods: list[dict[str, Any]] = spec["methods"]

    entries = [
        (
            method["fn_name"],
            typed_fn_name(method["fn_name"]),
            method["method"],
            request_type_name(method["fn_name"]),
        )
        for method in methods
    ]

    out: list[str] = []
    out.append("use serde::de::DeserializeOwned;")
    out.append("")
    out.append("use crate::Result;")
    out.append("use crate::types::advanced::*;")
    out.append("")
    out.append("#[cfg(feature = \"_blocking\")]")
    out.append("use crate::BlockingClient;")
    out.append("#[cfg(feature = \"_async\")]")
    out.append("use crate::Client;")
    out.append("")
    out.append("#[cfg(feature = \"_async\")]")
    out.append("macro_rules! define_async_methods {")
    out.append("    ($(($fn_name:ident, $typed_name:ident, $method:literal, $request_ty:ty)),* $(,)?) => {")
    out.append("        $(")
    out.append("            pub async fn $fn_name<R>(&self, request: &$request_ty) -> Result<R>")
    out.append("            where")
    out.append("                R: DeserializeOwned,")
    out.append("            {")
    out.append("                self.client.call_method($method, request).await")
    out.append("            }")
    out.append("")
    out.append("            pub async fn $typed_name(")
    out.append("                &self,")
    out.append("                request: &$request_ty,")
    out.append("            ) -> Result<<$request_ty as AdvancedRequest>::Response> {")
    out.append("                self.call_typed(request).await")
    out.append("            }")
    out.append("        )*")
    out.append("    };")
    out.append("}")
    out.append("")
    out.append("#[cfg(feature = \"_blocking\")]")
    out.append("macro_rules! define_blocking_methods {")
    out.append("    ($(($fn_name:ident, $typed_name:ident, $method:literal, $request_ty:ty)),* $(,)?) => {")
    out.append("        $(")
    out.append("            pub fn $fn_name<R>(&self, request: &$request_ty) -> Result<R>")
    out.append("            where")
    out.append("                R: DeserializeOwned,")
    out.append("            {")
    out.append("                self.client.call_method($method, request)")
    out.append("            }")
    out.append("")
    out.append("            pub fn $typed_name(")
    out.append("                &self,")
    out.append("                request: &$request_ty,")
    out.append("            ) -> Result<<$request_ty as AdvancedRequest>::Response> {")
    out.append("                self.call_typed(request)")
    out.append("            }")
    out.append("        )*")
    out.append("    };")
    out.append("}")
    out.append("")
    out.append("/// Additional Telegram Bot API methods with typed request models.")
    out.append("#[cfg(feature = \"_async\")]")
    out.append("#[derive(Clone)]")
    out.append("pub struct AdvancedService {")
    out.append("    client: Client,")
    out.append("}")
    out.append("")
    out.append("#[cfg(feature = \"_async\")]")
    out.append("impl AdvancedService {")
    out.append("    pub(crate) fn new(client: Client) -> Self {")
    out.append("        Self { client }")
    out.append("    }")
    out.append("")
    out.append("    /// Calls advanced methods using request-associated response type.")
    out.append("    pub async fn call_typed<Q>(&self, request: &Q) -> Result<Q::Response>")
    out.append("    where")
    out.append("        Q: AdvancedRequest,")
    out.append("    {")
    out.append("        self.client.call_method(Q::METHOD, request).await")
    out.append("    }")
    out.append("")
    out.append("    define_async_methods! {")
    for fn_name, typed_name, method_name, req_name in entries:
        out.append(f"        ({fn_name}, {typed_name}, \"{method_name}\", {req_name}),")
    out.append("    }")
    out.append("}")
    out.append("")
    out.append("/// Blocking additional Telegram Bot API methods with typed request models.")
    out.append("#[cfg(feature = \"_blocking\")]")
    out.append("#[derive(Clone)]")
    out.append("pub struct BlockingAdvancedService {")
    out.append("    client: BlockingClient,")
    out.append("}")
    out.append("")
    out.append("#[cfg(feature = \"_blocking\")]")
    out.append("impl BlockingAdvancedService {")
    out.append("    pub(crate) fn new(client: BlockingClient) -> Self {")
    out.append("        Self { client }")
    out.append("    }")
    out.append("")
    out.append("    /// Calls advanced methods using request-associated response type.")
    out.append("    pub fn call_typed<Q>(&self, request: &Q) -> Result<Q::Response>")
    out.append("    where")
    out.append("        Q: AdvancedRequest,")
    out.append("    {")
    out.append("        self.client.call_method(Q::METHOD, request)")
    out.append("    }")
    out.append("")
    out.append("    define_blocking_methods! {")
    for fn_name, typed_name, method_name, req_name in entries:
        out.append(f"        ({fn_name}, {typed_name}, \"{method_name}\", {req_name}),")
    out.append("    }")
    out.append("}")
    out.append("")

    return "\n".join(out)


def main() -> None:
    spec_path = resolve_spec_path()
    spec = json.loads(spec_path.read_text())
    TYPES_OUT.write_text(generate_types(spec))
    API_OUT.write_text(generate_api(spec))


if __name__ == "__main__":
    main()
