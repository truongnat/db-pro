# Responsive layout API

Import from `db_pro_ui::components::responsive_layout`.

```rust
Container::new().max_width(960.0).gutter(20.0).show(ui, |ui| {
    ui.label("Centered content");
});

ResponsiveGrid::new(240.0).gap(16.0).max_columns(4)
    .show(ui, records, |cell, record| draw_record(cell, record));
```

- `Container::new()` creates an unconstrained container. `.max_width(points)` and `.gutter(points)` set bounds. `.show<R>(ui, closure) -> (Response, R)` returns both the region response and closure result.
- `container_width(available_width, max_width, gutter) -> ContainerWidth` exposes the pure width calculation.
- `ResponsiveGrid::new(min_cell_width)` creates a grid. `.gap(points)` sets inter-cell spacing, `.max_columns(count)` limits columns, `.metrics(available_width)` returns calculated `GridMetrics`, and `.show(items, closure) -> Vec<R>` renders all supplied items and returns one closure result per item.
- `grid_metrics(available_width, min_cell_width, gap, max_columns) -> GridMetrics` exposes the pure column/size calculation.

The grid may choose fewer columns as the parent narrows; items remain in input order. It renders every item and does not virtualize. Parent scrolling and semantic labels belong to the caller.
