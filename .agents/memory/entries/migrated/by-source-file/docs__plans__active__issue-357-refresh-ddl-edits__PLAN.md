---
schema_version: 1
id: "MEM-SRC-efb65648fc5e60e7"
title: "Source mirror: docs/plans/active/issue-357-refresh-ddl-edits/PLAN.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-357-refresh-ddl-edits/PLAN.md"
source_sha256: "efb65648fc5e60e753bfff725283559377e1decc936835f6e9ee7213ecbab114"
mirror_body_sha256: "efb65648fc5e60e753bfff725283559377e1decc936835f6e9ee7213ecbab114"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Issue #357: Refresh DDL wipes edits without confirm; Open-in-Query can overwrite active buffer

Status: VERIFIED-FIXED (already fixed in edbcf7ae)

No code change required (file belongs to a sibling area anyway). Refresh now checks `table_ddl_dirty()` and raises `table_ddl_discard_confirm` instead of dropping the buffer; OpenInQuery uses `open_query_document_with` (new document).
