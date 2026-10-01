# AspectRatio API

Import with `use db_pro_ui::components::AspectRatio;`.

## Constructors

- `AspectRatio::new(width_over_height: f32)` stores a custom ratio.
- `AspectRatio::sixteen_nine()`, `four_three()`, and `square()` create common ratios.

## Rendering

`show<R>(self, ui: &mut Ui, add_contents: impl FnOnce(&mut Ui) -> R) -> (Response, R)` allocates the region, runs the closure inside a clipped child UI, and returns the allocation response and closure result. The response has hover sense; the container itself is not clickable or focusable.

```rust
let (response, ()) = AspectRatio::sixteen_nine().show(ui, |ui| {
    ui.centered_and_justified(|ui| ui.label("Preview"));
});
```

Ratios that are non-finite, non-positive, or no greater than `0.001` become `1:1`. The region uses the current available width without an independent maximum height, so callers should constrain the parent when needed. Child painting is clipped to the allocated region.
