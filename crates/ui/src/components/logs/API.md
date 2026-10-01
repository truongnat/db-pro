# Logs API

Import `LogEntry`, `LogLevel`, and `LogViewer` from `db_pro_ui::components::logs`.

```rust
let entries = [
    LogEntry::new("12:04:05", LogLevel::Info, "Query completed"),
    LogEntry::new("12:04:06", LogLevel::Warning, "Result was truncated"),
];
LogViewer::new(&entries, theme).show(ui);
```

- `LogLevel` supports `Info`, `Notice`, `Warning`, and `Error`.
- `LogEntry::new(timestamp, level, message)` stores the provided row data.
- `LogViewer::new(entries: &[LogEntry], theme)` borrows entries; `.show(ui) -> Response` paints the viewer.

The viewer preserves caller order and does not mutate entries. An empty slice displays the fixed `EMPTY_MESSAGE`. Filtering, ordering, persistence, and scrolling are caller responsibilities. Previous public constants `LOG_FONT_SIZE`, `EMPTY_FONT_SIZE`, `FRAME_RADIUS`, `FRAME_STROKE`, `FRAME_MARGIN_X`, `FRAME_MARGIN_Y`, and `ROW_GAP` were shared-token aliases and were removed; use the corresponding tokens from `db_pro_ui::tokens` when custom composition needs those values.
