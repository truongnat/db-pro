---
schema_version: 1
id: "MEM-SRC-e46d0968a4107a90"
title: "Source mirror: docs/plans/active/query-editor-zed-feel/CHECKLIST.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/query-editor-zed-feel/CHECKLIST.md"
source_sha256: "e46d0968a4107a90334407d72695739f46047ab32122c7268bdecdc668ca1805"
mirror_body_sha256: "e46d0968a4107a90334407d72695739f46047ab32122c7268bdecdc668ca1805"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Checklist

## Existing visual interaction

- [x] Manual scroll both axes with content size from buffer
- [x] Caret blink + reset on edit/move + repaint while focused
- [x] Keep caret in view after cursor moves
- [x] Softer frame / gutter / current-line / selection
- [x] Comfortable editor line height

## Reliable input and suggestions

- [x] Editor interaction policy separated from rendering
- [x] Ordinary text/IME input does not auto-open completion
- [x] Dot-triggered and `Ctrl/Cmd+Space` manual completion
- [x] AI prediction default off and no automatic scheduling while typing
- [x] Completion session bound to source document version
- [x] Invalid/stale replacement ranges fail closed
- [x] Completion detail/documentation shown on selection and hover
- [x] `Ctrl/Cmd+S` requests save without inserting text
- [x] Continuous prefix filtering with version-safe session refresh
- [x] SQL symbol hover outside the completion popup
- [x] Signature help for SQL functions

## Gates

- [x] `cargo fmt --all -- --check`
- [x] `cargo check -p db-pro-ui -p db-pro-native`
- [x] `cargo clippy -p db-pro-ui -p db-pro-native --all-targets -- -D warnings`
- [x] `cargo test -p db-pro-ui` — 536 passed
- [x] clean-code diff scanner reviewed — 0 failures, 4 warning classes
- [x] release build via performance audit — 31.2 MB binary
- [ ] Runtime visual/interaction evidence supplied by owner
