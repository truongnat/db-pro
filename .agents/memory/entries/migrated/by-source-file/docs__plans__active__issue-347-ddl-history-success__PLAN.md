---
schema_version: 1
id: "MEM-SRC-f609c5dfd5f3b58f"
title: "Source mirror: docs/plans/active/issue-347-ddl-history-success/PLAN.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-347-ddl-history-success/PLAN.md"
source_sha256: "f609c5dfd5f3b58f5510f7fc41bfe536dbd2e458300d11f4a7bd917f491c69b1"
mirror_body_sha256: "f609c5dfd5f3b58f5510f7fc41bfe536dbd2e458300d11f4a7bd917f491c69b1"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Issue #347: Execution history records success=true at dispatch

Status: VERIFIED-FIXED (already fixed in 9fe8706e)

No code change required. `apply_workbench_ddl` records `success=false` (pending) at dispatch and stores `(request_id, history_index)` in `workbench.pending_execution`; `finalize_execution` patches the entry on `DdlCompleted` / `QueryFailed`.
