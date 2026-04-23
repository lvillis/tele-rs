set shell := ["bash", "-euo", "pipefail", "-c"]

patch:
    cargo release patch --no-publish --execute

publish:
    cargo publish --workspace

check-generated:
    python3 scripts/gen_advanced.py
    cargo fmt --all
    git diff --exit-code -- crates/tele/src/types/advanced.rs crates/tele/src/api/advanced.rs

ci:
    just check-generated
    cargo fmt --all
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo nextest run --workspace --all-features
    cargo test --workspace --all-features --doc
    cargo doc --workspace --no-deps
