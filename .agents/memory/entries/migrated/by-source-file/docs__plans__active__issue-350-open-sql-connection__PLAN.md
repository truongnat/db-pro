---
schema_version: 1
id: "MEM-SRC-dc8d3a3a48eb0b93"
title: "Source mirror: docs/plans/active/issue-350-open-sql-connection/PLAN.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-350-open-sql-connection/PLAN.md"
source_sha256: "dc8d3a3a48eb0b938adaa3f15bb80bad838269d4af115deb0159ba6eb6c9b984"
mirror_body_sha256: "dc8d3a3a48eb0b938adaa3f15bb80bad838269d4af115deb0159ba6eb6c9b984"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Issue #350: Open in SQL Editor uses query context connection, not plan's connection

Status: VERIFIED-FIXED (already fixed in 9fe8706e)

No code change required. `OpenSql` handler binds the new document to `workbench.preview_connection_id` (captured at plan time in `run_plan`) and sets the document schema.
