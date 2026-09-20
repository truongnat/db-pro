//! Queries / Data / Problems / History sidebar activities.
use super::sidebar_data_view::{SidebarDataAction, SidebarDataContext};
use super::sidebar_problems_view::{SidebarProblemsAction, SidebarProblemsContext};
use super::sidebar_queries_view::{SidebarQueriesAction, SidebarQueriesContext};
use super::*;
use egui::RichText;
use lucide_icons::Icon;

impl DbProApp {
    pub(super) fn draw_queries(&mut self, ui: &mut egui::Ui) {
        let actions = {
            let context = SidebarQueriesContext {
                theme: self.theme,
                documents: &self.query.session.documents,
                active_tab: self.workspace.active_tab,
                active_document_index: self.query.session.active_document_index,
            };
            context.draw(ui)
        };
        for action in actions {
            match action {
                SidebarQueriesAction::NewQuery => self.new_query_document(),
                SidebarQueriesAction::NewScratch => self.new_scratch_query_document(),
                SidebarQueriesAction::Select(index) => {
                    self.switch_query_document(index);
                    self.workspace.active_tab = WorkspaceTab::Query;
                }
                SidebarQueriesAction::Duplicate(index) => self.duplicate_query_document(index),
                SidebarQueriesAction::Rename(index) => self.rename_query_document_inline(index),
                SidebarQueriesAction::Close(index) => self.request_close_query_document(index),
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
        for (label, snippet) in query_snippets::builtin_sql_snippets() {
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
        let actions = {
            let context = SidebarDataContext {
                theme: self.theme,
                pinned_tables: &self.schema.explorer.pinned_tables,
                recent_tables: &self.schema.explorer.recent_tables,
                selected_table: self.schema.explorer.selected_table.as_deref(),
            };
            context.draw(ui)
        };
        for action in actions {
            match action {
                SidebarDataAction::OpenData(table) => {
                    self.open_table(table);
                    self.table.state.table_view = TableView::Data;
                }
                SidebarDataAction::OpenStructure(table) => self.open_table_from_palette(table),
                SidebarDataAction::OpenQuery(table) => {
                    let schema = self.active_schema().to_owned();
                    self.new_query_document();
                    self.set_active_query_text(format!("SELECT *\nFROM {schema}.{table}\nLIMIT 100;"));
                    self.workspace.active_tab = WorkspaceTab::Query;
                    self.workspace.activity = Activity::Queries;
                    self.feedback.runtime_message = format!("Query ready for {table}");
                }
                SidebarDataAction::TogglePinned(table) => self.toggle_pinned_table(table),
                SidebarDataAction::RemoveRecent(table) => {
                    self.schema.explorer.remove_recent_table(&table);
                    self.feedback.runtime_message = format!("Removed {table} from recent");
                }
            }
        }
    }

    pub(super) fn draw_problems(&mut self, ui: &mut egui::Ui) {
        let entries = self.collect_problem_entries();
        let actions = {
            let context = SidebarProblemsContext {
                theme: self.theme,
                entries: &entries,
                severity_filter: self.query.editor.problems_severity_filter,
                source_filter: self.query.editor.problems_source_filter,
                selected: self
                    .query
                    .editor
                    .problems_selected
                    .as_ref()
                    .map(|(document_id, index)| (document_id.as_str(), *index)),
            };
            context.draw(ui)
        };
        for action in actions {
            match action {
                SidebarProblemsAction::SetSeverityFilter(filter) => {
                    self.query.editor.problems_severity_filter = filter;
                }
                SidebarProblemsAction::SetSourceFilter(filter) => {
                    self.query.editor.problems_source_filter = filter;
                }
                SidebarProblemsAction::Select(entry) => {
                    self.query.editor.problems_selected = Some((entry.document_id, entry.diagnostic_index));
                    if entry.document_index == usize::MAX {
                        self.open_workspace_sql_file(entry.document_title);
                        self.workspace.files_panel_tab = FilesPanelTab::Search;
                    } else {
                        self.navigate_to_problem(entry.document_index, entry.diagnostic_index);
                    }
                }
                SidebarProblemsAction::QuickFix {
                    document_index,
                    diagnostic_index,
                } => {
                    self.apply_problem_fix(document_index, diagnostic_index);
                }
            }
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
