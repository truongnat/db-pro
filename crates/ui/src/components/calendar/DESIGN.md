# Calendar design

`mod.rs` is the public facade. `ui.rs` owns the seven-column month grid, date-picker trigger, popup area, and painting. `handler.rs` owns Gregorian date and month decisions, popup open state, and keyboard activation policy. `config.rs` owns calendar-specific geometry; theme colors and shared tokens stay centralized.

`Calendar` reads and updates the caller's selected date and viewed year/month. `DatePicker` stores its open flag and viewed month under its stable ID in egui temporary state, draws the popup in a foreground `Area`, and persists navigation after each frame. A changed selection or Escape closes the popup; an outside pointer click dismisses it. Popup placement is clamped to the screen rectangle.

The month view measures its available width, shrinks cells down to a one-point floor in constrained layouts, and paints up to 42 day hit targets. Each date button reports its ISO date and selected state to egui accessibility tooling. Focused month/day controls activate with Enter or Space and show a focus ring. Reduced motion makes hover feedback immediate. Rendering is constant per month (six rows by seven cells), independent of the selected date.

Use a stable unique ID per picker and a parent scroll area where necessary. The picker exposes month navigation and date activation by pointer or keyboard; it does not provide arrow-key grid traversal. Light/dark surfaces and selected-day contrast use `DbProTheme` semantic colors.
