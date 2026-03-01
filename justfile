gen-advanced:
    python3 scripts/gen_advanced.py
    cargo fmt --all

check-generated:
    python3 scripts/gen_advanced.py
    cargo fmt --all
    git diff --exit-code -- crates/tele/src/types/advanced.rs crates/tele/src/api/advanced.rs

answer-check:
    cargo fmt --all
    cargo check --all-targets --all-features
    cargo clippy --all-targets --all-features -- -D warnings

ci:
    just check-generated
    cargo fmt --all --check
    cargo clippy --all-targets --all-features -- -D warnings
    cargo test --all-features
    cargo doc --no-deps

release-check:
    just ci
    cargo package -p tele-macros --allow-dirty

release-plan level='patch':
    cargo release {{level}} --workspace --no-confirm

release-run level='patch':
    cargo release {{level}} --workspace --no-confirm --execute

release-plan-unpublished:
    cargo release --workspace --unpublished --no-confirm

release-run-unpublished:
    cargo release --workspace --unpublished --no-confirm --execute
