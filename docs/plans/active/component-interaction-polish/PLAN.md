# Component interaction polish

State: RUNTIME_VERIFY. Source commit: `cd58b354a43f55458759c298bc053c5a4dd36c0b`. Baseline: `307c9cc0301f9d3de4f58a22c6d916789b483341`.

## Scope

Fix shared Button press painting, shared Alert dismiss alignment and native egui Label/TextEdit text selection colors requested by the owner. Keep existing geometry, hit targets, theme roles, fonts and egui renderer. No DB/provider operations or rs-ui migration.

## Acceptance

- Button background, border, icon, label and underline scale around one center; cached glyph layout and hit targets stay stable.
- Alert dismiss is at the trailing edge with short and wrapped descriptions.
- Native text selection uses solid blue and white selected glyphs; unselected glyphs retain their colors.
- Rust gates pass and runtime evidence is recorded honestly.

Implementation is delivered. Formal visual gate remains pending: full 1440×900/1920×1080 viewport heights and a native recording of the held-button animation.
