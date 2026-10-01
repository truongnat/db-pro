# SqlEditorToolbar

SQL query execution toolbar component providing query execution, query explanation, formatting, cancellation, and AI assistance actions.

## Public API

- `SqlEditorAction`: enum of toolbar actions:
  - `RunQuery`: execute the entire SQL document.
  - `RunSelection`: execute the currently selected SQL text block.
  - `ExplainQuery`: request an execution plan for the query.
  - `FormatSql`: format the SQL script with standard indentation.
  - `CancelQuery`: cancel the actively running query.
  - `AskAi`: open the AI SQL assistance prompt.
- `SqlEditorToolbar::new(is_running, has_selection, theme)`: creates a toolbar with running/selection state.
- `SqlEditorToolbar::show(ui)`: renders the toolbar surface and returns `Option<SqlEditorAction>` if an action was clicked.

## Behavior & Constraints

- **Execution State**: When `is_running = true`, `Cancel` replaces Run, Run Selection, Explain, and Format. `Ask AI` remains available at the trailing side.
- **Selection Gating**: The `Run Selection` button appears dynamically only when `has_selection = true` and the editor is not running.
- **Accessibility & Focus**: Every toolbar button uses the common `Button` component, supporting Tab key navigation, Space/Enter activation, and accessible descriptions (`access_label`).
- **Surface Theme**: Frame background, border stroke, and button styles use semantic theme tokens (`surface_panel`, `border_subtle`) matching DB Pro design tokens.

## Usage Example

```rust
use db_pro_ui::components::sql_editor::{SqlEditorAction, SqlEditorToolbar};

let is_running = false;
let has_selection = true;

if let Some(action) = SqlEditorToolbar::new(is_running, has_selection, theme).show(ui) {
    match action {
        SqlEditorAction::RunQuery => execute_query(),
        SqlEditorAction::RunSelection => execute_selected(),
        SqlEditorAction::ExplainQuery => explain_query(),
        SqlEditorAction::FormatSql => format_sql(),
        SqlEditorAction::CancelQuery => cancel_query(),
        SqlEditorAction::AskAi => open_ai_prompt(),
    }
}
```

## Layers

- `mod.rs`: public entry point and stable exports (`SqlEditorAction`, `SqlEditorToolbar`).
- `ui.rs`: egui layout rendering, frame composition, and action button interaction.
- `handler.rs`: pure decision logic, label/icon/variant resolvers, and visibility predicates.
- `config.rs`: toolbar-owned sizing, rounding, margin, and spacing constants.
