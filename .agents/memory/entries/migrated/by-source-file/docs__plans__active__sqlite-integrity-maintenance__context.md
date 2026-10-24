---
schema_version: 1
id: "MEM-SRC-fbd5ece4dacbb0ce"
title: "Source mirror: docs/plans/active/sqlite-integrity-maintenance/context.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/sqlite-integrity-maintenance/context.md"
source_sha256: "fbd5ece4dacbb0ce6c2d50494da56f97899de7d6d361d9ace9fa07f51ae5bc6c"
mirror_body_sha256: "fbd5ece4dacbb0ce6c2d50494da56f97899de7d6d361d9ace9fa07f51ae5bc6c"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Context

Baseline: 76b120724a7542de8b587a002811ddacad28156a. Owner delegated scope approval with continuous issue work.
- infrastructure/src/sqlite/monitoring.rs ignored schema and emitted per-table VACUUM; VacuumAnalyze ran VACUUM only.
- infrastructure/src/error.rs mapped unknown extended constraints to Check; core ConstraintType lacked PrimaryKey/Other.
- Full literal enum-consumer search found no exhaustive consumers requiring migration; LSP references failed (server exits). Extend taxonomy without changing existing error code/message identifiers.
- Reuse real SQLiteConnector/MonitoringPort in integration tests and rusqlite engine errors for mapper unit tests. No mocks/source-text tests.
- Existing core/infra error behavior for PostgreSQL unchanged. SQLite-specific checks do not prove PG behavior; #403 UI/provider acceptance separate.
- Prefer explicit unsupported table VACUUM over a dangerous database fallback. VACUUM plus ANALYZE are ordered, not atomic.
