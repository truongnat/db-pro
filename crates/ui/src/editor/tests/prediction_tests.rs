//! Tests for `../prediction.rs` (moved out of the source file).
use super::*;

#[test]
fn test_edit_prediction_accept_helpers() {
    let pred = EditPrediction::new(10, "WHERE id = 100\nLIMIT 10;", None);
    assert_eq!(pred.accept_full(), "WHERE id = 100\nLIMIT 10;");
    assert_eq!(pred.accept_next_word(), "WHERE");
    assert_eq!(pred.accept_next_line(), "WHERE id = 100\n");
}

#[test]
fn partial_accept_moves_anchor_after_replacement_range() {
    let mut pred = EditPrediction::with_range_and_version(8, (4, 8), "SELECT", None, 1);
    pred.consume(3);
    assert_eq!(pred.anchor, 7);
    assert_eq!(pred.replacement_range, (7, 7));
    assert_eq!(pred.text, "ECT");
}

#[test]
fn test_prediction_state_stale_rejection() {
    let mut state = PredictionState::new();
    state.document_version = 2;
    let req_id = RequestId(101);
    state.pending_request = Some(req_id);
    state.status = PredictionStatus::Generating(req_id);

    // Response from older version (v1) should be rejected
    let accepted = state.apply_response(req_id, 1, "SELECT 1;".to_owned(), 5);
    assert!(!accepted);
    assert_eq!(state.active_prediction, None);

    // Response from matching version (v2) should be accepted
    let accepted = state.apply_response(req_id, 2, "SELECT 1;".to_owned(), 5);
    assert!(accepted);
    assert_eq!(state.status, PredictionStatus::Ready);
    assert_eq!(state.active_prediction.as_ref().unwrap().text, "SELECT 1;");

    // Buffer changed -> invalidation
    state.notify_buffer_changed(3);
    assert_eq!(state.active_prediction, None);
    assert_eq!(state.status, PredictionStatus::Idle);
}

#[test]
fn context_fingerprint_is_stable_for_alias_map_order() {
    let mut first = AiSqlContext::default();
    first.table_aliases.insert("u".to_owned(), "users".to_owned());
    first.table_aliases.insert("o".to_owned(), "orders".to_owned());
    let mut second = first.clone();
    second.table_aliases.clear();
    second.table_aliases.insert("o".to_owned(), "orders".to_owned());
    second.table_aliases.insert("u".to_owned(), "users".to_owned());
    assert_eq!(first.fingerprint(3, 12), second.fingerprint(3, 12));
}

#[test]
fn overlap_normalization_removes_duplicate_prefix_and_suffix() {
    assert_eq!(
        normalize_prediction_overlap(
            "SELECT * FROM users WHERE ",
            " LIMIT 10",
            "WHERE active = true LIMIT 10"
        ),
        "active = true"
    );
    assert_eq!(
        normalize_prediction_overlap("ORDER BY created_at", " DESC", " DESC"),
        ""
    );
    assert_eq!(normalize_prediction_overlap("SELECT ", "", "users"), "users");
}

#[test]
fn overlap_is_utf8_safe_and_does_not_remove_whitespace_only_overlap() {
    assert_eq!(
        normalize_prediction_overlap("SELECT \"tên\" FROM khách_hàng", "", "khách_hàng WHERE"),
        " WHERE"
    );
    assert_eq!(normalize_prediction_overlap("SELECT ", "", " "), " ");
    assert_eq!(normalize_prediction_overlap("SELECT ", "", " WHERE"), " WHERE");
}

#[test]
fn partial_accept_handles_unicode_leading_whitespace() {
    let prediction = EditPrediction::new(0, "\u{00a0}tên FROM users", None);
    assert_eq!(prediction.accept_next_word(), "\u{00a0}tên");
}

#[test]
fn prediction_guard_rejects_empty_prose_and_unbounded_output() {
    assert!(!is_prediction_acceptable(""));
    assert!(!is_prediction_acceptable("Here is a suggestion"));
    assert!(!is_prediction_acceptable(&"x".repeat(MAX_PREDICTION_CHARS + 1)));
    assert!(is_prediction_acceptable("\n  WHERE active = true"));
}
