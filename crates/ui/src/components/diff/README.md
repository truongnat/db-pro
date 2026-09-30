# Diff Viewer

Monospace line-by-line diff inspector component for SQL migrations, schema modifications, and database content comparison.

## Public API

- `DiffViewer::new(title: &'a str, lines: &'a [DiffLine], theme: DbProTheme)`: Creates a new diff viewer container displaying a titled header with summary change badges (+X -Y) and colored line diffs.
- `.show(ui: &mut Ui) -> Response`: Renders the diff table inside a framed panel and returns the egui `Response`.
- `DiffLine::context(old_num: usize, new_num: usize, content: impl Into<String>)`: Creates an unchanged context line.
- `DiffLine::added(new_num: usize, content: impl Into<String>)`: Creates an added (+) diff line.
- `DiffLine::removed(old_num: usize, content: impl Into<String>)`: Creates a removed (-) diff line.
- `DiffLineType`: Enum with variants `Context`, `Added`, `Removed`.

## Behavior & Constraints

- Header bar displays the file or migration target title along with aggregate additions and deletions stats (e.g. `+3  -1`); the viewer exposes the title as its accessible label.
- Empty input shows a centered “No changes to display.” label instead of a blank body.
- Line types render distinct background highlights using semantic soft tokens (`theme.success_soft()`, `theme.danger_soft()`) and marker symbols (`+`, `-`, ` `) so that state is not communicated by color alone.
- Monospace line-number columns (old / new) share a three-character minimum gutter that expands for larger line numbers.
- Theme borders, stroke tokens (`STROKE_THIN`), and radii follow the design system foundations.

## Usage Example

```rust
use db_pro_ui::components::diff::{DiffLine, DiffViewer};
use db_pro_ui::DbProTheme;

let theme = DbProTheme::light();
let diff_lines = [
    DiffLine::context(1, 1, "CREATE TABLE users ("),
    DiffLine::removed(2, "    status VARCHAR(20)"),
    DiffLine::added(2, "    status user_status_enum NOT NULL DEFAULT 'active'"),
    DiffLine::context(3, 3, ");"),
];

DiffViewer::new("0042_alter_users_status.sql", &diff_lines, theme).show(ui);
```

## Layers

- `mod.rs`: Component entry point and public re-exports.
- `ui.rs`: egui layout, painting of rows, gutters, background highlights, and frame styling.
- `handler.rs`: Pure calculations for counting additions/removals, formatting summary badges, formatting line number columns, and resolving line visual styles.
- `config.rs`: Component-specific layout offsets, font sizes, and height constants.
