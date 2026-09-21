//! Files / workspace activity sidebar: root effects and tab composition.
use super::*;

#[path = "files_surface_view.rs"]
mod files_surface_view;

impl DbProApp {
    pub(super) fn draw_files_activity(&mut self, ui: &mut egui::Ui) {
        let actions = files_surface_view::FilesSurfaceContext {
            theme: self.theme,
            workspace: &self.workspace.files.ide_workspace,
            selected_tab: self.workspace.files_panel_tab,
        }
        .draw(ui);
        self.apply_files_surface_actions(actions);
        if self.workspace.files.ide_workspace.roots.is_empty() {
            return;
        }
        match self.workspace.files_panel_tab {
            FilesPanelTab::Tree => self.draw_files_tree_tab(ui),
            FilesPanelTab::Search => self.draw_files_search_tab(ui),
            FilesPanelTab::Migrations => self.draw_files_migrations_tab(ui),
            FilesPanelTab::Tasks => self.draw_files_tasks_tab(ui),
            FilesPanelTab::Graph => self.draw_files_graph_tab(ui),
            FilesPanelTab::Git => self.draw_files_git_tab(ui),
        }
        ui.add_space(10.0);
        self.draw_files_agent_context_strip(ui);
    }

    fn apply_files_surface_actions(&mut self, actions: Vec<files_surface_view::FilesSurfaceAction>) {
        for action in actions {
            match action {
                files_surface_view::FilesSurfaceAction::OpenFolder => self.request_open_workspace_folder(),
                files_surface_view::FilesSurfaceAction::Refresh => {
                    self.workspace.files.refresh(&mut self.feedback);
                }
                files_surface_view::FilesSurfaceAction::OpenRecent(path) => self.open_workspace_folder(path),
                files_surface_view::FilesSurfaceAction::SelectRoot(index) => {
                    if index < self.workspace.files.ide_workspace.roots.len() {
                        self.workspace.files.ide_workspace.active_root = index;
                    }
                }
                files_surface_view::FilesSurfaceAction::RemoveRoot => {
                    self.workspace.files.ide_workspace.remove_active_root();
                }
                files_surface_view::FilesSurfaceAction::SetTrusted(trusted) => {
                    self.workspace.files.ide_workspace.set_trusted(trusted);
                }
                files_surface_view::FilesSurfaceAction::SelectEnvironment(index) => {
                    if let Some(environment) = self.workspace.files.ide_workspace.environments.get(index) {
                        let name = environment.name.clone();
                        self.workspace.files.ide_workspace.active_environment = index;
                        self.feedback.runtime_message = format!("Environment → {name}");
                    }
                }
                files_surface_view::FilesSurfaceAction::Close => {
                    self.workspace
                        .files
                        .close(&mut self.workspace.shell, &mut self.feedback);
                }
                files_surface_view::FilesSurfaceAction::SelectTab(tab) => {
                    self.workspace.files_panel_tab = tab;
                    if tab == FilesPanelTab::Git {
                        self.workspace.files.refresh_git_status(&mut self.feedback);
                    }
                }
            }
        }
    }
}
