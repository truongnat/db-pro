//! Tests for `../query_helpers.rs` (moved out of the source file).
    use super::format_query_document;
    use crate::editor::{SelectionRange, SqlDialect};
    use crate::query::QueryDocument;

    #[test]
    fn format_query_selection_does_not_reformat_the_rest_of_the_buffer() {
        let sql = "SELECT 1 FROM t; SELECT 2 FROM u";
        let mut doc = QueryDocument::new("q", "Query", sql);
        doc.selection = SelectionRange::point(0);
        format_query_document(&mut doc, SqlDialect::Postgres);
        assert_eq!(doc.text(), sql, "an empty selection must not format the buffer");

        let start = sql.find("SELECT 2").expect("second statement");
        doc.selection = SelectionRange::new(start, sql.len());
        format_query_document(&mut doc, SqlDialect::Postgres);
        assert!(
            doc.text().starts_with("SELECT 1 FROM t; "),
            "text before the selection stays put: {}",
            doc.text()
        );
        assert!(
            doc.text().ends_with("SELECT 2\nFROM u"),
            "only the selection is formatted: {}",
            doc.text()
        );
    }