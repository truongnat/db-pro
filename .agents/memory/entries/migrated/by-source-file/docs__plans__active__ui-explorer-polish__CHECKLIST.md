---
schema_version: 1
id: "MEM-SRC-c9f0ce7b33431cfb"
title: "Source mirror: docs/plans/active/ui-explorer-polish/CHECKLIST.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/ui-explorer-polish/CHECKLIST.md"
source_sha256: "c9f0ce7b33431cfbde06e444295d1b76cebbaf1863b566ddc32d62bc7346274d"
mirror_body_sha256: "c9f0ce7b33431cfbde06e444295d1b76cebbaf1863b566ddc32d62bc7346274d"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Checklist — UI04 Explorer Polish

- [x] Audit category folder colors (Views, Functions, Triggers) and object rows in `explorer_folders.rs`.
- [x] Audit context menus in `explorer_view.rs`, `explorer_tree.rs`, and `explorer_details.rs`.
- [x] Ensure consistent theme tokens for object types in dark and light modes.
- [x] Run Rust quality gates (`cargo fmt`, `cargo check`, `cargo clippy`, `cargo test`).
- [x] Document findings and verification.
