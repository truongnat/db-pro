//! Tests for `../query_snippets.rs` (moved out of the source file).
    use super::builtin_sql_snippets;

    #[test]
    fn snippets_are_stable_and_non_empty() {
        let snippets = builtin_sql_snippets();

        assert_eq!(snippets.len(), 6);
        assert!(snippets
            .iter()
            .all(|(label, trigger, sql)| !label.is_empty() && !trigger.is_empty() && !sql.is_empty()));
        let insert = snippets.iter().find(|(label, _, _)| *label == "INSERT row").unwrap().2;
        assert!(insert.contains("($1, $2)"));
        assert!(insert.contains("${1:table_name}"));
    }