# Parameterized Transaction Session Lease for PostgreSQL

## State

`COMPLETED`

## Goal

Make `PostgresConnector::execute_parameterized_transaction` join the query tab's already-held
session connection (`entry.open_session`) instead of running parameterized table mutations on a
separate pool connection, so a staged row edit participates in the transaction the tab opened with
`BEGIN`.

## Problem Statement

When a query tab executes an explicit transaction (`BEGIN`), the connection is held in
`entry.open_session` for subsequent statements on that tab.
`execute_parameterized_transaction` ignored `entry.open_session` and always acquired a new pool
connection with `pool.begin()`. As a result:

1. Staged table row mutations executed while the tab has an open transaction ran on a completely
   separate database connection/session.
2. The mutation could not see uncommitted changes from the active transaction tab (or vice-versa).
3. With an explicit transaction open on the tab, table row mutations did not run inside that
   transaction, violating the session transaction boundary.
4. Joining is not a drop-in change: on a standalone connection the failure path calls
   `tx.rollback()`, which on a leased connection would destroy the caller's outer transaction.
   The joined path must undo only its own statements.

## Evidence & Failure Scenario

- **Code path**: `PostgresConnector::execute_parameterized_transaction` in
  `crates/infrastructure/src/postgres/connector.rs` called `pool.begin().await` while the sibling
  `execute_transaction` in the same file already dispatches through `checkout_if_held(...)`.
- **Failure scenario** (reproduced live in `pg_joined_parameterized_transaction_rolls_back_only_its_own_statements`):
  1. A tab issues `BEGIN`; the connection is saved in `entry.open_session`.
  2. The tab inserts a row — uncommitted, on connection `A`.
  3. The data grid stages a mutation, which goes through `execute_parameterized_transaction` and
     takes a *new* pool connection `B`.
  4. `B` cannot see `A`'s uncommitted row, so a uniqueness/foreign-key-guarded edit succeeds or
     fails on the wrong evidence, and its commit lands independently of `A`.
- **Severity**: P1 (data corruption / wrong database mutation; broken transaction semantics).

## Scope

1. `PostgresConnector::execute_parameterized_transaction` checks out `entry.open_session` via
   `checkout_if_held`; when a lease is returned, the run goes to the new
   `execute_parameterized_transaction_joined`, otherwise the existing standalone `pool.begin()`
   path is unchanged.
2. The joined run wraps itself in a savepoint whose name is generated per run by
   `transaction_session::savepoint_name()` (`dbpro_tx_<seq>`), never from user input.
   - Statement failure or an `expect_affected_rows` / `max_affected_rows` invariant breach →
     `ROLLBACK TO SAVEPOINT <name>`, outcome `LeftOpen`, lease returned to the tab.
   - Success → `RELEASE SAVEPOINT <name>` then `lease.succeed()`.
   - A savepoint undo that itself fails is not reported as `LeftOpen`: the outcome becomes
     `Unknown`, the error text carries the rollback failure, and the connection is discarded
     rather than handed back to the tab.
3. Pre-statement failures (backend pid, `search_path` push, the `SAVEPOINT` itself) report
   `phase = Begin`, not `Statement` with index 0, because `TableMutationExecution` rewrites a
   statement index onto the grid row it came from. `query_service::format_transaction_failure`
   gained the matching `(Begin, LeftOpen)` message arm.
4. Regression coverage lives in `crates/infrastructure/tests/pg_integration.rs` (live server), not
   in source-text assertions.

## Non-goals / provider scope

- **MySQL is out of scope for this change.** The same joined branch was implemented and tested
  against a live MySQL 8 server, then removed. See FINDINGS: MySQL's prepared-statement protocol
  rejects `BEGIN` and `SAVEPOINT` with `ERROR 1295`, so the branch cannot be reached by the code
  paths that use sqlx's prepared API, and shipping SQL the provider refuses is worse than not
  shipping it. The MySQL-side capability gap is recorded as a follow-up.
- **SQLite is not in scope.** `SqliteConnector::execute_parameterized_transaction` does not read a
  session slot at all (no `open_session` in `crates/infrastructure/src/sqlite/connector.rs`), and
  `transaction_session` is a PostgreSQL-only mechanism, so there is no SQLite behaviour to change or
  regress here. The SQLite side of the same session-lease question is a separate provider gap and is
  not claimed as covered by this PR.
- No change to the standalone (no lease) path, the safety/confirmation policy, or the
  `TransactionFailure` contract.

## Acceptance criteria

1. With a tab holding an open transaction, `execute_parameterized_transaction` runs on that tab's
   connection: its writes are visible inside the transaction and are undone by the tab's `ROLLBACK`
   (or kept by its `COMMIT`).
2. A failed statement inside the joined run undoes only the run's own statements, leaves the tab's
   transaction usable (no 25P02), and reports `phase=Statement`, correct `statement_index`,
   preceding results, and `outcome=LeftOpen`.
3. An undo that cannot be confirmed reports `outcome=Unknown` and does not return the connection to
   the tab.
4. `cargo check --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`,
   `cargo test --workspace`, `cargo +nightly fmt --all -- --check`,
   `python3 tools/cc-scan.py . --fail-on error` stay green.
