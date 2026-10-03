//! Static SQL snippets shared by the query editor, palette and sidebar.

pub(super) fn builtin_sql_snippets() -> &'static [(&'static str, &'static str, &'static str)] {
    &[
        ("SELECT table", "sel*", "SELECT *\nFROM table_name\nLIMIT 100;"),
        (
            "UPDATE by primary key",
            "upd*",
            "UPDATE table_name\nSET column_name = value\nWHERE id = 1;",
        ),
        (
            "INSERT row",
            "ins*",
            "INSERT INTO table_name (column_a, column_b)\nVALUES ($1, $2);",
        ),
        ("DELETE with WHERE", "del*", "DELETE FROM table_name\nWHERE id = $1;"),
        (
            "EXPLAIN ANALYZE",
            "exp*",
            "EXPLAIN (ANALYZE, BUFFERS)\nSELECT *\nFROM table_name\nWHERE id = $1;",
        ),
        (
            "CREATE INDEX",
            "idx*",
            "CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_table_column\nON table_name (column_name);",
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::builtin_sql_snippets;

    #[test]
    fn snippets_are_stable_and_non_empty() {
        let snippets = builtin_sql_snippets();

        assert_eq!(snippets.len(), 6);
        assert!(snippets
            .iter()
            .all(|(label, trigger, sql)| !label.is_empty() && !trigger.is_empty() && !sql.is_empty()));
    }
}
