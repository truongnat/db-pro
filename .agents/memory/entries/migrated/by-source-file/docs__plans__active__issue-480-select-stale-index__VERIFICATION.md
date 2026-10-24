---
schema_version: 1
id: "MEM-SRC-53e158dbce609ef9"
title: "Source mirror: docs/plans/active/issue-480-select-stale-index/VERIFICATION.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-480-select-stale-index/VERIFICATION.md"
source_sha256: "53e158dbce609ef9905dee8e54b540c787b13cacb107c40dd3ac366144ff011b"
mirror_body_sha256: "53e158dbce609ef9905dee8e54b540c787b13cacb107c40dd3ac366144ff011b"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# #480 Verification

- `cargo check -p db-pro-ui` → Finished, no errors (also `db-pro-app` for #510 caller).
- `cargo test -p db-pro-ui` → 687 passed, 0 failed.
- `cargo clippy -p db-pro-ui` → no warnings in touched files.
- `cargo fmt -p db-pro-ui -p db-pro-app -- --check` → clean.
- `bash scripts/check-ui-architecture.sh` → PASS.

## New regression tests
- `stale_selection_index_clamps_when_options_shrink`
