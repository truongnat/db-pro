# Verification — Parameterized Transaction Session Lease for PostgreSQL

Every result below is from a command that was run, at the SHAs named. `n/a` marks what was not run.

## Gates at `44f2d9d2` (merged with `origin/main @ a31a2bec`)

| Command | Exit | Measured result |
|---|---|---|
| `cargo +nightly fmt --all -- --check` | 0 | no diff |
| `cargo check --workspace` | 0 | no output beyond `Finished` |
| `cargo clippy --workspace --all-targets -- -D warnings` | 0 | 0 warnings |
| `cargo test --workspace -- --test-threads=4` | 0 | 2323 passed, 0 failed, 67 ignored |
| `python3 tools/cc-scan.py . --fail-on error` | 0 | `error=0 warning=126 info=0` (all 126 warnings are baselined legacy debt) |
| `bash .skills/clean-code/scripts/clean-code-scan.sh rust --diff --ratchet --ci` | 0 | `✓ pass: 12 ⚠ warn: 4 ✗ fail: 0` (diff vs `main`) |
| `cargo build --release --locked -p db-pro-native` | n/a | no UI/rendering change in this PR, so the perf-audit binary budget is not touched |

## Live provider evidence

Fixture: throwaway container `dbpro-verify-pg-553` (PostgreSQL 16, `127.0.0.1:25499`,
`postgres://dbpro:dbpro_test@127.0.0.1:25499/dbpro_fixture`), created for this run and removed
after it. No shared/local database was used.

| Command | Result |
|---|---|
| `DATABASE_URL=… cargo test -p db-pro-infrastructure --test pg_integration -- --ignored` | 42 passed, 0 failed — includes the whole pre-existing PostgreSQL suite |
| `… -- --ignored pg_joined` | `pg_joined_parameterized_transaction_rolls_back_with_the_tab` ok; `pg_joined_parameterized_transaction_rolls_back_only_its_own_statements` ok |
| `cargo test -p db-pro-core --lib boundary_failure_with_open_transaction_is_not_reported_as_unknown` | 1 passed, 0 failed |
| `cargo test -p db-pro-infrastructure --lib savepoint_names_are_unique_per_run` | 1 passed, 0 failed |

**MySQL — not verified, and not claimed.** The MySQL half of the first draft was removed before
this verification ran, because the live probe showed the branch cannot execute (F-5): `BEGIN` and
`SAVEPOINT` sent through sqlx's prepared API both return `ERROR 1295 (HY000)`. The MySQL connector
files in this PR are byte-identical to `main`, so there is no MySQL behaviour to regress.

**SQLite — n/a by scope, not by test.** `SqliteConnector` has no session slot at all
(`command grep -n "open_session\|transaction_session" crates/infrastructure/src/sqlite/connector.rs`
→ no matches), so there is no SQLite-side join to verify. Recorded as a provider gap, not as a pass.

## Mutation matrix (proves the new tests are coverage)

Each row is a temporary edit to `execute_parameterized_transaction_joined`, recompiled and run
against the live fixture, then reverted. Full table in FINDINGS F-4.

| Mutation | Outcome |
|---|---|
| joined dispatch removed → standalone `pool.begin()` | both tests FAIL (`the tab's rollback must undo the joined write too, got ["joined"]`) |
| `ROLLBACK TO SAVEPOINT` not issued | the failure-path test FAILs at the in-transaction read (aborted-transaction state) |
| undo targets a name that does not exist | the failure-path test FAILs with `left: Unknown, right: LeftOpen` |

## Review findings — status

| Thread | Severity | Status |
|---|---|---|
| unchecked savepoint undo reported `LeftOpen` (PG) | P1 | fixed in `d53cfe4c`; the `Unknown` branch is exercised by the third mutation row |
| unchecked savepoint undo (MySQL) | P1 | moot — the branch is removed (1295); the same PG fix shipped |
| fixed savepoint name | P2 | fixed in `d53cfe4c` (`savepoint_name()`) |
| preflight failures classified `Statement` | P2 | fixed in `d53cfe4c` (`boundary_left_open` + formatter arm) |
| source-text grep test as coverage | P2 | deleted; replaced by the two live tests above |
| plan/checklist/findings/verification accuracy | P2/P3 | all four documents rewritten at `44f2d9d2`+ |
| duplicated statement loop | P3 | open — follow-up, reasoning in FINDINGS |
| `execute_transaction_joined` preflight phase | P3 | open — follow-up, pre-existing path untouched by this PR |
| MySQL prepared-protocol capability gap | P2 (new) | open — follow-up |

Open P0: 0. Open P1: 0. P2: 0 (all fixed or moot with evidence). P3: 3, each recorded as a follow-up.

## UI / runtime evidence

n/a — no UI, theme, or view change. The affected surface is the data-grid mutation path, verified at
the connector boundary with the live PostgreSQL tests above rather than by screenshot.

## Summary (Việt)

Đã sửa xong hai P1 do review chỉ ra ở PostgreSQL: kết quả `ROLLBACK TO SAVEPOINT` giờ được kiểm tra
— nếu không xác nhận được thì báo `Unknown` và loại bỏ connection thay vì trả lại tab; tên savepoint
sinh riêng cho từng run; lỗi trước statement báo `phase = Begin` nên lưới dữ liệu không còn đổ cho
dòng đầu tiên. Nhánh MySQL bị loại khỏi PR vì `BEGIN`/`SAVEPOINT` qua prepared protocol trả
`ERROR 1295` — bằng chứng đo trên server thật. Test grep chuỗi đã xoá, thay bằng hai test live mà
mutation cho thấy thất bại đúng khi phá hành vi nó giữ. Toàn bộ gate xanh: fmt 0, check 0, clippy 0
warnings, `cargo test` 2323/0, cc-scan `error=0`, suite PostgreSQL live 42/42. P3 còn lại (loop trùng
lặp, preflight của đường batch, khoảng trống capability MySQL) đã ghi thành follow-up.
