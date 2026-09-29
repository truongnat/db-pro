# Table

Shared native `egui` table with optional row selection, sortable headers,
virtualized rows, aligned cells, and optional vertical grid lines.

## Public API

```rust
use crate::components::{Table, TableColumn, TableColumnAlign};

let columns = [
    TableColumn::new("Name").sortable(true),
    TableColumn::fixed("Count", 96.0).align(TableColumnAlign::Right),
];

Table::new(&columns, theme).show(ui, rows.len(), is_selected, toggle_all, toggle_row, sort, render_cell);
```

## Layers

- `mod.rs` keeps the public column/table builders and stable exports.
- `ui.rs` owns egui painting and interaction wiring.
- `handler.rs` owns typed column geometry, table height calculations, and
  select-all state decisions.
- `config.rs` centralizes table-local semantic dimensions; shared design tokens
  remain in `crates/ui/src/tokens.rs`.
- `tests.rs` covers the public builder contract. Geometry/state invariants are
  tested next to their handler helpers.

The refactor preserves the existing `components::table::{Table, TableColumn,
TableColumnAlign}` exports and the `Table::show` callback contract.
