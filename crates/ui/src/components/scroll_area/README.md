# ScrollArea

Native egui scroll container preserving the builder API of `ScrollArea`.
Configuration lives in `config.rs`, axis and visual lifecycle decisions live in `handler.rs`,
and rendering in `ui.rs`.

```rust
ScrollArea::new(theme).both().max_height(400.0).show(ui, |ui| {
    ui.label("Scrollable content");
});
```

`horizontal`, `vertical`, and `both` preserve egui's `[horizontal, vertical]` axis order.
The component temporarily removes the clip margin while egui paints scrollbars and restores
the caller's visuals before returning, so sibling widgets keep their original style.
