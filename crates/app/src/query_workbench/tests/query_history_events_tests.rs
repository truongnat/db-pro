//! Tests for `../query_history_events.rs` (moved out of the source file).
    use super::*;

    fn record(sql: &str) -> QueryHistoryRecord {
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
        }
    }

    /// `len + last id` is the palette fingerprint key: both halves must change
    /// appropriately across pushes and retention drains (#500).
    fn history_key(entries: &[UiQueryHistoryEntry]) -> String {
        format!(
            "{}:{}",
            entries.len(),
            entries.last().map(|entry| entry.id.as_str()).unwrap_or("-")
        )
    }

    #[test]
    fn history_is_capped_at_retention_limit() {
        let mut editor = QueryEditorState::default();
        for i in 0..QUERY_HISTORY_RETENTION + 25 {
            record_query_history(&mut editor, record(&format!("select {i}")));
        }

        assert_eq!(editor.query_history_entries.len(), QUERY_HISTORY_RETENTION);
        assert_eq!(editor.query_history_entries.first().unwrap().sql, "select 25");
        assert_eq!(
            editor.query_history_entries.last().unwrap().sql,
            format!("select {}", QUERY_HISTORY_RETENTION + 24)
        );
    }

    #[test]
    fn push_that_drains_the_front_still_changes_fingerprint_key() {
        // Regression (#500): `len` alone is unchanged by a push+drain at the
        // cap, so a len-only fingerprint left the palette index stale.
        let mut editor = QueryEditorState::default();
        for i in 0..QUERY_HISTORY_RETENTION {
            record_query_history(&mut editor, record(&format!("select {i}")));
        }
        let before = history_key(&editor.query_history_entries);

        record_query_history(&mut editor, record("select overflow"));

        let after = history_key(&editor.query_history_entries);
        assert_ne!(before, after);
        assert_eq!(editor.query_history_entries.len(), QUERY_HISTORY_RETENTION);
    }

    #[test]
    fn every_entry_gets_a_unique_stable_id() {
        let mut editor = QueryEditorState::default();
        for i in 0..40 {
            record_query_history(&mut editor, record("select 1"));
            assert_eq!(editor.query_history_entries.len(), i + 1);
        }

        let ids: std::collections::BTreeSet<&str> = editor
            .query_history_entries
            .iter()
            .map(|entry| entry.id.as_str())
            .collect();
        assert_eq!(ids.len(), 40);
    }
