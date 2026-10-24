---
schema_version: 1
id: "MEM-SRC-874f62549ee4801d"
title: "Source mirror: docs/plans/active/sqlite-integrity-maintenance/PLAN.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/sqlite-integrity-maintenance/PLAN.md"
source_sha256: "874f62549ee4801d5af06be690deffd77d9e7d328c5e832f1d15bc73570d5791"
mirror_body_sha256: "874f62549ee4801d5af06be690deffd77d9e7d328c5e832f1d15bc73570d5791"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# SQLite integrity and maintenance — #491 / #498 / #461 triage

Lifecycle: REVIEW. Baseline `76b120724a7542de8b587a002811ddacad28156a`. Real-engine verification and independent source review ACCEPT; publication pending.
Owner authorized continuous issue-based fixes without per-ticket approval.

## Requirements / context

- R1 (#491): reject SQLite table-target VACUUM/VACUUM ANALYZE before executing SQL, never silently expand to database scope. Database/schema VACUUM ANALYZE must run both real operations; quoted ANALYZE targets preserve caller scope.
- R2 (#498): classify actual PRIMARY KEY violations as PrimaryKey, preserve known UNIQUE/FK/NOT NULL/CHECK categories, and carry unknown SQLite extended constraint codes as Other(code), never falsely label them CHECK.
- R3 (#461 investigation): exercise public SQLiteConnector PRAGMA state, orphan insert refusal and ON DELETE CASCADE before deciding whether missing explicit initialization is a real defect with the bundled SQLite build. Do not edit actor.rs owned by draft PR #527 from source suspicion alone.

Existing patterns: SQLiteMonitoringPort owns provider SQL and existing identifier quoting; public connector is the shared real execution path. Error mapping stays in infrastructure/error.rs; ConstraintType is the core taxonomy. LSP reference request failed; complete literal ConstraintType:: search found no exhaustive external consumers. New tests remain adjacent/integration-level, never inline.

## Acceptance / DoD

- [x] R1: real provider smoke/test rejects per-table vacuum, performs both operations and preserves main/attached-schema scope.
- [x] R2: real engine PK → PrimaryKey, trigger → Other(1811), other known categories preserved; generic error envelope consumers compile.
- [x] R3: baseline and candidate fresh connector enforcement/cascade PASS. Bundled build enables FK by compile option; no speculative actor fix.
- [x] Integrated fmt/check/tests executed (1,999/0/43); corrected SQLite suite 4/0/0, independent source review ACCEPT. Inherited Clippy failure recorded separately.

## Tasks

- [x] Add meaningful regression cases; reproduce baseline with isolated public-provider smoke.
- [x] Fix maintenance/error taxonomy; preserve provider/consumer contracts.
- [ ] Publish exact slice and close individually satisfied issues with runtime evidence.

## Scope fences

No actor resilience/busy-timeout edits, no database-wide fallback for table operations, no UI redesign, no retries/telemetry, no binary value/export changes. VACUUM then ANALYZE are sequential provider operations, not an atomic transaction (VACUUM cannot run inside one). PostgreSQL maintenance behavior is unchanged; #403 already has a source-level guard but its full UI acceptance is not claimed here.
