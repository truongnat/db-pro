---
schema_version: 1
id: "MEM-SRC-6574d57ad0a927a8"
title: "Source mirror: docs/plans/active/issue-506-grapheme-word-select/VERIFICATION.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-506-grapheme-word-select/VERIFICATION.md"
source_sha256: "6574d57ad0a927a8a428a41150c40db7464b2ceecf1cbb93392d65d193fda753"
mirror_body_sha256: "6574d57ad0a927a8a428a41150c40db7464b2ceecf1cbb93392d65d193fda753"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# #506 Verification

- `cargo check -p db-pro-ui` → Finished, no errors (also `db-pro-app` for #510 caller).
- `cargo test -p db-pro-ui` → 687 passed, 0 failed.
- `cargo clippy -p db-pro-ui` → no warnings in touched files.
- `cargo fmt -p db-pro-ui -p db-pro-app -- --check` → clean.
- `bash scripts/check-ui-architecture.sh` → PASS.

## New regression tests
- `select_word_keeps_combining_marks_inside_the_word`
- `select_word_handles_decomposed_vietnamese_tone`
- `select_word_at_end_of_buffer_selects_last_word`
