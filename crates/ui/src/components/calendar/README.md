# Calendar

Native `egui` calendar and date-picker controls. The public API is re-exported
from `components::calendar`; `ui.rs` preserves the existing rendering and
selection behavior, while `handler.rs` contains pure date decisions.

```rust
let date = DatePicker::new("Start date", selected).show(ui);
```

`SimpleDate::new` clamps month and day to valid calendar bounds. Consumers
should keep the selected date in application state and provide an explicit
label when the picker is not self-describing.

## Review notes

The popup keeps the existing outside-click dismissal and compact date-grid
layout. Keyboard/focus behavior remains delegated to egui's native responses;
no shared token or component file is modified by this migration.
