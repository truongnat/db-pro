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
| File / surface inventory | `crates/core/src/domain/safety.rs` — `transaction_control_verb()` + unit tests; `crates/core/src/application/query_service.rs` — dispatch guard in `execute_multi` + `transaction_control_rejection()` + tests; `crates/infrastructure/tests/multistatement_transaction_control.rs` — new, 4 live SQLite tests; `crates/infrastructure/tests/pg_integration.rs` — `pg_password()` helper + `product_path`/`count_cell` + 3 live PG tests; `docs/09-architecture-decisions.md` §5 — "what a batch may contain"; `docs/release/audit-execution-safety.md` — matrix row resolved; plan directory `docs/plans/active/multi-statement-transaction-control/` |
| Acceptance mapping | "earlier writes survive a reported rollback" → `pg_connector_alone_reports_rolled_back_for_a_batch_that_committed` (pins the defect) + `pg_multi_statement_batch_with_commit_is_refused_and_writes_nothing` (proves the refusal); SQLite equivalents in `multistatement_transaction_control.rs`; classification unit tests in `safety.rs` |
| Commands and counts | the four test commands in §2 plus the gate commands recorded in VERIFICATION.md as they actually run |
| CI run IDs / status | not run |
| Known limitations | single statements are deliberately not refused (#224); the connector's `execute_transaction` still trusts its documented no-transaction-control assumption — now enforced one layer up |
| Migrations / config implications | none |
| Out-of-scope changes | none — `crates/ui/*` changes that appeared in the shared working tree belong to the #246 session and are NOT part of this branch |

## 4. Review outcome — pending external review (VPS/Kilo reviewer after PR publish)

## 5. Research / audit handoff

- Source date: 2026-09-15
- References: issue #147; `docs/release/evidence/v01-runtime/providers/51-multistatement-contract-criterion-audit.md`; `docs/09-architecture-decisions.md` §5; `docs/release/audit-execution-safety.md` §2 row 12 / §3; `crates/core/src/ports/db_connector.rs` (`execute_transaction` contract); measured probes on both providers (tables in PLAN.md)
- Factual findings: F1–F4 in FINDINGS.md, each with a live measurement behind it
- Inference: the dispatch refusal is the smallest place that restores the contract for both providers without inventing connector-level rollback semantics (F4)
- Decision / recommendation: option (1) from the issue — implemented
- Unresolved questions: none for this lane
- Downstream tasks activated: none

## 6. Tổng kết

Đã sửa #147 theo phương án (1): `execute_multi` từ chối batch chứa `BEGIN/COMMIT/ROLLBACK/SAVEPOINT/RELEASE` trước khi dispatch (`transaction_control_verb` + guard + 5 unit test). Defect được đo trực tiếp trên PostgreSQL 16 fixture (`RolledBack` với 1 row sống sót) và SQLite (`Unknown` với 1 row sống sót; `BEGIN` leading bị SQLite tự chặn) — pin bằng 4 test SQLite live + 3 test PG live, kèm characterisation test giữ nguyên hành vi connector làm bằng chứng gốc. Docs 09 §5 và audit matrix đã cập nhật; probe tạm đã xóa; branch sạch từ `a9244a2`, chỉ chứa file #147. Còn lại: chạy đủ gates + publish PR.
