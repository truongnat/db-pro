# Table API

`Table`, `TableColumn`, and `TableColumnAlign` are public through `components::table` and `components`.

## Columns

`TableColumn::new(title)` creates a flexible left-aligned, non-sortable column. `TableColumn::fixed(title, width)` creates a fixed-width column. Builders `width(f32)`, `align(Left|Center|Right)`, and `sortable(bool)` configure it; fields are public for direct construction.

## Table

`Table::new(&columns, theme)` defaults to no row selection, no active sort, default row height, and no vertical grid. Configure:

- `selectable(enabled, all_selected)` to show row checkboxes and select-all state.
- `indeterminate(bool)` for the mixed selection marker.
- `sort(Option<column_index>, descending)` to show the current sort state.
- `row_height(f32)` and `vertical_grid(bool)` for row geometry and dividers.

`show(ui, row_count, is_row_selected, on_toggle_all, on_toggle_row, on_sort, render_cell)` returns unit. `is_row_selected(row)` supplies the current selection state. The three `FnMut` callbacks receive the requested new select-all state, row index, or sorted column index. `render_cell(ui, row, column)` paints each visible cell; callers decide its contents and apply resulting state changes. The widget does not sort rows or store selection.

With `row_count == 0`, the empty state is shown. Offscreen rows are not painted or sent through `render_cell`, though row indices are still checked for visibility each frame. For very large datasets, page rows in the caller. A table with more columns than available width scrolls horizontally.
