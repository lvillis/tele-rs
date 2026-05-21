set shell := ["bash", "-euo", "pipefail", "-c"]

patch:
    cargo release patch --no-publish --execute

publish:
    cargo publish -p tele-macros --locked
    cargo publish -p tele --locked

check-generated:
    cargo run -p tele-codegen -- check-advanced

ci:
    just check-generated
    cargo fmt --all --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo nextest run --workspace --all-features
    cargo test --workspace --all-features --doc
    cargo doc --workspace --no-deps
