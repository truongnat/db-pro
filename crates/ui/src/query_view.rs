use super::*;
use std::time::Instant;

#[path = "query_layout_surface_view.rs"]
mod query_layout_surface_view;

/// Egress note shown with the AI prediction control (#242).
///
/// Inline prediction can still schedule without a click — default is `Subtle`
/// (`crates/ui/src/editor/prediction.rs`) so ghost text stays quieter until reveal,
/// and a request follows 300 ms after an edit or cursor move (`query_document.rs`).
/// The control that chooses the mode is where its data flow has to be stated. The
/// registry entry recording the same facts is `docs/release/known-limitations.md`
/// LIM-019; with no key configured the runtime answers "AI provider is not configured"
/// and nothing leaves the machine (`crates/runtime/src/worker.rs:1118`).
pub(super) const AI_PREDICTION_EGRESS_NOTE: &str =
    "Sends the SQL around your cursor and its schema context to your configured AI provider.";

#[path = "query_helpers.rs"]
mod query_helpers;
pub(crate) use query_helpers::{
    database_error_diagnostic, deduplicate_diagnostics, deduplicate_messages, format_query_document,
    prediction_replacement_range, write_file_atomically,
};

impl DbProApp {
    pub(super) fn draw_query(&mut self, ui: &mut egui::Ui) {
        egui::Frame::none()
            .inner_margin(egui::Margin {
                left: SPACE_XS,
                right: SPACE_XS,
                top: SPACE_XS,
                bottom: 0.0,
            })
            .show(ui, |ui| self.draw_query_content(ui));
    }

    fn draw_query_content(&mut self, ui: &mut egui::Ui) {
        self.refresh_query_diagnostics(ui);
        self.draw_query_context_chrome(ui);
        self.draw_query_transaction_chrome(ui);
        self.draw_visual_query_builder_surface(ui);
        let layout = query_layout_surface_view::calculate(query_layout_surface_view::QueryPanelLayoutContext {
            available_height: ui.available_height(),
            bottom_panel_open: self.workspace.bottom_panel_open,
            output_dock_maximized: self.query.editor.query_output_dock_maximized,
            bottom_panel_height: self.workspace.bottom_panel_height,
        });
        self.draw_query_editor_stack(ui, layout.editor_height);
        self.draw_query_optional_panels(ui);
        if layout.dock_open {
            self.draw_query_output_dock(ui, layout.dock_height);
        }
        self.draw_query_status_bar(ui);
        self.draw_dirty_close_dialog(ui.ctx());
        self.draw_save_as_dialog(ui.ctx());
    }

    fn refresh_query_diagnostics(&mut self, ui: &egui::Ui) {
        let driver = self.active_driver().to_owned();
        let lint = self.preferences.settings.editor.lint.clone();
        query_diagnostics_view::refresh_diagnostics(&mut self.query, &driver, &lint);
        if let Some(deadline) = self.query.editor.diagnostics_debounce_at {
            let remaining = deadline.saturating_duration_since(Instant::now());
            if !remaining.is_zero() {
                ui.ctx().request_repaint_after(remaining);
            }
        }
    }

    fn draw_query_context_chrome(&mut self, ui: &mut egui::Ui) {
        let doc_idx = self.query.session.active_document_index;
        let file_path = self
            .query
            .session
            .documents
            .get(doc_idx)
            .and_then(|document| document.file_path.clone());
        let connected = self.active_query_connection_id().is_some() && self.connection.lifecycle.is_connected();
        let connection_name = if connected {
            self.active_query_connection_name().to_owned()
        } else {
            "No connection".to_owned()
        };
        let schema = self.active_query_schema().to_owned();
        let environment = self
            .active_query_connection()
            .map(|connection| connection.environment.clone())
            .unwrap_or_default();
        let anchors = {
            let mut context = query_context_view::QueryContextViewContext {
                theme: self.theme,
                editor: &mut self.query.editor,
                file_path: file_path.as_deref(),
                connected,
                connection_name: &connection_name,
                schema: &schema,
                environment: &environment,
            };
            query_context_view::draw_context_strip(&mut context, ui)
        };
        if self.query.editor.query_context_picker_open {
            if let Some(anchor) = anchors.context_anchor {
                self.draw_query_context_picker(ui.ctx(), anchor);
            }
        }
        if self.query.editor.query_tools_open {
            if let Some(anchor) = anchors.more_anchor {
                self.draw_query_actions_menu(ui.ctx(), anchor);
            }
        }
    }

    fn draw_query_transaction_chrome(&mut self, ui: &mut egui::Ui) {
        let actions = query_transaction_surface_view::QueryTransactionSurfaceContext {
            theme: self.theme,
            execution: &mut self.query.execution,
        }
        .draw(ui);
        for action in actions {
            match action {
                query_transaction_surface_view::QueryTransactionAction::Transaction(action) => {
                    self.handle_transaction_action(action);
                }
                query_transaction_surface_view::QueryTransactionAction::DismissDisconnectGuard => {
                    self.query.execution.disconnect_txn_guard = false;
                }
            }
        }
    }

    fn draw_visual_query_builder_surface(&mut self, ui: &mut egui::Ui) {
        if !self.query.editor.visual_builder.open {
            return;
        }
        ui.add_space(SPACE_XS);
        egui::CollapsingHeader::new("Visual query builder")
            .default_open(true)
            .show(ui, |ui| self.draw_visual_query_builder(ui));
        ui.add_space(SPACE_XS);
    }

    fn draw_query_editor_stack(&mut self, ui: &mut egui::Ui, editor_height: f32) {
        ui.allocate_ui_with_layout(
            egui::vec2(ui.available_width(), editor_height),
            Layout::top_down(Align::Min),
            |ui| self.draw_query_editor(ui),
        );
        self.draw_floating_completion_popup(ui.ctx());
        let mut context = query_search_view::QuerySearchContext {
            theme: self.theme,
            editor: &mut self.query.editor,
            session: &mut self.query.session,
        };
        query_search_view::draw_editor_search_overlay(&mut context, ui.ctx());
    }

    fn draw_query_optional_panels(&mut self, ui: &mut egui::Ui) {
        if self.query.editor.snippets_open {
            self.draw_sql_snippets(ui);
        }
        if self.query.editor.query_params_panel_open {
            self.draw_sql_parameters_panel(ui);
        }
    }

    fn draw_query_context_picker(&mut self, ctx: &egui::Context, anchor: egui::Rect) {
        let doc_idx = self.query.session.active_document_index;
        let current_conn_id = self
            .query
            .session
            .documents
            .get(doc_idx)
            .and_then(|d| d.connection_id.clone())
            .or_else(|| self.connection.lifecycle.active_connection_id().map(str::to_owned));
        let current_schema = self.active_query_schema().to_owned();
        let available_schemas = if !self.schema.explorer.schema.schemas.is_empty() {
            self.schema.explorer.schema.schemas.clone()
        } else if !self.query_capabilities().allows(|caps| caps.schema.schemas) {
            vec!["main".to_string()]
        } else {
            vec!["public".to_string()]
        };
        let connections: Vec<(String, String, String)> = self
            .connection
            .catalog
            .iter()
            .map(|c| (c.id.clone(), c.name.clone(), c.environment.clone()))
            .collect();

        let context = query_context_picker_view::QueryContextPickerContext {
            theme: self.theme,
            current_connection_id: current_conn_id.as_deref(),
            current_schema: &current_schema,
            available_schemas: &available_schemas,
            connections: &connections,
        };
        if let Some(action) = query_context_picker_view::draw_picker(&context, ctx, anchor) {
            match action {
                query_context_picker_view::QueryContextPickerAction::SelectConnection(id) => {
                    self.set_document_connection(doc_idx, Some(id));
                    self.query.editor.query_context_picker_open = false;
                }
                query_context_picker_view::QueryContextPickerAction::SelectSchema(schema) => {
                    self.set_document_schema(doc_idx, Some(schema));
                    self.query.editor.query_context_picker_open = false;
                }
                query_context_picker_view::QueryContextPickerAction::Close => {
                    self.query.editor.query_context_picker_open = false;
                }
            }
        }
    }

    fn draw_query_output_dock(&mut self, ui: &mut egui::Ui, dock_height: f32) {
        let body_height = {
            let mut context = query_output_dock_surface_view::QueryOutputDockContext {
                theme: self.theme,
                workspace: &mut self.workspace,
                output: &mut self.query.output,
                session: &self.query.session,
                editor: &mut self.query.editor,
            };
            context.draw_chrome(ui, dock_height)
        };
        ui.allocate_ui_with_layout(
            egui::vec2(ui.available_width(), body_height),
            Layout::top_down(Align::Min),
            |ui| {
                let result = self.query.session.active_result().cloned();
                self.draw_output_pane(ui, result.as_ref());
            },
        );
    }

    fn draw_query_status_bar(&mut self, ui: &mut egui::Ui) {
        self.refresh_query_parameter_count();
        if let Some(action) = self.draw_query_status_surface(ui) {
            self.apply_query_status_bar_action(action);
        }
    }

    fn refresh_query_parameter_count(&mut self) {
        let param_key = (
            self.query.session.active_document_index,
            self.query.session.active_buffer_version(),
        );
        if self.query.editor.param_count_cache_key != Some(param_key) {
            self.query.editor.param_count_cache_key = Some(param_key);
            self.query.editor.param_count_cache =
                crate::query::discover_sql_parameters(self.query.session.active_text()).len();
        }
    }

    fn draw_query_status_surface(
        &self,
        ui: &mut egui::Ui,
    ) -> Option<query_status_bar_surface_view::QueryStatusBarAction> {
        let active_request_id = self
            .query
            .session
            .documents
            .get(self.query.session.active_document_index)
            .and_then(|document| match document.execution_state {
                QueryExecutionState::Running(request_id) => Some(request_id),
                _ => None,
            });
        let capabilities = self.query_capabilities();
        let cancel_reason =
            capabilities.feature_limitation(db_pro_core::domain::capabilities::CapabilityFeature::Cancel);
        query_status_bar_surface_view::draw_status_bar(
            &query_status_bar_surface_view::QueryStatusBarContext {
                theme: self.theme,
                connected: self.active_query_connection_id().is_some() && self.connection.lifecycle.is_connected(),
                active_request_id,
                cancel_supported: capabilities.allows(|value| value.query.cancel),
                cancel_reason: cancel_reason.as_deref(),
                modifier: Self::primary_modifier_label(),
                bottom_panel_open: self.workspace.bottom_panel_open,
                in_transaction: self.query.execution.query_in_transaction,
                transaction_pending: self.query.execution.query_txn_pending,
                auto_commit: self.query.execution.query_auto_commit,
                cursor_line: self.query.editor.query_cursor_line,
                cursor_column: self.query.editor.query_cursor_column,
                driver: self.active_query_driver(),
                schema: self.active_query_schema(),
                parameter_count: self.query.editor.param_count_cache,
                diagnostic_count: self.query.editor.diagnostics.len(),
            },
            ui,
        )
    }

    fn apply_query_status_bar_action(&mut self, action: query_status_bar_surface_view::QueryStatusBarAction) {
        use query_status_bar_surface_view::QueryStatusBarAction as Action;
        match action {
            Action::RunControl(action) => match action {
                query_run_control_view::QueryRunControlAction::Run => {
                    self.dispatch_query();
                }
                query_run_control_view::QueryRunControlAction::Cancel(request_id) => self.cancel_query(request_id),
                query_run_control_view::QueryRunControlAction::ReportUnsupportedCancel(reason) => {
                    self.feedback.runtime_message = reason;
                }
                query_run_control_view::QueryRunControlAction::ReportDisconnected => {
                    self.feedback.runtime_message = "Connect to a database before running a query".to_owned();
                }
            },
            Action::ShowOutput => self.workspace.bottom_panel_open = true,
            Action::ToggleTransaction => {
                self.query.execution.query_txn_bar_open = !self.query.execution.query_txn_bar_open;
            }
            Action::ToggleParameters => {
                self.query.editor.query_params_panel_open = !self.query.editor.query_params_panel_open;
            }
            Action::ShowDiagnostics => self.show_query_diagnostics(),
        }
    }

    fn show_query_diagnostics(&mut self) {
        self.workspace.bottom_panel_open = true;
        if let Some(doc_id) = self
            .query
            .session
            .documents
            .get(self.query.session.active_document_index)
            .map(|document| document.id.clone())
        {
            self.query.output.set_for_document_and_activate_if_active(
                &doc_id,
                self.query
                    .session
                    .active_document()
                    .map(|document| document.id.as_str()),
                OutputTab::Messages,
            );
        }
    }
}

impl DbProApp {
    /// Keyword / table / column completion list (legacy inline card — floating popup is canonical).
    #[allow(dead_code)]
    fn draw_sql_completion(&mut self, ui: &mut egui::Ui) {
        card_frame(self.theme).show(ui, |ui| {
            ui.label(RichText::new("SQL completion").strong());
            let uses_positional = !self.query_capabilities().allows(|caps| caps.query.numbered_parameters);
            let mut candidates = vec![
                "SELECT".to_owned(),
                "FROM".to_owned(),
                "WHERE".to_owned(),
                "JOIN".to_owned(),
                "GROUP BY".to_owned(),
                "ORDER BY".to_owned(),
                "LIMIT".to_owned(),
                "COUNT(*)".to_owned(),
            ];
            if uses_positional {
                candidates.extend(["GLOB", "strftime", "WITHOUT ROWID"].into_iter().map(str::to_owned));
            } else {
                candidates.extend(
                    ["ILIKE", "RETURNING", "jsonb_build_object"]
                        .into_iter()
                        .map(str::to_owned),
                );
            }
            candidates.extend(self.active_schema_table_names());
            candidates.extend(self.active_schema_column_names());
            candidates.extend(self.schema.explorer.schema.views.iter().map(|view| view.name.clone()));
            candidates.extend(
                self.schema
                    .explorer
                    .schema
                    .functions
                    .iter()
                    .map(|function| function.name.clone()),
            );
            for keyword in candidates.iter() {
                if ui
                    .selectable_label(false, keyword)
                    .on_hover_text("Insert SQL keyword or expression")
                    .clicked()
                {
                    self.append_to_active_query(keyword);
                    self.query.editor.completion_open = false;
                }
            }
        });
    }

    /// Quick SQL snippet inserters.
    fn draw_sql_snippets(&mut self, ui: &mut egui::Ui) {
        card_frame(self.theme).show(ui, |ui| {
            ui.label(RichText::new("SQL snippets").strong());
            for (label, snippet) in query_snippets::builtin_sql_snippets() {
                if Button::new(self.theme)
                    .text(*label)
                    .variant(ButtonVariant::Secondary)
                    .size(ButtonSize::Sm)
                    .show(ui)
                    .clicked()
                {
                    self.insert_snippet(snippet);
                    self.query.editor.snippets_open = false;
                }
            }
        });
    }

    /// Parser diagnostics for the current SQL (legacy list — gutter + status count are canonical).
    #[allow(dead_code)]
    fn draw_diagnostics(&mut self, ui: &mut egui::Ui) {
        if self.query.editor.diagnostics.is_empty() {
            return;
        }
        ui.colored_label(
            self.theme.warning,
            format!("Diagnostics · {}", self.query.editor.diagnostics.len()),
        );
        for diagnostic in &self.query.editor.diagnostics {
            ui.colored_label(self.theme.warning, format!("• {diagnostic}"));
        }
    }

    /// Discovered bind placeholders for the active document (#225 discovery slice).
    fn draw_sql_parameters_panel(&mut self, ui: &mut egui::Ui) {
        let supports_parameters = self.query_capabilities().allows(|caps| caps.query.parameters);
        let mut context = query_parameters_view::QueryParametersContext {
            theme: self.theme,
            session: &mut self.query.session,
            supports_parameters,
        };
        query_parameters_view::draw_parameters_panel(&mut context, ui);
    }
}

#[cfg(test)]
#[path = "query_view_tests.rs"]
mod query_view_tests;
