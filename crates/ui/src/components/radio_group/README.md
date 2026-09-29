# RadioGroup

Radio group component for selecting a single option from mutually exclusive choices in native egui views.

## Public API

- `RadioGroupOption::new(value, label)`: creates a radio option with typed value and display label.
- `RadioGroupOption::description(desc)`: attaches an optional secondary descriptive line below the label.
- `RadioGroupOption::disabled(disabled)`: disables interaction and displays muted styles.
- `RadioGroup::new(theme)`: creates a radio group builder.
- `RadioGroup::horizontal(bool)`: switches between vertical stacked (default) and wrapping horizontal layouts.
- `RadioGroup::label(label)`: adds an accessible name for the whole group.
- `RadioGroup::option(opt)`: adds an option to the group.
- `RadioGroup::show(ui, selected)`: renders the group, updates mutable selection on click or keyboard navigation, and returns `Some(new_value)` when selection changes.

## Behavior & Constraints

- Clicking an unselected, enabled radio option updates `*selected` and returns `Some(new_value)`.
- Space/Enter activates the focused option; arrow keys move focus and selection, wrapping while skipping disabled options.
- Clicking an already selected option or a disabled option produces no mutation or change event.
- Horizontal options wrap when the available viewport width is exhausted.
- Layout spacing uses component constants for consistent horizontal (`16.0px`) and vertical (`8.0px`) rhythm; the unused cross-axis gap is centralized as `NO_ITEM_GAP`.
- Each option delegates visual rendering and option-level semantics to [`Radio`](../selection.rs); `.label(...)` supplies the group-level accessible name.

## Usage Example

```rust
use db_pro_ui::components::radio_group::{RadioGroup, RadioGroupOption};

#[derive(Clone, PartialEq, Debug)]
enum Engine {
    Postgres,
    Sqlite,
}

let mut selected = Engine::Postgres;

RadioGroup::new(theme)
    .label("Database engine")
    .option(RadioGroupOption::new(Engine::Postgres, "PostgreSQL").description("Recommended for production"))
    .option(RadioGroupOption::new(Engine::Sqlite, "SQLite"))
    .show(ui, &mut selected);
```

## Layers

- `mod.rs`: public entry point and re-exports.
- `ui.rs`: egui layout, list iteration, and Radio delegation.
- `handler.rs`: pure decision logic for option selection mutations and orientation item spacing.
- `config.rs`: spacing constants for horizontal and vertical layouts.
