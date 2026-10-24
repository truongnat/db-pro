---
schema_version: 1
id: "MEM-SRC-b6b31641fdf71689"
title: "Source mirror: docs/plans/active/issue-496-497-fk-dropdown/CHECKLIST.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-496-497-fk-dropdown/CHECKLIST.md"
source_sha256: "b6b31641fdf7168995884f19c4bb8db13c27f8ebe695f2e066c55c066550d518"
mirror_body_sha256: "b6b31641fdf7168995884f19c4bb8db13c27f8ebe695f2e066c55c066550d518"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Checklist — #496 + #497

- [x] `dropdown_applies` = 1:1 columns only; composite → `None` (text edit)
- [x] `lookup_sql` emits `ORDER BY <first to_column> LIMIT 500`
- [x] `FK_LOOKUP_LIMIT` constant shared by SQL + truncation note
- [x] `open_referenced_row` still navigates composite FKs
- [x] Tests in `table/tests/table_fk_lookup_tests.rs`
- [x] `cargo test -p db-pro-app`
- [x] `cargo check -p db-pro-app`
