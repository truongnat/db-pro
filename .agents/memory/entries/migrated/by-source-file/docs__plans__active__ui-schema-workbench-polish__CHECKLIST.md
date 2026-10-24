---
schema_version: 1
id: "MEM-SRC-2fdd6f501d59ff02"
title: "Source mirror: docs/plans/active/ui-schema-workbench-polish/CHECKLIST.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/ui-schema-workbench-polish/CHECKLIST.md"
source_sha256: "2fdd6f501d59ff028ed56fb3c2a2e881f09bfb30761692ce7de08ba0cf284ad8"
mirror_body_sha256: "2fdd6f501d59ff028ed56fb3c2a2e881f09bfb30761692ce7de08ba0cf284ad8"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Checklist — UI07 Table/Object/Schema Workbench Quality

- [x] Standardize schema workbench planning action buttons in `schema_workbench_form.rs`.
- [x] Standardize schema object workbench actions (routine source editor, execution, DDL preview, drop confirmation) in `schema_object_view.rs`.
- [x] Standardize table DDL tab actions (Refresh, Open in Query, Copy) in `table_ddl_view.rs`.
- [x] Standardize table metadata filter clear and jump actions in `table_metadata_view.rs`.
- [x] Standardize table workspace welcome and toolbar actions in `table_view.rs`.
- [x] Extend non-regression tests in `crates/ui/src/components/mod.rs`.
- [x] Run full workspace quality gates (`cargo fmt`, `cargo check`, `cargo clippy`, `cargo test`, `cargo build --release --locked -p db-pro-native`).
- [x] Record verification evidence.
