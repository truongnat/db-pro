# ScrollArea API

Import `ScrollArea` from `db_pro_ui::components::scroll_area`.

```rust
ScrollArea::new(theme).both().max_height(400.0).show(ui, |ui| {
    ui.label("Scrollable content");
});
```

- `ScrollArea::new(theme)` defaults to vertical scrolling, no horizontal scrolling, and `auto_shrink([false, false])`.
- `.horizontal(enabled)` and `.vertical(enabled)` set axes independently; `.both()` enables both.
- `.auto_shrink([horizontal, vertical])` uses egui's axis order.
- `.max_height(points)` applies egui's maximum-height constraint.
- `.show<R>(ui, closure) -> R` renders the native egui scroll area and returns the child closure value.

The theme argument is retained for constructor compatibility; scrollbar visuals currently come from egui. `max_height` is passed to egui as supplied, so callers should use a finite nonnegative value. Scroll offsets, content, and any keyboard focus handling remain owned by egui and the caller's child widgets.
