# HoverCard

Floating overlay component for previewing rich contextual information when hovering or focusing on a trigger element.

## Public API

- `HoverCard::new(id, theme)`: creates a new hover card with the given stable ID and theme.
- `HoverCard::width(width)`: sets the card width in points (default: `300.0`, clamped to available screen width).
- `HoverCard::open_delay(delay)`: sets the delay in seconds before opening (default: `0.20s`).
- `HoverCard::close_delay(delay)`: sets the grace delay in seconds before closing when pointer/focus leaves (default: `0.15s`).
- `HoverCard::show(ui, trigger, content)`: renders the trigger widget closure and conditionally renders the floating content overlay. Returns `(Response, Option<R>)`.

## Behavior & Constraints

- **Accessibility & Focus**: Trigger responds to both pointer hover and keyboard focus (`trigger_resp.has_focus()`), ensuring keyboard users can inspect card content.
- **Grace Period**: Moving the pointer between the trigger and the card within `close_delay` keeps the card open so interactive content inside the card can be clicked or selected.
- **Escape dismissal**: Pressing Escape while the card is open closes it immediately, surrenders trigger focus, clears pending timers, and suppresses reopening while the trigger/card remains active. Reopening is allowed after the pointer and focus leave the card interaction.
- **Delay bounds**: Negative, NaN, and infinite delays mean no delay; finite delays are clamped to the component's 60-second timer maximum. This also bounds repaint scheduling safely.
- **Collision & Overflow**: The card clamps horizontally within the screen viewport (with `SCREEN_EDGE_INSET`). When space below the trigger is insufficient, it automatically flips upward above the trigger.
- **Theme Tokens**: Floating surface styling uses `theme.surface_floating`, `theme.border_subtle`, standard shadow, and border radius.

## Usage Example

```rust
use db_pro_ui::components::hover_card::HoverCard;

let (trigger_resp, card_result) = HoverCard::new("user_profile_hover", theme)
    .width(280.0)
    .open_delay(0.2)
    .close_delay(0.15)
    .show(
        ui,
        |ui| ui.label("Hover or focus for details"),
        |ui| {
            ui.heading("Schema Details");
            ui.label("Table: public.users");
            ui.label("Rows: 1,420,000");
        },
    );
```

## Layers

- `mod.rs`: public entry point and stable `HoverCard` export.
- `ui.rs`: egui `Area` and `Frame` rendering, interaction binding, and repaint triggering.
- `handler.rs`: pure open/close timer transitions, collision detection, position calculation, and width clamping.
- `config.rs`: HoverCard-specific defaults and dimensions.
