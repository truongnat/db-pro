use lucide_icons::Icon;

use crate::components::button::ButtonVariant;

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
#[path = "tests/handler_tests.rs"]
mod tests;
