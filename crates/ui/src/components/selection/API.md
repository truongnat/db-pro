# Selection API

Import controls from `db_pro_ui::components::selection` or the `components` re-exports.

```rust
let checked = Checkbox::new(&mut enabled, "Enabled", theme)
    .description("Use encrypted connections")
    .show(ui);
let changed = checked.changed();
```

- `Checkbox::new(&mut bool, label, theme)`: `.description(text)`, `.enabled(bool)`, `.focusable(bool)`, `.show(ui) -> egui::Response`.
- `Switch::new(&mut bool, theme)`: `.label(text)`, `.description(text)`, `.enabled(bool)`, `.show(ui) -> egui::Response`.
- `Radio::new(selected, label, theme)`: `.description(text)`, `.enabled(bool)`, `.show(ui) -> egui::Response`. Selection is supplied by the caller; the Radio does not mutate group state.
- `Slider::new(&mut f32, RangeInclusive<f32>, theme)`: `.label(text)`, `.show_value(bool)`, `.width(f32)`, `.show(ui) -> egui::Response`.

Checkbox and Switch update their bound boolean on an enabled activation and mark the response changed. Their response also carries focus and interaction information. Radio returns its click response only. Slider updates its bound value within the supplied range using egui's slider behavior. Width is a requested layout width, so narrow parents can constrain the available space. Descriptive text is plain text; it is not parsed as markup.

## Layers

`ui.rs` handles egui layout and painting; `handler.rs` contains interaction and row-metric decisions; `config.rs` contains component-specific values.
