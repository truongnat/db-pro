//! Tests for `../search_service.rs` (moved out of the source file).
    use super::super::PaletteAction;
    use super::*;
    use lucide_icons::Icon;

    fn item(title: &str, subtitle: &str) -> PaletteItem {
        PaletteItem {
            icon: Icon::Table2,
            title: title.to_owned(),
            subtitle: subtitle.to_owned(),
            shortcut: None,
            action: PaletteAction::Welcome,
        }
    }

    #[test]
    fn ranking_prefers_prefix_and_exact_title() {
        let entries = vec![
            (SearchKind::Table, item("users_archive", "Open table")),
            (SearchKind::Table, item("users", "Open table")),
            (SearchKind::Column, item("user_id", "Insert column")),
            (SearchKind::Command, item("Export results", "Export current results")),
        ];
        let ranked = SearchService::filter_rank(&entries, "users", SearchScope::All, 10);
        assert_eq!(ranked[0].title, "users");
        assert!(ranked.iter().any(|i| i.title == "users_archive"));
    }

    #[test]
    fn scope_filter_excludes_other_kinds() {
        let entries = vec![
            (SearchKind::Table, item("orders", "Open table")),
            (SearchKind::Command, item("New query", "Create scratch")),
            (SearchKind::Agent, item("Ask Agent", "Agent action")),
        ];
        let schema_only = SearchService::filter_rank(&entries, "", SearchScope::Schema, 10);
        assert_eq!(schema_only.len(), 1);
        assert_eq!(schema_only[0].title, "orders");

        let agent_only = SearchService::filter_rank(&entries, "", SearchScope::Agent, 10);
        assert_eq!(agent_only.len(), 1);
        assert_eq!(agent_only[0].title, "Ask Agent");
    }

    #[test]
    fn index_invalidates_when_fingerprint_changes() {
        let mut index = SearchIndex::default();
        index.ensure("fp-a".into(), || vec![(SearchKind::Table, item("a", "t"))]);
        assert_eq!(index.entries().len(), 1);
        assert_eq!(index.fingerprint(), "fp-a");
        index.ensure("fp-a".into(), || {
            vec![(SearchKind::Table, item("should-not-rebuild", "t"))]
        });
        assert_eq!(index.entries()[0].1.title, "a");
        index.invalidate();
        assert!(index.is_empty());
        index.ensure("fp-b".into(), || vec![(SearchKind::Table, item("b", "t"))]);
        assert_eq!(index.entries()[0].1.title, "b");
    }

    #[test]
    fn large_schema_search_is_bounded() {
        let entries: Vec<_> = (0..5_000)
            .map(|i| (SearchKind::Table, item(&format!("table_{i:04}"), "Open table")))
            .collect();
        let ranked = SearchService::filter_rank(&entries, "table_42", SearchScope::Schema, 80);
        assert!(ranked.len() <= 80);
        assert!(ranked
            .iter()
            .any(|i| i.title.contains("0042") || i.title.contains("42")));
    }