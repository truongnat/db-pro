---
schema_version: 1
id: "MEM-SRC-c9400cd5972835bc"
title: "Source mirror: docs/plans/active/issue-505-filtered-selection-arrows/CHECKLIST.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-505-filtered-selection-arrows/CHECKLIST.md"
source_sha256: "c9400cd5972835bc3c614016cec53e8a8ea1979c22db891ccbbfb1e6d9eedd1c"
mirror_body_sha256: "c9400cd5972835bc3c614016cec53e8a8ea1979c22db891ccbbfb1e6d9eedd1c"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Checklist — #505

- [x] `row_position` is Option-aware; no 0-default for arrows
- [x] ArrowDown → first visible row when selection not in projection
- [x] ArrowUp → last visible row when selection not in projection
- [x] Home/End/Left/Right keep prior fallback behavior
- [x] Tests in `result_grid/tests/mod_tests.rs`
- [x] `cargo test -p db-pro-app`
- [x] `cargo check -p db-pro-app`
