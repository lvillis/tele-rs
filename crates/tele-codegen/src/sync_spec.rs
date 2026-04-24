use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use regex::Regex;
use reqwest::blocking::Client;
use scraper::{Html, Selector};

use crate::spec::{BotApiSpec, MethodSpec, ParamSpec};

const DEFAULT_BOT_API_URL: &str = "https://core.telegram.org/bots/api";

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
        .build()?
        .get(&source_url)
        .send()?
        .error_for_status()?
        .text()?;

    let synced = build_synced_spec(&html, &source_url, &existing_spec)?;
    fs::write(&spec_path, serde_json::to_string_pretty(&synced)? + "\n")?;
    fs::write(&fixture_path, synced.all_methods.join("\n") + "\n")?;
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

    let heading_re = Regex::new(
        r##"<h4><a class="anchor" name="[^"]+" href="#[^"]+"><i class="anchor-icon"></i></a>([^<]+)</h4>"##,
    )?;
    let paragraph_selector = parse_selector("p")?;
    let table_selector = parse_selector("table")?;
    let header_selector = parse_selector("th")?;
    let row_selector = parse_selector("tbody tr")?;
    let cell_selector = parse_selector("td")?;

    let mut methods = Vec::new();
    let headings = heading_re
        .captures_iter(html)
        .filter_map(|captures| {
            let whole = captures.get(0)?;
            let heading = normalize_ws(captures.get(1)?.as_str());
            Some((whole.start(), whole.end(), heading))
        })
        .collect::<Vec<_>>();

    for (index, (_, section_start, heading)) in headings.iter().enumerate() {
        if !looks_like_method_name(heading) {
            continue;
        }

        let section_end = headings
            .get(index + 1)
            .map(|(next_start, _, _)| *next_start)
            .unwrap_or(html.len());
        let section_html = &html[*section_start..section_end];
        let fragment = Html::parse_fragment(section_html);
        let Some(first_paragraph) = fragment.select(&paragraph_selector).next() else {
            continue;
        };
        let summary = text_content(first_paragraph.html().as_str());

        let params = fragment
            .select(&table_selector)
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
            .map(|table| {
                table
                    .select(&row_selector)
                    .filter_map(|row| {
                        let cells = row
                            .select(&cell_selector)
                            .map(|cell| normalize_ws(&cell.text().collect::<String>()))
                            .collect::<Vec<_>>();
                        (cells.len() == 4).then(|| ParamSpec {
                            name: cells[0].clone(),
                            field_name: rust_field_name(&cells[0]),
                            required: cells[2] == "Yes",
                            type_raw: cells[1].clone(),
                            type_rust: infer_type_rust(&cells[0], &cells[1]),
                        })
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        methods.push(MethodSpec {
            fn_name: to_snake_case(heading),
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

fn looks_like_method_name(value: &str) -> bool {
    let mut chars = value.chars();
    matches!(chars.next(), Some(first) if first.is_ascii_lowercase())
        && value.chars().all(|ch| ch.is_ascii_alphanumeric())
}

fn text_content(html_fragment: &str) -> String {
    let fragment = Html::parse_fragment(html_fragment);
    normalize_ws(&fragment.root_element().text().collect::<String>())
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
    if matches!(
        name,
        "type"
            | "match"
            | "loop"
            | "move"
            | "ref"
            | "self"
            | "Self"
            | "super"
            | "crate"
            | "where"
            | "async"
            | "await"
            | "dyn"
            | "use"
            | "mod"
            | "trait"
            | "impl"
            | "fn"
            | "struct"
            | "enum"
    ) {
        return format!("r#{name}");
    }
    name.to_owned()
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

    #[test]
    fn extracts_methods_from_official_like_fragment() {
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

        let methods = parse_official_methods(html);
        assert!(methods.is_ok());
        let methods = match methods {
            Ok(methods) => methods,
            Err(_) => return,
        };
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
    fn sync_preserves_existing_advanced_methods_and_adopts_new_methods() {
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

        let synced = build_synced_spec(html, "https://core.telegram.org/bots/api", &existing_spec);
        assert!(synced.is_ok());
        let synced = match synced {
            Ok(synced) => synced,
            Err(_) => return,
        };

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
    }
}
