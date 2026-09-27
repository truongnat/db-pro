# ScrollArea

Native egui scroll container preserving the builder API of `ScrollArea`.
Configuration lives in `config.rs`, visual lifecycle helpers in `handler.rs`, and rendering in `ui.rs`.

```rust
ScrollArea::new(theme).both().max_height(400.0).show(ui, |ui| {
    ui.label("Scrollable content");
});
```
