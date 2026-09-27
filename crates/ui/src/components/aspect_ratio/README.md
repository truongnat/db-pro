# AspectRatio

`AspectRatio` reserves a native egui region whose width follows the current available UI width and whose height is derived from a fixed width/height ratio.

## Public API

- `AspectRatio::new(ratio)` — creates a container for a custom `width / height` ratio. Ratios at or below `0.001` fall back to square behavior, matching the pre-migration component.
- `AspectRatio::sixteen_nine()` — convenience constructor for `16:9`.
- `AspectRatio::four_three()` — convenience constructor for `4:3`.
- `AspectRatio::square()` — convenience constructor for `1:1`.
- `show(ui, |ui| { ... })` — allocates the region, clips a child UI to it, and returns the allocation `Response` plus the closure return value.

## Behavior

The component does not paint by itself. It allocates a hover-sense rectangle, creates a clipped child UI with the same bounds, then lets the caller draw content inside that child. Layout and clipping stay in `ui.rs`; ratio sanitization and size calculation are pure handler decisions with focused tests.

## Example

```rust
use db_pro_ui::components::AspectRatio;

let (response, _) = AspectRatio::sixteen_nine().show(ui, |ui| {
    ui.label("16:9 canvas");
});

assert!(response.rect.is_finite());
```
