# RadioGroup design

`mod.rs` re-exports the builder from `ui.rs`. The UI builds each option from the shared `selection::Radio`, records egui responses and caller values, and applies click/keyboard signals through `handler.rs`. The handler owns activation eligibility, arrow navigation, disabled-option skipping, and orientation spacing. `config.rs` contains the group-specific gaps.

Keyboard arrows follow the group orientation, wrap around, and skip disabled choices. Space or Enter selects the focused enabled option. Clicking an enabled unselected option updates the caller-owned value and returns the new value; selecting the current option or activating a disabled option is inert. The group-level label is attached to its accessibility metadata.

Each option is rendered once per frame and the response/value list is retained for keyboard routing in that frame. Horizontal mode wraps with the available width; vertical mode stacks choices. Very large option sets are not virtualized and should use a different interaction pattern.

Use concise option labels and descriptions. Disabled options remain visible and muted. Focus follows arrow navigation, and the selection radio uses the active theme for contrast. This component has no animated transition.
