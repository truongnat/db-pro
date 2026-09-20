//! Files / workspace activity sidebar: tree, search, migrations, tasks, graph.
use super::*;
use crate::components::button::{Button, ButtonSize, ButtonVariant};
use crate::segmented_control;
use egui::{vec2, Align, Layout, RichText};
use lucide_icons::Icon;

impl DbProApp {
    pub(super) fn draw_files_activity(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            section_label(ui, "WORKSPACE", self.theme);
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if Button::new(self.theme)
                    .icon(Icon::FolderOpen)
                    .variant(ButtonVariant::Ghost)
                    .size(ButtonSize::IconSm)
                    .tooltip("Add / open folder")
                    .show(ui)
                    .clicked()
                {
                    self.request_open_workspace_folder();
                }
                if !self.workspace.files.ide_workspace.roots.is_empty()
                    && Button::new(self.theme)
                        .icon(Icon::RefreshCw)
                        .variant(ButtonVariant::Ghost)
                        .size(ButtonSize::IconSm)
                        .tooltip("Refresh tree")
                        .show(ui)
                        .clicked()
                {
                    self.workspace.files.refresh(&mut self.feedback);
                }
            });
        });
        ui.add_space(6.0);

        if self.workspace.files.ide_workspace.roots.is_empty() {
            ui.label(
                RichText::new("Open a folder to browse SQL, migrations, and project files.")
                    .small()
                    .color(self.theme.text_muted),
            );
            ui.add_space(8.0);
            if Button::new(self.theme)
                .icon(Icon::FolderOpen)
                .text("Open Folder")
                .variant(ButtonVariant::Secondary)
                .size(ButtonSize::Sm)
                .show(ui)
                .clicked()
            {
                self.request_open_workspace_folder();
            }
            if !self.workspace.files.ide_workspace.recent_roots.is_empty() {
                ui.add_space(12.0);
                section_label(ui, "RECENT", self.theme);
                ui.add_space(6.0);
                let recent = self.workspace.files.ide_workspace.recent_roots.clone();
                for path in recent.into_iter().take(8) {
                    let label = path
                        .file_name()
                        .map(|name| name.to_string_lossy().into_owned())
                        .unwrap_or_else(|| path.display().to_string());
                    if sidebar_item(ui, Icon::Folder, &label, false, self.theme)
                        .on_hover_text(path.display().to_string())
                        .clicked()
                    {
                        self.open_workspace_folder(path);
                    }
                }
            }
            return;
        }

        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = vec2(4.0, 4.0);
            for (index, root) in self.workspace.files.ide_workspace.roots.clone().into_iter().enumerate() {
                let label = root
                    .path
                    .file_name()
                    .map(|name| name.to_string_lossy().into_owned())
                    .unwrap_or_else(|| root.path.display().to_string());
                let selected = self.workspace.files.ide_workspace.active_root == index;
                if ui.selectable_label(selected, label).clicked() {
                    self.workspace.files.ide_workspace.active_root = index;
                }
            }
            if Button::new(self.theme)
                .text("+ Root")
                .variant(ButtonVariant::Ghost)
                .size(ButtonSize::Sm)
                .show(ui)
                .clicked()
            {
                self.request_open_workspace_folder();
            }
            if Button::new(self.theme)
                .text("Remove")
                .variant(ButtonVariant::Ghost)
                .size(ButtonSize::Sm)
                .show(ui)
                .clicked()
            {
                self.workspace.files.ide_workspace.remove_active_root();
            }
        });
        ui.add_space(4.0);
        if let Some(path) = self.workspace.files.ide_workspace.primary_path() {
            ui.label(
                RichText::new(path.display().to_string())
                    .small()
                    .monospace()
                    .color(self.theme.text_secondary),
            );
        }
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            let trusted = self.workspace.files.ide_workspace.is_trusted();
            if ui.selectable_label(trusted, "Trusted").clicked() {
                self.workspace.files.ide_workspace.set_trusted(true);
            }
            if ui.selectable_label(!trusted, "Untrusted").clicked() {
                self.workspace.files.ide_workspace.set_trusted(false);
            }
            if Button::new(self.theme)
                .icon(Icon::X)
                .text("Close")
                .variant(ButtonVariant::Ghost)
                .size(ButtonSize::Sm)
                .show(ui)
                .clicked()
            {
                self.workspace
                    .files
                    .close(&mut self.workspace.shell, &mut self.feedback);
            }
        });
        ui.add_space(4.0);
        ui.horizontal_wrapped(|ui| {
            for (index, env) in self
                .workspace
                .files
                .ide_workspace
                .environments
                .clone()
                .into_iter()
                .enumerate()
            {
                let selected = self.workspace.files.ide_workspace.active_environment == index;
                if ui.selectable_label(selected, &env.name).clicked() {
                    self.workspace.files.ide_workspace.set_active_environment(index);
                    self.feedback.runtime_message = format!("Environment → {}", env.name);
                }
            }
        });
        if let Some(drift) = self.workspace.files.ide_workspace.schema_drift_message.clone() {
            ui.label(RichText::new(drift).small().color(self.theme.warning));
        }
        if let Some(error) = self.workspace.files.ide_workspace.last_error.clone() {
            ui.label(RichText::new(error).small().color(self.theme.danger));
        }

        ui.add_space(6.0);
        let tab_labels = ["Tree", "Search", "Migrations", "Tasks", "Graph", "Git"];
        let selected_tab = match self.workspace.files_panel_tab {
            FilesPanelTab::Tree => 0,
            FilesPanelTab::Search => 1,
            FilesPanelTab::Migrations => 2,
            FilesPanelTab::Tasks => 3,
            FilesPanelTab::Graph => 4,
            FilesPanelTab::Git => 5,
        };
        if let Some(next) = segmented_control(ui, &tab_labels, selected_tab, false, self.theme) {
            self.workspace.files_panel_tab = match next {
                1 => FilesPanelTab::Search,
                2 => FilesPanelTab::Migrations,
                3 => FilesPanelTab::Tasks,
                4 => FilesPanelTab::Graph,
                5 => FilesPanelTab::Git,
                _ => FilesPanelTab::Tree,
            };
            if self.workspace.files_panel_tab == FilesPanelTab::Git {
                self.workspace.files.refresh_git_status(&mut self.feedback);
            }
        }
        ui.add_space(6.0);

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
}
