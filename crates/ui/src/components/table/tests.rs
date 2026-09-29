use super::{Table, TableColumn, TableColumnAlign};
use crate::DbProTheme;

#[test]
fn column_builder_preserves_public_configuration() {
    let column = TableColumn::new("updated_at")
        .width(144.0)
        .align(TableColumnAlign::Right)
        .sortable(true);

    assert_eq!(column.title, "updated_at");
    assert_eq!(column.width, Some(144.0));
    assert_eq!(column.align, TableColumnAlign::Right);
    assert!(column.sortable);
}

#[test]
fn table_builder_keeps_the_default_row_height_and_options() {
    let columns = [TableColumn::new("id")];
    let table = Table::new(&columns, DbProTheme::light())
        .selectable(true, true)
        .indeterminate(true)
        .sort(Some(0), true)
        .row_height(52.0)
        .vertical_grid(true);

    assert!(table.selectable);
    assert!(table.all_selected);
    assert!(table.indeterminate);
    assert_eq!(table.sort_column, Some(0));
    assert!(table.sort_desc);
    assert_eq!(table.row_height, 52.0);
    assert!(table.show_vertical_grid);
}
