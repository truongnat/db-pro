# Transaction design

`mod.rs` exposes `TransactionBar`, `DestructiveOperationDialog`, and `TransactionAction`. `ui.rs` lays out status and confirmation surfaces; `handler.rs` derives display status, resolves available transaction actions, and validates destructive confirmation; `config.rs` contains transaction-specific measurements. Database execution and authoritative transaction state remain outside this component.

## Event flow

Each frame, `TransactionBar` derives a display state from `auto_commit`, `in_transaction`, and pending mutation count. It shows Begin only when not in a transaction and auto-commit is off; it shows Commit and Rollback while in a transaction. A click becomes an optional `TransactionAction`; the caller must dispatch it and update the inputs from actual runtime state. Status precedence is auto-commit, inactive, uncommitted (pending count > 0), then active.

`DestructiveOperationDialog` draws through the shared Dialog, borrows caller-owned open/input state, and enables its destructive button only when trimmed input equals the required keyword exactly (case-sensitive). Cancel closes the dialog and returns false; valid confirmation closes it and returns true; invalid or absent confirmation returns false and leaves it open.

## Rendering and interaction limits

The bar paints one status line, optional pending badge, isolation label, and the currently available action buttons. The confirmation adds one shared modal dialog and one text input. Work is constant per frame apart from egui text measurement. The bar does not expose a pending/busy state or disable actions during an in-flight database request; callers must avoid rendering stale clickable controls while work is pending. Long isolation names, labels, or object names can exceed the compact horizontal bar, so callers should shorten or constrain them for narrow layouts.

The destructive dialog inherits Dialog's Escape and focus behavior. It does not perform a database mutation or prove that the caller's eventual operation is atomic or safe.
