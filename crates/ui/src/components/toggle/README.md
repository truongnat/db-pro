# Toggle

Native egui two-state controls with a caller-owned `bool` and a single-select `ToggleGroup`.
The public API is re-exported from `components::toggle` and `components`.

```rust
let mut compact = false;
Toggle::new(&mut compact, theme)
    .label("Compact")
    .variant(ToggleVariant::Outline)
    .show(ui);

let mut view = View::Data;
ToggleGroup::new(theme)
    .item(ToggleGroupItem::new(View::Data).label("Data"))
    .item(ToggleGroupItem::new(View::Structure).label("Structure"))
    .show_single(ui, &mut view);
```

`Toggle` changes the supplied value only when enabled and clicked. `ToggleGroup` keeps the
currently selected item active and returns `Some(value)` only when a different item is chosen.
The UI layer measures egui text and paints controls; `handler.rs` owns size, appearance,
rounding, width, and state-transition decisions. Component-owned dimensions and thresholds
live in `config.rs`.
