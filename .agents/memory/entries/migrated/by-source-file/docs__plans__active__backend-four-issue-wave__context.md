---
schema_version: 1
id: "MEM-SRC-d3da5acbf921b10d"
title: "Source mirror: docs/plans/active/backend-four-issue-wave/context.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/backend-four-issue-wave/context.md"
source_sha256: "d3da5acbf921b10d1f41f0e6d2c0fd29568143f4908133a454c887c4a1e31c0f"
mirror_body_sha256: "d3da5acbf921b10d1f41f0e6d2c0fd29568143f4908133a454c887c4a1e31c0f"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Context

Baseline bceb735727f81180dad678defee1000bc547d97b. Sources traced: core/domain/event_trigger.rs and app/tools/event_trigger_activity_view.rs; infra/postgres/cross_connection.rs→connector.rs→runtime/api.rs; infra/mysql/connector.rs connect/test share open_pool; core/application/delimited_transfer.rs shared source→TransferService. Existing adjacent tests and opt-in provider integration tests are reused. LSP references failed (server exits0), literal callsite scan used; semantic search judge unavailable and reported. Decisions/risk/alternatives: PLAN.md; issue-specific discovered facts: FINDINGS.md.
