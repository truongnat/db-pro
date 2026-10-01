# HoverCard API

```rust
let (trigger_response, content_result) = HoverCard::new("schema-preview", theme)
    .width(280.0)
    .open_delay(0.2)
    .close_delay(0.15)
    .show(
        ui,
        |ui| ui.button("Table details"),
        |ui| ui.label("public.users · 12 columns"),
    );
```

- `HoverCard::new(id, theme)` requires a stable unique string ID.
- `.width(points)` requests card width (default 300 points), clamped to the viewport.
- `.open_delay(seconds)` and `.close_delay(seconds)` set hover/open and leave/close timing (defaults 0.20 and 0.15 seconds). Negative, NaN, or infinite values become immediate; finite values are capped at 60 seconds.
- `.show(ui, trigger, content) -> (egui::Response, Option<R>)` always runs the trigger closure, and runs the content closure only while the card is open. The first value is the trigger response; the second is the content result.

Hover or keyboard focus opens after the delay. Moving into the card keeps it open during the grace period. Escape closes immediately and suppresses reopening until trigger/card interaction ends. The card is non-modal; it does not capture outside clicks or trap focus. Content is height-constrained and scrollable. Position flips above the trigger when space below is insufficient and remains within the screen inset where possible.

## Layers

`ui.rs` owns egui trigger, timer wiring, and Area/Frame presentation; `handler.rs` owns timer and placement decisions; `config.rs` contains local delay/geometry values.
