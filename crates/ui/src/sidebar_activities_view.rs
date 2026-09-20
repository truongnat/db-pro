//! Queries / Data / Problems / History sidebar activities.
use super::sidebar_data_view::{SidebarDataAction, SidebarDataContext};
use super::sidebar_problems_view::{SidebarProblemsAction, SidebarProblemsContext};
use super::sidebar_queries_view::{SidebarQueriesAction, SidebarQueriesContext};
use super::sidebar_query_library_view::{SidebarQueryLibraryAction, SidebarQueryLibraryContext};
use super::sidebar_query_shortcuts_view::{SidebarQueryShortcutAction, SidebarQueryShortcutsContext};
use super::*;
use egui::RichText;

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
        let shortcut_actions = SidebarQueryShortcutsContext { theme: self.theme }.draw(ui);
        for action in shortcut_actions {
            match action {
                SidebarQueryShortcutAction::InsertSnippet(snippet) => {
                    self.insert_snippet(&snippet);
                    self.workspace.active_tab = WorkspaceTab::Query;
                    self.workspace.activity = Activity::Queries;
                }
                SidebarQueryShortcutAction::NewScratch => self.new_scratch_query_document(),
            }
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
        let actions = {
            let context = SidebarQueryLibraryContext {
                theme: self.theme,
                saved_queries: &self.query.library.saved_queries,
                query_folders: &self.query.library.query_folders,
                history: &self.query.editor.query_history,
            };
            context.draw_saved_queries(ui)
        };
        for action in actions {
            self.apply_sidebar_query_library_action(ui, action);
        }
        self.draw_delete_saved_query_confirmation(ui);
    }

    fn apply_sidebar_query_library_action(&mut self, ui: &mut egui::Ui, action: SidebarQueryLibraryAction) {
        match action {
            SidebarQueryLibraryAction::NewQuery => self.new_query_document(),
            SidebarQueryLibraryAction::OpenQuery(sql) | SidebarQueryLibraryAction::OpenHistory(sql) => {
                self.set_active_query_text(sql);
                self.workspace.active_tab = WorkspaceTab::Query;
            }
            SidebarQueryLibraryAction::CopySql { name, sql } => {
                ui.output_mut(|output| output.copied_text = sql);
                self.feedback.runtime_message = format!("Copied SQL for `{name}`");
            }
            SidebarQueryLibraryAction::Rename(query) => self.rename_saved_query(&query),
            SidebarQueryLibraryAction::RequestDelete(id) => {
                self.overlay.delete_confirmation_id = Some(id);
            }
            SidebarQueryLibraryAction::RequestDeleteFolder(id) => {
                self.overlay.folder_delete_confirmation = id;
            }
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
        let actions = {
            let context = SidebarQueryLibraryContext {
                theme: self.theme,
                saved_queries: &self.query.library.saved_queries,
                query_folders: &self.query.library.query_folders,
                history: &self.query.editor.query_history,
            };
            context.draw_local_history(ui)
        };
        for action in actions {
            self.apply_sidebar_query_library_action(ui, action);
        }
    }
}
