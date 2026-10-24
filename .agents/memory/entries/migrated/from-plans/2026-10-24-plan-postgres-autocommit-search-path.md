---
schema_version: 1
id: "MEM-PLAN-postgres-autocommit-search-path-6489c25ebceb"
title: "Plan archive: Postgres Autocommit Search Path"
type: decision
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, decision, postgres-autocommit-search-path]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "6489c25ebceb113dffefce51fff97beea132d91c5b4e95decbe2f7acef36d4fd"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/postgres-autocommit-search-path/PLAN.md"
  - "docs/plans/active/postgres-autocommit-search-path/FINDINGS.md"
  - "docs/plans/active/postgres-autocommit-search-path/VERIFICATION.md"
  - "docs/plans/active/postgres-autocommit-search-path/AGENT_EVIDENCE.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/postgres-autocommit-search-path/`. Loại suy luận: `decision`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

# Postgres autocommit search_path — PLAN

- Status: REVIEW (backend fix verified live on PostgreSQL; app UAT pending)
- Reported: with tenant `tenant1` selected in the query tab, `SELECT * FROM m_ctrl_universal_value;`
  failed with `relation "m_ctrl_universal_value" does not exist`.
- Root cause: `push_session_search_path` installed the editor schema with
  `set_config('search_path', ..., is_local = true)` unconditionally. PostgreSQL applies a local
  setting only to the current transaction; outside one it expires at the end of the `set_config`
  statement, so the following `query` / `execute` / `explain` still resolved names against the old
  path. The unconditional local scope came from #456 (`424258ce`), which needed it to stop
  search_path leakage when a timed-out future is dropped before the restore.
- Fix:
  - `push_session_search_path` / `pop_session_search_path` take the connection's transaction state:
    `is_local = true` inside a held tab transaction, `false` on an autocommit connection
    (`crates/infrastructure/src/postgres/search_path.rs:18`, `:51`).
  - An autocommit run that installed a session-wide path arms
    `SessionLease::discard_if_abandoned()` (`crates/infrastructure/src/postgres/connector.rs:176`,
    the shared `begin_scoped_run`; EXPLAIN at `:1014`); a completed restore calls `allow_reuse()`
    (`:193`). A future dropped between push and
    pop closes the connection through `close_on_drop` instead of returning the leaked path to the
    pool (`crates/infrastructure/src/transaction_session.rs:101`-`:117`), preserving #456.
  - Batch and joined transaction paths stay transaction-local (`connector.rs:217`, `:280`).
- Providers: PostgreSQL only. SQLite, MySQL and SQL Server paths are untouched.
- Out of scope: the explorer-vs-document schema fallback in run preparation still sends the
  document schema only; the query-tab picker sets that document schema, so this report is fixed.


## Source: `FINDINGS.md`

# Postgres autocommit search_path — FINDINGS

- Severity: P1 — provider-specific correctness, major user flow broken. Any unqualified table in
  the selected tenant schema failed on PostgreSQL; qualified names still worked, which hides the
  regression from tests that qualify.
- Evidence:
  - Live PostgreSQL reproduction (`set_config(..., true)` in autocommit):
    the next statement reported `"$user", public` as `current_setting('search_path')` and
    `SELECT * FROM m_ctrl_universal_value` failed; the same sequence with `is_local = false`
    returned the tenant row.
  - Connector-level regression test (added, failed before the fix):
    `pg_autocommit_run_resolves_unqualified_tables_in_active_query_schema` failed with
    `QuerySyntaxAt { message: "relation \"scoped_values\" does not exist", position: 13 }`.
- Failure scenario: query tab selects `tenant1`; the run command carries `schema: Some("tenant1")`;
  the connector installs it locally in autocommit; PostgreSQL discards it with the installing
  statement; `SELECT * FROM m_ctrl_universal_value` resolves against `"$user", public` and fails.
- Scope: `crates/infrastructure/src/postgres/search_path.rs` and the query / execute / explain /
  joined-batch call sites in `crates/infrastructure/src/postgres/connector.rs`, plus the session
  lease in `crates/infrastructure/src/transaction_session.rs`.
- Existing coverage: the parameterized-transaction test only exercised an explicit transaction,
  where `SET LOCAL` is correct; no test covered an autocommit query with a schema. The app already
  asserts that the tenant schema is dispatched (`app_tests.rs` tenant RunQuery test), so the loss
  was provider-side.
- Provider impact: PostgreSQL fixed and verified live. SQLite (no schemas beyond `main`), MySQL
  (database-qualified) and SQL Server (default schema) are unaffected by this code path.
- Runtime testability: three live regression tests on the local PostgreSQL 18 fixture; plus unit
  tests for the abandoned/restored lease. Native-app UAT against a real tenant database remains.


## Source: `VERIFICATION.md`

# Postgres autocommit search_path — VERIFICATION

- Verified against the uncommitted tree on HEAD `ebba5ca74590abf50c0c3f72daf3e7248ff90755`; the
  exact code-snapshot SHA is recorded in `AGENT_EVIDENCE.md` §1 (plan docs are not part of it).
- Files:
  - `crates/infrastructure/src/postgres/search_path.rs`
  - `crates/infrastructure/src/postgres/connector.rs`
  - `crates/infrastructure/src/transaction_session.rs`
  - `crates/infrastructure/src/tests/transaction_session_tests.rs`
  - `crates/infrastructure/tests/pg_integration.rs`

## Live PostgreSQL regression evidence

Fixture: local PostgreSQL 18, `DATABASE_URL=postgres://postgres:postgres@127.0.0.1:5432/postgres`.

| Test | Before fix | After fix |
|------|-----------|-----------|
| `pg_autocommit_run_resolves_unqualified_tables_in_active_query_schema` | FAIL — `relation "scoped_values" does not exist` | PASS |
| `pg_transaction_run_resolves_unqualified_tables_in_active_query_schema` | n/a (new) | PASS |
| `pg_autocommit_timeout_discards_the_schema_scoped_connection` | n/a (new) | PASS |

Commands (all exit 0):
- `DATABASE_URL=... cargo test -p db-pro-infrastructure --test pg_integration run_resolves -- --ignored`
  → 2 passed / 0 failed, 38 filtered out.
- `DATABASE_URL=... cargo test -p db-pro-infrastructure --test pg_integration pg_autocommit_timeout_discards_the_schema_scoped_connection -- --ignored`
  → 1 passed / 0 failed (4.09 s).
- `cargo test -p db-pro-infrastructure --lib transaction_session` → 7 passed / 0 failed, including
  `abandoned_session_scoped_run_discards_the_connection` and
  `completed_session_scoped_run_keeps_the_connection`.

## Workspace gates

| Gate | Result |
|------|--------|
| `cargo +nightly fmt --all -- --check` | PASS (exit 0) |
| `cargo check --workspace` | PASS (exit 0) |
| `cargo clippy --workspace --all-targets -- -D warnings` | PASS (exit 0) |
| `cargo test --workspace` | PASS — 2243 passed / 0 failed / 65 ignored across 27 test binaries (exit 0) |
| `python3 tools/cc-scan.py crates/infrastructure/src/postgres/connector.rs crates/infrastructure/src/postgres/search_path.rs crates/infrastructure/src/transaction_session.rs --fail-on error` | FAIL — 1 inherited error: `execute_batch` is 76 lines (unchanged by this work; the same finding is present at HEAD). No new findings from this change. |

## Limitations

- Native-app UAT (tenant picker → RunQuery → runtime → connector against a real tenant database)
  is not yet recorded; the connector boundary and the app dispatch schema are verified separately.
- SQLite/MySQL/SQL Server were not re-run for this change; their code paths are untouched.
- `execute_batch` carries a pre-existing clean-code `HUGE_FUNCTION` error at HEAD; it was not
  touched to keep this fix scoped.



## Source: `AGENT_EVIDENCE.md`

# Agent evidence — Postgres tenant schema (autocommit search_path)

## 1. Claim

| Field | Value |
|---|---|
| Agent identity | qoder · postgres provider lane |
| Issue(s) | user report (no issue number): tenant selected in the query tab, `SELECT * FROM m_ctrl_universal_value;` failed with `relation "m_ctrl_universal_value" does not exist` |
| Task state | `Review` |
| Baseline SHA | `ebba5ca74590abf50c0c3f72daf3e7248ff90755` |
| Code snapshot SHA | tree `2ad4e7a9c98a6adcfe56445dc63df80006b9642c` — exact and reproducible (parent `ebba5ca7`, contains only the working-tree changes under `crates/`; built with `git commit-tree` from a temporary index, branch not moved). Handoff commit object wrapping that tree: `4e7c96b3634ab51698338b9a826d4e002e620b39` |
| Branch / PR | `main` · single commit `fix(postgres): resolve unqualified tables in the active tenant schema` · no PR |
| Scope interpretation | Fix the tenant-schema query failure on PostgreSQL by making the connector's `search_path` push/pop follow the connection's transaction state, without reopening the #456 leak. |
| Out of scope | explorer-vs-document schema fallback in run preparation; refactoring the inherited `execute_batch` huge function; native-app UAT recording; SQLite/MySQL/SQL Server paths |

## 2. Progress checkpoint

- Current HEAD: `ebba5ca74590abf50c0c3f72daf3e7248ff90755`
- Completed:
  - [x] Root cause proven live (`is_local = true` expires with the `set_config` statement in autocommit)
  - [x] Push/pop scoped by connection transaction state (`search_path.rs:18`, `:51`)
  - [x] #456 preserved: autocommit runs discard the connection when abandoned before the restore
  - [x] Connector regression tests: autocommit, held transaction, timeout leak
  - [x] Shared `begin_scoped_run` / `finish_scoped_run` helpers remove the duplicated preamble/restore
- Remaining: native-app UAT (tenant picker → RunQuery → connector against a real tenant database).
- Findings / risks:
  - P1 (fixed): autocommit schema run resolved unqualified names against `"$user", public`.
  - P2 (inherited, not introduced): `execute_batch` is 76 lines and trips the clean-code
    `HUGE_FUNCTION` error at HEAD (`connector.rs:537` after this change; unchanged code).
- Tests already run:
  - `DATABASE_URL=... cargo test -p db-pro-infrastructure --test pg_integration run_resolves -- --ignored` → 2 passed / 0 failed / 38 filtered, exit 0
  - `DATABASE_URL=... cargo test -p db-pro-infrastructure --test pg_integration pg_autocommit_timeout_discards_the_schema_scoped_connection -- --ignored` → 1 passed / 0 failed, exit 0
  - `cargo test -p db-pro-infrastructure --lib transaction_session` → 7 passed / 0 failed, exit 0
  - `cargo test --workspace` → 2243 passed / 0 failed / 65 ignored across 27 binaries, exit 0
- Dependency / blocker changes: none.

## 3. Implementation handoff / review request

| Field | Value |
|---|---|
| Exact SHA | tree `2ad4e7a9c98a6adcfe56445dc63df80006b9642c` (code snapshot; see §1) |
| Commit list | `fix(postgres): resolve unqualified tables in the active tenant schema` (this handoff commit) |
| File / surface inventory | `crates/infrastructure/src/postgres/search_path.rs` — push/pop take `in_transaction` and bind `is_local`; `crates/infrastructure/src/postgres/connector.rs` — `begin_scoped_run` (`:159`) arms the lease for autocommit (`:176`), `finish_scoped_run` (`:182`) restores and allows reuse (`:193`); `query` (`:452`), `execute` (`:488`), `explain` (`:1009`) use them / the same contract; batch paths pass transaction state (`:217`, `:280`); `crates/infrastructure/src/transaction_session.rs` — `DiscardOnDrop`, `discard_if_abandoned` (`:101`), `allow_reuse` (`:105`), close on abandoned drop (`:115`); `crates/infrastructure/src/tests/transaction_session_tests.rs` — 2 lease tests; `crates/infrastructure/tests/pg_integration.rs` — 3 live tests (`:729`, `:782`, `:838`) |
| Acceptance mapping | unqualified query in tenant schema works in autocommit → `pg_autocommit_run_resolves_unqualified_tables_in_active_query_schema`; held transaction keeps `SET LOCAL` → `pg_transaction_run_resolves_unqualified_tables_in_active_query_schema`; timeout/cancel leak closed → `pg_autocommit_timeout_discards_the_schema_scoped_connection` + `abandoned_session_scoped_run_discards_the_connection` |
| Commands and counts | see §2; gates: `cargo +nightly fmt --all -- --check` exit 0, `cargo check --workspace` exit 0, `cargo clippy --workspace --all-targets -- -D warnings` exit 0, `cargo test --workspace` exit 0 (2243/0/65) |
| CI run IDs / status | not run |
| Known limitations | native-app UAT pending; `execute_batch` inherited clean-code error; other providers untouched |
| Migrations / config implications | none |
| Out-of-scope changes | none |

## 4. Review outcome

n/a — pending independent review. The implementer's own verification is §2/§3, not an approval.

## 5. Research / audit handoff

n/a — implementation lane.

## 6. Tổng kết (Vietnamese summary)

Đã sửa lỗi tenant schema trên PostgreSQL: `search_path` trước đây luôn được đặt bằng `SET LOCAL`
(`is_local = true`), nên ở chế độ autocommit nó hết hiệu lực ngay sau câu `set_config` và
`SELECT * FROM m_ctrl_universal_value` vẫn tìm trong `"$user", public`. Nay push/pop bám theo trạng
thái transaction của connection: trong transaction giữ `SET LOCAL`, ngoài transaction đặt session-wide
rồi khôi phục; nếu future bị bỏ dở trước khi khôi phục, connection bị đóng thay vì trả về pool (giữ
nguyên bảo vệ của #456). Bằng chứng: 3 test PostgreSQL live pass (gồm timeout 4.09 s), 7 test lease
pass, toàn bộ workspace 2243 passed / 0 failed / 65 ignored. Còn lại: UAT trên app native với database
tenant thật; lỗi clean-code `execute_batch` là tồn tại từ HEAD, không do thay đổi này.

