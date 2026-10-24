---
schema_version: 1
id: "MEM-PLAN-sqlite-integrity-maintenance-e3034d04d4a1"
title: "Plan archive: Sqlite Integrity Maintenance"
type: decision
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, decision, sqlite-integrity-maintenance]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "e3034d04d4a19aaedf5b0aa7d2f552c273c38b513421d4743dd63d4f848d80f0"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/sqlite-integrity-maintenance/PLAN.md"
  - "docs/plans/active/sqlite-integrity-maintenance/FINDINGS.md"
  - "docs/plans/active/sqlite-integrity-maintenance/VERIFICATION.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/sqlite-integrity-maintenance/`. Loại suy luận: `decision`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

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


## Source: `FINDINGS.md`

# Agent evidence — SQLite integrity / maintenance

## 1. Claim
omp parent; #491/#498 implementation and #461 investigation. In Progress on main; baseline 76b120724a7542de8b587a002811ddacad28156a. Excludes UI, PR #527 and unrelated provider changes.

## 2. Progress checkpoint
Baseline source proves ignored schema/per-table VACUUM and unknown constraints mapped to Check. Code/tests added; execution pending. Exact paths/decisions in context.md.

## 3. Implementation handoff / review request
Changed core domain/error.rs, infrastructure error.rs plus adjacent tests, SQLite monitoring.rs, new integration test sqlite_integrity_maintenance.rs and this plan. Gate and runtime results pending, not passing. Parent owns commit/push/issue closure.

## 4. Review outcome
n/a — not independently reviewed. No broad provider/UI completion claims.

## 5. Research / audit handoff
Issue text #491/#498/#461 read 2026-10-16. Source baseline exact SHA above; enum consumer search finds no exhaustive matches outside mapper/tests. #403 already has PostgreSQL guard but complete UI evidence not gathered. #461 bundled-default claim still unverified.

## 6. Tổng kết
Đã sửa scope VACUUM/ANALYZE và taxonomy lỗi SQLite; đang chờ chạy regression/smoke tích hợp. Chưa đóng #491/#498/#461, không suy diễn runtime từ source.


## Source: `VERIFICATION.md`

# Verification — SQLite integrity / maintenance

Baseline: `76b120724a7542de8b587a002811ddacad28156a`; candidate working-tree change, final SHA recorded at publication.

## Executed public provider smoke

Separate standalone Rust programs depend on public SQLiteConnector and SqliteMonitoringPort, not private helpers or mocked SQL. Same source ran against exact baseline export and candidate.

Baseline: `cargo run --offline --manifest-path /tmp/db-pro-cluster-probes/baseline-sqlite/Cargo.toml` → exit 101, expected assertion failure after observing:

```text
foreign_keys=true orphan_rejected=true cascade=true
table_vacuum_rejected=false vacuum_analyze_has_stats=false attached_schema_stats=true
primary_key_category=Check trigger_category=Check
```

Candidate: `cargo run --offline --manifest-path /tmp/db-pro-cluster-probes/candidate-sqlite/Cargo.toml` → exit 0:

```text
foreign_keys=true orphan_rejected=true cascade=true
table_vacuum_rejected=true vacuum_analyze_has_stats=true attached_schema_stats=true
primary_key_category=PrimaryKey trigger_category=Other(1811)
PASS: standalone public SQLite connector and monitoring smoke
```

The baseline/candidate harness initially used unsupported CellValue equality and failed compilation; corrected both identically to pattern matching. Only runnable outputs above count as runtime evidence.

## Automated / review evidence

- Integrated `cargo +nightly fmt --all && cargo check --workspace && cargo test --workspace`: exit 0, 1,999 passed / 0 failed / 43 ignored (`artifact://93`), includes three real engine mapper cases.
- After independent reviewer found default VacuumAnalyze could ANALYZE attached databases, fixed second leg to quoted main when no schema supplied and extended existing fixture with populated aux database. `cargo test -p db-pro-infrastructure --test sqlite_integrity_maintenance` → 4 passed / 0 failed / 0 ignored, exit 0 after correction (`artifact://109`). It proves main stats count=3 while aux has no sqlite_stat1, scoped quoted-table stats in aux=2, explicit unsupported per-table vacuum, fresh connector FK enforcement and cascade.
- Independent reviewer ReviewScheduleSshSqlite rechecked corrected source: ACCEPT, no unresolved introduced findings. Source review is not runtime proof.
- Workspace Clippy baseline failures remain separately recorded in agent-replay-step-budget/VERIFICATION.md; no suppression or unrelated lint repair here.

## #461 actual bundled default

No product fix is needed for the alleged default-OFF behavior on this repository build: both exact baseline and candidate fresh public connectors report foreign_keys=1, reject orphan inserts and execute ON DELETE CASCADE. Cargo.toml:46 pins rusqlite with bundled; installed libsqlite3-sys 0.30.1 build.rs:123 explicitly sets `-DSQLITE_DEFAULT_FOREIGN_KEYS=1`. This is source corroboration of actual engine evidence, not an inference from another SQLite binary. Added an independent regression guarding real enforcement; did not touch actor.rs or draft PR #527. No claim that unrelated system SQLite builds have the same default.

## Boundaries

VACUUM then ANALYZE are ordered separate operations, not atomic. SQLite exercised independently. PostgreSQL mapper/maintenance unchanged; full #403 UI acceptance not claimed. MySQL/SQLServer n/a for these SQLite-specific fixes. UI unchanged; no screenshots or native interaction claim. PrimaryKey and Other(i32) extend core classification while retaining generic error code/message-id consumers. No CI pass claimed.

