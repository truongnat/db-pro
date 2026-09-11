# Verification

## Automated

- `cargo fmt --all -- --check` passed.
- `cargo check -p db-pro-ui -p db-pro-native` passed.
- `cargo test -p db-pro-ui` passed: 10 tests.
- `cargo check --workspace` passed.
- `cargo clippy --workspace --all-targets -- -D warnings` passed.
- `cargo test --workspace` passed with no failures; the PostgreSQL integration set remains ignored because it needs an external provider.

## Runtime

Native runtime review passed with the raw `cargo run -p db-pro-native` binary:

- light shell and Agent empty state rendered in `/tmp/db-pro-agent-v3-empty.png`;
- active SQLite context rendered as `Native Test · SQLite · 2 tables`;
- read-only SQL response rendered in `/tmp/db-pro-agent-sql.png`;
- mutation warning rendered in `/tmp/db-pro-agent-mutation.png`;
- Insert into Query activated the Query tab and populated the editor without executing the draft.

The provider/API-backed Codex integration is intentionally a later slice; this
plan only covers the offline native copilot contract and safe draft workflow.
