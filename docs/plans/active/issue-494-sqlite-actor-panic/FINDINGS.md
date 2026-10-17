# Findings — issue #494

- `SqliteActor::run` dispatched 9 handler variants directly; a panic in any of
  them killed the dedicated thread while `SqliteHandle` stayed clonable —
  callers then saw only `oneshot recv error`/`SendError` with no context.
- Handlers are `&self` methods, so `AssertUnwindSafe` is required
  (`rusqlite::Connection` is `!RefUnwindSafe`). Post-panic we only send an error
  through the responder and keep looping; the connection object is never
  destructively replaced.
- No data-path `unwrap`/`expect` found in `handle_execute`, `handle_introspect`
  (and its `introspect.rs` helpers), or `handle_explain`/`handle_raw_query` —
  all use `unwrap_or`-style fallbacks. The audit required no conversions.
- `TransactionFailure` contract: `statement_index = statements.len()` is the
  documented sentinel for transaction-level failures; `outcome: Unknown` is the
  honest state after a mid-transaction panic (may have left an open
  transaction — dropping the `tx` borrows doesn't run since `&self` methods
  can't outlive; any open transaction on `conn` remains open, next command can
  still proceed as `is_autocommit` checks handle it in `handle_execute_transaction`).
