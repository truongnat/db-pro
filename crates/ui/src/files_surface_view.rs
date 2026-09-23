//! Workspace-files shell presentation and typed navigation intents.
use super::super::ide_workspace::IdeWorkspaceState;
use super::super::*;
use crate::components::button::{Button, ButtonSize, ButtonVariant};
use crate::segmented_control;
use egui::{vec2, Align, Layout, RichText};
use lucide_icons::Icon;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum FilesSurfaceAction {
    OpenFolder,
    Refresh,
    OpenRecent(PathBuf),
    SelectRoot(usize),
    RemoveRoot,
    SetTrusted(bool),
    SelectEnvironment(usize),
    Close,
    SelectTab(FilesPanelTab),
}

pub(super) struct FilesSurfaceContext<'a> {
    pub(super) theme: DbProTheme,
    pub(super) workspace: &'a IdeWorkspaceState,
    pub(super) selected_tab: FilesPanelTab,
}

impl FilesSurfaceContext<'_> {
    pub(super) fn draw(&self, ui: &mut egui::Ui) -> Vec<FilesSurfaceAction> {
        let mut actions = Vec::new();
        self.draw_header(ui, &mut actions);
        if self.workspace.roots.is_empty() {
            self.draw_empty_state(ui, &mut actions);
            return actions;
        }
        self.draw_workspace_chrome(ui, &mut actions);
        actions
    }

    fn draw_header(&self, ui: &mut egui::Ui, actions: &mut Vec<FilesSurfaceAction>) {
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
                    actions.push(FilesSurfaceAction::OpenFolder);
                }
                if !self.workspace.roots.is_empty()
                    && Button::new(self.theme)
                        .icon(Icon::RefreshCw)
                        .variant(ButtonVariant::Ghost)
                        .size(ButtonSize::IconSm)
                        .tooltip("Refresh tree")
                        .show(ui)
                        .clicked()
                {
                    actions.push(FilesSurfaceAction::Refresh);
                }
            });
        });
        ui.add_space(6.0);
    }

    fn draw_empty_state(&self, ui: &mut egui::Ui, actions: &mut Vec<FilesSurfaceAction>) {
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
            actions.push(FilesSurfaceAction::OpenFolder);
        }
        if self.workspace.recent_roots.is_empty() {
            return;
        }
        ui.add_space(12.0);
        section_label(ui, "RECENT", self.theme);
        ui.add_space(6.0);
        for path in self.workspace.recent_roots.iter().take(8) {
            let label = path
                .file_name()
                .map(|name| name.to_string_lossy().into_owned())
                .unwrap_or_else(|| path.display().to_string());
            if sidebar_item(ui, Icon::Folder, &label, false, self.theme)
                .on_hover_text(path.display().to_string())
                .clicked()
            {
                actions.push(FilesSurfaceAction::OpenRecent(path.clone()));
            }
        }
    }

    fn draw_workspace_chrome(&self, ui: &mut egui::Ui, actions: &mut Vec<FilesSurfaceAction>) {
        self.draw_roots(ui, actions);
        self.draw_workspace_status(ui, actions);
        self.draw_tabs(ui, actions);
    }

    fn draw_roots(&self, ui: &mut egui::Ui, actions: &mut Vec<FilesSurfaceAction>) {
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = vec2(4.0, 4.0);
            for (index, root) in self.workspace.roots.iter().enumerate() {
                let label = root
                    .path
                    .file_name()
                    .map(|name| name.to_string_lossy().into_owned())
                    .unwrap_or_else(|| root.path.display().to_string());
                if ui
                    .selectable_label(self.workspace.active_root == index, label)
                    .clicked()
                {
                    actions.push(FilesSurfaceAction::SelectRoot(index));
                }
            }
            if Button::new(self.theme)
                .text("+ Root")
                .variant(ButtonVariant::Ghost)
                .size(ButtonSize::Sm)
                .show(ui)
                .clicked()
            {
                actions.push(FilesSurfaceAction::OpenFolder);
            }
            if Button::new(self.theme)
                .text("Remove")
                .variant(ButtonVariant::Ghost)
                .size(ButtonSize::Sm)
                .show(ui)
                .clicked()
            {
                actions.push(FilesSurfaceAction::RemoveRoot);
            }
        });
    }

    fn draw_workspace_status(&self, ui: &mut egui::Ui, actions: &mut Vec<FilesSurfaceAction>) {
        ui.add_space(4.0);
        if let Some(path) = self.workspace.primary_path() {
            ui.label(
                RichText::new(path.display().to_string())
                    .small()
                    .monospace()
                    .color(self.theme.text_secondary),
            );
        }
        ui.add_space(4.0);
        self.draw_trust_controls(ui, actions);
        ui.add_space(4.0);
        self.draw_environments(ui, actions);
        if let Some(drift) = &self.workspace.schema_drift_message {
            ui.label(RichText::new(drift).small().color(self.theme.warning));
        }
        if let Some(error) = &self.workspace.last_error {
            ui.label(RichText::new(error).small().color(self.theme.danger));
        }
        ui.add_space(6.0);
    }

    fn draw_trust_controls(&self, ui: &mut egui::Ui, actions: &mut Vec<FilesSurfaceAction>) {
        ui.horizontal(|ui| {
            let trusted = self.workspace.is_trusted();
            if ui.selectable_label(trusted, "Trusted").clicked() {
                actions.push(FilesSurfaceAction::SetTrusted(true));
            }
            if ui.selectable_label(!trusted, "Untrusted").clicked() {
                actions.push(FilesSurfaceAction::SetTrusted(false));
            }
            if Button::new(self.theme)
                .icon(Icon::X)
                .text("Close")
                .variant(ButtonVariant::Ghost)
                .size(ButtonSize::Sm)
                .show(ui)
                .clicked()
            {
                actions.push(FilesSurfaceAction::Close);
            }
        });
    }

    fn draw_environments(&self, ui: &mut egui::Ui, actions: &mut Vec<FilesSurfaceAction>) {
        ui.horizontal_wrapped(|ui| {
            for (index, environment) in self.workspace.environments.iter().enumerate() {
                if ui
                    .selectable_label(self.workspace.active_environment == index, &environment.name)
                    .clicked()
                {
                    actions.push(FilesSurfaceAction::SelectEnvironment(index));
                }
            }
        });
    }

    fn draw_tabs(&self, ui: &mut egui::Ui, actions: &mut Vec<FilesSurfaceAction>) {
        let labels = ["Tree", "Search", "Migrations", "Tasks", "Graph", "Git"];
        let selected = match self.selected_tab {
            FilesPanelTab::Tree => 0,
            FilesPanelTab::Search => 1,
            FilesPanelTab::Migrations => 2,
            FilesPanelTab::Tasks => 3,
            FilesPanelTab::Graph => 4,
            FilesPanelTab::Git => 5,
        };
        if let Some(index) = segmented_control(ui, &labels, selected, false, self.theme) {
            let tab = match index {
                1 => FilesPanelTab::Search,
                2 => FilesPanelTab::Migrations,
                3 => FilesPanelTab::Tasks,
                4 => FilesPanelTab::Graph,
                5 => FilesPanelTab::Git,
                _ => FilesPanelTab::Tree,
            };
            actions.push(FilesSurfaceAction::SelectTab(tab));
        }
        ui.add_space(6.0);
    }
}
