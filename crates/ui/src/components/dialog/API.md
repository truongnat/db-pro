# Dialog API

Import from `components::dialog`. The compatibility module `components::dialog::modal` re-exports `Dialog`, `DialogActionLabels`, `close_icon_button`, and `dialog_actions`.

## Dialog

```rust
let result = Dialog::new(&mut open, "Delete table", theme)
    .description("This action cannot be undone.")
    .id_salt(("delete-table", table_id))
    .show(ui, |ui| {
        ui.label("Delete this table and its data?");
        dialog_actions(ui, theme, DialogActionLabels {
            secondary: "Cancel",
            primary: "Delete",
        })
    });
```

- `new(&mut bool, title, theme)` borrows the caller's visibility state; `title` accepts a value convertible to `Cow<str>`.
- `description(text)` adds optional descriptive text. `width(f32)` sets the requested card width. `id_salt(Hash)` distinguishes simultaneous instances at one call site; use a stable value.
- `show(ui, closure)` provides a scrollable body. `show_ctx(ctx, closure)` is the context-based equivalent when there is no parent `Ui`.
- `show_framed(ui, closure)` and `show_framed_ctx(ctx, closure)` provide `DialogFrame`, whose `body` method creates the scroll region and whose `footer` method creates a fixed action area.
- Each show method returns `None` while closed or fully hidden during the close transition. Otherwise it returns `Some(value)` from the content closure. Dismissal sets the borrowed open flag to `false`; business actions are still the caller's responsibility.
- Escape and backdrop dismissal apply only to the topmost dialog. Clicking inside the card does not dismiss it. When a backdrop click is reported without a pointer location, current behavior treats it as outside the card and dismisses.

## Sheet

`Sheet::new(&mut open, title, theme).width(f32).id_salt(hash).show(ui, closure)` renders a right-side panel and returns `Option<R>` with the same closed/open semantics as `Dialog`. Escape closes only the topmost modal. Backdrop clicks intentionally do not close sheets; provide a visible close action for pointer users.

## Helpers

- `DialogFrame::body(closure)` returns the closure value and places its content in the bounded scroll area.
- `DialogFrame::footer(closure)` returns the closure value and lays out a sticky footer.
- `DialogActionLabels { secondary, primary }` supplies labels for the standard two-action row. `dialog_actions(...)` returns `(secondary_clicked, primary_clicked)` and does not perform either action.
- `close_icon_button(ui, theme)` returns the egui `Response`; callers decide whether a click changes application state.

All content callbacks run during egui rendering. Keep state changes in the caller and use persistent state outside the callback when an action must survive frames. The dialog does not restore focus to its trigger after closing. A width smaller than the component's minimum content width is raised to that minimum; screen fit still depends on the parent viewport.
