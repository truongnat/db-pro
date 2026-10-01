# Badge API

The component is exported as `db_pro_ui::components::{Badge, BadgeVariant}`. Advanced helpers are available from `db_pro_ui::components::badge`.

```rust
Badge::new("Connected", theme)
    .variant(BadgeVariant::Success)
    .dot(true)
    .show(ui);
```

## Widget

- `Badge::new(text, theme)` accepts any text convertible to `Cow<'a, str>`.
- `.variant(BadgeVariant)` selects `Default`, `Secondary`, `Outline`, `Destructive`, `Success`, `Warning`, or `Info` (default: `Default`).
- `.dot(bool)` adds a status dot (default: `false`).
- `.icon(Icon)` adds a Lucide icon when no dot is enabled.
- `.compact(bool)` selects compact density (default: `false`).
- `.show(&mut Ui) -> Response` paints the badge and returns a hover-sense response; it does not trigger an action.

## Supporting public types

`BadgePalette::from_variant(variant, &theme)` exposes the resolved fill, text, border, and dot colors. `BadgeMetrics::from_compact(compact)` exposes metric calculation, including `leading_size`, `calculate_size`, `leading_gap`, and `text_position`; these are low-level layout helpers rather than alternate widgets.

Badge text is the accessibility label. It has no selected, disabled, or busy state and is not keyboard-focusable. The widget does not wrap or truncate its text. Compact mode changes visual density; it does not change semantics.
