# Alert

Native `egui` alert and confirmation dialog components. The public API is
re-exported from `components::alert`; rendering remains in `ui.rs`, while
interaction decisions and component defaults live in the neighboring layers.

```rust
let action = AlertDialog::new("Delete connection", "This cannot be undone.", theme).show(ui, &mut open);
```

Dialogs return `AlertDialogAction::Confirm` or `Cancel`; callers own the open
state. Destructive mode is opt-in with `.destructive(true)`, and backdrop clicks
cannot cancel a destructive dialog. Escape cancels an open dialog; Cancel receives
initial focus when no other widget owns focus. Use `.id_salt(key)` when multiple
dialogs can be mounted with the same title. The egui area implementation does
not yet trap or restore focus, so native keyboard/accessibility runtime review is
still required.

## Review notes

Dismissible alerts place the labeled close button beside the wrapped copy and
return its response so callers can remove the alert. Dialog width is clamped to
the viewport after frame padding, and action buttons wrap at narrow widths.
Variant styling uses `DbProTheme` semantics; the component does not run the
confirmed action itself.
