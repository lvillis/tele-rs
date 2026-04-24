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

fn parse_expected_methods(text: &str, path: &Path) -> Option<BTreeSet<String>> {
    if path.extension().is_some_and(|ext| ext == "json") {
        let spec: MethodCoverageSpec = serde_json::from_str(text).ok()?;
        if spec.all_methods.is_empty() {
            return None;
        }
        return Some(spec.all_methods.into_iter().collect());
    }

    let methods = text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(ToOwned::to_owned)
        .collect::<BTreeSet<_>>();

    (!methods.is_empty()).then_some(methods)
}

fn collect_rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_rust_files(&path, out);
            continue;
        }
        if path.extension().is_some_and(|ext| ext == "rs") {
            out.push(path);
        }
    }
}

#[test]
fn telegram_bot_api_methods_are_fully_covered() -> Result<(), Box<dyn std::error::Error>> {
    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = crate_root
        .parent()
        .and_then(|path| path.parent())
        .map(Path::to_path_buf)
        .unwrap_or_else(|| crate_root.clone());

    let mut candidate_paths = Vec::new();
    if let Ok(path) = env::var("TELE_METHOD_COVERAGE_SPEC_PATH") {
        candidate_paths.push(PathBuf::from(path));
    }
    candidate_paths.push(workspace_root.join("crates/tele-codegen/spec/bot_api.json"));

    let mut expected_methods = None;
    for path in &candidate_paths {
        if let Ok(text) = fs::read_to_string(path)
            && let Some(methods) = parse_expected_methods(&text, path)
        {
            expected_methods = Some(methods);
            break;
        }
    }

    let expected_methods = match expected_methods {
        Some(methods) => methods,
        None => {
            assert!(
                !EMBEDDED_METHOD_SPEC.trim().is_empty(),
                "embedded method spec fixture is empty"
            );
            parse_expected_methods(EMBEDDED_METHOD_SPEC, Path::new("bot_api_all_methods.txt"))
                .ok_or_else(|| {
                    std::io::Error::other("embedded method spec fixture must contain methods")
                })?
        }
    };

    let api_dir = crate_root.join("src/api");
    let mut api_files = Vec::new();
    collect_rust_files(&api_dir, &mut api_files);

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
