# Developer Tools Components

Developer-oriented utility and presentation components providing interactive or stylized system widgets (Terminal output blocks, Progress rings).

## Public API

- `TerminalBlock::new(output, theme)`: creates a styled terminal card with macOS-style window control dots.
  - `.title(title)`: sets a custom title in the terminal header bar (defaults to `"Terminal Output"`).
  - `.show(ui)`: renders the terminal container and returns the `egui::Response`.
- `ProgressRing::new(progress, radius, theme)`: creates a circular progress meter with clamped progress [0.0, 1.0].
  - `.show(ui)`: renders the ring track and progress arc and returns the `egui::Response`.

## Behavior & Constraints

- `TerminalBlock` is styled with the editor background (`surface_editor`) and a distinct panel header (`surface_panel`).
- Output text is formatted in monospace with `FONT_SIZE_MONO_SM` and wrapped within standard padding.
- `ProgressRing` uses smooth circular arcs starting from 12 o'clock (-90°), sweeping clockwise to match progress percentage.
- All colors and strokes are mapped through `DbProTheme`.

## Usage Example

```rust
use db_pro_ui::components::dev_tools::{ProgressRing, TerminalBlock};

// Rendering terminal output:
TerminalBlock::new("SELECT * FROM pg_stat_activity;", theme)
    .title("psql query")
    .show(ui);

// Rendering circular progress:
ProgressRing::new(0.85, 24.0, theme).show(ui);
```

## Layers

- `mod.rs`: component entry point and public re-exports.
- `config.rs`: sizing constants (header height, dot sizes, progress ring track width).
- `handler.rs`: pure math and data transformations (arc calculations, dot coordinate positioning, title resolution).
- `ui.rs`: egui rendering and widget integration.
