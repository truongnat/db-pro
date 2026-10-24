---
schema_version: 1
id: "MEM-SRC-17c67418118c027d"
title: "Source mirror: docs/plans/completed/query-workspace-zed-shell/CHECKLIST.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/completed/query-workspace-zed-shell/CHECKLIST.md"
source_sha256: "17c67418118c027d8a43f87e4c7215006505d6bcfdb663d1ab59e4c16a6d23b1"
mirror_body_sha256: "17c67418118c027d8a43f87e4c7215006505d6bcfdb663d1ab59e4c16a6d23b1"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Checklist — query-workspace-zed-shell

## Plan

- [x] PLAN.md written
- [x] Branch `feature/query-workspace-zed-shell`

## Implement

- [x] Editor-first `draw_query` composition
- [x] Context chips replace ComboBoxes
- [x] Query status strip with Run/Stop
- [x] Output dock gated; open on result/error
- [x] Hide permanent diagnostics / params / inline completion
- [x] Find as floating overlay
- [x] Skip duplicate shell output panel on Query tab
- [x] Builder moved to More menu

## Verify

- [x] `cargo fmt` (touched files)
- [x] `cargo check -p db-pro-ui`
- [x] `cargo clippy -p db-pro-ui --all-targets -- -D warnings`
- [x] `cargo test -p db-pro-ui --lib` (523 passed)
- [x] Runtime screenshots A–J recorded in `screenshots/` (1280x800 and 1440x900 light and dark)

## Review

- [x] Self-review P0/P1/P2
- [x] PR referencing PLAN / CHECKLIST / VERIFICATION
