---
schema_version: 1
id: "MEM-SRC-229799903955d635"
title: "Source mirror: docs/plans/active/ui-dialogs-forms-consistency/CHECKLIST.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/ui-dialogs-forms-consistency/CHECKLIST.md"
source_sha256: "229799903955d635c41e8b692e03434f40cf080e098705d4ed9c4ee3ed94ddb8"
mirror_body_sha256: "229799903955d635c41e8b692e03434f40cf080e098705d4ed9c4ee3ed94ddb8"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Checklist — UI08 Dialog, Form, and Confirmation Consistency

- [x] Standardize query dialogs (destructive confirmation, export, overwrite, save as, unsaved query) in `query_dialogs_view.rs`.
- [x] Standardize connection dialogs and form buttons (delete confirmations, presets, URI import, browse) in `connection_view.rs`.
- [x] Standardize inspector window close buttons in `table_metadata_view.rs`.
- [x] Extend non-regression tests in `crates/ui/src/components/mod.rs`.
- [x] Run full workspace quality gates (`cargo fmt`, `cargo check`, `cargo clippy`, `cargo test`, `cargo build --release --locked -p db-pro-native`).
- [x] Record verification evidence.
