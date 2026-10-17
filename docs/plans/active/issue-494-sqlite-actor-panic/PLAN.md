# Issue #494 — SqliteActor panic kills the command loop

## Problem
`SqliteActor::run` called `handle_*` methods directly. A panic in any handler
unwound out of the loop, killed the actor thread, and every queued responder
silently returned a generic oneshot `SendError`/`RecvError` — the connection
looked alive but was dead.

## Fix
In `crates/infrastructure/src/sqlite/actor.rs`:

- Every handler dispatch in `run()` is wrapped in
  `std::panic::catch_unwind(std::panic::AssertUnwindSafe(...))`.
- New free helpers:
  - `dispatch_guarded(&actor, |actor| ...)` for handlers returning
    `Result<T, DbError>` — panic becomes
    `Err(DbError::Internal("sqlite actor handler panicked: <msg>"))`.
  - `dispatch_transaction_guarded(&actor, stmt_count, |actor| ...)` for the two
    transaction handlers returning `Result<_, TransactionFailure>` — panic
    becomes a `TransactionFailure` with `phase: Validation`,
    `statement_index: statements.len()`, `outcome: Unknown` (post-panic
    transaction state is indeterminate), and the panic message in `error`.
  - `panic_message` extracts `&str`/`String` panic payloads.
- `Interrupt` and `Shutdown` arms unchanged (no handler call).
- `#[cfg(test)]`-gated `SqliteCommand::Poison` added so a test can inject a
  panic through the real dispatch loop.

## Unwrap/expect audit (issue asks: fix if trivial)
- `handle_execute`, `handle_introspect` (`introspect.rs`),
  `handle_explain`, `handle_raw_query`, `handle_execute_statement*`:
  no `unwrap`/`expect` on data paths — only `unwrap_or*`/`unwrap_or_default`
  safe fallbacks. Nothing to convert.
- `handle_execute_parameterized_transaction` uses `unwrap_or_default` inside an
  already-built error message — safe.
