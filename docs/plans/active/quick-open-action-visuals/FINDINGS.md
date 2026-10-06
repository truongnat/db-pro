# Findings

At baseline `57cea2bf746e132564cde952cc06d0d04d796aa0`, the selected Quick Open scope was painted by `ui.selectable_label`. `DbProTheme::apply` sets `Visuals::override_text_color`, which egui bakes into the label galley before selected widget visuals are applied. The baseline regression test observed `#1A1C1F` in the selected glyph while the selection foreground was `#FFFFFF`.

P2, fixed in `78d914887642dd400c7e2048db9a24d0403619c8`: selected filter labels now pass the current egui selection foreground explicitly. The separate selected-row outline and scope-filter hover/active strokes were removed.

P2, fixed in `78d914887642dd400c7e2048db9a24d0403619c8`: Button Outline, disabled, and loading palettes emitted decorative strokes. All action Button variants now stay stroke-free through hover interpolation, loading, and disabled states. The dedicated keyboard focus ring remains.

Dark-theme solid buttons can still have black text by design. `DbProTheme::text_on_solid` chooses black when the accent fill is bright enough to provide stronger normal-text contrast; this is separate from selected-label color.

No P0/P1 findings. No database or provider impact.
