# Multi-statement batches with transaction control — re-measured on the merged fix (#147)

- Session: issue-queue pass 6, 2026-09-15
- Issue: **#147** ([P1][RC1][Query] Decide and enforce multi-statement partial-write semantics)
- Fix commit: **`9d5bebaa`** (`fix(query): refuse multi-statement batches with transaction control
  (#147)`), merged into `main` as `f66d800d`; contract recorded in
  `docs/09-architecture-decisions.md` §5
- **Re-measurement base head:** `main @ d4df6373` (worktree clean)
- **Outcome:** the P1 truth-gap measured by pass 4 is closed on `main`. The exact batch that reported
  `RolledBack` while one row survived is now **refused before dispatch, with nothing written**. The
  measured behaviour matches the documented contract, so the issue's closure stands and is no longer
  an unverified one.

## 1. The gap this re-verifies

Pass 4 measured, against the live PostgreSQL fixture (`providers/51-multistatement-contract-criterion-audit.md`):

```text
execute_transaction(["BEGIN","INSERT …","COMMIT","SELECT no_such_column …"])
  → phase=Statement index=3 outcome=RolledBack,  1 row surviving
control (no explicit control)
  → outcome=RolledBack,  0 rows surviving
```

The envelope claimed a rollback that did not happen: the batch's own `COMMIT` ended the app-owned
transaction, and the statements after it ran outside the wrapper. The issue stayed open on a
**contract choice** (reject / honour / weaken the claim); `9d5bebaa` takes the first option — reject
transaction control inside a multi-statement batch, which fails closed.

## 2. Re-measurement on `main @ d4df6373` — the issue's own batch, through the product path

Temporary probe (`crates/infrastructure/tests/tmp_issue147_measurement.rs`), run against the live
PostgreSQL 16.15 fixture on `127.0.0.1:55432` (`dbpro_fixture`; password per the repo runbook,
redacted), **deleted before the pass-6 commits** — the tree carries no probe. The probe builds the
real product path (`QueryService` + real `PostgresConnector` + real `SQLiteMetaStore` + a registry
entry for the connection) and reads the table back through a second connection.

```text
$ DATABASE_URL=postgres://dbpro:<fixture-pw>@127.0.0.1:55432/dbpro_fixture \
  cargo test -p db-pro-infrastructure --test tmp_issue147_measurement -- --ignored --nocapture

running 1 test
PROBE product-path error_index=0 code=QUERY_FAILED message=`BEGIN` is not allowed inside a
  multi-statement batch: the batch runs in one transaction, so `BEGIN` would end that transaction
  and leave the statements before it committed. Remove the transaction control, or run each
  statement separately.
PROBE product-path results_len=0
PROBE product-path result_kinds_len=0
PROBE surviving_rows_after_product_path=0
PROBE control error_index=1 message=transaction rolled back: query syntax error: column
  "no_such_column" does not exist
PROBE control surviving_rows=0
test probe_exact_issue_batch_through_the_product_path ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s
```

The script measured is the issue's shape, semicolon-separated as the editor sends it:

```sql
BEGIN; INSERT INTO ms_tx_issue147 (id) VALUES (1); COMMIT; SELECT no_such_column FROM ms_tx_issue147
```

| | Before (`main @ 3889970`, pass 4) | **On `main @ d4df6373` (pass 6)** |
|---|---|---|
| Envelope | `RolledBack`, index 3 | **refused up front, index 0, naming `` `BEGIN` ``** |
| `results` returned | earlier results preserved | **empty — no statement ran** |
| Rows surviving in the table | **1** | **0** |
| Control batch without control (`INSERT; SELECT bad`) | `RolledBack`, 0 rows | `transaction rolled back: …`, **0 rows** (unchanged) |

The control case matters for the same reason it did in pass 4: the refusal replaced nothing — an
ordinary mutating batch still runs atomically and still reports the confirmed rollback.

## 3. Committed evidence (no probe needed)

| Command | Result |
|---|---|
| `cargo test -p db-pro-infrastructure --test pg_integration -- --ignored multi_statement` | **2 passed** — `pg_multi_statement_batch_with_commit_is_refused_and_writes_nothing` (product refusal, 0 rows), `pg_multi_statement_batch_without_transaction_control_still_rolls_back` |
| `cargo test -p db-pro-infrastructure --test pg_integration -- --ignored connector_alone` | **1 passed** — `pg_connector_alone_reports_rolled_back_for_a_batch_that_committed`, the characterisation test that keeps the *defect shape* pinned one layer down (connector unchanged by the fix) |
| `cargo test -p db-pro-infrastructure --test multistatement_transaction_control` | **4 passed** — the SQLite half, including `sqlite_batch_that_commits_itself_is_refused_and_writes_nothing` and `sqlite_connector_alone_still_lets_a_script_commit_itself` |
| `cargo test -p db-pro-core --lib transaction_control` / `safety` | the classifier (`transaction_control_verb`, `crates/core/src/domain/safety.rs:141`) is covered by the domain suite |

The dispatch rule itself: `crates/core/src/application/query_service.rs:226-256` — a batch with more
than one statement and any mutation is refused before dispatch when any statement carries
transaction control, with `results`/`result_kinds` empty and the offending index named. A single
statement is deliberately not covered (manual transaction control is `OUT_OF_SCOPE_V01` as #224).

## 4. What this does not claim

- **No live MySQL half.** The provider that carries the transaction-control rule in this pass is
  PostgreSQL and SQLite. MySQL is registered but cannot be reached from the UI (see #235's re-opened
  remainder), so no MySQL batch execution was measured.
- **Not an atomicity claim for arbitrary SQL.** The connector's own `execute_transaction` still
  documents that a script which commits itself escapes the wrapper; the fix is one layer up, by
  refusing such a batch, and `pg_connector_alone_reports_rolled_back_for_a_batch_that_committed`
  exists so that removing the refusal fails loudly.
- **No UI-frame evidence.** The refusal is a runtime/domain behaviour; nothing in this pass drove the
  egui surface for it, and no claim is made about its wording in the UI.
