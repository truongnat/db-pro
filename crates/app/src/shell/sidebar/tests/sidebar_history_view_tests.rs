//! Tests for `../sidebar_history_view.rs` (moved out of the source file).
    use super::*;

    #[test]
    fn day_groups_use_local_calendar() {
        let today = chrono::Local::now().date_naive();
        assert!(day_label(Some(today), today).starts_with("TODAY"));
        let yesterday = today.pred_opt().unwrap();
        assert!(day_label(Some(yesterday), today).starts_with("YESTERDAY"));
        assert_eq!(day_label(None, today), "UNDATED");
    }

    #[test]
    fn truncate_marks_long_text() {
        assert_eq!(truncate("short", 10), "short");
        assert!(truncate("a much longer line of text", 10).ends_with('…'));
    }