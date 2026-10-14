//! Tests for `../activity_bar_view.rs` (moved out of the source file).
    use super::*;

    #[test]
    fn activity_clicks_emit_navigation_intents_without_mutating_state() {
        assert!(matches!(
            activity_action(Some(Activity::Queries), "SQL Scripts & Projects"),
            Some(ActivityBarAction::OpenQuery)
        ));
        assert!(matches!(
            activity_action(Some(Activity::Tools), "Tools & Management Hub"),
            Some(ActivityBarAction::SelectActivity(Activity::Tools))
        ));
        assert!(matches!(
            activity_action(Some(Activity::Explorer), "Database Navigator"),
            Some(ActivityBarAction::SelectActivity(Activity::Explorer))
        ));
        assert!(matches!(
            activity_action(None, "AI Copilot (Agent)"),
            Some(ActivityBarAction::ToggleAgent)
        ));
    }