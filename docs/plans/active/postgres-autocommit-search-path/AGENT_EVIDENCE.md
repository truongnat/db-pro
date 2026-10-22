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
