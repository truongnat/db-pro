//! Query output composition and the result-grid shell.
use super::*;

impl DbProApp {
    /// Selects an output pane and applies the pane's explicit action at the root boundary.
    pub(super) fn draw_output_pane(&mut self, ui: &mut egui::Ui, result: Option<&UiQueryResult>) {
        let active_tab = self.query.output.active_tab_for_document(
            self.query
                .session
                .active_document()
                .map(|document| document.id.as_str()),
        );
        match active_tab {
            OutputTab::Results => self.draw_results_pane(ui, result),
            OutputTab::Chart => self.draw_chart_pane(ui, result),
            OutputTab::Messages => self.draw_messages_pane(ui),
            OutputTab::Explain => self.draw_explain_pane(ui),
            OutputTab::History => self.draw_history_pane(ui),
        }
    }

    /// Results grid plus its row-count / export header.
    pub(super) fn draw_results_pane(&mut self, ui: &mut egui::Ui, result: Option<&UiQueryResult>) {
        let active_result_index = self
            .query
            .session
            .documents
            .get(self.query.session.active_document_index)
            .map_or(0, |document| document.active_result_index);
        let context = query_results_surface_view::QueryResultsSurfaceContext {
            theme: self.theme,
            result_count: self.query.session.active_result_count(),
            active_result_index,
            result,
        };
        if let Some(action) = query_results_surface_view::draw_results(&context, ui, |ui, result| {
            self.draw_result_grid(ui, result);
        }) {
            match action {
                query_results_surface_view::QueryResultsSurfaceAction::SelectResult(index) => {
                    self.set_active_query_result(index);
                }
                query_results_surface_view::QueryResultsSurfaceAction::OpenExport => {
                    self.overlay.export_open = true;
                }
            }
        }
        self.draw_export_dialog(ui, result);
        self.draw_destructive_run_dialog(ui);
    }

    fn draw_messages_pane(&mut self, ui: &mut egui::Ui) {
        let mut context = query_output_panes_view::QueryOutputPanesContext {
            theme: self.theme,
            session: &mut self.query.session,
        };
        query_output_panes_view::draw_messages_pane(&mut context, ui);
    }

    fn draw_chart_pane(&mut self, ui: &mut egui::Ui, result: Option<&UiQueryResult>) {
        let mut context = query_output_panes_view::QueryOutputPanesContext {
            theme: self.theme,
            session: &mut self.query.session,
        };
        query_output_panes_view::draw_chart_pane(&mut context, ui, result);
    }

    fn draw_explain_pane(&mut self, ui: &mut egui::Ui) {
        let action = {
            let mut context = query_output_actions_view::QueryOutputActionsContext {
                theme: self.theme,
                session: &self.query.session,
                editor: &mut self.query.editor,
                execution: &mut self.query.execution,
                feedback: &mut self.feedback,
            };
            query_output_actions_view::draw_explain_pane(&mut context, ui)
        };
        if let Some(action) = action {
            self.apply_query_output_action(action);
        }
    }

    fn draw_history_pane(&mut self, ui: &mut egui::Ui) {
        let action = {
            let mut context = query_output_actions_view::QueryOutputActionsContext {
                theme: self.theme,
                session: &self.query.session,
                editor: &mut self.query.editor,
                execution: &mut self.query.execution,
                feedback: &mut self.feedback,
            };
            query_output_actions_view::draw_history_pane(&mut context, ui)
        };
        if let Some(action) = action {
            self.apply_query_output_action(action);
        }
    }

    fn apply_query_output_action(&mut self, action: query_output_actions_view::QueryOutputAction) {
        match action {
            query_output_actions_view::QueryOutputAction::Explain => self.explain_query(),
            query_output_actions_view::QueryOutputAction::ExplainAnalyze => self.explain_query_analyze(),
            query_output_actions_view::QueryOutputAction::OpenHistory(entry, run) => {
                self.open_history_entry(&entry, run);
            }
        }
    }
}
