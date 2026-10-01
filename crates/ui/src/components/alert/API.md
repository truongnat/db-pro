# Alert API

`Alert`, `AlertDialog`, `AlertDialogAction`, and `AlertVariant` are re-exported from `components::alert` and `components`.

## Inline alert

```rust
if let Some(close) = Alert::new("Connection lost", "Reconnect to continue.", theme)
    .variant(AlertVariant::Warning)
    .dismissable(true)
    .show(ui)
{
    if close.clicked() {
        dismiss_notice();
    }
}
```

`Alert::new(title, description, theme)` creates a non-dismissible Default alert. `title_only(title, theme)` omits the description. Builders select `variant(Default|Info|Success|Warning|Destructive)`, an optional Lucide `icon`, and `dismissable(bool)`. `show(ui)` returns `Some(Response)` for a visible close control, or `None` when non-dismissible. Clicking does not remove application state automatically. Long title/description text wraps.

## AlertDialog

`AlertDialog::new(title, description, theme)` defaults to Continue/Cancel labels and non-destructive behavior. Configure `confirm_label(text)`, `cancel_label(text)`, `destructive(bool)`, and a stable `id_salt(hash)` when rendering more than one instance. Call `show(ctx, &mut open)` each frame while managing the `open` flag in the caller.

- Closed: returns `None`.
- Confirm button: sets `open = false`, returns `Some(AlertDialogAction::Confirm)`.
- Cancel button or Escape: sets `open = false`, returns `Some(AlertDialogAction::Cancel)`.
- Backdrop click: same Cancel result when `destructive(false)`; ignored when destructive.

Destructive styling does not execute or authorize an operation by itself. Awaiting work/busy state, focus restoration, and retry policy belong to the caller; the API does not provide a busy flag or focus trap.
