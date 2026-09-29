# Selection

Native selection controls (`Checkbox`, `Switch`, `Radio`, and `Slider`). Rendering remains in `ui.rs`; interaction decisions and row metrics live in `handler.rs`.

```rust
let response = Checkbox::new(&mut checked, "Enabled", theme).show(ui);
```

The public API is re-exported by `components`.
