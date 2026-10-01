# HoverCard design

`mod.rs` re-exports the builder. `ui.rs` calls the trigger closure, tracks temporary open/hover/timer state under the caller's stable ID, and renders a foreground `Area` and scrollable frame. `handler.rs` owns delay sanitization, open/close transitions, width clamping, and collision placement. `config.rs` contains HoverCard-specific timing and sizing defaults.

Pointer hover and keyboard focus start an open timer. When active state ends, the close grace timer lets the pointer move into the card. The handler resolves timer state from egui time and the UI schedules repaint for only the remaining deadline. Escape closes immediately, clears timers, suppresses reopening while the trigger/card remains active, and surrenders focus to prevent an immediate reopen loop.

The card is measured before final placement. Its width is clamped to the viewport, its content is height-constrained and scrollable, and it flips above the trigger when space below is insufficient; otherwise its position is clamped inside screen insets. Rendering cost follows caller content and occurs each frame while open. There is no continuous animation loop.

The trigger must expose meaningful text or its own accessible label. Focus makes the card available to keyboard users; Escape dismisses it. The card is non-modal and does not trap focus. Floating surface, border, and shadow use the active theme and local dimensions.
