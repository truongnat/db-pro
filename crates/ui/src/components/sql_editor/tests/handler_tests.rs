//! Tests for `../handler.rs` (moved out of the source file).
use super::*;

#[test]
fn test_action_labels() {
    assert_eq!(action_label(SqlEditorAction::RunQuery), "Run");
    assert_eq!(action_label(SqlEditorAction::RunSelection), "Run Selection");
    assert_eq!(action_label(SqlEditorAction::ExplainQuery), "Explain");
    assert_eq!(action_label(SqlEditorAction::FormatSql), "Format");
    assert_eq!(action_label(SqlEditorAction::CancelQuery), "Cancel");
    assert_eq!(action_label(SqlEditorAction::AskAi), "Ask AI");
}

#[test]
fn test_action_icons() {
    assert_eq!(
        action_icon(SqlEditorAction::RunQuery).map(char::from),
        Some(char::from(Icon::Play))
    );
    assert!(action_icon(SqlEditorAction::RunSelection).is_none());
    assert_eq!(
        action_icon(SqlEditorAction::ExplainQuery).map(char::from),
        Some(char::from(Icon::ChartNoAxesCombined))
    );
    assert_eq!(
        action_icon(SqlEditorAction::FormatSql).map(char::from),
        Some(char::from(Icon::AlignLeft))
    );
    assert_eq!(
        action_icon(SqlEditorAction::CancelQuery).map(char::from),
        Some(char::from(Icon::Square))
    );
    assert_eq!(
        action_icon(SqlEditorAction::AskAi).map(char::from),
        Some(char::from(Icon::Sparkles))
    );
}

#[test]
fn test_action_variants() {
    assert_eq!(action_variant(SqlEditorAction::RunQuery), ButtonVariant::Default);
    assert_eq!(action_variant(SqlEditorAction::RunSelection), ButtonVariant::Outline);
    assert_eq!(action_variant(SqlEditorAction::ExplainQuery), ButtonVariant::Outline);
    assert_eq!(action_variant(SqlEditorAction::FormatSql), ButtonVariant::Ghost);
    assert_eq!(action_variant(SqlEditorAction::CancelQuery), ButtonVariant::Destructive);
    assert_eq!(action_variant(SqlEditorAction::AskAi), ButtonVariant::Outline);
}

#[test]
fn test_running_and_selection_visibility_rules() {
    // When query is running:
    assert!(should_show_cancel(true));
    assert!(!should_show_idle_actions(true));
    assert!(!should_show_run_selection(true, true));
    assert!(!should_show_run_selection(true, false));

    // When idle with selection:
    assert!(!should_show_cancel(false));
    assert!(should_show_idle_actions(false));
    assert!(should_show_run_selection(false, true));

    // When idle without selection:
    assert!(!should_show_run_selection(false, false));
}
