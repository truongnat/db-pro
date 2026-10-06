# Calendar API

```rust
let mut selected = None;
let response = DatePicker::new("start_date", &mut selected, theme)
    .placeholder("Choose a start date")
    .enabled(true)
    .show(ui);
```

- `SimpleDate { year, month, day }` is a copyable date value. `SimpleDate::new(year, month, day)` clamps month to `1..=12` and day to the valid range for that month. `parse(text) -> Option<SimpleDate>` accepts exactly `YYYY-MM-DD` and rejects invalid dates; `to_iso_string()` formats that representation.
- `Calendar::new(&mut Option<SimpleDate>, &mut year, &mut month, theme)` renders an inline calendar. Invalid viewed months are normalized to January; `.show(ui) -> egui::Response` returns the calendar frame response and changes selection on date activation.
- `Calendar::for_temporal_value(&mut String, date_only, &mut year, &mut month, theme)` renders the same calendar for a raw ISO date/datetime editor, with Monday-first weekdays and a `Today` action. Selecting a date preserves the existing time suffix for datetime values; if no time is present it uses `00:00:00`.
- `DatePicker::new(id, &mut Option<SimpleDate>, theme)` creates a closed-by-default picker. `.placeholder(text)` changes the empty trigger label; `.enabled(bool)` disables opening and clears stale popup state; `.show(ui) -> egui::Response` returns the trigger response.
- Date helpers `is_leap_year`, `days_in_month`, `day_of_week`, `previous_month`, and `next_month` are re-exported from `components::calendar`.

The caller owns the selected date. Date activation updates it and marks the picker response changed; DatePicker closes after selection. Adjacent-month dates are selectable and update the viewed month. Click outside and Escape also close the popup. Focused trigger, month arrows, and dates activate with Enter/Space; arrow-key traversal between days is not implemented. The trigger publishes the selected ISO date or placeholder as its accessible label, while each date cell publishes its ISO date and selected state. Use a stable ID unique among sibling pickers.

## Layers

`ui.rs` owns egui rendering and popup interaction; `handler.rs` owns date, month, popup, and activation decisions; `config.rs` owns calendar-specific measurements.
