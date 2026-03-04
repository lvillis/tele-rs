check-generated:
    python3 scripts/gen_advanced.py
    cargo fmt --all
    git diff --exit-code -- crates/tele/src/types/advanced.rs crates/tele/src/api/advanced.rs

answer-check:
    cargo fmt --all --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings

verify: answer-check

ci:
    just check-generated
    just answer-check
    cargo test --workspace --all-features
    cargo doc --workspace --no-deps

release-check:
    just ci
    cargo package --workspace --all-features --allow-dirty --no-verify
