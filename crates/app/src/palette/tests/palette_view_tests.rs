//! Tests for `../palette_view.rs` (moved out of the source file).
    use super::*;
    use super::super::events::QueryHistoryRecord;
    use super::super::query_history_events::record_query_history;

    fn push_history(app: &mut DbProApp, sql: &str) {
        record_query_history(
            &mut app.query.editor,
            QueryHistoryRecord {
                sql: sql.to_owned(),
                connection_id: None,
                schema: None,
                started_at: "2026-10-17T00:00:00Z".to_owned(),
                status: UiQueryHistoryStatus::Success,
                duration_ms: 1,
                row_count: None,
                affected_rows: None,
                error_code: None,
                error_summary: None,
            },
        );
    }

    #[test]
    fn fingerprint_changes_when_push_drains_history_front() {
        // Regression (#500): at the 500-entry retention cap a push drains the
        // front, so `len` alone was a stale fingerprint for the palette index.
        let mut app = DbProApp::default();
        for i in 0..500 {
            push_history(&mut app, &format!("select {i}"));
        }
        let before = app.search_fingerprint();
        push_history(&mut app, "select overflow");
        assert_eq!(app.query.editor.query_history_entries.len(), 500);
        assert_ne!(before, app.search_fingerprint());
    }

    #[test]
    fn palette_lists_history_newest_first_with_entry_id_actions() {
        let mut app = DbProApp::default();
        for i in 0..5 {
            push_history(&mut app, &format!("select marker_{i}"));
        }
        let newest_id = app.query.editor.query_history_entries.last().unwrap().id.clone();
        let oldest_id = app.query.editor.query_history_entries.first().unwrap().id.clone();

        let history: Vec<_> = app
            .palette_entries(PaletteMode::QuickOpen)
            .into_iter()
            .filter(|(kind, _)| *kind == SearchKind::History)
            .collect();

        assert_eq!(history.len(), 5);
        assert!(history[0].1.title.contains("marker_4"));
        assert!(history[4].1.title.contains("marker_0"));
        match &history[0].1.action {
            PaletteAction::OpenHistoryEntry(id) => assert_eq!(id, &newest_id),
            other => panic!("expected OpenHistoryEntry, got {other:?}"),
        }
        match &history[4].1.action {
            PaletteAction::OpenHistoryEntry(id) => assert_eq!(id, &oldest_id),
            other => panic!("expected OpenHistoryEntry, got {other:?}"),
        }
    }

    #[test]
    fn open_history_entry_action_opens_the_matching_entry_not_a_position() {
        // Regression (#500): position-based actions opened the wrong entry
        // after the front of history was drained.
        let mut app = DbProApp::default();
        for i in 0..502 {
            push_history(&mut app, &format!("select target_{i}"));
        }
        let target = app.query.editor.query_history_entries[0].clone();
        assert_eq!(target.sql, "select target_2");

        let documents_before = app.query.session.documents.len();
        app.execute_palette_action(
            PaletteAction::OpenHistoryEntry(target.id.clone()),
            &egui::Context::default(),
        );

        assert_eq!(app.query.session.documents.len(), documents_before + 1);
        assert_eq!(
            app.query.session.documents.last().unwrap().text(),
            target.sql
        );
    }

    #[test]
    fn open_history_entry_action_reports_missing_id() {
        let mut app = DbProApp::default();
        let documents_before = app.query.session.documents.len();
        app.execute_palette_action(
            PaletteAction::OpenHistoryEntry("no-such-entry".to_owned()),
            &egui::Context::default(),
        );
        assert_eq!(app.query.session.documents.len(), documents_before);
        assert_eq!(
            app.feedback.runtime_message,
            "History entry is no longer available"
        );
    }
    #[test]
    fn open_table_from_palette_clears_table_scoped_data_state() {
        // Regression: the palette path skipped reset_for_table, so the previous
        // table's sort column rode into the next LoadTableData — e.g. opening
        // products (ORDER BY "id") then role_permissions produced
        // `ORDER BY "id"` on a table with no `id` column.
        let mut app = DbProApp::default();
        app.schema.explorer.selected_table = Some("products".to_owned());
        app.table.data_query.sorts.push(UiTableDataSort {
            column: "id".to_owned(),
            descending: false,
        });
        app.table.data_query.filter_column = "id".to_owned();
        app.table.data_query.sql_condition_draft = "id = 1".to_owned();
        app.table.data_query.error = Some("stale error".to_owned());
        app.table.data_query.request = Some(RequestId(1));

        app.open_table_from_palette("role_permissions".to_owned());

        assert!(app.table.data_query.sorts.is_empty());
        assert!(app.table.data_query.filter_column.is_empty());
        assert!(app.table.data_query.sql_condition_draft.is_empty());
        assert!(app.table.data_query.error.is_none());
        assert!(app.table.data_query.request.is_none());
        assert!(app.table.data_query.result.is_none());
        assert_eq!(
            app.schema.explorer.selected_table.as_deref(),
            Some("role_permissions")
        );
    }
