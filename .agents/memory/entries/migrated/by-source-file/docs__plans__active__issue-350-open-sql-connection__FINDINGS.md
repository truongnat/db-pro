---
schema_version: 1
id: "MEM-SRC-9dcdd6cf75e0c522"
title: "Source mirror: docs/plans/active/issue-350-open-sql-connection/FINDINGS.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-350-open-sql-connection/FINDINGS.md"
source_sha256: "9dcdd6cf75e0c522c4673a5153e2cd02c6de2f469307e9a2251a49af69292ada"
mirror_body_sha256: "9dcdd6cf75e0c522c4673a5153e2cd02c6de2f469307e9a2251a49af69292ada"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Findings — #350

schema_workbench_actions.rs:21-38. `preview_connection_id` stored at :102-103. Handler calls `set_document_connection(index, Some(connection_id))` and `set_document_schema`. The handling lives in schema_workbench_actions.rs (my file), not events_query_dispatch.
