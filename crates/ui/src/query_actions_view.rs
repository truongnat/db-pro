use super::query_view::AI_PREDICTION_EGRESS_NOTE;
use super::*;
use std::time::Instant;

impl DbProApp {
    /// Query actions menu anchored to the compact query context strip.
    pub(super) fn draw_query_actions_menu(&mut self, ctx: &egui::Context, anchor: egui::Rect) {
        let menu_width = 264.0;
        let menu_position = egui::pos2((anchor.right() - menu_width).max(8.0), anchor.bottom() + 4.0);
        let mut close_menu = false;
        let menu = egui::Area::new(egui::Id::new("query_actions_menu"))
            .order(egui::Order::Foreground)
            .fixed_pos(menu_position)
            .show(ctx, |ui| {
                egui::Frame {
                    fill: self.theme.surface_elevated,
                    inner_margin: egui::Margin::same(8.0),
                    rounding: egui::Rounding::same(8.0),
                    stroke: egui::Stroke::new(1.0, self.theme.border_subtle),
                    ..Default::default()
                }
                .show(ui, |ui| {
                    ui.set_min_width(menu_width);
                    ui.label(
                        RichText::new("Query actions")
                            .small()
                            .strong()
                            .color(self.theme.text_muted),
                    );
                    ui.add_space(4.0);
                    close_menu |= self.draw_query_run_actions(ui, ctx);
                    ui.separator();
                    ui.label(RichText::new("Editor").small().strong().color(self.theme.text_muted));
                    ui.add_space(4.0);
                    close_menu |= self.draw_query_editor_actions(ui);
                    ui.label(
                        RichText::new(format!("Editor font · {} px", self.query.editor.editor_font_size))
                            .small()
                            .color(self.theme.text_muted),
                    );
                });
            });
        let clicked_outside = ctx.input(|input| {
            input.pointer.any_click()
                && input
                    .pointer
                    .interact_pos()
                    .is_some_and(|position| !menu.response.rect.contains(position) && !anchor.contains(position))
        });
        if clicked_outside || close_menu {
            self.query.editor.query_tools_open = false;
        }
    }

    /// Run / format / explain / agent / save entries.
    fn draw_query_run_actions(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) -> bool {
        let mut close_menu = false;
        if menu_button_with_icon(
            ui,
            Icon::Play,
            if self.query.session.selected_text.is_empty() {
                "Run query"
            } else {
                "Run selection"
            },
            self.theme,
        )
        .clicked()
        {
            self.dispatch_query();
            close_menu = true;
        }
        if menu_button_with_icon(ui, Icon::WandSparkles, "Format SQL", self.theme).clicked() {
            self.format_active_query();
            close_menu = true;
        }
        if menu_button_with_icon(ui, Icon::ChartNoAxesCombined, "Explain query", self.theme).clicked() {
            self.explain_query();
            close_menu = true;
        }
        if menu_button_with_icon(ui, Icon::Bot, "Ask Agent", self.theme).clicked() {
            self.open_agent_prompt(
                if self.query.session.selected_text.trim().is_empty() {
                    "Explain the current SQL and suggest improvements"
                } else {
                    "Explain the selected SQL and suggest improvements"
                },
                ctx,
            );
            close_menu = true;
        }
        if menu_button_with_icon(ui, Icon::Save, "Save query", self.theme).clicked() {
            self.save_query_document();
            close_menu = true;
        }
        if menu_button_with_icon(ui, Icon::Save, "Save query as…", self.theme).clicked() {
            self.open_save_as_dialog();
            close_menu = true;
        }
        let builder_label = if self.query.editor.visual_builder.open {
            "Hide visual query builder"
        } else {
            "Visual query builder"
        };
        if menu_button_with_icon(ui, Icon::LayoutTemplate, builder_label, self.theme).clicked() {
            self.query.editor.visual_builder.open = !self.query.editor.visual_builder.open;
            close_menu = true;
        }
        close_menu
    }

    /// Editor-specific query actions and preferences.
    pub(super) fn draw_query_editor_actions(&mut self, ui: &mut egui::Ui) -> bool {
        let mut close_menu = false;
        if menu_button_with_icon(ui, Icon::Search, "Find in SQL", self.theme).clicked() {
            self.query.editor.editor_search_open = !self.query.editor.editor_search_open;
            close_menu = true;
        }
        let txn_label = if self.query.execution.query_txn_bar_open {
            "Hide transaction controls"
        } else {
            "Show transaction controls"
        };
        if menu_button_with_icon(ui, Icon::GitBranch, txn_label, self.theme).clicked() {
            self.query.execution.query_txn_bar_open = !self.query.execution.query_txn_bar_open;
            close_menu = true;
        }
        if menu_button_with_icon(ui, Icon::Minus, "Decrease font size", self.theme).clicked() {
            self.query.editor.editor_font_size = (self.query.editor.editor_font_size - 1.0).max(10.0);
        }
        if menu_button_with_icon(ui, Icon::Plus, "Increase font size", self.theme).clicked() {
            self.query.editor.editor_font_size = (self.query.editor.editor_font_size + 1.0).min(24.0);
        }
        if menu_button_with_icon(ui, Icon::Bot, "Generate SQL Prediction", self.theme).clicked() {
            if self.preferences.prediction_mode != PredictionMode::Off {
                if let Some(doc) = self
                    .query
                    .session
                    .documents
                    .get_mut(self.query.session.active_document_index)
                {
                    doc.schedule_prediction_with_mode(Instant::now(), true);
                }
            }
            close_menu = true;
        }
        ui.horizontal(|ui| {
            ui.label(RichText::new("AI prediction").small().color(self.theme.text_muted));
            for (mode, label) in [
                (PredictionMode::Off, "Off"),
                (PredictionMode::Subtle, "Subtle"),
                (PredictionMode::Eager, "Eager"),
            ] {
                if ui
                    .selectable_label(self.preferences.prediction_mode == mode, label)
                    .clicked()
                {
                    self.preferences.prediction_mode = mode;
                    if mode == PredictionMode::Off {
                        self.cancel_prediction_for_document(self.query.session.active_document_index);
                    }
                }
            }
        });
        ui.label(
            RichText::new(AI_PREDICTION_EGRESS_NOTE)
                .font(font_caption())
                .color(self.theme.text_muted),
        );
        ui.add_space(4.0);
        if menu_button_with_icon(ui, Icon::FileCode2, "SQL snippets", self.theme).clicked() {
            self.query.editor.snippets_open = !self.query.editor.snippets_open;
            close_menu = true;
        }
        ui.horizontal(|ui| {
            input(
                ui,
                &mut self.query.library.query_folder,
                "folder (optional)",
                150.0,
                self.theme,
            );
            if Button::new(self.theme)
                .text("New folder")
                .variant(ButtonVariant::Secondary)
                .size(ButtonSize::Sm)
                .show(ui)
                .clicked()
            {
                self.create_query_folder();
            }
        });
        close_menu
    }

    fn create_query_folder(&mut self) {
        let Some(connection) = self.active_connection().cloned() else {
            return;
        };
        let request_id = self.task_bridge.next_request_id();
        let command = match self.query.library.create_folder_command(request_id, connection.id) {
            Ok(command) => command,
            Err(error) => {
                self.feedback.runtime_message = error;
                return;
            }
        };
        self.dispatch_command(command);
        self.feedback.runtime_message = "Creating query folder…".to_owned();
    }

    pub(crate) fn insert_snippet(&mut self, snippet: &str) {
        self.cancel_prediction_for_document(self.query.session.active_document_index);
        if let Some(doc) = self
            .query
            .session
            .documents
            .get_mut(self.query.session.active_document_index)
        {
            let offset = doc.cursor.offset.min(doc.buffer.len_bytes());
            let insertion = if offset > 0 && !doc.buffer.text()[..offset].ends_with('\n') {
                format!("\n{snippet}")
            } else {
                snippet.to_owned()
            };
            doc.buffer.insert(offset, &insertion);
            let new_offset = offset + insertion.len();
            doc.cursor = crate::editor::CursorPosition::from_offset(&doc.buffer, new_offset);
            doc.selection = crate::editor::SelectionRange::point(new_offset);
            doc.dirty = true;
            self.query.editor.query_cursor_line = doc.cursor.line + 1;
            self.query.editor.query_cursor_column = doc.cursor.col + 1;
        }
        self.workspace.active_tab = WorkspaceTab::Query;
        self.refresh_diagnostics();
        self.feedback.runtime_message = "Snippet inserted".to_owned();
    }
}
