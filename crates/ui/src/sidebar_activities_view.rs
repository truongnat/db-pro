//! Queries / Data / Problems / History sidebar activities.
use super::sidebar_data_view::{SidebarDataAction, SidebarDataContext};
use super::sidebar_problems_view::{SidebarProblemsAction, SidebarProblemsContext};
use super::sidebar_queries_surface_view::{SidebarQueriesSurfaceAction, SidebarQueriesSurfaceContext};
use super::sidebar_query_library_view::SidebarQueryLibraryAction;
use super::*;

impl DbProApp {
    pub(super) fn draw_queries(&mut self, ui: &mut egui::Ui) {
        let actions = {
            let context = SidebarQueriesSurfaceContext {
                theme: self.theme,
                documents: &self.query.session.documents,
                active_tab: self.workspace.active_tab,
                active_document_index: self.query.session.active_document_index,
                saved_queries: &self.query.library.saved_queries,
                query_folders: &self.query.library.query_folders,
                history: &self.query.editor.query_history,
                delete_confirmation_id: self.overlay.delete_confirmation_id.as_deref(),
            };
            context.draw(ui)
        };
        for action in actions {
            match action {
                SidebarQueriesSurfaceAction::OpenQuery(action) => self.apply_sidebar_query_action(action),
                SidebarQueriesSurfaceAction::Library(action) => self.apply_sidebar_query_library_action(ui, action),
                SidebarQueriesSurfaceAction::Shortcut(action) => self.apply_sidebar_query_shortcut_action(action),
            }
        }
    }

    fn apply_sidebar_query_action(&mut self, action: sidebar_queries_view::SidebarQueriesAction) {
        match action {
            sidebar_queries_view::SidebarQueriesAction::NewQuery => self.new_query_document(),
            sidebar_queries_view::SidebarQueriesAction::NewScratch => self.new_scratch_query_document(),
            sidebar_queries_view::SidebarQueriesAction::Select(index) => {
                self.switch_query_document(index);
                self.workspace.active_tab = WorkspaceTab::Query;
            }
            sidebar_queries_view::SidebarQueriesAction::Duplicate(index) => self.duplicate_query_document(index),
            sidebar_queries_view::SidebarQueriesAction::Rename(index) => self.rename_query_document_inline(index),
            sidebar_queries_view::SidebarQueriesAction::Close(index) => self.request_close_query_document(index),
        }
    }

    fn apply_sidebar_query_shortcut_action(
        &mut self,
        action: sidebar_query_shortcuts_view::SidebarQueryShortcutAction,
    ) {
        match action {
            sidebar_query_shortcuts_view::SidebarQueryShortcutAction::InsertSnippet(snippet) => {
                self.insert_snippet(&snippet);
                self.workspace.active_tab = WorkspaceTab::Query;
                self.workspace.activity = Activity::Queries;
            }
            sidebar_query_shortcuts_view::SidebarQueryShortcutAction::NewScratch => {
                self.new_scratch_query_document();
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
                        self.workspace.files.select_panel_tab(FilesPanelTab::Search);
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
        let context = SidebarQueriesSurfaceContext {
            theme: self.theme,
            documents: &self.query.session.documents,
            active_tab: self.workspace.active_tab,
            active_document_index: self.query.session.active_document_index,
            saved_queries: &self.query.library.saved_queries,
            query_folders: &self.query.library.query_folders,
            history: &self.query.editor.query_history,
            delete_confirmation_id: self.overlay.delete_confirmation_id.as_deref(),
        };
        for action in context.draw_history(ui) {
            match action {
                SidebarQueriesSurfaceAction::Library(action) => self.apply_sidebar_query_library_action(ui, action),
                SidebarQueriesSurfaceAction::OpenQuery(action) => self.apply_sidebar_query_action(action),
                SidebarQueriesSurfaceAction::Shortcut(action) => self.apply_sidebar_query_shortcut_action(action),
            }
        }
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
            SidebarQueryLibraryAction::ConfirmDelete(id) => {
                let request_id = self.task_bridge.next_request_id();
                self.dispatch_command(self.query.library.delete_query_command(request_id, id));
                self.overlay.delete_confirmation_id = None;
            }
            SidebarQueryLibraryAction::CancelDelete => {
                self.overlay.delete_confirmation_id = None;
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
}
