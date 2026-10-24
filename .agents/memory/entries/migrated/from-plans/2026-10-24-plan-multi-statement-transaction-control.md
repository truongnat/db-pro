---
schema_version: 1
id: "MEM-PLAN-multi-statement-transaction-cont-22710b8bedff"
title: "Plan archive: Multi Statement Transaction Control"
type: decision
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, decision, multi-statement-transaction-control]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "22710b8bedff8e4b1ff0692399165fd3e888233252e1d811d578a4d95a8f6b6d"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/completed/multi-statement-transaction-control/PLAN.md"
  - "docs/plans/completed/multi-statement-transaction-control/FINDINGS.md"
  - "docs/plans/completed/multi-statement-transaction-control/VERIFICATION.md"
  - "docs/plans/completed/multi-statement-transaction-control/AGENT_EVIDENCE.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/completed/multi-statement-transaction-control/`. Loại suy luận: `decision`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

# Plan — #147: `execute_multi` must refuse a batch that carries its own transaction control

State: COMPLETED

## Problem

`QueryService::execute_multi` runs a batch inside one wrapper transaction. When a statement in the
batch carries its own `BEGIN` / `COMMIT` / `ROLLBACK`, the user's statement ends (or replaces) the
wrapper's transaction. On PostgreSQL the wrapper then reports `RolledBack` for the failing batch
while the earlier writes are actually committed — the envelope lies, and the product shows a
rollback that never happened.

## Measured defect (live, this branch)

PostgreSQL 16.15 (fixture `dbpro-v01-pg-fixture`, port 55432), probe measured on this branch:

| Shape | Failure index | Reported outcome | Rows surviving |
|---|---|---|---|
| `BEGIN; INSERT; COMMIT; SELECT bad` | 3 | `RolledBack` | **1 (the insert)** |
| `INSERT; COMMIT; SELECT bad` | 2 | `RolledBack` | **1 (the insert)** |
| `INSERT; SELECT bad` (control) | 1 | `RolledBack` | 0 (correct) |
| `INSERT; ROLLBACK; SELECT bad` | 2 | `RolledBack` | 0 (no harm) |

SQLite 3.x (same probe, measured on this branch):

| Shape | Failure index | Reported outcome | Rows surviving |
|---|---|---|---|
| `BEGIN; INSERT; COMMIT; SELECT bad` | 0 (`BEGIN` refused: "cannot start a transaction within a transaction") | `RolledBack` | 0 |
| `INSERT; COMMIT; SELECT bad` | 2 | `Unknown` ("cannot rollback - no transaction is active") | **1 (the insert)** |
| `INSERT; SELECT bad` (control) | 1 | `RolledBack` | 0 (correct) |
| `INSERT; ROLLBACK; SELECT bad` | 2 | `Unknown` | 0 (no harm) |

Both providers break the "a failed batch leaves nothing behind" contract whenever the script commits
itself; PostgreSQL additionally reports `RolledBack` for a batch that committed.

## Fix (fail-closed at dispatch)

`QueryService::execute_multi` refuses a batch whose statements carry transaction control, before
dispatching anything:

- `transaction_control_verb()` (core/domain/safety.rs) classifies a statement's leading verb as
  `BEGIN` / `COMMIT` / `ROLLBACK` / `SAVEPOINT` / `RELEASE`, using the same lexical rules as the
  existing `statement_classifies_as_mutation` (comments stripped, keywords case-insensitive, short
  quoted lexemes).
- The refusal is reported as a failed batch (`results` empty, `error` = statement index +
  message) and nothing is dispatched, so no partial state is possible by construction.
- Single statements are NOT refused (per #224, a single statement is a direct script execution
  surface) — only the multi-statement path whose contract is atomicity.
- The connector itself is unchanged; its `execute_transaction` documents the assumption instead
  (`crates/core/src/ports/db_connector.rs`).

## Tests

- Core unit tests (5 new): verb classification + dispatch refusal.
- `crates/infrastructure/tests/multistatement_transaction_control.rs` (4 live SQLite tests):
  product refusal, contract without control, and the two connector characterisations measured above.
- `crates/infrastructure/tests/pg_integration.rs` (3 new live PG tests, `#[ignore]`): product
  refusal, contract without control, connector characterisation (`RolledBack` + 1 surviving row).

## Out of scope

- Single-statement transaction control (direct execution path) — tracked by #224.
- Any change to connector transaction semantics.


## Source: `FINDINGS.md`

# Findings — #147

## F1 (P1, confirmed): `RolledBack` is reported for a batch that committed (PostgreSQL)

- Evidence: probe on this branch (PG 16.15 fixture), shapes A and B — `Err(phase=Statement,
  outcome=RolledBack)` with 1 row surviving.
- Why it matters: the product shows "rolled back" while the write is committed. A user who trusts
  the envelope retries an operation that already took effect, or deletes rows believing they are
  still there.
- Fingerprint (GH CLI, this branch): `gh issue view 147` — "and earlier writes actually survive"
  matches the measured shapes exactly.

## F2 (P1, confirmed): a batch can commit outside the wrapper transaction (both providers)

- PostgreSQL: any shape carrying `COMMIT` survives its writes (table above).
- SQLite: shape B (`INSERT; COMMIT; SELECT bad`) survives, and the envelope reports `Unknown`
  because the wrapper has nothing left to roll back.
- The harmful shape differs per provider (SQLite refuses a leading `BEGIN` itself), which is why
  the fix refuses the whole class at dispatch instead of special-casing a verb.

## F3 (accepted limitation): single statements are not covered

- Per #224, a single statement is a direct script execution surface (`execute`, not
  `execute_multi`). `BEGIN`, `COMMIT`, `ROLLBACK` typed alone keep working there. The multi-
statement path is the one whose contract is atomicity.

## F4 (root cause, from source)

- `QueryService::execute_multi` wraps all statements in one `execute_transaction` call and maps the
  connector envelope 1:1 into the UiEvent; the envelope's `RolledBack` therefore reaches the UI
  even when the wrapped transaction no longer exists at rollback time. The connector cannot detect
  this in general (its rollback is a no-op against a server that already committed), so the
  guarantee has to be created at dispatch — refuse the class, and the wrapper transaction always
  exists for the whole batch.


## Source: `VERIFICATION.md`

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


## Source: `AGENT_EVIDENCE.md`

# Agent evidence — #147 `execute_multi` refuses a batch that carries its own transaction control

## 1. Claim

| Field | Value |
|---|---|
| Agent identity | zcode · execution-safety lane (session continuation for #147) |
| Issue(s) | #147 (option 1: fail-closed rejection at dispatch); #224 untouched |
| Task state | In Progress → Ready for review |
| Baseline SHA | `a9244a28c13a7cf51302096662840e2479affbd5` — everything below is asserted against this tree |
| Branch / PR | `fix/multi-statement-transaction-control` (from `a9244a2`) · PR pending publish |
| Scope interpretation | refuse a multi-statement batch whose statements contain transaction control, before dispatch, so the documented all-or-nothing contract becomes true by construction |
| Out of scope | single-statement direct execution (#224); any change to `Connector::execute_transaction` semantics; the archived React frontend |

## 2. Progress checkpoint

- Current HEAD: `a9244a28c13a7cf51302096662840e2479affbd5` + staged #147 files (commit follows)
- Completed acceptance rows: [x] measured defect on both providers, [x] dispatch refusal, [x] live SQLite tests, [x] live PostgreSQL tests, [x] docs/09 §5 + audit matrix row, [x] probes removed
- Remaining acceptance rows: [ ] gates (fmt/check/clippy/test workspace/native release/perf-scan) on this exact tree, [ ] PR published
- Findings / risks: none open for this lane; see FINDINGS.md F1–F4 (all measured, all addressed)
- Tests already run (this session):
  - `cargo test -p db-pro-infrastructure --test multistatement_transaction_control` → 4 passed / 0 failed, exit 0
  - `cargo test -p db-pro-infrastructure --test pg_integration -- --ignored 'pg_multi_statement'` (DATABASE_URL=fixture) → 2 passed / 0 failed, exit 0
  - `... -- --ignored 'pg_connector_alone'` → 1 passed / 0 failed, exit 0
  - `cargo test -p db-pro-core` → 308 passed / 0 failed, exit 0 (post-restore of the core files)
- Dependency / blocker changes: the earlier blocker (foreign branch `fix/rc1-result-grid-selection-lookup-cache` carrying another session's UI work) is cleared — this branch was recreated from `a9244a2` and carries only #147 files. The #147 core/test work itself survived in checkpoint commit `0eb2382` (cline checkpoint) and was restored from there after the branch switch.

## 3. Implementation handoff / review request

| Field | Value |
|---|---|
| Exact SHA | filled after the commit lands (this table is updated in the same push) |
| Commit list | `fix(core): refuse a multi-statement batch that carries its own transaction control (#147)` |
| File / surface inventory | `crates/core/src/domain/safety.rs` — `transaction_control_verb()` + unit tests; `crates/core/src/application/query_service.rs` — dispatch guard in `execute_multi` + `transaction_control_rejection()` + tests; `crates/infrastructure/tests/multistatement_transaction_control.rs` — new, 4 live SQLite tests; `crates/infrastructure/tests/pg_integration.rs` — `pg_password()` helper + `product_path`/`count_cell` + 3 live PG tests; `docs/architecture/09-architecture-decisions.md` §5 — "what a batch may contain"; `docs/release/audit-execution-safety.md` — matrix row resolved; plan directory `docs/plans/active/multi-statement-transaction-control/` |
| Acceptance mapping | "earlier writes survive a reported rollback" → `pg_connector_alone_reports_rolled_back_for_a_batch_that_committed` (pins the defect) + `pg_multi_statement_batch_with_commit_is_refused_and_writes_nothing` (proves the refusal); SQLite equivalents in `multistatement_transaction_control.rs`; classification unit tests in `safety.rs` |
| Commands and counts | the four test commands in §2 plus the gate commands recorded in VERIFICATION.md as they actually run |
| CI run IDs / status | not run |
| Known limitations | single statements are deliberately not refused (#224); the connector's `execute_transaction` still trusts its documented no-transaction-control assumption — now enforced one layer up |
| Migrations / config implications | none |
| Out-of-scope changes | none — `crates/ui/*` changes that appeared in the shared working tree belong to the #246 session and are NOT part of this branch |

## 4. Review outcome — pending external review (VPS/Kilo reviewer after PR publish)

## 5. Research / audit handoff

- Source date: 2026-09-15
- References: issue #147; `docs/release/evidence/v01-runtime/providers/51-multistatement-contract-criterion-audit.md`; `docs/architecture/09-architecture-decisions.md` §5; `docs/release/audit-execution-safety.md` §2 row 12 / §3; `crates/core/src/ports/db_connector.rs` (`execute_transaction` contract); measured probes on both providers (tables in PLAN.md)
- Factual findings: F1–F4 in FINDINGS.md, each with a live measurement behind it
- Inference: the dispatch refusal is the smallest place that restores the contract for both providers without inventing connector-level rollback semantics (F4)
- Decision / recommendation: option (1) from the issue — implemented
- Unresolved questions: none for this lane
- Downstream tasks activated: none

## 6. Tổng kết

Đã sửa #147 theo phương án (1): `execute_multi` từ chối batch chứa `BEGIN/COMMIT/ROLLBACK/SAVEPOINT/RELEASE` trước khi dispatch (`transaction_control_verb` + guard + 5 unit test). Defect được đo trực tiếp trên PostgreSQL 16 fixture (`RolledBack` với 1 row sống sót) và SQLite (`Unknown` với 1 row sống sót; `BEGIN` leading bị SQLite tự chặn) — pin bằng 4 test SQLite live + 3 test PG live, kèm characterisation test giữ nguyên hành vi connector làm bằng chứng gốc. Docs 09 §5 và audit matrix đã cập nhật; probe tạm đã xóa; branch sạch từ `a9244a2`, chỉ chứa file #147. Còn lại: chạy đủ gates + publish PR.

