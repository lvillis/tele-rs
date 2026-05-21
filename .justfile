set shell := ["bash", "-euo", "pipefail", "-c"]

patch:
    cargo release patch --no-publish --execute

publish:
    cargo publish --workspace

check-generated:
    cargo run -p tele-codegen -- check-advanced

check-features:
    cargo check -p tele --all-targets
    cargo check -p tele --all-targets --no-default-features --features async-tls-rustls-ring
    cargo check -p tele --all-targets --no-default-features --features blocking-tls-rustls-ring
    cargo check -p tele --all-targets --no-default-features --features bot,async-tls-rustls-ring
    cargo check -p tele --all-targets --no-default-features --features axum,macros,redis-session,postgres-session,async-tls-rustls-ring

ci:
    just check-generated
    just check-features
    cargo fmt --all --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo nextest run --workspace --all-features
    cargo test --workspace --all-features --doc
    cargo doc --workspace --no-deps
