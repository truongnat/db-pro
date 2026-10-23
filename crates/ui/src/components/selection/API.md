# Selection API

Import controls from `db_pro_ui::components::selection` or the `components` re-exports.

```rust
let checked = Checkbox::new(&mut enabled, "Enabled", theme)
    .description("Use encrypted connections")
    .show(ui);
let changed = checked.changed();
```

- `Checkbox::new(&mut bool, label, theme)`: `.description(text)`, `.enabled(bool)`, `.focusable(bool)`, `.compact(bool)`, `.show(ui) -> egui::Response`.
- `Switch::new(&mut bool, theme)`: `.label(text)`, `.description(text)`, `.enabled(bool)`, `.show(ui) -> egui::Response`.
- `Radio::new(selected, label, theme)`: `.description(text)`, `.enabled(bool)`, `.compact(bool)`, `.show(ui) -> egui::Response`. Selection is supplied by the caller; the Radio does not mutate group state.
- `Slider::new(&mut f32, RangeInclusive<f32>, theme)`: `.label(text)`, `.show_value(bool)`, `.width(f32)`, `.show(ui) -> egui::Response`.

Checkbox and Switch update their bound boolean on an enabled activation and mark the response changed. Their response also carries focus and interaction information. Radio returns its click response only. Slider updates its bound value within the supplied range using egui's slider behavior. Width is a requested layout width, so narrow parents can constrain the available space. Descriptive text is plain text; it is not parsed as markup.

`Checkbox` and `Radio` default to the width policy of their parent layout: a horizontal row
allocates the measured label/description width so siblings stay adjacent, while a vertical
container keeps the full row as the click target. `.compact(bool)` overrides that default either
way.

A compact row is capped at the parent's available width, so a narrow cell wraps its label instead of
overflowing it. The one exception is the control glyph plus its gap (about 24 pt): a parent narrower
than that still receives that minimum, because the indicator cannot be drawn in less. Non-compact
rows keep the pre-existing full-width policy and are not capped.

## Layers

`ui.rs` handles egui layout and painting; `handler.rs` contains interaction and row-metric decisions; `config.rs` contains component-specific values.
