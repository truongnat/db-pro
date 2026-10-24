---
schema_version: 1
id: "MEM-SRC-d144299f3327a4ea"
title: "Source mirror: docs/plans/active/issue-509-search-unicode-offsets/VERIFICATION.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-509-search-unicode-offsets/VERIFICATION.md"
source_sha256: "d144299f3327a4ea9cb6710d2cb808b5f1fcf03a9b9b5401c5aba8da51d95bcf"
mirror_body_sha256: "d144299f3327a4ea9cb6710d2cb808b5f1fcf03a9b9b5401c5aba8da51d95bcf"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# #509 Verification

- `cargo check -p db-pro-ui` → Finished, no errors (also `db-pro-app` for #510 caller).
- `cargo test -p db-pro-ui` → 687 passed, 0 failed.
- `cargo clippy -p db-pro-ui` → no warnings in touched files.
- `cargo fmt -p db-pro-ui -p db-pro-app -- --check` → clean.
- `bash scripts/check-ui-architecture.sh` → PASS.

## New regression tests
- `editor::search::tests` (7 tests incl. İ expansion + Kelvin K offsets)
- `search_match_ranges_are_original_text_byte_offsets`
