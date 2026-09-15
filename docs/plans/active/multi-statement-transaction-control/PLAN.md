# Plan — #147: `execute_multi` must refuse a batch that carries its own transaction control

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
