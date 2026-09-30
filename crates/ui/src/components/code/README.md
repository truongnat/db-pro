# Code (`InlineCode` & `CodeBlock`)

Monospace code presentation components designed for developer workflows, SQL snippets, query results, and database definitions.

## Public API

### `InlineCode`
- `InlineCode::new(text: &'a str, theme: DbProTheme)`: Creates a new inline code snippet with subtle background styling and primary text contrast.
- `.show(ui: &mut Ui) -> Response`: Renders the inline code chip and returns the egui `Response`.

### `CodeBlock`
- `CodeBlock::new(code: &'a str, theme: DbProTheme)`: Creates a code block with language header, line numbers, and copy action.
- `.language(lang: &'a str)`: Sets the language identifier displayed in the header badge (defaults to `"SQL"`).
- `.show_line_numbers(show: bool)`: Toggles line number gutter visibility (defaults to `true`).
- `.show(ui: &mut Ui) -> Response`: Renders the full code block container and returns the outer `Response`.

## Behavior & Constraints

- `InlineCode` renders with subtle borders and padding matching workstation metadata density.
- `CodeBlock` features an integrated header bar with language badge and an interactive copy button with a 2-second "Copied" feedback state.
- Gutter width dynamically scales based on total line count to prevent number clipping.
- Copy button exposes accessible `WidgetInfo` labeled with the action state ("Copy" / "Copied").
- Monospace typography uses `FontId::monospace` with clear contrast tokens (`text_primary` for code, `text_secondary` for language/copy, `text_tertiary` for line numbers).

## Usage Example

```rust
use db_pro_ui::components::code::{CodeBlock, InlineCode};
use db_pro_ui::DbProTheme;

let theme = DbProTheme::light();

// Inline snippet
InlineCode::new("SELECT * FROM users WHERE status = 'active';", theme).show(ui);

// Full code block
CodeBlock::new("CREATE TABLE customers (\n    id SERIAL PRIMARY KEY,\n    name TEXT NOT NULL\n);", theme)
    .language("sql")
    .show_line_numbers(true)
    .show(ui);
```

## Layers

- `mod.rs`: Component entry point and public re-exports.
- `ui.rs`: egui layout, painting, frame allocation, and interaction plumbing.
- `handler.rs`: Pure calculations for copy button state, timeout evaluation, gutter sizing, and line number formatting.
- `config.rs`: Component-specific sizing, padding, and duration constants.
