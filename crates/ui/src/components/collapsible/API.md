# Collapsible API

```rust
let mut open = false;
let (response, body_result) = Collapsible::new(&mut open, theme)
    .title("Advanced connection settings")
    .id(ui.id().with("advanced-settings"))
    .show(ui, |ui| ui.label("Pool size and TLS options"));
```

- `Collapsible::new(&mut bool, theme)` binds caller-owned open state.
- `.title(text)`, `.icon(Icon)`, `.badge(text)`, `.disabled(bool)`, and `.id(egui::Id)` configure the header. Supply unique IDs for reorderable/dynamic siblings.
- `.show(ui, content) -> (egui::Response, Option<R>)` renders the header and runs the body closure while open. The body result is `None` while closed.

Click and focused Enter/Space toggle the value unless disabled. The response exposes the header role and expanded state. Reduced motion opens/closes immediately, without body-height or opacity animation. The widget does not manage child IDs or application state beyond the supplied boolean.

## Layers

`ui.rs` owns egui layout and painting; `handler.rs` owns state and badge geometry; `config.rs` contains component-local measurements; `components::disclosure` holds shared disclosure rendering.
