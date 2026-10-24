---
schema_version: 1
id: "MEM-SRC-59bf4bebf30fb996"
title: "Source mirror: docs/plans/completed/native-ai-agent/VERIFICATION.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/completed/native-ai-agent/VERIFICATION.md"
source_sha256: "59bf4bebf30fb996a2a9982947cfbf0ffb2531f23aa53f77ca8a11ebebbabb9c"
mirror_body_sha256: "59bf4bebf30fb996a2a9982947cfbf0ffb2531f23aa53f77ca8a11ebebbabb9c"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
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
