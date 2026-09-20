//! Queries / Data / Problems / History sidebar activities.
use super::*;
use egui::{Align, Layout, RichText};
use lucide_icons::Icon;

impl DbProApp {
    pub(super) fn draw_queries(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            section_label(ui, "OPEN QUERIES", self.theme);
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if compact_icon_button(ui, Icon::FilePlus2, self.theme)
                    .on_hover_text("New scratch query")
                    .clicked()
                {
                    self.new_scratch_query_document();
                }
                if compact_icon_button(ui, Icon::Plus, self.theme)
                    .on_hover_text("New query")
                    .clicked()
                {
                    self.new_query_document();
                }
            });
        });
        ui.add_space(8.0);

        for (index, document) in self.query.session.documents.clone().into_iter().enumerate() {
            let selected =
                self.workspace.active_tab == WorkspaceTab::Query && self.query.session.active_document_index == index;
            let unsaved = document.is_dirty();
            let title = if unsaved {
                format!("{}  •", document.title)
            } else {
                document.title.clone()
            };
            let response = sidebar_item(ui, Icon::FileCode2, &title, selected, self.theme);
            let is_ctx = is_context_menu_triggered(&response, ui);
            let mut close_requested = false;
            let mut duplicate_requested = false;
            let mut rename_requested = false;
            let theme = self.theme;
            context_action_menu(ui, &response, theme, |ui, close_menu| {
                if ctx_menu_item(ui, Some(Icon::Copy), "Duplicate query", None, theme.text_primary, theme).clicked() {
                    duplicate_requested = true;
                    *close_menu = true;
                }
                if ctx_menu_item(ui, Some(Icon::Pencil), "Rename tab", None, theme.text_primary, theme).clicked() {
                    rename_requested = true;
                    *close_menu = true;
                }
                if self.query.session.documents.len() > 1
                    && ctx_menu_item(ui, Some(Icon::Trash2), "Close query", None, theme.danger, theme).clicked()
                {
                    close_requested = true;
                    *close_menu = true;
                }
            });
            if response.clicked() && !is_ctx {
                self.switch_query_document(index);
                self.workspace.active_tab = WorkspaceTab::Query;
            }
            if duplicate_requested {
                self.duplicate_query_document(index);
            }
            if rename_requested {
                self.rename_query_document_inline(index);
            }
            if close_requested {
                self.request_close_query_document(index);
            }
        }

        ui.add_space(14.0);
        section_label(ui, "SAVED QUERIES", self.theme);
        ui.add_space(6.0);
        self.draw_saved_queries_section(ui);

        ui.add_space(14.0);
        section_label(ui, "HISTORY", self.theme);
        ui.add_space(6.0);
        self.draw_local_history_section(ui);

        ui.add_space(14.0);
        section_label(ui, "SNIPPETS", self.theme);
        ui.add_space(6.0);
        for (label, snippet) in Self::builtin_sql_snippets() {
            if sidebar_item(ui, Icon::Braces, label, false, self.theme)
                .on_hover_text(*snippet)
                .clicked()
            {
                self.insert_snippet(snippet);
                self.workspace.active_tab = WorkspaceTab::Query;
                self.workspace.activity = Activity::Queries;
            }
        }

        ui.add_space(14.0);
        section_label(ui, "SCRATCH", self.theme);
        ui.add_space(6.0);
        ui.label(
            RichText::new("Scratch tabs are disposable — use New scratch for throwaway SQL.")
                .small()
                .color(self.theme.text_muted),
        );
        if compact_button_with_icon(ui, Icon::FilePlus2, "Open scratch SQL", self.theme).clicked() {
            self.new_scratch_query_document();
        }
    }

    pub(super) fn draw_data_activity(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            section_label(ui, "PINNED TABLES", self.theme);
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                badge(
                    ui,
                    &self.schema.explorer.pinned_tables.len().to_string(),
                    self.theme.surface_hover,
                    self.theme.text_muted,
                );
            });
        });
        ui.add_space(8.0);
        if self.schema.explorer.pinned_tables.is_empty() {
            ui.label(
                RichText::new("Pin tables from Explorer or Quick Open for fast reopen.")
                    .small()
                    .color(self.theme.text_muted),
            );
        } else {
            let pinned = self.schema.explorer.pinned_tables.clone();
            for table in pinned {
                self.draw_data_table_row(ui, &table, true);
            }
        }

        ui.add_space(16.0);
        ui.horizontal(|ui| {
            section_label(ui, "RECENT TABLES", self.theme);
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                badge(
                    ui,
                    &self.schema.explorer.recent_tables.len().to_string(),
                    self.theme.surface_hover,
                    self.theme.text_muted,
                );
            });
        });
        ui.add_space(8.0);
        if self.schema.explorer.recent_tables.is_empty() {
            ui.label(
                RichText::new("Tables you open appear here in most-recent order.")
                    .small()
                    .color(self.theme.text_muted),
            );
        } else {
            let recent = self.schema.explorer.recent_tables.clone();
            for table in recent {
                self.draw_data_table_row(ui, &table, false);
            }
        }

        ui.add_space(12.0);
        ui.label(
            RichText::new("Right-click for open, pin, or remove")
                .small()
                .color(self.theme.text_muted),
        );
    }

    fn draw_data_table_row(&mut self, ui: &mut egui::Ui, table: &str, from_pinned: bool) {
        let selected = self.schema.explorer.selected_table.as_deref() == Some(table);
        let icon = if from_pinned { Icon::Pin } else { Icon::Table2 };
        let response = sidebar_item(ui, icon, table, selected, self.theme);
        let is_ctx = is_context_menu_triggered(&response, ui);

        let mut open_data = false;
        let mut open_structure = false;
        let mut open_query = false;
        let mut toggle_pin = false;
        let mut remove_recent = false;
        let is_pinned = self.schema.explorer.pinned_tables.iter().any(|item| item == table);
        let pin_label = if is_pinned { "Unpin table" } else { "Pin table" };

        context_action_menu(ui, &response, self.theme, |ui, close_menu| {
            if ctx_menu_item(
                ui,
                Some(Icon::Table2),
                "Open data",
                None,
                self.theme.text_primary,
                self.theme,
            )
            .clicked()
            {
                open_data = true;
                *close_menu = true;
            }
            if ctx_menu_item(
                ui,
                Some(Icon::Columns3),
                "Open structure",
                None,
                self.theme.text_primary,
                self.theme,
            )
            .clicked()
            {
                open_structure = true;
                *close_menu = true;
            }
            if ctx_menu_item(
                ui,
                Some(Icon::FileCode2),
                "New query for table",
                None,
                self.theme.text_primary,
                self.theme,
            )
            .clicked()
            {
                open_query = true;
                *close_menu = true;
            }
            ui.separator();
            if ctx_menu_item(
                ui,
                Some(Icon::Pin),
                pin_label,
                None,
                self.theme.text_primary,
                self.theme,
            )
            .clicked()
            {
                toggle_pin = true;
                *close_menu = true;
            }
            if !from_pinned
                && ctx_menu_item(
                    ui,
                    Some(Icon::Trash2),
                    "Remove from recent",
                    None,
                    self.theme.text_primary,
                    self.theme,
                )
                .clicked()
            {
                remove_recent = true;
                *close_menu = true;
            }
        });

        if response.clicked() && !is_ctx {
            open_data = true;
        }

        if open_data {
            self.open_table(table.to_owned());
            self.table.state.table_view = TableView::Data;
        }
        if open_structure {
            self.open_table_from_palette(table.to_owned());
        }
        if open_query {
            let schema = self.active_schema().to_owned();
            self.new_query_document();
            self.set_active_query_text(format!("SELECT *\nFROM {schema}.{table}\nLIMIT 100;"));
            self.workspace.active_tab = WorkspaceTab::Query;
            self.workspace.activity = Activity::Queries;
            self.feedback.runtime_message = format!("Query ready for {table}");
        }
        if toggle_pin {
            self.toggle_pinned_table(table.to_owned());
        }
        if remove_recent {
            self.schema.explorer.remove_recent_table(table);
            self.feedback.runtime_message = format!("Removed {table} from recent");
        }
    }

    pub(super) fn draw_problems(&mut self, ui: &mut egui::Ui) {
        let entries = self.collect_problem_entries();
        let error_count = entries
            .iter()
            .filter(|entry| entry.severity == crate::editor::DiagnosticSeverity::Error)
            .count();
        let warning_count = entries
            .iter()
            .filter(|entry| entry.severity == crate::editor::DiagnosticSeverity::Warning)
            .count();

        ui.horizontal(|ui| {
            section_label(ui, "PROBLEMS", self.theme);
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                badge(
                    ui,
                    &format!("{error_count}E · {warning_count}W"),
                    self.theme.surface_hover,
                    self.theme.text_muted,
                );
            });
        });
        ui.add_space(8.0);

        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 6.0;
            egui::ComboBox::from_id_salt("problems_severity_filter")
                .selected_text(match self.query.editor.problems_severity_filter {
                    ProblemsSeverityFilter::All => "All",
                    ProblemsSeverityFilter::Errors => "Errors",
                    ProblemsSeverityFilter::Warnings => "Warnings",
                })
                .width(96.0)
                .show_ui(ui, |ui| {
                    for (filter, label) in [
                        (ProblemsSeverityFilter::All, "All"),
                        (ProblemsSeverityFilter::Errors, "Errors"),
                        (ProblemsSeverityFilter::Warnings, "Warnings"),
                    ] {
                        ui.selectable_value(&mut self.query.editor.problems_severity_filter, filter, label);
                    }
                });
            egui::ComboBox::from_id_salt("problems_source_filter")
                .selected_text(match self.query.editor.problems_source_filter {
                    ProblemsSourceFilter::All => "All sources",
                    ProblemsSourceFilter::Parser => "Parser",
                    ProblemsSourceFilter::Lint => "Lint",
                    ProblemsSourceFilter::Delimiter => "Delimiter",
                    ProblemsSourceFilter::Database => "Database",
                })
                .width(120.0)
                .show_ui(ui, |ui| {
                    for (filter, label) in [
                        (ProblemsSourceFilter::All, "All sources"),
                        (ProblemsSourceFilter::Parser, "Parser"),
                        (ProblemsSourceFilter::Lint, "Lint"),
                        (ProblemsSourceFilter::Delimiter, "Delimiter"),
                        (ProblemsSourceFilter::Database, "Database"),
                    ] {
                        ui.selectable_value(&mut self.query.editor.problems_source_filter, filter, label);
                    }
                });
        });
        ui.add_space(8.0);

        let filtered: Vec<_> = entries
            .into_iter()
            .filter(|entry| self.problem_matches_filters(entry))
            .collect();

        if filtered.is_empty() {
            ui.add_space(24.0);
            ui.vertical_centered(|ui| {
                ui.label(icon_text(Icon::TriangleAlert, "", self.theme.text_muted));
                ui.add_space(8.0);
                ui.label(RichText::new("No problems in open documents").color(self.theme.text_secondary));
                ui.label(
                    RichText::new("Lint, parser, and execution diagnostics appear here.")
                        .small()
                        .color(self.theme.text_muted),
                );
            });
            return;
        }

        let mut navigate: Option<(usize, usize)> = None;
        let mut last_doc: Option<usize> = None;
        for entry in &filtered {
            if last_doc != Some(entry.document_index) {
                last_doc = Some(entry.document_index);
                ui.add_space(6.0);
                ui.label(
                    RichText::new(&entry.document_title)
                        .small()
                        .strong()
                        .color(self.theme.text_muted),
                );
            }
            let selected = self.query.editor.problems_selected.as_ref()
                == Some(&(entry.document_id.clone(), entry.diagnostic_index));
            let (icon, _color) = match entry.severity {
                crate::editor::DiagnosticSeverity::Error => (Icon::AlertCircle, self.theme.danger),
                crate::editor::DiagnosticSeverity::Warning => (Icon::TriangleAlert, self.theme.warning),
                crate::editor::DiagnosticSeverity::Information | crate::editor::DiagnosticSeverity::Hint => {
                    (Icon::Info, self.theme.text_muted)
                }
            };
            let source = match entry.source {
                crate::editor::DiagnosticSource::Parser => "parser",
                crate::editor::DiagnosticSource::Lint => "lint",
                crate::editor::DiagnosticSource::Delimiter => "delimiter",
                crate::editor::DiagnosticSource::Database => "database",
            };
            let label = format!(
                "L{}:{}  {}  · {source}",
                entry.line + 1,
                entry.column + 1,
                entry.message
            );
            let response = sidebar_item(ui, icon, &label, selected, self.theme);
            if response.clicked() {
                self.query.editor.problems_selected = Some((entry.document_id.clone(), entry.diagnostic_index));
                if entry.document_index == usize::MAX {
                    // Workspace-indexed diagnostic (#269): open the SQL file if possible.
                    self.open_workspace_sql_file(entry.document_title.clone());
                    self.workspace.files_panel_tab = FilesPanelTab::Search;
                } else {
                    navigate = Some((entry.document_index, entry.diagnostic_index));
                }
            }
            if entry.has_fix {
                ui.horizontal(|ui| {
                    ui.add_space(12.0);
                    if compact_button(ui, "Quick fix", self.theme).clicked() {
                        self.apply_problem_fix(entry.document_index, entry.diagnostic_index);
                    }
                });
            }
        }
        if let Some((doc_index, diagnostic_index)) = navigate {
            self.navigate_to_problem(doc_index, diagnostic_index);
        }
    }

    pub(super) fn draw_history(&mut self, ui: &mut egui::Ui) {
        ui.label(
            RichText::new("Saved queries")
                .small()
                .strong()
                .color(self.theme.text_muted),
        );
        self.draw_saved_queries_section(ui);
        ui.separator();
        ui.label(
            RichText::new("Local history")
                .small()
                .strong()
                .color(self.theme.text_muted),
        );
        self.draw_local_history_section(ui);
    }

    /// Saved queries, grouped by folder, plus the pending-delete confirmation.
    fn draw_saved_queries_section(&mut self, ui: &mut egui::Ui) {
        if self.query.library.saved_queries.is_empty() {
            self.draw_empty_saved_queries(ui);
            return;
        }
        let saved = self.query.library.saved_queries.clone();
        let mut groups: Vec<(String, Vec<UiSavedQuerySummary>)> = Vec::new();
        for query in saved {
            let folder = query.folder.clone().unwrap_or_else(|| "Unfiled".to_owned());
            if let Some((_, queries)) = groups.iter_mut().find(|(name, _)| name == &folder) {
                queries.push(query);
            } else {
                groups.push((folder, vec![query]));
            }
        }
        for (folder, queries) in groups {
            self.draw_saved_query_folder(ui, folder, queries);
        }
        self.draw_delete_saved_query_confirmation(ui);
    }

    fn draw_empty_saved_queries(&mut self, ui: &mut egui::Ui) {
        card_frame(self.theme).show(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.label(icon_text(Icon::Bookmark, "", self.theme.accent));
                ui.add_space(6.0);
                ui.label(RichText::new("No saved queries yet").strong());
                ui.label(
                    RichText::new("Save a query to keep it close at hand.")
                        .small()
                        .color(self.theme.text_muted),
                );
                ui.add_space(8.0);
                if compact_button_with_icon(ui, Icon::Plus, "New query", self.theme).clicked() {
                    self.new_query_document();
                }
            });
        });
    }

    /// One collapsible folder of saved queries, with a folder-level context menu.
    fn draw_saved_query_folder(&mut self, ui: &mut egui::Ui, folder: String, queries: Vec<UiSavedQuerySummary>) {
        let folder_id = self
            .query
            .library
            .query_folders
            .iter()
            .find(|item| item.name == folder)
            .map(|item| item.id.clone());
        let mut delete_requested = false;
        let header = egui::collapsing_header::CollapsingState::load_with_default_open(
            ui.ctx(),
            ui.make_persistent_id(("saved-query-folder", folder.as_str())),
            true,
        )
        .show_header(ui, |ui| {
            ui.label(icon_text(Icon::FolderOpen, &folder, self.theme.text_primary));
            ui.label(
                RichText::new(format!("{} queries", queries.len()))
                    .small()
                    .color(self.theme.text_muted),
            );
        });
        let (_, header_response, _) = header.body(|ui| {
            for query in queries {
                self.draw_saved_query_entry(ui, &query);
            }
        });
        let theme = self.theme;
        context_action_menu(ui, &header_response.response, theme, |ui, close_menu| {
            if folder_id.is_some()
                && ctx_menu_item(ui, Some(Icon::Trash2), "Delete folder", None, theme.danger, theme).clicked()
            {
                delete_requested = true;
                *close_menu = true;
            }
        });
        if delete_requested {
            self.overlay.folder_delete_confirmation = folder_id;
        }
    }

    fn draw_saved_query_entry(&mut self, ui: &mut egui::Ui, query: &UiSavedQuerySummary) {
        let query_response = sidebar_item(ui, Icon::FileCode2, &query.name, false, self.theme);
        let is_ctx = is_context_menu_triggered(&query_response, ui);
        let mut rename_requested = false;
        let mut delete_requested = false;
        let mut copy_sql = false;
        let theme = self.theme;
        context_action_menu(ui, &query_response, theme, |ui, close_menu| {
            if ctx_menu_item(ui, Some(Icon::Play), "Open in Editor", None, theme.text_primary, theme).clicked() {
                *close_menu = true;
            }
            if ctx_menu_item(ui, Some(Icon::Copy), "Copy SQL", None, theme.text_primary, theme).clicked() {
                copy_sql = true;
                *close_menu = true;
            }
            ui.separator();
            if ctx_menu_item(ui, Some(Icon::Pencil), "Rename query", None, theme.text_primary, theme).clicked() {
                rename_requested = true;
                *close_menu = true;
            }
            if ctx_menu_item(ui, Some(Icon::Trash2), "Delete query", None, theme.danger, theme).clicked() {
                delete_requested = true;
                *close_menu = true;
            }
        });
        if query_response.clicked() && !is_ctx {
            self.set_active_query_text(query.sql.clone());
            self.workspace.active_tab = WorkspaceTab::Query;
        }
        if copy_sql {
            ui.output_mut(|o| o.copied_text = query.sql.clone());
            self.feedback.runtime_message = format!("Copied SQL for `{}`", query.name);
        }
        if rename_requested {
            self.rename_saved_query(query);
        }
        if delete_requested {
            self.overlay.delete_confirmation_id = Some(query.id.clone());
        }
    }

    fn rename_saved_query(&mut self, query: &UiSavedQuerySummary) {
        let request_id = self.task_bridge.next_request_id();
        let name = if self.query.library.query_folder.trim().is_empty() {
            format!("{} (renamed)", query.name)
        } else {
            self.query.library.query_folder.trim().to_owned()
        };
        self.dispatch_command(
            self.query
                .library
                .rename_query_command(request_id, query.id.clone(), name),
        );
    }

    fn draw_delete_saved_query_confirmation(&mut self, ui: &mut egui::Ui) {
        let Some(id) = self.overlay.delete_confirmation_id.clone() else {
            return;
        };
        ui.colored_label(self.theme.warning, "Delete this saved query?");
        ui.horizontal(|ui| {
            if compact_button(ui, "Confirm delete", self.theme).clicked() {
                let request_id = self.task_bridge.next_request_id();
                self.dispatch_command(self.query.library.delete_query_command(request_id, id));
                self.overlay.delete_confirmation_id = None;
            }
            if compact_button(ui, "Cancel", self.theme).clicked() {
                self.overlay.delete_confirmation_id = None;
            }
        });
    }

    fn draw_local_history_section(&mut self, ui: &mut egui::Ui) {
        if self.query.editor.query_history.is_empty() {
            ui.label(RichText::new("No queries run yet").color(self.theme.text_muted));
            return;
        }
        let history = self.query.editor.query_history.clone();
        for query in history.iter().rev() {
            let title = query.lines().next().unwrap_or("query");
            if sidebar_item(ui, Icon::History, title, false, self.theme)
                .on_hover_text("Open query from local history")
                .clicked()
            {
                self.set_active_query_text(query.clone());
                self.workspace.active_tab = WorkspaceTab::Query;
            }
            ui.add_space(12.0);
        }
    }
}
