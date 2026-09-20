use super::*;

/// Owns query-editor interaction, visual-builder drafts and diagnostics caches.
#[derive(Debug)]
pub(crate) struct QueryEditorState {
    pub(super) editor_search: String,
    pub(super) editor_search_open: bool,
    pub(super) query_editor_focused: bool,
    pub(super) query_focus_editor_on_open: bool,
    pub(super) query_cursor_line: usize,
    pub(super) query_cursor_column: usize,
    pub(super) editor_font_size: f32,
    pub(super) query_tools_open: bool,
    pub(super) query_context_picker_open: bool,
    pub(super) query_params_panel_open: bool,
    pub(super) query_output_dock_maximized: bool,
    pub(super) query_editor_rect: egui::Rect,
    pub(super) completion_open: bool,
    pub(super) snippets_open: bool,
    pub(super) visual_builder: super::visual_query_builder_state::VisualQueryBuilderState,
    pub(super) diagnostics: Vec<String>,
    pub(super) diagnostics_cache_key: Option<(usize, u64)>,
    pub(super) diagnostics_cache_driver: String,
    pub(super) diagnostics_lint_structured: Vec<crate::editor::Diagnostic>,
    pub(super) diagnostics_debounce_key: Option<(usize, u64)>,
    pub(super) diagnostics_debounce_at: Option<std::time::Instant>,
    pub(super) diagnostics_exec_fp: Option<(usize, usize)>,
    pub(super) param_count_cache_key: Option<(usize, u64)>,
    pub(super) param_count_cache: usize,
    pub(super) problems_severity_filter: ProblemsSeverityFilter,
    pub(super) problems_source_filter: ProblemsSourceFilter,
    pub(super) problems_selected: Option<(String, usize)>,
    pub(super) query_history: Vec<String>,
    pub(super) query_history_entries: Vec<UiQueryHistoryEntry>,
    pub(super) query_history_search: String,
}

impl Default for QueryEditorState {
    fn default() -> Self {
        Self {
            editor_search: String::new(),
            editor_search_open: false,
            query_editor_focused: false,
            query_focus_editor_on_open: true,
            query_cursor_line: 1,
            query_cursor_column: 1,
            editor_font_size: 14.0,
            query_tools_open: false,
            query_context_picker_open: false,
            query_params_panel_open: false,
            query_output_dock_maximized: false,
            query_editor_rect: egui::Rect::NOTHING,
            completion_open: false,
            snippets_open: false,
            visual_builder: super::visual_query_builder_state::VisualQueryBuilderState::default(),
            diagnostics: Vec::new(),
            diagnostics_cache_key: None,
            diagnostics_cache_driver: String::new(),
            diagnostics_lint_structured: Vec::new(),
            diagnostics_debounce_key: None,
            diagnostics_debounce_at: None,
            diagnostics_exec_fp: None,
            param_count_cache_key: None,
            param_count_cache: 0,
            problems_severity_filter: ProblemsSeverityFilter::All,
            problems_source_filter: ProblemsSourceFilter::All,
            problems_selected: None,
            query_history: Vec::new(),
            query_history_entries: Vec::new(),
            query_history_search: String::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_query_editor_state_starts_with_safe_editor_defaults() {
        let state = QueryEditorState::default();

        assert_eq!(state.query_cursor_line, 1);
        assert_eq!(state.query_cursor_column, 1);
        assert_eq!(state.editor_font_size, 14.0);
        assert_eq!(state.visual_builder.where_op, "=");
        assert_eq!(state.visual_builder.limit, "100");
        assert!(state.diagnostics.is_empty());
    }
}
