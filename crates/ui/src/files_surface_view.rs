//! Workspace-files shell presentation and typed navigation intents.
use super::super::ide_workspace::IdeWorkspaceState;
use super::super::*;
use crate::components::button::{Button, ButtonSize, ButtonVariant};
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
        self.draw_workspace_card(ui, actions);
        ui.add_space(SPACE_SM);
        self.draw_tabs(ui, actions);
    }

    fn draw_workspace_card(&self, ui: &mut egui::Ui, actions: &mut Vec<FilesSurfaceAction>) {
        egui::Frame {
            fill: self.theme.surface_elevated,
            stroke: egui::Stroke::new(1.0, self.theme.border_subtle),
            inner_margin: egui::Margin::same(SPACE_SM),
            rounding: egui::Rounding::same(RADIUS_MD),
            ..Default::default()
        }
        .show(ui, |ui| {
            let active_root_name = self.workspace.roots.get(self.workspace.active_root).map(|root| {
                root.path
                    .file_name()
                    .map(|name| name.to_string_lossy().into_owned())
                    .unwrap_or_else(|| root.path.display().to_string())
            }).unwrap_or_else(|| "Workspace".to_owned());

            // Header row: Folder icon + Name + Action buttons
            ui.horizontal(|ui| {
                ui.label(icon_text(Icon::FolderTree, "", self.theme.accent));
                ui.label(RichText::new(&active_root_name).font(font_ui_label()).strong().color(self.theme.text_primary));
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if Button::new(self.theme)
                        .icon(Icon::X)
                        .variant(ButtonVariant::Ghost)
                        .size(ButtonSize::IconSm)
                        .tooltip("Close workspace")
                        .show(ui)
                        .clicked()
                    {
                        actions.push(FilesSurfaceAction::Close);
                    }
                    if self.workspace.roots.len() > 1
                        && Button::new(self.theme)
                            .icon(Icon::Trash2)
                            .variant(ButtonVariant::Ghost)
                            .size(ButtonSize::IconSm)
                            .tooltip("Remove active root")
                            .show(ui)
                            .clicked()
                    {
                        actions.push(FilesSurfaceAction::RemoveRoot);
                    }
                });
            });

            // Path row
            if let Some(path) = self.workspace.primary_path() {
                let path_str = path.display().to_string();
                let truncated = crate::components::truncate_ellipsis(&path_str, 32);
                ui.add_space(SPACE_XXS);
                ui.label(
                    RichText::new(&truncated)
                        .font(font_caption())
                        .monospace()
                        .color(self.theme.text_muted),
                )
                .on_hover_text(&path_str);
            }

            ui.add_space(SPACE_XS);

            // Badges row: Trust status toggle + Environment
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing = vec2(SPACE_XS, SPACE_XXS);
                let trusted = self.workspace.is_trusted();
                let trust_label = if trusted { "Trusted" } else { "Untrusted" };
                let trust_variant = if trusted {
                    crate::components::badge::BadgeVariant::Success
                } else {
                    crate::components::badge::BadgeVariant::Warning
                };
                let resp = ui.scope(|ui| {
                    crate::components::badge::Badge::new(trust_label, self.theme)
                        .variant(trust_variant)
                        .compact(true)
                        .show(ui)
                }).response;
                if resp.on_hover_cursor(egui::CursorIcon::PointingHand).on_hover_text("Click to toggle workspace trust").clicked() {
                    actions.push(FilesSurfaceAction::SetTrusted(!trusted));
                }

                for (index, environment) in self.workspace.environments.iter().enumerate() {
                    let is_active = self.workspace.active_environment == index;
                    let env_variant = if is_active {
                        crate::components::badge::BadgeVariant::Default
                    } else {
                        crate::components::badge::BadgeVariant::Secondary
                    };
                    let resp = ui.scope(|ui| {
                        crate::components::badge::Badge::new(&environment.name, self.theme)
                            .variant(env_variant)
                            .compact(true)
                            .show(ui)
                    }).response;
                    if resp.on_hover_cursor(egui::CursorIcon::PointingHand).clicked() && !is_active {
                        actions.push(FilesSurfaceAction::SelectEnvironment(index));
                    }
                }
            });

            // Multiple roots switcher
            if self.workspace.roots.len() > 1 {
                ui.add_space(SPACE_XXS);
                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing = vec2(SPACE_XS, SPACE_XXS);
                    ui.label(RichText::new("Roots:").font(font_caption()).color(self.theme.text_muted));
                    for (index, root) in self.workspace.roots.iter().enumerate() {
                        let label = root
                            .path
                            .file_name()
                            .map(|name| name.to_string_lossy().into_owned())
                            .unwrap_or_else(|| root.path.display().to_string());
                        let is_active = self.workspace.active_root == index;
                        if ui.selectable_label(is_active, label).clicked() && !is_active {
                            actions.push(FilesSurfaceAction::SelectRoot(index));
                        }
                    }
                });
            }

            if let Some(drift) = &self.workspace.schema_drift_message {
                ui.add_space(SPACE_XXS);
                ui.label(RichText::new(drift).font(font_caption()).color(self.theme.warning));
            }
            if let Some(error) = &self.workspace.last_error {
                ui.add_space(SPACE_XXS);
                ui.label(RichText::new(error).font(font_caption()).color(self.theme.danger));
            }
        });
    }

    fn draw_tabs(&self, ui: &mut egui::Ui, actions: &mut Vec<FilesSurfaceAction>) {
        let tabs = [
            (FilesPanelTab::Tree, Icon::FolderTree, "Files"),
            (FilesPanelTab::Search, Icon::Search, "Search"),
            (FilesPanelTab::Migrations, Icon::Database, "Migrate"),
            (FilesPanelTab::Tasks, Icon::ListCheck, "Tasks"),
            (FilesPanelTab::Graph, Icon::GitGraph, "Graph"),
            (FilesPanelTab::Git, Icon::GitBranch, "Git"),
        ];

        let avail_w = ui.available_width();
        let col_w = ((avail_w - SPACE_XS * 2.0) / 3.0).max(60.0);

        egui::Grid::new("files_panel_subtabs_grid")
            .spacing(vec2(SPACE_XS, SPACE_XS))
            .show(ui, |ui| {
                for (i, (tab, icon, label)) in tabs.iter().enumerate() {
                    let is_active = self.selected_tab == *tab;
                    let (rect, resp) = ui.allocate_exact_size(vec2(col_w, 28.0), egui::Sense::click());
                    let resp = resp.on_hover_cursor(egui::CursorIcon::PointingHand);
                    let hovered = resp.hovered();

                    let bg_fill = if is_active {
                        self.theme.surface_active
                    } else if hovered {
                        self.theme.surface_hover
                    } else {
                        self.theme.surface_panel
                    };

                    ui.painter().rect_filled(rect, egui::Rounding::same(RADIUS_SM), bg_fill);
                    ui.painter().rect_stroke(
                        rect,
                        egui::Rounding::same(RADIUS_SM),
                        egui::Stroke::new(1.0, if is_active { self.theme.border_default } else { self.theme.border_subtle }),
                    );

                    let text_color = if is_active || hovered {
                        self.theme.text_primary
                    } else {
                        self.theme.text_secondary
                    };

                    let icon_color = if is_active {
                        self.theme.accent
                    } else {
                        self.theme.text_muted
                    };

                    ui.painter().text(
                        egui::pos2(rect.left() + 6.0, rect.center().y),
                        egui::Align2::LEFT_CENTER,
                        char::from(*icon).to_string(),
                        font_icon(ICON_XS),
                        icon_color,
                    );

                    ui.painter().text(
                        egui::pos2(rect.left() + 20.0, rect.center().y),
                        egui::Align2::LEFT_CENTER,
                        *label,
                        font_caption(),
                        text_color,
                    );

                    if resp.clicked() && !is_active {
                        actions.push(FilesSurfaceAction::SelectTab(*tab));
                    }

                    if (i + 1) % 3 == 0 {
                        ui.end_row();
                    }
                }
            });

        ui.add_space(SPACE_XS);
    }
}
