use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

const EMBEDDED_METHOD_SPEC: &str = include_str!("fixtures/bot_api_all_methods.txt");

#[derive(Debug, Deserialize)]
struct MethodCoverageSpec {
    all_methods: Vec<String>,
}

fn parse_expected_methods(
    text: &str,
    path: &Path,
) -> Result<Option<BTreeSet<String>>, Box<dyn std::error::Error>> {
    if path.extension().is_some_and(|ext| ext == "json") {
        let spec: MethodCoverageSpec = serde_json::from_str(text).map_err(|source| {
            format!(
                "failed to parse method coverage spec `{}`: {source}",
                path.display()
            )
        })?;
        if spec.all_methods.is_empty() {
            return Err(format!(
                "method coverage spec `{}` must contain at least one method",
                path.display()
            )
            .into());
        }
        return Ok(Some(spec.all_methods.into_iter().collect()));
    }

    let methods = text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(ToOwned::to_owned)
        .collect::<BTreeSet<_>>();

    Ok((!methods.is_empty()).then_some(methods))
}

fn load_expected_methods(
    path: &Path,
) -> Result<Option<BTreeSet<String>>, Box<dyn std::error::Error>> {
    match fs::read_to_string(path) {
        Ok(text) => parse_expected_methods(&text, path),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(format!(
            "failed to read method coverage spec `{}`: {error}",
            path.display()
        )
        .into()),
    }
}

fn load_required_expected_methods(
    path: &Path,
) -> Result<BTreeSet<String>, Box<dyn std::error::Error>> {
    load_expected_methods(path)?.ok_or_else(|| {
        format!(
            "method coverage spec `{}` must exist and contain at least one method",
            path.display()
        )
        .into()
    })
}

fn collect_rust_files(dir: &Path, out: &mut Vec<PathBuf>) -> std::io::Result<()> {
    let entries = fs::read_dir(dir)?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_rust_files(&path, out)?;
            continue;
        }
        if path.extension().is_some_and(|ext| ext == "rs") {
            out.push(path);
        }
    }

    Ok(())
}

#[test]
fn telegram_bot_api_methods_are_fully_covered() -> Result<(), Box<dyn std::error::Error>> {
    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = crate_root
        .parent()
        .and_then(|path| path.parent())
        .map(Path::to_path_buf)
        .unwrap_or_else(|| crate_root.clone());

    let default_spec_path = workspace_root.join("crates/tele-codegen/spec/bot_api.json");
    let expected_methods = match env::var("TELE_METHOD_COVERAGE_SPEC_PATH") {
        Ok(path) => Some(load_required_expected_methods(&PathBuf::from(path))?),
        Err(_) => load_expected_methods(&default_spec_path)?,
    };

    let expected_methods = match expected_methods {
        Some(methods) => methods,
        None => {
            assert!(
                !EMBEDDED_METHOD_SPEC.trim().is_empty(),
                "embedded method spec fixture is empty"
            );
            let Some(methods) =
                parse_expected_methods(EMBEDDED_METHOD_SPEC, Path::new("bot_api_all_methods.txt"))?
            else {
                return Err(std::io::Error::other(
                    "embedded method spec fixture must contain methods",
                )
                .into());
            };
            methods
        }
    };

    let api_dir = crate_root.join("src/api");
    let mut api_files = Vec::new();
    collect_rust_files(&api_dir, &mut api_files)?;

    let mut api_sources = Vec::new();
    let mut unreadable_api_files = Vec::new();
    for path in api_files {
        match fs::read_to_string(&path) {
            Ok(source) => api_sources.push(source),
            Err(_) => unreadable_api_files.push(path),
        }
    }
    assert!(
        unreadable_api_files.is_empty(),
        "failed to read api source files: {unreadable_api_files:?}"
    );

    let mut covered_methods = BTreeSet::new();
    for method in &expected_methods {
        let needle = format!("\"{method}\"");
        if api_sources.iter().any(|source| source.contains(&needle)) {
            covered_methods.insert(method.clone());
        }
    }

    let missing_methods: Vec<String> = expected_methods
        .difference(&covered_methods)
        .cloned()
        .collect();

    assert!(
        missing_methods.is_empty(),
        "missing Telegram Bot API methods in service layer: {missing_methods:?}"
    );

    Ok(())
}

#[test]
fn json_method_spec_parse_errors_are_not_silent() {
    let result = parse_expected_methods("{", Path::new("broken.json"));

    assert!(result.is_err());
}

#[test]
fn json_method_spec_requires_methods() {
    let result = parse_expected_methods(r#"{"all_methods":[]}"#, Path::new("empty.json"));

    assert!(result.is_err());
}

#[test]
fn required_method_spec_missing_is_error() {
    let result = load_required_expected_methods(Path::new(
        "fixtures/definitely-missing-method-coverage-spec.txt",
    ));

    assert!(result.is_err());
}
