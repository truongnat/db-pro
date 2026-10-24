---
schema_version: 1
id: "MEM-SRC-c5093e92ced84479"
title: "Source mirror: docs/plans/archived/qa-p2-readonly-connection-grid/CHECKLIST.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/archived/qa-p2-readonly-connection-grid/CHECKLIST.md"
source_sha256: "c5093e92ced844799f7a4088bba3fa0107adfb579200b9555ed11ef76cca6c01"
mirror_body_sha256: "c5093e92ced844799f7a4088bba3fa0107adfb579200b9555ed11ef76cca6c01"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# CHECKLIST — QA-P2-08 Read-Only Connection Data Grid Visual Affordance

- [x] Check connection `readonly` flag in `DataSection`
- [x] Guard `handleCellSave`, `handleDeleteRow`, `handleBatchDelete`, `handleRowSave` when connection is read-only
- [x] Pass `isReadonlyConnection` to `DataGrid`
- [x] Calculate `canEdit = pkColumns.length > 0 && !isReadonlyConnection` in `DataGrid`
- [x] Render read-only connection banner in `DataGrid`
- [x] Add i18n key `dataGrid.readOnlyConnection` in `en.json` and `ja.json`
- [x] Add unit tests in `data-grid.test.tsx`
- [x] Run all frontend quality gates (`typecheck`, `lint`, `format:check`, `check:tokens`, `test`)
- [x] Run backend quality gates (`cargo test`)
