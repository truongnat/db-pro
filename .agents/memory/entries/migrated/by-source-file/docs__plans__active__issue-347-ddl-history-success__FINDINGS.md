---
schema_version: 1
id: "MEM-SRC-c677ef3169dfe40b"
title: "Source mirror: docs/plans/active/issue-347-ddl-history-success/FINDINGS.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-347-ddl-history-success/FINDINGS.md"
source_sha256: "c677ef3169dfe40b914c90ad40f4ea2833bce5e6fc1089efd130a870ab6c696c"
mirror_body_sha256: "c677ef3169dfe40b914c90ad40f4ea2833bce5e6fc1089efd130a870ab6c696c"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Findings — #347

schema_workbench_actions.rs:178-179, schema_workbench.rs:242-254 (`finalize_execution`), wired in runtime/operation_events.rs:220 (success) and runtime/runtime_event_handlers.rs:234 (failure).
