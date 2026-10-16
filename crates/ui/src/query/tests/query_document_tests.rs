//! Tests for `../query_document.rs` (moved out of the source file).
use super::*;

#[test]
fn test_query_document_resolve_executable_sql() {
    let mut doc = QueryDocument::new("doc-1", "Doc 1", "SELECT 1;\nSELECT 2;\nSELECT 3;");

    // 1. Cursor on first statement
    doc.cursor.set_offset(&doc.buffer, 3);
    let (sql, _) = doc.resolve_executable_sql();
    assert_eq!(sql, "SELECT 1;");

    // 2. Cursor on second statement
    doc.cursor.set_offset(&doc.buffer, 12);
    let (sql, _) = doc.resolve_executable_sql();
    assert_eq!(sql, "SELECT 2;");

    // 3. Selection active
    doc.selection.anchor = 0;
    doc.selection.active = 19;
    let (sql, _) = doc.resolve_executable_sql();
    assert_eq!(sql, "SELECT 1;\nSELECT 2;");
}

#[test]
fn blank_line_after_a_semicolon_runs_that_statement() {
    let mut doc = QueryDocument::new("doc-1", "Doc 1", "SELECT 1;\n\nSELECT 2;");
    doc.cursor.set_offset(&doc.buffer, 10);
    let (sql, _) = doc.resolve_executable_sql();
    assert_eq!(sql, "SELECT 1;");
}

#[test]
fn test_query_document_search_matches() {
    let mut doc = QueryDocument::new("doc-1", "Doc 1", "SELECT id, name FROM users WHERE id = 1;");
    doc.search.query = "id".to_owned();
    let text = doc.text().to_owned();
    doc.search.update_matches(&text);
    assert_eq!(doc.search.matches.len(), 2);
    assert_eq!(doc.search.matches[0], (7, 9));
    assert_eq!(doc.search.matches[1], (33, 35));

    let next = doc.search.next_match();
    assert_eq!(next, Some((33, 35)));
    let prev = doc.search.prev_match();
    assert_eq!(prev, Some((7, 9)));
}

#[test]
fn query_document_search_keeps_unicode_expansion_offsets_in_original_text() {
    let mut doc = QueryDocument::new("doc-unicode", "Unicode", "İx i̇");
    let text = doc.text().to_owned();
    doc.search.query = "x".to_owned();
    doc.search.update_matches(&text);
    assert_eq!(doc.search.matches, vec![(2, 3)]);

    doc.search.query = "i".to_owned();
    doc.search.update_matches(&text);
    assert_eq!(doc.search.matches, vec![(0, 2), (4, 5)]);
}

#[test]
fn test_query_document_serde_roundtrip() {
    let mut doc = QueryDocument::new("q-1", "My Query", "SELECT * FROM users;");
    doc.connection_id = Some("conn-123".to_owned());
    doc.schema = Some("public".to_owned());
    doc.cursor.set_offset(&doc.buffer, 14);
    doc.selection = crate::editor::SelectionRange::new(7, 14);
    doc.scroll = 42.5;

    let serialized = serde_json::to_string(&doc).unwrap();
    let deserialized: QueryDocument = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized.id, "q-1");
    assert_eq!(deserialized.title, "My Query");
    assert_eq!(deserialized.text(), "SELECT * FROM users;");
    assert_eq!(deserialized.connection_id.as_deref(), Some("conn-123"));
    assert_eq!(deserialized.schema.as_deref(), Some("public"));
    assert_eq!(deserialized.cursor.offset, 14);
    assert_eq!(deserialized.selection.anchor, 7);
    assert_eq!(deserialized.selection.active, 14);
    assert_eq!(deserialized.scroll, 42.5);
    assert!(!deserialized.is_dirty());
}

#[test]
fn undoing_to_saved_snapshot_clears_dirty_state() {
    let mut doc = QueryDocument::new("q-1", "Query", "SELECT 1;");
    doc.mark_saved();

    doc.set_text("SELECT 2;");
    assert!(doc.is_dirty());

    doc.set_text("SELECT 1;");
    assert!(!doc.is_dirty());
}

#[test]
fn prediction_schedule_is_due_only_after_debounce_and_can_be_consumed() {
    let mut doc = QueryDocument::new("doc-1", "Doc 1", "SELECT 1");
    let now = Instant::now();
    doc.schedule_prediction(now);
    assert!(!doc.prediction_is_due(now));
    assert!(doc.prediction_is_due(now + SQL_PREDICTION_DEBOUNCE));
    assert!(doc.take_prediction_schedule());
    assert!(!doc.prediction_is_due(now + SQL_PREDICTION_DEBOUNCE));
}

#[test]
fn cached_prediction_is_reused_only_for_the_same_fingerprint() {
    let mut doc = QueryDocument::new("doc-1", "Doc 1", "SELECT ");
    let prediction = EditPrediction::new(7, "* FROM users", None);
    let now = Instant::now();
    doc.cache_prediction(41, prediction.clone(), now);
    assert_eq!(doc.take_cached_prediction(41, now).as_ref(), Some(&prediction));
    assert!(doc.take_cached_prediction(42, now).is_none());
}

#[test]
fn manual_prediction_schedule_bypasses_debounce() {
    let mut doc = QueryDocument::new("doc-1", "Doc 1", "SELECT ");
    let now = Instant::now();
    doc.schedule_prediction_with_mode(now, true);
    assert!(doc.prediction_is_due(now));
    assert!(doc.take_prediction_manual());
}

#[test]
fn prediction_request_is_deduped_within_cache_window() {
    let mut doc = QueryDocument::new("doc-1", "Doc 1", "SELECT ");
    let now = Instant::now();
    doc.prediction_last_request_fingerprint = Some(7);
    doc.prediction_last_request_at = Some(now);
    assert!(doc.should_dedupe_prediction(7, now));
    assert!(!doc.should_dedupe_prediction(8, now));
}
