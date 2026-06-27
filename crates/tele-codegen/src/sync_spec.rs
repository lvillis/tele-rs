use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use regex::Regex;
use reqwest::blocking::Client;
use scraper::{ElementRef, Html, Selector};

use crate::fs_util::write_text_if_changed;
use crate::spec::{BotApiSpec, MethodSpec, ParamSpec};

const DEFAULT_BOT_API_URL: &str = "https://core.telegram.org/bots/api";
const SYNC_REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

pub(crate) fn sync(source_url: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .ok_or("failed to resolve workspace root")?;
    let spec_path = manifest_dir.join("spec").join("bot_api.json");
    let fixture_path = workspace_root.join("crates/tele/tests/fixtures/bot_api_all_methods.txt");
    let source_url = source_url
        .or_else(|| std::env::var("TELE_BOT_API_SOURCE_URL").ok())
        .unwrap_or_else(|| DEFAULT_BOT_API_URL.to_owned());

    let existing_bytes = fs::read(&spec_path).map_err(|error| {
        format!(
            "failed to read existing Bot API spec at {}: {error}",
            spec_path.display()
        )
    })?;
    let existing_spec = serde_json::from_slice::<BotApiSpec>(&existing_bytes)?;
    existing_spec
        .validate()
        .map_err(|error| -> Box<dyn std::error::Error> { error.into() })?;

    let html = Client::builder()
        .user_agent("tele-codegen/0")
        .timeout(SYNC_REQUEST_TIMEOUT)
        .build()?
        .get(&source_url)
        .send()?
        .error_for_status()?
        .text()?;

    let synced = build_synced_spec(&html, &source_url, &existing_spec)?;
    write_text_if_changed(&spec_path, &(serde_json::to_string_pretty(&synced)? + "\n"))?;
    write_text_if_changed(&fixture_path, &(synced.all_methods.join("\n") + "\n"))?;
    Ok(())
}

fn build_synced_spec(
    html: &str,
    generated_from: &str,
    existing_spec: &BotApiSpec,
) -> Result<BotApiSpec, Box<dyn std::error::Error>> {
    let version = parse_version(html)?;
    let official_methods = parse_official_methods(html)?;
    let all_methods = official_methods
        .iter()
        .map(|method| method.method.clone())
        .collect::<Vec<_>>();
    let parsed_methods = all_methods
        .iter()
        .map(String::as_str)
        .collect::<HashSet<_>>();
    let missing_existing_methods = existing_spec
        .all_methods
        .iter()
        .filter(|method| !parsed_methods.contains(method.as_str()))
        .collect::<Vec<_>>();
    if !missing_existing_methods.is_empty() {
        return Err(format!(
            "refusing to sync Bot API spec because existing methods disappeared from parsed docs: {}",
            missing_existing_methods
                .iter()
                .map(|method| method.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        )
        .into());
    }
    if all_methods.len() < existing_spec.all_methods.len() {
        return Err(format!(
            "refusing to sync Bot API spec because parsed method count shrank from {} to {}",
            existing_spec.all_methods.len(),
            all_methods.len()
        )
        .into());
    }

    let existing_advanced = existing_spec
        .advanced_methods
        .iter()
        .map(|method| method.method.as_str())
        .collect::<HashSet<_>>();
    let known_methods = existing_spec
        .all_methods
        .iter()
        .map(String::as_str)
        .collect::<HashSet<_>>();

    let advanced_methods = official_methods
        .into_iter()
        .filter(|method| {
            existing_advanced.contains(method.method.as_str())
                || !known_methods.contains(method.method.as_str())
        })
        .collect::<Vec<_>>();

    let spec = BotApiSpec {
        version,
        generated_from: generated_from.to_owned(),
        all_methods,
        advanced_methods,
    };
    spec.validate()
        .map_err(|error| -> Box<dyn std::error::Error> { error.into() })?;
    Ok(spec)
}

fn parse_version(html: &str) -> Result<String, Box<dyn std::error::Error>> {
    let version = Regex::new(r"Bot API ([0-9]+\.[0-9]+)")?
        .captures(html)
        .and_then(|captures| captures.get(1))
        .map(|matched| format!("Bot API {}", matched.as_str()))
        .ok_or("failed to find Bot API version in official docs")?;
    Ok(version)
}

fn parse_official_methods(html: &str) -> Result<Vec<MethodSpec>, Box<dyn std::error::Error>> {
    fn parse_selector(value: &str) -> Result<Selector, Box<dyn std::error::Error>> {
        Selector::parse(value)
            .map_err(|error| format!("failed to parse selector `{value}`: {error:?}").into())
    }

    let document = Html::parse_document(html);
    let heading_selector = parse_selector("h4")?;
    let paragraph_selector = parse_selector("p")?;
    let table_selector = parse_selector("table")?;
    let header_selector = parse_selector("th")?;
    let row_selector = parse_selector("tbody tr")?;
    let cell_selector = parse_selector("td")?;

    let mut methods = Vec::new();

    for heading_element in document.select(&heading_selector) {
        let heading = normalize_ws(&heading_element.text().collect::<String>());
        if !looks_like_method_name(&heading) {
            continue;
        }

        let section = collect_method_section(heading_element);
        let Some(first_paragraph) = section.iter().find_map(|element| {
            if element.value().name() == "p" {
                Some(*element)
            } else {
                element.select(&paragraph_selector).next()
            }
        }) else {
            return Err(format!("method `{heading}` is missing a summary paragraph").into());
        };
        let summary = normalize_ws(&first_paragraph.text().collect::<String>());

        let params = section
            .iter()
            .flat_map(|element| {
                let current = (element.value().name() == "table").then_some(*element);
                current.into_iter().chain(element.select(&table_selector))
            })
            .find(|table| {
                let headers = table
                    .select(&header_selector)
                    .map(|cell| normalize_ws(&cell.text().collect::<String>()))
                    .collect::<Vec<_>>();
                headers
                    == [
                        "Parameter".to_owned(),
                        "Type".to_owned(),
                        "Required".to_owned(),
                        "Description".to_owned(),
                    ]
            })
            .map(|table| parse_parameter_rows(&heading, table, &row_selector, &cell_selector))
            .transpose()?
            .unwrap_or_default();

        methods.push(MethodSpec {
            fn_name: to_snake_case(&heading),
            method: heading.clone(),
            return_desc: extract_return_desc(&summary),
            params,
        });
    }

    if methods.is_empty() {
        return Err("failed to parse any Bot API methods from official docs".into());
    }

    Ok(methods)
}

fn parse_parameter_rows(
    method: &str,
    table: ElementRef<'_>,
    row_selector: &Selector,
    cell_selector: &Selector,
) -> Result<Vec<ParamSpec>, Box<dyn std::error::Error>> {
    let mut params = Vec::new();

    for row in table.select(row_selector) {
        let cells = row
            .select(cell_selector)
            .map(|cell| normalize_ws(&cell.text().collect::<String>()))
            .collect::<Vec<_>>();
        if cells.len() != 4 {
            return Err(format!(
                "method `{method}` has malformed parameter row with {} cells",
                cells.len()
            )
            .into());
        }

        let required = parse_required_cell(method, &cells[0], &cells[2])?;
        params.push(ParamSpec {
            name: cells[0].clone(),
            field_name: rust_field_name(&cells[0]),
            required,
            type_raw: cells[1].clone(),
            type_rust: infer_type_rust(&cells[0], &cells[1]),
        });
    }

    Ok(params)
}

fn parse_required_cell(
    method: &str,
    param: &str,
    value: &str,
) -> Result<bool, Box<dyn std::error::Error>> {
    match value {
        "Yes" => Ok(true),
        "Optional" => Ok(false),
        _ => Err(format!(
            "method `{method}` parameter `{param}` has invalid Required value `{value}`"
        )
        .into()),
    }
}

fn collect_method_section<'a>(heading: ElementRef<'a>) -> Vec<ElementRef<'a>> {
    let mut section = Vec::new();
    let mut sibling = heading.next_sibling();

    while let Some(node) = sibling {
        sibling = node.next_sibling();
        let Some(element) = ElementRef::wrap(node) else {
            continue;
        };
        if element.value().name() == "h4" {
            break;
        }
        section.push(element);
    }

    section
}

fn looks_like_method_name(value: &str) -> bool {
    let mut chars = value.chars();
    matches!(chars.next(), Some(first) if first.is_ascii_lowercase())
        && value.chars().all(|ch| ch.is_ascii_alphanumeric())
}

fn normalize_ws(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn extract_return_desc(summary: &str) -> String {
    for needle in ["On success, returns ", "Returns "] {
        if let Some(start) = summary.find(needle) {
            let tail = &summary[start + needle.len()..];
            let end = tail.find('.').unwrap_or(tail.len());
            return tail[..end].trim().to_owned();
        }
    }
    String::new()
}

fn rust_field_name(name: &str) -> String {
    if is_rust_keyword(name) {
        return format!("{name}_");
    }
    name.to_owned()
}

fn is_rust_keyword(name: &str) -> bool {
    matches!(
        name,
        "Self"
            | "abstract"
            | "as"
            | "async"
            | "await"
            | "become"
            | "box"
            | "break"
            | "const"
            | "continue"
            | "crate"
            | "do"
            | "dyn"
            | "else"
            | "enum"
            | "extern"
            | "false"
            | "final"
            | "fn"
            | "for"
            | "gen"
            | "if"
            | "impl"
            | "in"
            | "let"
            | "loop"
            | "macro"
            | "match"
            | "mod"
            | "move"
            | "mut"
            | "override"
            | "priv"
            | "pub"
            | "ref"
            | "return"
            | "self"
            | "static"
            | "struct"
            | "super"
            | "trait"
            | "true"
            | "try"
            | "type"
            | "typeof"
            | "union"
            | "unsafe"
            | "unsized"
            | "use"
            | "virtual"
            | "where"
            | "while"
            | "yield"
    )
}

fn infer_type_rust(name: &str, type_raw: &str) -> String {
    let normalized = normalize_ws(type_raw);
    match normalized.as_str() {
        "Boolean" => "bool".to_owned(),
        "Float" => "f64".to_owned(),
        "String" | "InputFile or String" | "String or InputFile" => "String".to_owned(),
        "Integer" => {
            if is_user_id(name) {
                "UserId".to_owned()
            } else if is_message_id(name) {
                "MessageId".to_owned()
            } else {
                "i64".to_owned()
            }
        }
        "Integer or String" | "String or Integer" => {
            if is_chat_id(name) {
                "ChatId".to_owned()
            } else {
                "String".to_owned()
            }
        }
        "Array of Integer" => {
            if name.ends_with("message_ids") {
                "Vec<MessageId>".to_owned()
            } else {
                "Vec<i64>".to_owned()
            }
        }
        "Array of String" => "Vec<String>".to_owned(),
        _ => "Value".to_owned(),
    }
}

fn is_user_id(name: &str) -> bool {
    name == "user_id" || name.ends_with("_user_id")
}

fn is_chat_id(name: &str) -> bool {
    name == "chat_id" || name.ends_with("_chat_id")
}

fn is_message_id(name: &str) -> bool {
    name == "message_id" || name.ends_with("_message_id")
}

fn to_snake_case(value: &str) -> String {
    let mut out = String::new();
    let chars = value.chars().collect::<Vec<_>>();
    for (index, ch) in chars.iter().enumerate() {
        if ch.is_uppercase() {
            if index > 0 {
                let prev = chars[index - 1];
                let next = chars.get(index + 1).copied();
                if prev.is_ascii_lowercase() || next.is_some_and(|next| next.is_ascii_lowercase()) {
                    out.push('_');
                }
            }
            out.extend(ch.to_lowercase());
        } else {
            out.push(*ch);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    type TestResult = Result<(), Box<dyn std::error::Error>>;

    #[test]
    fn extracts_methods_from_official_like_fragment() -> TestResult {
        let html = r##"
            <h4><a class="anchor" name="getme" href="#getme"><i class="anchor-icon"></i></a>getMe</h4>
            <p>Use this method to get basic information about the bot. Returns a User object.</p>
            <table class="table">
              <thead><tr><th>Parameter</th><th>Type</th><th>Required</th><th>Description</th></tr></thead>
              <tbody></tbody>
            </table>
            <h4><a class="anchor" name="getavailablegifts" href="#getavailablegifts"><i class="anchor-icon"></i></a>getAvailableGifts</h4>
            <p>Returns the list of gifts that can be sent by the bot to users and channel chats. Requires no parameters. Returns a Gifts object.</p>
            <h4><a class="anchor" name="user" href="#user"><i class="anchor-icon"></i></a>User</h4>
            <p>This object represents a Telegram user or bot.</p>
        "##;

        let methods = parse_official_methods(html)?;
        assert_eq!(methods.len(), 2);
        assert_eq!(methods[0].method, "getMe");
        assert_eq!(methods[0].fn_name, "get_me");
        assert_eq!(methods[0].return_desc, "a User object");
        assert_eq!(methods[1].method, "getAvailableGifts");
        assert_eq!(methods[1].fn_name, "get_available_gifts");
        assert_eq!(
            methods[1].return_desc,
            "the list of gifts that can be sent by the bot to users and channel chats"
        );
        Ok(())
    }

    #[test]
    fn extracts_methods_from_flexible_heading_markup() -> TestResult {
        let html = r##"
            <h4 id="getme">
              <a href="#getme" name="getme" class="anchor">
                <i class="anchor-icon"></i>
              </a>
              getMe
            </h4>
            <p>Use this method to get basic information about the bot. Returns a User object.</p>
            <table class="table">
              <thead>
                <tr>
                  <th>Parameter</th>
                  <th>Type</th>
                  <th>Required</th>
                  <th>Description</th>
                </tr>
              </thead>
              <tbody>
                <tr>
                  <td>user_id</td>
                  <td>Integer</td>
                  <td>Yes</td>
                  <td>User identifier</td>
                </tr>
              </tbody>
            </table>
        "##;

        let methods = parse_official_methods(html)?;

        assert_eq!(methods.len(), 1);
        assert_eq!(methods[0].method, "getMe");
        assert_eq!(methods[0].params.len(), 1);
        assert_eq!(methods[0].params[0].type_rust, "UserId");
        Ok(())
    }

    #[test]
    fn rejects_malformed_parameter_rows() -> TestResult {
        let html = r##"
            <h4><a class="anchor" name="sendmessage" href="#sendmessage"><i class="anchor-icon"></i></a>sendMessage</h4>
            <p>Use this method to send text messages. On success, returns a Message object.</p>
            <table class="table">
              <thead><tr><th>Parameter</th><th>Type</th><th>Required</th><th>Description</th></tr></thead>
              <tbody>
                <tr><td>chat_id</td><td>Integer or String</td><td>Yes</td></tr>
              </tbody>
            </table>
        "##;

        let error = match parse_official_methods(html) {
            Ok(methods) => {
                return Err(
                    format!("expected malformed rows to fail sync, got {methods:?}").into(),
                );
            }
            Err(error) => error,
        };

        assert!(error.to_string().contains("malformed parameter row"));
        Ok(())
    }

    #[test]
    fn rejects_unknown_required_marker() -> TestResult {
        let html = r##"
            <h4><a class="anchor" name="sendmessage" href="#sendmessage"><i class="anchor-icon"></i></a>sendMessage</h4>
            <p>Use this method to send text messages. On success, returns a Message object.</p>
            <table class="table">
              <thead><tr><th>Parameter</th><th>Type</th><th>Required</th><th>Description</th></tr></thead>
              <tbody>
                <tr><td>chat_id</td><td>Integer or String</td><td>Maybe</td><td>Target chat</td></tr>
              </tbody>
            </table>
        "##;

        let error = match parse_official_methods(html) {
            Ok(methods) => {
                return Err(format!(
                    "expected unknown Required marker to fail sync, got {methods:?}"
                )
                .into());
            }
            Err(error) => error,
        };

        assert!(error.to_string().contains("invalid Required value"));
        Ok(())
    }

    #[test]
    fn infers_common_parameter_types() {
        assert_eq!(infer_type_rust("chat_id", "Integer or String"), "ChatId");
        assert_eq!(infer_type_rust("user_id", "Integer"), "UserId");
        assert_eq!(infer_type_rust("message_id", "Integer"), "MessageId");
        assert_eq!(
            infer_type_rust("message_ids", "Array of Integer"),
            "Vec<MessageId>"
        );
        assert_eq!(infer_type_rust("payload", "InlineKeyboardMarkup"), "Value");
    }

    #[test]
    fn rust_field_names_avoid_edition_keywords() {
        assert_eq!(rust_field_name("type"), "type_");
        assert_eq!(rust_field_name("self"), "self_");
        assert_eq!(rust_field_name("crate"), "crate_");
        assert_eq!(rust_field_name("gen"), "gen_");
        assert_eq!(rust_field_name("chat_id"), "chat_id");
    }

    #[test]
    fn sync_preserves_existing_advanced_methods_and_adopts_new_methods() -> TestResult {
        let existing_spec = BotApiSpec {
            version: "Bot API 9.4".to_owned(),
            generated_from: "https://core.telegram.org/bots/api".to_owned(),
            all_methods: vec!["getMe".to_owned(), "sendMessage".to_owned()],
            advanced_methods: vec![MethodSpec {
                fn_name: "send_message".to_owned(),
                method: "sendMessage".to_owned(),
                return_desc: "a Message object".to_owned(),
                params: vec![],
            }],
        };
        let html = r##"
            <p>Bot API 9.5</p>
            <h4><a class="anchor" name="getme" href="#getme"><i class="anchor-icon"></i></a>getMe</h4>
            <p>Use this method to get basic information about the bot. Returns a User object.</p>
            <h4><a class="anchor" name="sendmessage" href="#sendmessage"><i class="anchor-icon"></i></a>sendMessage</h4>
            <p>Use this method to send text messages. On success, returns a Message object.</p>
            <table class="table">
              <thead><tr><th>Parameter</th><th>Type</th><th>Required</th><th>Description</th></tr></thead>
              <tbody></tbody>
            </table>
            <h4><a class="anchor" name="sendpaidmedia" href="#sendpaidmedia"><i class="anchor-icon"></i></a>sendPaidMedia</h4>
            <p>Use this method to send paid media. On success, returns a Message object.</p>
            <table class="table">
              <thead><tr><th>Parameter</th><th>Type</th><th>Required</th><th>Description</th></tr></thead>
              <tbody></tbody>
            </table>
        "##;

        let synced = build_synced_spec(html, "https://core.telegram.org/bots/api", &existing_spec)?;

        assert_eq!(
            synced.all_methods,
            vec!["getMe", "sendMessage", "sendPaidMedia"]
        );
        assert_eq!(
            synced
                .advanced_methods
                .iter()
                .map(|method| method.method.as_str())
                .collect::<Vec<_>>(),
            vec!["sendMessage", "sendPaidMedia"]
        );
        Ok(())
    }

    #[test]
    fn sync_rejects_method_count_shrink() {
        let existing_spec = BotApiSpec {
            version: "Bot API 9.4".to_owned(),
            generated_from: "https://core.telegram.org/bots/api".to_owned(),
            all_methods: vec!["getMe".to_owned(), "sendMessage".to_owned()],
            advanced_methods: vec![MethodSpec {
                fn_name: "send_message".to_owned(),
                method: "sendMessage".to_owned(),
                return_desc: "a Message object".to_owned(),
                params: vec![],
            }],
        };
        let html = r##"
            <p>Bot API 9.5</p>
            <h4><a class="anchor" name="getme" href="#getme"><i class="anchor-icon"></i></a>getMe</h4>
            <p>Use this method to get basic information about the bot. Returns a User object.</p>
        "##;

        assert!(
            build_synced_spec(html, "https://core.telegram.org/bots/api", &existing_spec).is_err()
        );
    }

    #[test]
    fn sync_rejects_missing_existing_methods_even_when_count_does_not_shrink() -> TestResult {
        let existing_spec = BotApiSpec {
            version: "Bot API 9.4".to_owned(),
            generated_from: "https://core.telegram.org/bots/api".to_owned(),
            all_methods: vec!["getMe".to_owned(), "sendMessage".to_owned()],
            advanced_methods: vec![MethodSpec {
                fn_name: "send_message".to_owned(),
                method: "sendMessage".to_owned(),
                return_desc: "a Message object".to_owned(),
                params: vec![],
            }],
        };
        let html = r##"
            <p>Bot API 9.5</p>
            <h4><a class="anchor" name="getme" href="#getme"><i class="anchor-icon"></i></a>getMe</h4>
            <p>Use this method to get basic information about the bot. Returns a User object.</p>
            <h4><a class="anchor" name="sendpaidmedia" href="#sendpaidmedia"><i class="anchor-icon"></i></a>sendPaidMedia</h4>
            <p>Use this method to send paid media. On success, returns a Message object.</p>
        "##;

        let result = build_synced_spec(html, "https://core.telegram.org/bots/api", &existing_spec);
        assert!(result.is_err());
        let error = match result {
            Ok(spec) => return Err(format!("expected sync to fail, got {spec:?}").into()),
            Err(error) => error,
        };
        assert!(error.to_string().contains("sendMessage"));
        Ok(())
    }
}
