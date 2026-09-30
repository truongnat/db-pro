# Separator

`Separator` provides subtle horizontal and vertical divider lines conforming to DB Pro design tokens, with optional centered text labels for horizontal dividers.

## Public API

- `Separator::horizontal(theme)` — creates a full-width horizontal divider with standard margin.
- `Separator::vertical(theme)` — creates a vertical divider constrained to parent row height with standard margin.
- `label(&'a str)` — sets an optional centered text label for horizontal dividers.
- `thickness(f32)` — overrides divider line stroke thickness.
- `margin(f32)` — overrides outer padding around the separator. Negative and non-finite thickness/margin values are replaced with orientation-appropriate safe defaults; valid dimensions retain existing behavior. A thickness of zero intentionally hides the line.
- `show(ui)` — allocates the separator geometry and paints lines/labels into `egui::Ui`. Separators are non-interactive. A labeled horizontal divider exposes its text as a label to egui accessibility output; unlabeled/vertical dividers are decorative and intentionally add no accessibility node because egui 0.29 has no separator widget role.
- `SeparatorOrientation` — orientation enum (`Horizontal`, `Vertical`).

## Architecture & Layering

- `mod.rs` — Public re-exports for `Separator` and `SeparatorOrientation`.
- `config.rs` — Component-local constants: line thickness, margins, label font size, padding, and height bounds.
- `handler.rs` — Pure geometric calculations for bounding box sizing, line segment coordinates, and centered label placement.
- `ui.rs` — egui widget allocation, text layout, and painter rendering using theme tokens.

Labels are intentionally rendered only for horizontal separators. Supporting vertical labels would expand the component's behavior/API contract and is not part of this API. Empty labels render as plain dividers. Long labels are centered and clipped to the allocated width so they cannot paint over neighboring controls.

## Example

```rust
use db_pro_ui::DbProTheme;
use db_pro_ui::components::{Separator, SeparatorOrientation};

let theme = DbProTheme::light();

// Standard horizontal line
Separator::horizontal(theme).show(ui);

// Horizontal divider with centered label
Separator::horizontal(theme)
    .label("OR")
    .show(ui);

// Vertical divider between toolbar items
Separator::vertical(theme).show(ui);
```
