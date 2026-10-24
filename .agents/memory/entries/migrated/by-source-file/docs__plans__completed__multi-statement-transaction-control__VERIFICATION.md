---
schema_version: 1
id: "MEM-SRC-e00a62cd29d854d7"
title: "Source mirror: docs/plans/completed/multi-statement-transaction-control/VERIFICATION.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/completed/multi-statement-transaction-control/VERIFICATION.md"
source_sha256: "e00a62cd29d854d7c158628b723578e5dd9138485a413e31261084a76259fbde"
mirror_body_sha256: "e00a62cd29d854d7c158628b723578e5dd9138485a413e31261084a76259fbde"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Verification — #147

## Gates (this branch, before commit)

| Command | Result |
|---|---|
| `cargo fmt --all -- --check` | pending |
| `cargo check --workspace` | pending |
| `cargo clippy --workspace --all-targets -- -D warnings` | pending |
| `cargo test --workspace` | pending |
| `cargo build --release --locked -p db-pro-native` | pending |
| `bash .skills/perf-audit/scripts/perf-scan.sh` | pending |

(update this table as each gate actually runs; do not tick without executing)

## Live database evidence (collected)

- SQLite: `cargo test -p db-pro-infrastructure --test multistatement_transaction_control`
  → 4 passed / 0 failed.
- PostgreSQL: fixture `dbpro-v01-pg-fixture` (docker, `postgres:16`, port 55432);
  `DATABASE_URL=postgres://dbpro:...@127.0.0.1:55432/dbpro_fixture cargo test
  -p db-pro-infrastructure --test pg_integration -- --ignored pg_multi_statement` → 2 passed;
  `-- --ignored pg_connector_alone` → 1 passed. Characterisation pins the measured defect
  (`RolledBack` + 1 surviving row) on the connector itself, unchanged by the fix.

## Probes

Temporary probe files (`probe_ms_tx.rs`, `probe_ms_tx_pg.rs`) measured all four batch shapes on both
providers (tables in PLAN.md) and were deleted after the measurements were folded into tests.
