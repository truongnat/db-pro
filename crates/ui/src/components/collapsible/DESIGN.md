# Collapsible design

`mod.rs` re-exports `Collapsible`. `ui.rs` owns egui header allocation, interaction, focus, text measurement, and painting. `handler.rs` owns caller-state toggles and badge geometry. `config.rs` contains disclosure-specific dimensions; common disclosure surface, focus ring, chevron, and body layout live in `components::disclosure`.

The builder binds directly to a caller-owned `bool`. Pointer click or focused Enter/Space becomes a handler activation, which updates the state only when enabled. The UI publishes `CollapsingHeader` role and expanded state, then uses shared `CollapsingState` to render body content. A stable supplied ID keeps focus and animation identity attached to the same item when sibling order changes.

The header measures title/badge widths each frame, reserves trailing chevron space, and clips long titles to the remaining width. Open content is rendered inside the shared padded frame. Reduced motion bypasses the height clip and opacity tween, while hover and chevron states change immediately. No content is rendered while closed in reduced-motion mode.

The header should have a concise title and stable sibling identity. Disabled controls retain muted styling and do not receive focus or activation. Semantic surfaces, text, and focus colors come from `DbProTheme`.
