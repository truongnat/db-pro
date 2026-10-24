---
schema_version: 1
id: "MEM-SRC-46b0c0964a84b3a1"
title: "Source mirror: docs/plans/active/ui-shell-hierarchy/VERIFICATION.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/ui-shell-hierarchy/VERIFICATION.md"
source_sha256: "46b0c0964a84b3a141866d7df776dba6e5cbf95fb70e99f7e6c4be0943e7382f"
mirror_body_sha256: "46b0c0964a84b3a141866d7df776dba6e5cbf95fb70e99f7e6c4be0943e7382f"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Verification — UI03 Shell Hierarchy

## Test Matrix

- `cargo fmt --all -- --check` -> PASS
- `cargo check --workspace` -> PASS
- `cargo clippy --workspace --all-targets -- -D warnings` -> PASS
- `cargo test --workspace` -> PASS (483 passed, 0 failed in db-pro-ui)
- `cargo build --release --locked -p db-pro-native` -> PASS
