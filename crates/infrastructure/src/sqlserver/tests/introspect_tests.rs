use super::*;

#[test]
fn catalog_foreign_key_actions_use_sql_spellings() {
    for (update, delete, expected_update, expected_delete) in [
        ("NO_ACTION", "NO_ACTION", "NO ACTION", "NO ACTION"),
        ("SET_NULL", "SET_DEFAULT", "SET NULL", "SET DEFAULT"),
        ("SET_DEFAULT", "SET_NULL", "SET DEFAULT", "SET NULL"),
        ("CASCADE", "CASCADE", "CASCADE", "CASCADE"),
    ] {
        let row = Row(vec![CellValue::Text(update.into()), CellValue::Text(delete.into())]);
        assert_eq!(foreign_key_action(&row, 0).unwrap(), expected_update);
        assert_eq!(foreign_key_action(&row, 1).unwrap(), expected_delete);
    }
}
