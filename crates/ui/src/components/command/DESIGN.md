# Command design

`mod.rs` is the public facade for `CommandInput`, `CommandItem`, `CommandGroup`, and `CommandEmpty`. `ui.rs` lays out and paints the input and rows; `handler.rs` owns geometry, accessibility-label composition, actionable/selected/disabled decisions, and token-based colors; `config.rs` contains component-specific sizes. The search field border uses shared `tokens::STROKE_THIN` directly instead of keeping a duplicate alias.

## Event flow and cost

The caller owns query filtering, current selection, and command dispatch. `CommandInput::show` allocates one fixed-height row, places a child single-line `TextEdit`, and unions the row and editor responses. A caller updates its query through the borrowed string, filters commands, then draws rows. Each `CommandItem::show` allocates one response and paints only the title, optional subtitle, icon, shortcut, and hover/selected background. The returned response is the activation signal; `id` is metadata and does not define egui identity.

Disabled rows use hover-only sensing and disabled accessibility metadata, so they cannot be activated and do not animate into a selected state. `selected` changes the button metadata and visual emphasis; it does not mutate selection. `CommandGroup` adds a heading around caller content. `CommandEmpty` paints a no-results message. Per-frame cost is O(number of rows rendered) plus text layout; the component does not filter or virtualize the list.

## Accessibility, motion, and narrow layouts

The input remains an egui text editor with a placeholder. Rows publish a composed button label including title, subtitle, and shortcut; callers should give titles that make the action clear. Shortcuts are visual hints and are not dispatched here. Text is clipped to leave room for an optional shortcut; very narrow widths can leave no visible title area. Hover emphasis uses shared motion helpers; callers own keyboard navigation and command invocation.
