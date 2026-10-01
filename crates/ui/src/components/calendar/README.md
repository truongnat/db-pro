# Calendar

Native `egui` calendar and date-picker controls. The public API is re-exported
from `components::calendar`; `ui.rs` preserves the existing rendering and
selection behavior, while `handler.rs` contains pure date decisions.

```rust
let mut selected_date = Some(SimpleDate::new(2026, 10, 1));
let response = DatePicker::new("start_date", &mut selected_date, theme)
    .placeholder("Choose a start date")
    .show(ui);
```

`SimpleDate::new` clamps month and day to valid calendar bounds. Consumers
keep the selected date in application state and provide a stable unique picker
ID. The trigger label includes the selected ISO date or the placeholder.

## Review notes

The popup keeps outside-click dismissal, clamps its placement to the viewport,
closes after selection or Escape, and preserves month navigation between frames.
Its custom-painted trigger, month buttons, and date cells expose accessibility
names and selected state; focused controls activate with Enter or Space. Focus
rings remain visible, and hover/open transitions respect reduced motion.
