---
schema_version: 1
id: "MEM-SRC-565611b6cdd27bdb"
title: "Source mirror: docs/plans/active/sqlserver-provider/VERIFICATION.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/sqlserver-provider/VERIFICATION.md"
source_sha256: "565611b6cdd27bdbc2000dec1df38fb40f8d00d546416b45e623f1f41d0a9b51"
mirror_body_sha256: "565611b6cdd27bdbc2000dec1df38fb40f8d00d546416b45e623f1f41d0a9b51"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# SQL Server provider verification (#260)

## Baseline

- SHA: `cc399501338b7f38342a1bb12b9990e048537312`
- State: `RUNTIME_VERIFY`
- Implementation SHA: `774a4e7f6ce8d88939988711e8fadf0588f1c3c1`

## Commands

| Command | Result |
|---|---|
| `cargo fmt --all -- --check` | PASS |
| `cargo check --workspace` | PASS |
| `cargo clippy --workspace --all-targets -- -D warnings` | PASS |
| `cargo test --workspace --quiet` | PASS: 1164 passed, 0 failed, 42 ignored |
| `cargo build --release --locked -p db-pro-native` | PASS |
| `bash .skills/clean-code/scripts/clean-code-scan.sh rust --diff --ratchet --ci` | PASS: 13 pass, 3 warnings, 0 fail |
| `cargo test -p db-pro-infrastructure --test sqlserver_integration -- --list` | PASS: 4 ignored fixture tests registered |
| `cargo test -p db-pro-infrastructure sqlserver -- --nocapture` | PASS: 2 provider unit tests |

## Provider matrix

| Provider | Supported operation | Automated evidence | Live/runtime evidence | Capability gate |
|---|---|---|---|---|
| SQL Server | connect/query/execute/batch/transaction/introspection/dialect implemented | unit tests PASS; 4 ignored fixture tests registered | BLOCKED: no `SQLSERVER_URL` or live SQL Server fixture | explicit capability matrix in `db-pro-core` |
| PostgreSQL | existing support | NOT RE-RUN | NOT VERIFIED in this task | unchanged/out of scope |
| SQLite | existing support | NOT RE-RUN | NOT VERIFIED in this task | unchanged/out of scope |

## Runtime evidence

No native UI screenshot/recording or live SQL Server evidence has been collected.
Docker was checked on 2026-09-17: no SQL Server container/image was available in
the environment. The fixture command is documented in
`crates/infrastructure/tests/sqlserver_integration.rs`.

## Issue-wave evidence — 2026-10-22 (HEAD `24bae059`)

The 2026-09-17 BLOCKED rows above are superseded: a live SQL Server 2022 CU27
fixture now runs on this host (`dbpro-mssql-fixture`, provisioned idempotently by
`scripts/provision-test-fixtures.sh`; decoder fixture under `fixtures/mssql/`).

- **P1 fixed — cancellation/timeout (`75dd8854`)**: tiberius has no TDS attention
  signal, so cancel is `KILL <spid>` (session tracked via `fetch_spid` +
  `ActiveSqlServer` Drop-guard, mirroring the PG backend-pid pattern) plus
  connection rotation; `run_with_deadline` yields the canonical `DbError::QueryTimeout`.
  Live SQL Server suite 8/8 at that SHA.
- **P1 found and fixed — `explain` never worked (`24bae059`)**: the first live
  SHOWPLAN test proved the one-batch wrapper hit server error 1067 and the RPC
  path silently executed the query instead of explaining it. Rewritten as three
  separate `SET SHOWPLAN_TEXT` simple-query batches with session restore;
  parameterized EXPLAIN is now explicit `Unsupported` (no literal inlining).
- **Honest capabilities (`abd1d324`)**: `partitions`, `object_dependencies`,
  `sequences` flipped `true→false` with exact reason strings — the APIs behind
  them are PostgreSQL-only in this build.
- **Dialect-correct snippets (`e946773e`)**: explorer/table-data sample SQL emits
  `SELECT TOP (100)` on SQL Server instead of leaking `LIMIT 100`.
- **Conformance live (`24bae059`)**: the shared provider-contract decoder suite
  now parses `sqlserver://` targets and runs against the live fixture.

Measured at combined HEAD `24bae059` (2026-10-22):
`sqlserver_integration --include-ignored` **9/0**, `mysql_integration` **12/0**,
`pg_integration` **37/0**, `provider_contract_conformance` **13/0 for each of
PostgreSQL, MySQL, SQL Server targets**, workspace **2,244 / 0 / 62 ignored**,
clippy `--workspace --all-targets -- -D warnings` **0**, fmt PASS, ui-architecture
PASS, locked native release build PASS, clean-code diff-ratchet PASS.

Still open (keeps this plan `RUNTIME_VERIFY`, tracked out-of-plan):
native UI runtime captures of SQL Server surfaces at 1280×800/1440×900/1920×1080,
introspection completeness follow-ups (`sys.databases`/`sys.sequences`, trigger
timing/event/function body, routine parameter types, `SslMode::VerifyFull`,
`datetime2` sub-second precision, DB→DB transfer) — filed against #260 closure.
