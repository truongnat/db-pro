# Separator API

Import `Separator` and `SeparatorOrientation` from `db_pro_ui::components`.

```rust
Separator::horizontal(theme).label("Advanced options").show(ui);

ui.horizontal(|ui| {
    ui.label("Schema");
    Separator::vertical(theme).show(ui);
    ui.label("Table");
});
```

- `Separator::horizontal(theme)` creates a full-width horizontal divider with `SPACE_SM` margin.
- `Separator::vertical(theme)` creates a divider sized to the parent row, capped at 24 points, with a 6-point margin.
- `.label(&str)` sets text; only horizontal orientation paints it.
- `.thickness(f32)` overrides line stroke width. Zero intentionally hides the line.
- `.margin(f32)` overrides outer padding.
- `.show(ui) -> Response` allocates and paints the divider.
- `SeparatorOrientation` is `Horizontal` or `Vertical`.

Negative or non-finite thickness/margin values fall back to safe defaults. A horizontal label is exposed as label metadata; other separators are decorative and add no accessibility node. The widget is not focusable or actionable.
