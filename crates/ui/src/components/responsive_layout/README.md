# Responsive layout

Native `egui` layout primitives for bounded containers and equal-width responsive grids.

- `Container` constrains and centers content within the local available width.
- `ResponsiveGrid` chooses the largest column count whose cells satisfy the configured minimum.
- Grid cells wrap row-wise and are bounded to their allocated width.

Use the public exports from `components::responsive_layout`; configuration normalization and
layout decisions remain internal implementation details.
