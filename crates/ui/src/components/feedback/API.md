# Feedback API

Import the widgets and helpers from `db_pro_ui::components::feedback`.

```rust
Progress::new(0.65, theme).label("Import progress").show(ui);
Progress::indeterminate(theme).label("Loading schema").show(ui);
Spinner::new(theme).label("Connecting").show(ui);
kbd_combo(ui, &["Ctrl", "Shift", "P"], theme);
separator_with_text(ui, "OR", theme);
```

- `Progress::new(fraction, theme)` creates determinate progress; `.label(text)`, `.height(points)`, `.animated(bool)`, `.is_indeterminate(bool)`, `.show(ui) -> f32` are available. `show` returns the fill fraction drawn, or `0.0` when indeterminate.
- `Progress::indeterminate(theme)` creates indeterminate progress with the default `Loading` accessible label.
- `Spinner::new(theme)` supports `.label(text)`, `.size(points)`, `.color(Color32)`, and `.show(ui)`. It is an indicator, not an input, and `show` returns no value.
- `kbd_badge(ui, shortcut, theme)` paints one shortcut badge; `kbd_combo(ui, keys, theme)` paints the keys joined with plus signs.
- `separator_with_text(ui, text, theme)` paints a caption between two rules.

Non-finite progress becomes zero; finite progress is clamped to `0..=1`. Invalid or negative heights use the default, while zero is allowed. Spinner size is floored to a safe minimum. `DbProTheme::reduce_motion` makes determinate progress immediate and keeps indeterminate Progress/Spinner still. The former public token aliases `KBD_RADIUS`, `KBD_STROKE_WIDTH`, and `SEPARATOR_STROKE_WIDTH` were removed; use `RADIUS_XS` and `STROKE_THIN` from `tokens.rs`.
