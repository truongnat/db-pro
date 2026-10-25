# Findings — UI03 Shell Hierarchy

## Audit Findings

1. `crates/ui/src/sidebar_view.rs`:
   - The "+ New query" button was custom painted with manual painter commands instead of using the canonical button tokens or standard button styling.
   - The workspace selector dropdown used raw `interact` with manual background repaint instead of canonical interactive component styling.
   - Hardcoded sizes and magic numbers (e.g. `FontId::proportional(13.5)`, `Rounding::same(6.0)`, etc.) where `tokens.rs` / `DbProTheme` provide unified design tokens.

2. `crates/ui/src/navigation_view.rs`:
   - The driver tag badge was hand-crafted with raw `egui::Frame` instead of using the canonical `Badge` component or standardized tokens.
   - Topbar search box center placeholder and sizing had slight alignment inconsistencies with standard inputs.
   - Statusbar buttons used legacy `compact_icon_button` where `Button::new(theme).icon(...).variant(ButtonVariant::Ghost).size(ButtonSize::IconSm)` is canonical.

3. `crates/ui/src/activity_bar_view.rs`:
   - Bottom settings button spacing had `ui.available_height() - 44.0` calculation that could jitter on resize.

## Wave 2 Findings — Shell Contrast & Region Elevation Flatness

4. User Evidence (`Screenshot 2026-10-25 at 15.20.06.png`):
   - In Light Mode, the interface reads completely flat: Activity Bar, Sidebar, CentralPanel, and Bottom Dock share `#f3f3f3` fill with zero separator lines (`show_separator_line(false)` and `bordered: false`).
   - Welcome cards (`welcome_surface_view.rs`) use `border_subtle` (`#f1f1f1`) on `#f3f3f3`, resulting in an imperceptible 1.08:1 contrast edge.
   - Solution: Add structural separator borders (`border_separator` / `border_default`) at shell panel boundaries (Sidebar right, Topbar bottom, Output dock top) and sharpen Welcome card resting borders.
