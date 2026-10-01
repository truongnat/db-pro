# Command API

The command primitives are re-exported through `components::command` and `components`.

## Input and rows

- `CommandInput::new(&mut query, theme)` edits a caller-owned `String`. `placeholder(text)` replaces the default hint. `show(ui)` returns the union of the allocated row and text editor `Response`s; filtering and clearing the query are caller responsibilities.
- `CommandItem::new(id, title)` creates an enabled, unselected row. Chain `subtitle(text)`, `icon(Icon)`, `shortcut(text)`, `disabled(bool)`, and `selected(bool)`. Public fields allow direct construction. `show(ui, theme)` returns an egui `Response`; check `.clicked()` to dispatch. `id` is descriptive metadata and does not set widget identity.
- `CommandGroup::new(heading).show(ui, theme, closure)` paints a heading and returns the closure's value.
- `CommandEmpty::new(theme).text(message).show(ui)` paints an empty-state message; `show` returns unit.

Disabled items remain hoverable but cannot be clicked. A selected disabled item does not keep the active-selection background. Selection is presentational; no command is executed by these widgets. The list is not internally filtered, keyboard-navigated, or virtualized.
