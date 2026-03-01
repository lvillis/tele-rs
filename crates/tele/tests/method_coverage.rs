use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

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
fn telegram_bot_api_methods_are_fully_covered() {
    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = crate_root
        .parent()
        .and_then(|path| path.parent())
        .map(Path::to_path_buf)
        .unwrap_or_else(|| crate_root.clone());

    let spec_path = workspace_root.join(".docs/spec/telegram_bot_api_9_4_all_methods.txt");
    let expected_text = fs::read_to_string(&spec_path).unwrap_or_default();
    assert!(
        !expected_text.is_empty(),
        "failed to read method spec file `{}`",
        spec_path.display()
    );

    let expected_methods: BTreeSet<String> = expected_text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(ToOwned::to_owned)
        .collect();

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
}
