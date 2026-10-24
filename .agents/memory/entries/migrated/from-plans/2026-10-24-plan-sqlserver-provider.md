---
schema_version: 1
id: "MEM-PLAN-sqlserver-provider-717a8a25c8a2"
title: "Plan archive: Sqlserver Provider"
type: decision
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, decision, sqlserver-provider]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "717a8a25c8a2795c7baeb0060e541a6c72fab9b3d72e73357023a3cf49bad7ba"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/sqlserver-provider/PLAN.md"
  - "docs/plans/active/sqlserver-provider/FINDINGS.md"
  - "docs/plans/active/sqlserver-provider/VERIFICATION.md"
  - "docs/plans/active/sqlserver-provider/AGENT_EVIDENCE.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/sqlserver-provider/`. Loại suy luận: `decision`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

# SQL Server provider (#260)

## State

`RUNTIME_VERIFY`

## Goal

Add a real SQL Server provider adapter behind the existing native provider seam so
connection, query, canonical value decoding, schema introspection, table data
operations, capability gating, and SQL Server identifier/parameter syntax are
implemented by the provider rather than by UI branches.

## Evidence baseline

- Baseline: `cc399501338b7f38342a1bb12b9990e048537312`
- GitHub issue: #260
- Dependency: #234 provider contract is present in the codebase, but its live
  acceptance status is not assumed from historical release notes.
- Existing gap: `DriverType` and `CompositeConnector` have no SQL Server arm;
  workspace `sqlx` has no TDS feature.
- Implementation SHA: `774a4e7f6ce8d88939988711e8fadf0588f1c3c1`.

## Scope

- Add a SQL Server driver identity and provider factory.
- Add a TDS-backed connector using the existing `DbConnector` port.
- Implement typed query parameters and canonical row decoding without lossy
  conversion for decimal, bigint, uniqueidentifier, datetimeoffset, binary, or
  JSON-like values.
- Implement SQL Server dialect quoting and parameter placeholders.
- Implement schema/table/column/PK/FK/index/check/trigger/view/routine
  introspection through SQL Server catalogs.
- Reuse the existing runtime/UI provider dispatch and capability model.
- Add focused unit/conformance coverage and a fixture-driven integration lane.

## Non-goals

- PL/pgSQL debugging (#252).
- SQL Server-specific administration, backup/restore, or security workspaces
  beyond explicit capability refusals.
- Claiming issue completion without a live SQL Server fixture and native UI
  evidence where the issue requires them.

## Acceptance criteria

1. SQL Server connection lifecycle and representative query execution work.
2. Query cancellation is either implemented safely or capability-gated with a
   user-facing reason.
3. Canonical values preserve exact decimal/bigint/uniqueidentifier/
   datetimeoffset/binary semantics.
4. Catalog introspection returns stable schemas, tables, columns, keys, indexes,
   checks, triggers, views, and routines where supported.
5. Table reads and mutations use the shared safety/change-set path and SQL Server
   dialect.
6. Unsupported PostgreSQL-only features are capability-gated with explicit
   reasons.
7. Provider/unit/conformance tests pass, and a live SQL Server fixture proves the
   supported matrix.

## Completion gate

The code and automated gates are complete at
`774a4e7f6ce8d88939988711e8fadf0588f1c3c1`. The plan remains under
`docs/plans/active/` because the environment has no live SQL Server fixture and
no native UI screenshots/runtime recording at the required sizes. The issue must
not be marked completed until those evidence lanes run.


## Source: `FINDINGS.md`

# SQL Server provider findings (#260)

## Baseline findings

| Severity | Finding | Evidence | Disposition |
|---|---|---|---|
| P1 | There is no SQL Server provider arm in the core driver enum or composite connector. | `cc399501338b7f38342a1bb12b9990e048537312`; `crates/core/src/domain/connection.rs`, `crates/infrastructure/src/connector.rs` | Resolved in `774a4e7f6ce8d88939988711e8fadf0588f1c3c1`. |
| P1 | The workspace has no TDS driver dependency, so a UI-only or enum-only change would be non-functional. | `cc399501338b7f38342a1bb12b9990e048537312`; root `Cargo.toml` | Resolved in `774a4e7f6ce8d88939988711e8fadf0588f1c3c1` with Tiberius/Tokio compat. |
| P1 | Issue acceptance requires live SQL Server evidence, but no SQL Server fixture is currently running in the environment. | GitHub #260 implementation-audit comments; `docker ps` on 2026-09-17 | Open; fixture lane required. |
| P2 | SQL Server-specific administration and PostgreSQL-only features need explicit capability reasons rather than generic errors. | `cc399501338b7f38342a1bb12b9990e048537312`; `crates/core/src/domain/capabilities.rs` | Resolved in `774a4e7f6ce8d88939988711e8fadf0588f1c3c1`; automated capability tests pass. |

## Review findings

No independent review has run yet. Self-review at `774a4e7f6ce8d88939988711e8fadf0588f1c3c1` found no P0/P1 regression; live fixture and native UI evidence remain blocked.


## Source: `VERIFICATION.md`

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


## Source: `AGENT_EVIDENCE.md`

# Agent evidence — #260 SQL Server provider

## 1. Claim

| Field | Value |
|---|---|
| Agent identity | codex · SQL Server provider implementation lane |
| Issue(s) | #260 |
| Task state | Blocked |
| Baseline SHA | `cc399501338b7f38342a1bb12b9990e048537312` |
| Branch / PR | `fix/sqlserver-provider` · PR not opened |
| Scope interpretation | Build a real TDS-backed provider through the existing `DbConnector` and capability seams, with tests and fixture evidence. |
| Out of scope | #252 debugger, SQL Server administration/backup/security workspaces, issue completion without live fixture/UI evidence. |

## 2. Progress checkpoint

- Current HEAD: `774a4e7f6ce8d88939988711e8fadf0588f1c3c1`
- Completed acceptance rows: provider identity/factory, TDS lifecycle, query/execute/transactions, typed mapping, dialect, capabilities, catalog introspection, UI selection, unit/fixture test lane.
- Remaining acceptance rows: live SQL Server fixture and native UI/runtime evidence.
- Findings / risks: live SQL Server fixture absent; cancellation is explicitly capability-gated; independent review has not run.
- Tests already run: workspace 1164 passed/0 failed/42 ignored; fmt, check, clippy, release native build, clean-code scan, and SQL Server unit tests passed.
- Dependency / blocker changes: #234 provider seams are consumed; #260 cannot be marked complete until `SQLSERVER_URL` fixture evidence is collected.

## 3. Implementation handoff / review request

Ready for review with runtime-verification blocker recorded; do not mark #260 completed yet.

## 4. Review outcome

No independent review has run. Self-review verdict: ACCEPT WITH P2 / BLOCKED runtime evidence; P0=0, P1=0 introduced, P2=1 (missing live fixture/UI evidence).

## 5. Research / audit handoff

- Source date: 2026-09-17.
- Source URLs / references: GitHub issue #260 and its implementation-audit comments; `crates/core/src/ports/db_connector.rs`; `crates/infrastructure/src/connector.rs`; `Cargo.toml`.
- Factual findings: implementation SHA `774a4e7f6ce8d88939988711e8fadf0588f1c3c1` adds the SQL Server driver, TDS adapter, connector, introspection, capability matrix, UI path, and ignored fixture tests; no live SQL Server container is available.
- Inference: tiberius is a viable candidate adapter, but TLS/authentication and live fixture compatibility remain to be proven in this repository.
- Decision / recommendation: keep the plan in `RUNTIME_VERIFY`; run the four ignored tests plus native UI evidence against the canonical fixture before closing #260.
- Unresolved questions: which SQL Server image/version and authentication mode should be the canonical CI fixture; whether cancellation can be safely supported by the selected TDS client.
- Downstream tasks activated: none.

## 6. Tổng kết (Vietnamese summary)

Đã triển khai provider SQL Server thật trên commit `774a4e7f6ce8d88939988711e8fadf0588f1c3c1`, gồm TDS lifecycle, query/transaction, typed decoding, introspection, capability gates, UI selection và integration lane. Tất cả gate tự động đã pass; issue vẫn chưa hoàn tất vì môi trường chưa có SQL Server live fixture và native UI evidence.

