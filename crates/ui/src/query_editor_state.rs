use super::*;

/// Owns query-editor interaction, visual-builder drafts and diagnostics caches.
#[derive(Debug)]
pub(crate) struct QueryEditorState {
    pub(crate) editor_search: String,
    pub(crate) editor_search_open: bool,
    pub(crate) query_editor_focused: bool,
    pub(crate) query_focus_editor_on_open: bool,
    pub(crate) query_cursor_line: usize,
    pub(crate) query_cursor_column: usize,
    pub(crate) editor_font_size: f32,
    pub(crate) query_tools_open: bool,
    pub(crate) query_context_picker_open: bool,
    pub(crate) query_params_panel_open: bool,
    pub(crate) query_output_dock_maximized: bool,
    pub(crate) query_editor_rect: egui::Rect,
    pub(crate) completion_open: bool,
    pub(crate) snippets_open: bool,
    pub(crate) visual_query_builder_open: bool,
    pub(crate) visual_query_model: crate::query::visual_builder::VisualQueryModel,
    pub(crate) visual_query_sql_preview: String,
    pub(crate) visual_query_error: Option<String>,
    pub(crate) visual_query_add_table: String,
    pub(crate) visual_query_join_table: String,
    pub(crate) visual_query_join_left: String,
    pub(crate) visual_query_join_right: String,
    pub(crate) visual_query_col_ref: String,
    pub(crate) visual_query_col_alias: String,
    pub(crate) visual_query_col_agg: String,
    pub(crate) visual_query_where_left: String,
    pub(crate) visual_query_where_op: String,
    pub(crate) visual_query_where_value: String,
    pub(crate) visual_query_order: String,
    pub(crate) visual_query_order_desc: bool,
    pub(crate) visual_query_limit: String,
    pub(crate) visual_query_offset: String,
    pub(crate) diagnostics: Vec<String>,
    pub(crate) diagnostics_cache_key: Option<(usize, u64)>,
    pub(crate) diagnostics_cache_driver: String,
    pub(crate) diagnostics_lint_structured: Vec<crate::editor::Diagnostic>,
    pub(crate) diagnostics_debounce_key: Option<(usize, u64)>,
    pub(crate) diagnostics_debounce_at: Option<std::time::Instant>,
    pub(crate) diagnostics_exec_fp: Option<(usize, usize)>,
    pub(crate) param_count_cache_key: Option<(usize, u64)>,
    pub(crate) param_count_cache: usize,
    pub(crate) problems_severity_filter: ProblemsSeverityFilter,
    pub(crate) problems_source_filter: ProblemsSourceFilter,
    pub(crate) problems_selected: Option<(String, usize)>,
    pub(crate) query_history: Vec<String>,
    pub(crate) query_history_entries: Vec<UiQueryHistoryEntry>,
    pub(crate) query_history_search: String,
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
            visual_query_builder_open: false,
            visual_query_model: crate::query::visual_builder::VisualQueryModel::default(),
            visual_query_sql_preview: String::new(),
            visual_query_error: None,
            visual_query_add_table: String::new(),
            visual_query_join_table: String::new(),
            visual_query_join_left: String::new(),
            visual_query_join_right: String::new(),
            visual_query_col_ref: String::new(),
            visual_query_col_alias: String::new(),
            visual_query_col_agg: String::new(),
            visual_query_where_left: String::new(),
            visual_query_where_op: "=".to_owned(),
            visual_query_where_value: String::new(),
            visual_query_order: String::new(),
            visual_query_order_desc: false,
            visual_query_limit: "100".to_owned(),
            visual_query_offset: String::new(),
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
        assert_eq!(state.visual_query_where_op, "=");
        assert_eq!(state.visual_query_limit, "100");
        assert!(state.diagnostics.is_empty());
    }
}
