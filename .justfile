check-generated:
    python3 scripts/gen_advanced.py
    cargo fmt --all
    git diff --exit-code -- crates/tele/src/types/advanced.rs crates/tele/src/api/advanced.rs

ci:
    just check-generated
    cargo fmt --all --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace --all-features
    cargo doc --workspace --no-deps
