# Feedback design

Progress and Spinner are semantic progress indicators. They attach a labeled `ProgressIndicator` role; determinate Progress reports 0–100, while indeterminate states omit a value. Keyboard badges and the labeled separator are display-only composition helpers.

The theme's reduced-motion preference disables Progress interpolation and moving indeterminate beams, and holds Spinner at a stable angle. Skeleton uses the same shared animation decision helper. A static centered beam keeps indeterminate work visible without continuous repaint. Shared radius and stroke values come directly from `tokens.rs`; local config owns component dimensions and timing inputs.

Progress and Spinner paint a constant number of primitives. The separator measures one caption and paints two lines; `kbd_combo` cost follows the number of keys. Accessible text is supplied by the caller through `.label(...)` for progress indicators; visual-only helpers do not create action semantics.
