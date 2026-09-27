# Command Components

Native egui primitives for rendering a command-search field, grouped command rows, and an empty result state. Filtering, keyboard navigation, selection state, and command dispatch remain the caller's responsibility.

## Public API

- `CommandInput`: edits the caller-owned query and returns the union of its row and text-edit responses.
- `CommandItem`: renders title, optional subtitle/icon/shortcut, and hover/selected/disabled states. `id` is metadata only and does not currently set egui response identity.
- `CommandGroup`: renders a heading around caller-provided content.
- `CommandEmpty`: renders the no-results message.

The existing names and builder APIs remain available through both `components::command::*` and `components::*`.

## Interaction contract

- Disabled items use hover sensing only, expose disabled button semantics, use disabled colors, and do not show a pointing-hand cursor. A disabled selected row does not receive the active-selection background.
- Selected state is exposed as button metadata and affects visual emphasis only; callers still own activation and selection updates.
- The accessible row label includes title, subtitle, and shortcut. Long title/subtitle text is clipped before the reserved shortcut region.
- `CommandInput::show` unions the allocated row response with the child `TextEdit` response so both areas remain observable.
- Geometry and colors are calculated through `config.rs` and `handler.rs`; egui widget allocation/layout/painting stays in `ui.rs`.

## Example

```rust
use crate::components::{CommandGroup, CommandInput, CommandItem};

let mut query = String::new();
CommandInput::new(&mut query, theme).show(ui);

CommandGroup::new("RECENT")
    .show(ui, theme, |ui| {
        CommandItem::new("open-query", "Open query")
            .shortcut("⌘K")
            .selected(true)
            .show(ui, theme);
    });
```
