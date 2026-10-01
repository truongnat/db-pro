# Transaction API

`TransactionBar`, `TransactionAction`, and `DestructiveOperationDialog` are re-exported from `components::transaction` and `components`.

## TransactionBar

`TransactionBar::new(in_transaction, pending_mutations, theme)` takes state values for the current frame. Builders set `auto_commit(bool)` (default false) and `isolation_level(&str)` (default `READ COMMITTED`). `show(ui) -> Option<TransactionAction>` returns:

- `Begin` when Begin Transaction is clicked while inactive and auto-commit is off.
- `Commit` or `Rollback` when the corresponding button is clicked during an active transaction.
- `None` when no eligible action was clicked.

The component reports intent only. Dispatch the action through the application/runtime, then update `in_transaction`, `pending_mutations`, and `auto_commit` from the resulting authoritative state. There is no pending/busy argument; gate repeat actions in the caller while a request is in flight.

## DestructiveOperationDialog

`DestructiveOperationDialog::new(&mut open, title, warning_text, target_object, required_keyword, &mut input_keyword, theme).show(ui) -> bool` borrows visibility and editable confirmation state. It shows the exact required keyword and leaves the caller's input available across frames. Leading/trailing whitespace is trimmed; matching is otherwise case-sensitive and exact.

- Returns `true` and closes only when the enabled destructive action is clicked with a valid keyword.
- Returns `false` and closes when Cancel is clicked.
- Returns `false` and remains open when confirmation is invalid or no action occurs.
- Escape or a backdrop click follows Dialog cancellation and closes the borrowed open flag; `show` then returns `false`.

The caller owns the actual operation and all success/failure feedback. The widget does not support a busy state; prevent duplicate operations in caller state after confirmation.
