//! Tests for `../query_editor_panel.rs` (moved out of the source file).
    use super::query_editor_support::apply_completion_item;
    use super::*;

    fn table_completion(replacement_range: (usize, usize)) -> CompletionItem {
        CompletionItem {
            label: "users".to_owned(),
            insert_text: "users".to_owned(),
            kind: CompletionItemKind::Table,
            detail: Some("Table · public.users".to_owned()),
            documentation: Some("Table with 3 columns".to_owned()),
            replacement_range,
            sort_score: 900,
        }
    }

    #[test]
    fn completion_applies_only_to_the_document_version_that_produced_it() {
        let mut document = QueryDocument::new("query-1", "Query 1", "SELECT * FROM us");
        let version = document.buffer.version();
        let item = table_completion((14, 16));
        document.completion.open(
            16,
            version,
            egui::Pos2::ZERO,
            "us".to_owned(),
            vec![item.clone()],
            crate::editor::CompletionTriggerKind::Manual,
        );

        document.buffer.insert(16, "x");
        assert!(!apply_completion_item(&mut document, &item, SqlDialect::Postgres));
        assert_eq!(document.buffer.text(), "SELECT * FROM usx");
        assert!(!document.completion.is_open);
    }

    #[test]
    fn current_completion_replaces_only_its_declared_prefix() {
        let mut document = QueryDocument::new("query-1", "Query 1", "SELECT * FROM us");
        let version = document.buffer.version();
        let item = table_completion((14, 16));
        document.completion.open(
            16,
            version,
            egui::Pos2::ZERO,
            "us".to_owned(),
            vec![item.clone()],
            crate::editor::CompletionTriggerKind::Manual,
        );

        assert!(apply_completion_item(&mut document, &item, SqlDialect::Postgres));
        assert_eq!(document.buffer.text(), "SELECT * FROM users");
        assert_eq!(document.cursor.offset, document.buffer.len_bytes());
        assert!(!document.completion.is_open);
    }