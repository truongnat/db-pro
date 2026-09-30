# Logs

Execution log viewer preserving `LogEntry`, `LogLevel`, and `LogViewer` APIs.
Rendering is isolated in `ui.rs`; level presentation decisions are tested in `handler.rs`; layout tokens are in `config.rs`.

```rust
let entry = LogEntry::new("12:04:05", LogLevel::Info, "Query completed");
LogViewer::new(&[entry], theme).show(ui);
```

`LogViewer` does not own filtering or persistence. Callers provide the ordered entries;
`handler.rs` owns the level-to-icon/color policy and `ui.rs` owns egui layout/painting.
