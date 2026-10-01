# Transaction components

`TransactionBar` displays caller-provided transaction state and returns an action for the caller to execute. `DestructiveOperationDialog` requires typing an exact keyword before returning confirmation.

```rust
if let Some(action) = TransactionBar::new(in_transaction, pending_changes, theme)
    .auto_commit(auto_commit)
    .isolation_level("READ COMMITTED")
    .show(ui)
{
    dispatch_transaction_action(action);
}
```

The bar can return Begin, Commit, or Rollback when the corresponding control is available. It never runs database work itself. The destructive dialog closes on cancellation and returns `true` only after the trimmed, case-sensitive confirmation text exactly matches the required keyword. See `DESIGN.md` and `API.md` for state precedence, timing, and limitations.
