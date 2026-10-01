# Toggle design

`mod.rs` is the public facade. `ui.rs` owns egui sizing, focus, and painting for `Toggle`, `ToggleGroup`, and `ToggleGroupItem`; `handler.rs` owns size tokens, appearances, width calculation, and state transitions. `config.rs` contains toggle-specific rounding and motion geometry. Shared colors and typography come from `DbProTheme` and tokens.

Standalone Toggle writes to a caller-owned boolean on enabled click or keyboard activation, then paints the resulting state. ToggleGroup paints a contiguous row, computes each item's width from its measured label/icon, and changes the caller's typed value only when a different item is clicked. Disabled Toggle controls are inert and visually muted.

The row is measured and painted on each frame. Long labels may widen the group; place it in a wrapping container or use concise labels and tooltips in narrow layouts. Groups are not virtualized.

## Accessibility and motion

Toggle responses expose the pressed state and use visible labels or icon tooltips as names. Theme tokens distinguish active, hover, disabled, and focus states. Hover animation uses the shared reduced-motion-aware helper. ToggleGroup is pointer-selectable; use a RadioGroup when arrow-key selection and radio semantics are needed.
