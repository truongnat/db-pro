# Findings — Parameterized Transaction Session Lease for PostgreSQL

Verified against a disposable fixture (PostgreSQL 16 container on `127.0.0.1:25499`, MySQL 8
container on `127.0.0.1:23399`), at base `5aee61bb` and at the first implementation `9a024d03`.

## F-1 — the joined path issued an unchecked savepoint undo (P1, external review, fixed)

`9a024d03:crates/infrastructure/src/postgres/connector.rs` ran
`let _ = sqlx::query("ROLLBACK TO SAVEPOINT dbpro_parameterized_tx").execute(…)` and returned
`outcome = LeftOpen` regardless of the result.

Failure scenario: the undo itself can fail (connection dropped, or the server is already in an
unrecoverable state). The run then reports "statement failed; the open transaction is still held" —
`query_service::format_transaction_failure` prints exactly that for `(Statement, LeftOpen)` — while
nothing has been verified about the transaction. The tab keeps using a connection whose state is
unknown, and `multi_query_execution.rs:76` only invalidates the schema cache on `Unknown`, so the
stale-UI guard is skipped too.

Fix: the undo result is inspected. On failure the outcome becomes `Unknown`, the message is
`<original error>; savepoint rollback failed: <rollback error>`, and `lease.discard_if_abandoned()`
keeps the connection out of the tab's slot. This matches the house precedent in the batch path
(`match rollback { Ok(()) => RolledBack, Err(_) => Unknown }`).

## F-2 — a fixed savepoint name (P2, external review, fixed)

`SAVEPOINT dbpro_parameterized_tx` / `RELEASE dbpro_parameterized_tx` reused one literal name for
every run on every connection.

Two runs can be open through the same tab transaction, and a caller may hold its own savepoint
across a run. `RELEASE` targets a name, not an occurrence, so the second run's release drops the
outer savepoint that shares the name.

Measured, not assumed: a psql probe on PostgreSQL 16 showed that duplicate savepoint names
*shadow* — `RELEASE` of the inner copy reveals the outer one, so PG is not corrupted by the reuse.
MySQL 8 behaves differently (a duplicate `SAVEPOINT x` discards the previous one, and `ROLLBACK TO
SAVEPOINT x` on a released name errors with 1305). The name is therefore generated per run
(`transaction_session::savepoint_name()` → `dbpro_tx_<seq>`), which is correct by construction on
both semantics and holds no user-visible identifiers (the value never comes from input).

**Test that was removed rather than shipped:** a PG test asserting a user savepoint survives a
joined run passed identically with and without the fix — because PG shadowing makes the case
non-discriminating. Keeping it would have been false coverage of the defect it claims to guard.
Replaced by a unit test on the name generator plus the live joined-run regression tests.

## F-3 — a setup failure was blamed on the first mutation (P2, external review, fixed)

The joined path reported its preflight failures — `SELECT pg_backend_pid()`, the `search_path`
push, and the `SAVEPOINT` itself — through `left_open(0, …)`, i.e. `phase = Statement` with
`statement_index = 0`.

That is not cosmetic. `TableMutationExecution` (`crates/core/src/application/table_mutation_execution.rs:68`)
rewrites a `Statement` index through `indexed_mutations`, so a connection-level failure was
attributed to the first row of the user's grid, and the row was marked as the one that failed
although it never ran.

Fix: a `boundary_left_open()` constructor for pre-statement failures — `phase = Begin`,
`statement_index = 0`, `outcome = LeftOpen` (the caller's transaction genuinely is still open).
`query_service::format_transaction_failure` had no arm for that pair and would have printed
"final outcome is unknown", so the arm was added with a unit test
(`boundary_failure_with_open_transaction_is_not_reported_as_unknown`).

The pre-existing `execute_transaction_joined` in the same file has the same shape for its own
preflight; it is outside this fix and is listed under follow-ups rather than changed here.

## F-4 — coverage that did not cover anything (caught by mutation, fixed)

The first draft of this PR shipped its "coverage" as a source-text assertion —
`owned_execute_parameterized_transaction_checks_session_lease` grepped `connector.rs` for the
strings `checkout_if_held` and `execute_parameterized_transaction_joined`. It passed on code that
never dispatches (flagged as P2 in review) and was deleted.

Replacing it with a live PG test exposed a second, worse problem: **the first live test also passed
with the join disabled.** Two causes, both measurement errors on my side, not code errors:

- an instrumented probe was inserted at the first `let in_transaction = lease.was_held();` in the
  file, which is in `execute_batch_joined`, so the joined parameterized path looked unreached;
- the mutation anchored on `if let Some(lease) = joined {`, which occurs twice (the batch path and
  the parameterized path), so the "join disabled" build still ran the joined code.

Both tests were then re-run against a mutation that drops the joined dispatch for the parameterized
method only. Measured matrix, all on the live fixture:

| Mutation | `rolls_back_with_the_tab` | `rolls_back_only_its_own_statements` |
|---|---|---|
| none (shipped code) | ok | ok |
| joined dispatch removed → standalone `pool.begin()` | **FAIL** — `the tab's rollback must undo the joined write too, got ["joined"]` | **FAIL** at the in-transaction read |
| `ROLLBACK TO SAVEPOINT` not issued | ok | **FAIL** at the in-transaction read (25P02, aborted transaction) |
| undo targets a name that does not exist | ok | **FAIL** with `left: Unknown, right: LeftOpen` |

The last row is the F-1 branch exercised end-to-end: an undo that cannot be confirmed is reported
`Unknown` instead of `LeftOpen`. No test is claimed as coverage unless a mutation of the code it
guards makes it fail.

## F-5 — MySQL cannot execute this branch at all (P1-for-the-PR, removed from scope)

The same joined implementation was written for `MySqlConnector` (the diff that first landed) and
driven against a live MySQL 8 server:

```text
sqlx::query("BEGIN").execute(&mut *conn)
  → ERROR 1295 (HY000): This command is not supported in the prepared statement protocol yet
sqlx::query("SAVEPOINT dbpro_parameterized_tx").execute(&mut *conn)
  → ERROR 1295 (HY000): same
```

`sqlx::query(...).execute()` uses the prepared protocol, so the MySQL joined branch is
unreachable-by-construction: reaching it requires the tab to have opened a transaction, and the
statement that would open it is the statement MySQL refuses here. Probes confirmed the text
protocol does accept both, and swapping to `sqlx::raw_sql(...)` inside the `#[async_trait]`
`DbConnector` method does not compile ("implementation of `Executor` is not general enough").

Decision: drop the MySQL half from this PR. Shipping SQL the provider rejects — or restructuring
the connector trait mid-PR to accommodate it — is a larger change than the P1 being fixed. The
MySQL files in this branch are byte-identical to `main` again.

**Follow-up (open, P2):** MySQL transaction-control and savepoint statements must go through the
text protocol, or the capability must be gated with a clear reason instead of emitting SQL that
fails with 1295. Tracked as its own change; a query tab on MySQL cannot hold an explicit
transaction through the prepared path today.

## Follow-ups opened by review (not fixed here)

1. **MySQL control statements** — transaction control and savepoints on MySQL must go through the
   text protocol or be capability-gated with a reason; today a prepared `BEGIN`/`SAVEPOINT` fails
   with 1295 (F-5).
2. **Shared statement loop** (P3) — the joined run repeats the standalone loop's statement execution
   and the `expect_affected_rows` / `max_affected_rows` checks. `cc-scan` reports the pair as a
   14-line `DUPLICATE_BLOCK` (`crates/infrastructure/src/postgres/connector.rs:790` vs `:964`). The
   extraction the reviewer asks for is real but lands on the standalone path's per-branch
   `tx.rollback()`, which has different transaction semantics; doing it inside a P1 fix would widen
   the diff and the regression surface beyond this plan's scope.
3. **`execute_transaction_joined` preflight phase** (P3) — the pre-existing batched-SQL joined path
   reports its own backend-pid and `search_path` setup failures as `left_open(0, …)`, the same
   misattribution described in F-3. Fixing it changes behaviour this PR does not otherwise touch.

## Not defects — recorded as observed

- The standalone path (no lease) is untouched; `pool.begin()` semantics and the existing
  `execute_parameterized_transaction_failure_rolls_back_replace_delete` PG test still hold.
- `transaction_control()` deliberately does not recognise `ROLLBACK TO SAVEPOINT …`, so a joined
  run's statements stay on the leased connection rather than being reinterpreted as transaction
  control. Correct as-is.
- `DiscardOnDrop` on the lease exists because of #456 (session-wide settings must not reach the
  next pool user); the joined path preserves that by returning the lease to the tab on success and
  discarding it when the outcome is unknown.

## Baseline impact

`.clean-code-baseline.json` unchanged; the joined method is new code inside an existing
`impl DbConnector` block, so no new module boundary is introduced.
