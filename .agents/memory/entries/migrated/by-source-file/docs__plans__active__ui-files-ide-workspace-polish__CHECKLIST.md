---
schema_version: 1
id: "MEM-SRC-4cb7e855adb99827"
title: "Source mirror: docs/plans/active/ui-files-ide-workspace-polish/CHECKLIST.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/ui-files-ide-workspace-polish/CHECKLIST.md"
source_sha256: "4cb7e855adb99827631919193867a29f08cda6fb40edb8893ef945c605e2e185"
mirror_body_sha256: "4cb7e855adb99827631919193867a29f08cda6fb40edb8893ef945c605e2e185"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Checklist — UI09 Files/IDE Workspace Polish

- [x] Standardize workspace root and folder actions in `files_activity_view.rs`.
- [x] Standardize scratch buffer, new file, and new folder actions in `files_activity_view.rs`.
- [x] Standardize search/replace and refactor actions in `files_activity_view.rs`.
- [x] Standardize sandbox and suite runner actions in `files_activity_view.rs`.
- [x] Standardize git staging, diff, and commit actions in `files_activity_view.rs`.
- [x] Standardize task creation buttons in `tasks_view.rs`.
- [x] Extend non-regression tests in `crates/ui/src/components/mod.rs`.
- [x] Run full workspace quality gates (`cargo fmt`, `cargo check`, `cargo clippy`, `cargo test`, `cargo build --release --locked -p db-pro-native`).
- [x] Record verification evidence.
