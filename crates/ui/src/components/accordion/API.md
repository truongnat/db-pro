# Accordion API

```rust
let mut open_item: Option<String> = None;
let item = AccordionItem::new("connection", "Connection settings")
    .icon(lucide_icons::Icon::Database)
    .badge("PostgreSQL");
let body = Accordion::new(theme).show_single(ui, item, &mut open_item, true, |ui| {
    ui.label("Connection details");
});
```

- `Accordion::new(theme)` creates a renderer.
- `AccordionItem::new(id, title)` defines a unique item. `.icon(Icon)`, `.badge(text)`, and `.disabled(bool)` are optional builders.
- `.show_single(ui, item, &mut Option<String>, collapsible, content) -> Option<R>` renders one item. With `collapsible=true`, activating the open item closes it; when false, the selected item stays open.
- `.show_multi(ui, item, &mut BTreeSet<String>, content) -> Option<R>` renders one independently controlled item. Opening inserts its ID and closing removes it.
- `AccordionType::{Single { collapsible }, Multiple}` is exported for API compatibility; choose the corresponding render method explicitly.

Content closure results are returned only when egui renders the body; while closed, the result is `None`. Focused enabled headers activate on Enter/Space; clicks use the same state transitions. Disabled items ignore activation and retain their previous multi-open membership. IDs must be unique within the same UI scope. Shared disclosure motion is immediate when `theme.reduce_motion` is true.

## Layers

`ui.rs` owns egui presentation; `handler.rs` owns open-state decisions; `config.rs` owns local badge/chevron dimensions; `disclosure.rs` is shared presentation support.
