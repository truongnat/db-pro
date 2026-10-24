---
schema_version: 1
id: "MEM-SRC-6c75e7ff7e9e427d"
title: "Source mirror: docs/plans/active/issue-487-null-vs-empty-delimited/CHECKLIST.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-487-null-vs-empty-delimited/CHECKLIST.md"
source_sha256: "6c75e7ff7e9e427d120a77064fb491db03b5893dbfb259ef9be12550f6baba94"
mirror_body_sha256: "6c75e7ff7e9e427d120a77064fb491db03b5893dbfb259ef9be12550f6baba94"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Checklist — #487

- [x] `UiCell::Null` → `\N` in `format_cell_delimited` (CSV + TSV + row copy)
- [x] `Text("")` still an empty field; escaping/quoting unchanged
- [x] `copy_cell_at`: empty string copyable; NULL copies `\N`
- [x] Tests: `result_grid_export_tests`, `result_grid_clipboard_tests`, updated
      `app_tests` + `result_grid_record_tests`
- [x] `cargo test -p db-pro-app`
- [x] `cargo check -p db-pro-app`
