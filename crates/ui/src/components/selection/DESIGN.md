# Selection design

`mod.rs` exposes `Checkbox`, `Switch`, `Radio`, and `Slider`. `ui.rs` owns egui allocation, layout, focus, and painting. `handler.rs` owns the switch activation decision and checkbox row-height calculation; `config.rs` contains component-specific dimensions. Colors and shared spacing come from `DbProTheme` and tokens.

Checkbox and Switch mutate caller-owned booleans after an enabled click or keyboard activation, then mark the egui response changed. Radio is a presentational single option: it reports a click but leaves group state management to its caller. Slider binds a caller-owned `f32` to an inclusive range and delegates pointer/key behavior to egui.

Each visible widget is measured and painted every frame. Descriptions add a second text row for Checkbox, Switch, and Radio. Layout remains in the parent UI; use a wrapping or scroll container when labels or descriptions may exceed a narrow panel.

## Accessibility and motion

Controls publish widget role/value metadata and use theme colors for enabled, disabled, and focus states. Checkbox and Switch accept optional descriptions; Switch requires its label for a meaningful accessible name. Reduced motion is respected by selection controls that animate through the shared animation helper. Check color contrast with the active theme and keep enough surrounding space for the native hit target.
