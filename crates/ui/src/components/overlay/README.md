# Overlay

Tooltip, popover, dropdown, toast, and context-menu surfaces. `ui.rs` owns egui layout and painting; `config.rs` contains overlay-local layout constants; `handler.rs` contains pure dismissal decisions. Use the types re-exported by `components`.

```rust
Tooltip::new("Copy", theme).show(&response);
```
