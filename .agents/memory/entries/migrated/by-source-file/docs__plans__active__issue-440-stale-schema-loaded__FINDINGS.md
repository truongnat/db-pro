---
schema_version: 1
id: "MEM-SRC-b502c84e0072469a"
title: "Source mirror: docs/plans/active/issue-440-stale-schema-loaded/FINDINGS.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-440-stale-schema-loaded/FINDINGS.md"
source_sha256: "b502c84e0072469aa55958024f9130afaf2215db72f0d151eda4e7016925cf7f"
mirror_body_sha256: "b502c84e0072469aa55958024f9130afaf2215db72f0d151eda4e7016925cf7f"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Findings — #440

`schema_events.rs:48-53` previously only rejected when a *different* request id was pending; `None` (post-disconnect) fell through and applied the stale schema.
`request_schema_introspection` (app/mod.rs:434) always sets `schema_request` before dispatch, so a legit response always has `Some(request_id)` pending. `handle_schema_request_failure` already used exact-match (`!= Some`).
Updated two app tests that sent SchemaLoaded without a pending request. Added `schema_loaded_after_disconnect_is_dropped` regression test.
Same `is_some_and` shape exists in `connection/connection_events.rs:75-80` (`on_connected`) — outside my file ownership; flagged to parent.
