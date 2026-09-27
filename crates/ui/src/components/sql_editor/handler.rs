use crate::components::button::ButtonVariant;
use lucide_icons::Icon;

/// Actions that can be triggered from the SQL Editor toolbar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SqlEditorAction {
    RunQuery,
    RunSelection,
    ExplainQuery,
    FormatSql,
    CancelQuery,
    AskAi,
}

/// Returns the user-facing text label for an action.
pub fn action_label(action: SqlEditorAction) -> &'static str {
    match action {
        SqlEditorAction::RunQuery => "Run",
        SqlEditorAction::RunSelection => "Run Selection",
        SqlEditorAction::ExplainQuery => "Explain",
        SqlEditorAction::FormatSql => "Format",
        SqlEditorAction::CancelQuery => "Cancel",
        SqlEditorAction::AskAi => "Ask AI",
    }
}

/// Returns the Lucide icon associated with an action.
pub fn action_icon(action: SqlEditorAction) -> Option<Icon> {
    match action {
        SqlEditorAction::RunQuery => Some(Icon::Play),
        SqlEditorAction::RunSelection => None,
        SqlEditorAction::ExplainQuery => Some(Icon::ChartNoAxesCombined),
        SqlEditorAction::FormatSql => Some(Icon::AlignLeft),
        SqlEditorAction::CancelQuery => Some(Icon::Square),
        SqlEditorAction::AskAi => Some(Icon::Sparkles),
    }
}

/// Returns the visual button variant for an action.
pub fn action_variant(action: SqlEditorAction) -> ButtonVariant {
    match action {
        SqlEditorAction::RunQuery => ButtonVariant::Default,
        SqlEditorAction::RunSelection => ButtonVariant::Outline,
        SqlEditorAction::ExplainQuery => ButtonVariant::Outline,
        SqlEditorAction::FormatSql => ButtonVariant::Ghost,
        SqlEditorAction::CancelQuery => ButtonVariant::Destructive,
        SqlEditorAction::AskAi => ButtonVariant::Outline,
    }
}

/// Returns the accessible label or description for screen readers and tooltips.
pub fn action_access_label(action: SqlEditorAction) -> &'static str {
    match action {
        SqlEditorAction::RunQuery => "Execute current query (⌘Enter)",
        SqlEditorAction::RunSelection => "Execute selected SQL statements (⇧⌘Enter)",
        SqlEditorAction::ExplainQuery => "Generate query execution plan",
        SqlEditorAction::FormatSql => "Format SQL document (⇧⌥F)",
        SqlEditorAction::CancelQuery => "Cancel running query execution",
        SqlEditorAction::AskAi => "Open AI SQL assistant (⌘I)",
    }
}

/// Returns whether the Cancel button should be displayed instead of normal run actions.
pub fn should_show_cancel(is_running: bool) -> bool {
    is_running
}

/// Returns whether idle execution buttons (Run, Explain, Format) should be shown.
pub fn should_show_idle_actions(is_running: bool) -> bool {
    !is_running
}

/// Returns whether the "Run Selection" button should be visible.
pub fn should_show_run_selection(is_running: bool, has_selection: bool) -> bool {
    !is_running && has_selection
}

#[cfg(test)]
mod tests {
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
}
