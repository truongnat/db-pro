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
