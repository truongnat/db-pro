// cc-scan:allow-file HUGE_FUNCTION,LONG_FUNCTION,HARD_COMPLEXITY,COMPLEXITY,DEEP_NESTING
// cc-scan:allow-file HARD_PARAMS,TOO_MANY_PARAMS,DUPLICATE_BLOCK
// egui painter/view file: fns are linear layout code; branches are per-state paint variants.
//! Workspace-files shell presentation and typed navigation intents.
use super::super::ide_workspace::IdeWorkspaceState;
use super::super::*;
use crate::components::button::{Button, ButtonSize, ButtonVariant};
use crate::components::{Select, SelectSize, SelectVariant};
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
            ui.label(RichText::new("WORKSPACE").font(font_caption()).strong().color(self.theme.text_secondary));
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if Button::new(self.theme)
                    .icon(Icon::FolderOpen)
                    .variant(ButtonVariant::Ghost)
                    .size(ButtonSize::IconSm)
                    .tooltip("Add / open folder")
                    .access_label("Add / open folder")
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
                        .tooltip(t!("common.refresh"))
                        .access_label(t!("common.refresh"))
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
        ui.label(RichText::new("RECENT").font(font_caption()).strong().color(self.theme.text_secondary));
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
            inner_margin: egui::Margin::same(SPACE_SM as i8),
            corner_radius: egui::CornerRadius::same(RADIUS_MD as u8),
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
                ui.label(
                    RichText::new(char::from(Icon::FolderTree).to_string())
                        .font(font_icon(ICON_XS))
                        .color(self.theme.accent),
                );
                // cc-scan:allow LINE_TOO_LONG — literal must not wrap
                ui.label(RichText::new(crate::components::truncate_ellipsis(&active_root_name, 28)).font(font_ui_label()).strong().color(self.theme.text_primary));
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if Button::new(self.theme)
                        .icon(Icon::X)
                        .variant(ButtonVariant::Ghost)
                        .size(ButtonSize::IconSm)
                        .tooltip("Close workspace")
                        .access_label("Close workspace")
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
                            .access_label("Remove active root")
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

            // Trust status: read-only state on the left, explicit toggle on the
            // right — a badge that was secretly a button confused the env row.
            let trusted = self.workspace.is_trusted();
            let (trust_icon, trust_color, trust_label) = if trusted {
                (Icon::ShieldCheck, self.theme.success, "Trusted")
            } else {
                (Icon::ShieldAlert, self.theme.warning, "Untrusted")
            };
            ui.horizontal(|ui| {
                ui.label(RichText::new(char::from(trust_icon).to_string()).font(font_icon(ICON_XS)).color(trust_color));
                ui.label(RichText::new(trust_label).font(font_caption()).color(trust_color));
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let (toggle_label, toggle_hint) = if trusted {
                        ("Revoke", "Mark workspace as untrusted")
                    } else {
                        ("Trust", "Trust this workspace to enable writes and tasks")
                    };
                    if Button::new(self.theme)
                        .text(toggle_label)
                        .variant(ButtonVariant::Ghost)
                        .size(ButtonSize::Sm)
                        .tooltip(toggle_hint)
                        .show(ui)
                        .clicked()
                    {
                        actions.push(FilesSurfaceAction::SetTrusted(!trusted));
                    }
                });
            });

            if !self.workspace.environments.is_empty() {
                ui.add_space(SPACE_XS);
                let env_names: Vec<String> = self
                    .workspace
                    .environments
                    .iter()
                    .map(|environment| environment.name.clone())
                    .collect();
                let mut selected = self
                    .workspace
                    .active_environment
                    .min(env_names.len().saturating_sub(1));
                Select::new("workspace_environment", &mut selected, &env_names)
                    .theme(self.theme)
                    .size(SelectSize::Sm)
                    .variant(SelectVariant::Outline)
                    .label("Environment")
                    .show(ui);
                if selected != self.workspace.active_environment {
                    actions.push(FilesSurfaceAction::SelectEnvironment(selected));
                }
            }

            // Multiple roots switcher — badge chips, same language as environments above
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
                        let resp = ui
                            .scope(|ui| {
                                crate::components::badge::Badge::new(&label, self.theme)
                                    .variant(if is_active {
                                        crate::components::badge::BadgeVariant::Default
                                    } else {
                                        crate::components::badge::BadgeVariant::Secondary
                                    })
                                    .compact(true)
                                    .show(ui)
                            })
                            .response
                            .on_hover_cursor(egui::CursorIcon::PointingHand)
                            .on_hover_text(root.path.display().to_string());
                        if resp.clicked() && !is_active {
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
            (FilesPanelTab::Migrations, Icon::Database, "Migrations"),
            (FilesPanelTab::Tasks, Icon::ListCheck, "Tasks"),
            (FilesPanelTab::Graph, Icon::GitGraph, "Graph"),
            (FilesPanelTab::Git, Icon::GitBranch, "Git"),
        ];

        // Same icon-tab language as the output strip: icon-only, label in the
        // tooltip — a 3×2 labelled grid overflowed the sidebar visually.
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = egui::vec2(SPACE_XS, 0.0);
            for (tab, icon, label) in tabs {
                let is_active = self.selected_tab == tab;
                let bg_color = if is_active {
                    self.theme.surface_active
                } else {
                    egui::Color32::TRANSPARENT
                };
                let icon_color = if is_active {
                    self.theme.accent
                } else {
                    self.theme.text_muted
                };
                let response = egui::Frame::NONE
                    .fill(bg_color)
                    .corner_radius(egui::CornerRadius::same(RADIUS_SM as u8))
                    .inner_margin(egui::Margin::symmetric(SPACE_SM as i8, SPACE_XS as i8))
                    .show(ui, |ui| {
                        ui.label(
                            RichText::new(char::from(icon).to_string())
                                .font(egui::FontId::new(12.0, egui::FontFamily::Name("lucide".into())))
                                .color(icon_color),
                        );
                    })
                    .response
                    .interact(egui::Sense::click())
                    .on_hover_cursor(egui::CursorIcon::PointingHand)
                    .on_hover_text(label);
                if response.clicked() && !is_active {
                    actions.push(FilesSurfaceAction::SelectTab(tab));
                }
            }
        });
        ui.add_space(SPACE_XS);
    }
}
