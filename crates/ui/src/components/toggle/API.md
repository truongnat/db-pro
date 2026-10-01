# Toggle API

```rust
let mut compact = false;
let response = Toggle::new(&mut compact, theme)
    .label("Compact")
    .variant(ToggleVariant::Outline)
    .enabled(true)
    .show(ui);

let mut view = View::Data;
let changed = ToggleGroup::new(theme)
    .size(ToggleSize::Sm)
    .item(ToggleGroupItem::new(View::Data).label("Data"))
    .item(ToggleGroupItem::new(View::Structure).label("Structure"))
    .show_single(ui, &mut view);
```

- `Toggle::new(&mut bool, theme)`: `.label(text)`, `.icon(Icon)`, `.variant(ToggleVariant)`, `.size(ToggleSize)`, `.enabled(bool)`, `.show(ui) -> egui::Response`.
- `ToggleGroupItem::new(value)`: `.label(text)`, `.icon(Icon)`, `.tooltip(text)`.
- `ToggleGroup::new(theme)`: `.size(ToggleSize)`, repeated `.item(item)`, `.show_single(ui, &mut selected) -> Option<T>`.

Toggle mutates its bound bool only for an enabled activation; inspect `Response::changed()` to detect the state change. ToggleGroup returns and writes `Some(value)` only when a different value is activated, and returns `None` for no change or an empty group. Values require `Clone + PartialEq`. Group members do not expose an enabled/disabled option. This is a click-driven segmented control; it does not implement arrow-key navigation.

## Layers

`ui.rs` owns egui interaction and painting; `handler.rs` owns state, sizing, and appearance decisions; `config.rs` owns toggle-specific constants.
