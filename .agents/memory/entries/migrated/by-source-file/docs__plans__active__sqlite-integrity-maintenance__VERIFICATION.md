---
schema_version: 1
id: "MEM-SRC-e1bc1922ed6d8a3e"
title: "Source mirror: docs/plans/active/sqlite-integrity-maintenance/VERIFICATION.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/sqlite-integrity-maintenance/VERIFICATION.md"
source_sha256: "e1bc1922ed6d8a3ef9af70e91e7cf8967d378e17f2008fb2bd5ac187d0a09d08"
mirror_body_sha256: "e1bc1922ed6d8a3ef9af70e91e7cf8967d378e17f2008fb2bd5ac187d0a09d08"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
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
