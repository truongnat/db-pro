//! Left activity rail (48px): Explorer / Files / Query / … settings.
use super::*;

pub(super) enum ActivityBarAction {
    OpenSchemaWorkbench,
    ToggleAgent,
}

pub(super) fn draw_activity_bar(
    ctx: &egui::Context,
    theme: DbProTheme,
    workspace: &mut WorkspaceFeatureState,
) -> Option<ActivityBarAction> {
    let mut action = None;
    egui::SidePanel::left("activity_bar")
        .resizable(false)
        .exact_width(48.0)
        .show_separator_line(false)
        .frame(egui::Frame {
            fill: theme.surface_panel,
            inner_margin: egui::Margin::ZERO,
            outer_margin: egui::Margin::ZERO,
            stroke: egui::Stroke::NONE,
            rounding: egui::Rounding::ZERO,
            shadow: egui::Shadow::NONE,
        })
        .show(ctx, |ui| {
            let full = ui.max_rect();
            ui.painter()
                .rect_filled(full, egui::Rounding::ZERO, theme.surface_panel);
            let line_x = ui.painter().round_to_pixel_center(full.right() - 1.0);
            ui.painter()
                .vline(line_x, full.y_range(), egui::Stroke::new(1.0, theme.border_subtle));

            ui.vertical_centered(|ui| {
                ui.add_space(SPACE_SM);
                for (activity, icon, hint) in [
                    (Some(Activity::Explorer), Icon::Database, "Explorer"),
                    (Some(Activity::Files), Icon::FolderOpen, "Files"),
                    (Some(Activity::Queries), Icon::FileCode2, "Queries"),
                    (Some(Activity::Data), Icon::Table2, "Data"),
                    (Some(Activity::History), Icon::History, "History"),
                    (Some(Activity::Problems), Icon::TriangleAlert, "Problems"),
                    (Some(Activity::Transfers), Icon::Upload, "Transfers"),
                    (Some(Activity::Monitor), Icon::Gauge, "Monitor"),
                    (Some(Activity::Security), Icon::Shield, "Security"),
                    (Some(Activity::Diagram), Icon::ArrowRightLeft, "ER diagram"),
                    (Some(Activity::Schema), Icon::Boxes, "Schema workbench"),
                    (Some(Activity::Compare), Icon::GitCompare, "Schema compare"),
                    (Some(Activity::Tasks), Icon::ListTodo, "Saved tasks"),
                    (None, Icon::Bot, "Agent (Copilot)"),
                ] {
                    let active = activity.is_some_and(|value| workspace.activity == value)
                        || (hint == "Queries" && workspace.active_tab == WorkspaceTab::Query)
                        || (hint == "Agent (Copilot)" && workspace.agent_open);
                    let response = icon_button(ui, icon, active, theme);
                    if response.on_hover_text(hint).clicked() {
                        match (activity, hint) {
                            (Some(value), _) => {
                                workspace.activity = value;
                                workspace.sidebar_open = true;
                                if value == Activity::Queries {
                                    workspace.active_tab = WorkspaceTab::Query;
                                } else if value == Activity::Diagram {
                                    workspace.active_tab = WorkspaceTab::Diagram;
                                } else if value == Activity::Schema {
                                    action = Some(ActivityBarAction::OpenSchemaWorkbench);
                                } else if value == Activity::Compare {
                                    workspace.active_tab = WorkspaceTab::SchemaCompare;
                                }
                            }
                            (None, "Agent (Copilot)") => action = Some(ActivityBarAction::ToggleAgent),
                            _ => {}
                        }
                    }
                    ui.add_space(SPACE_XS);
                }
                ui.add_space((ui.available_height() - 44.0).max(0.0));
                let settings = icon_button(ui, Icon::Settings2, workspace.activity == Activity::Settings, theme);
                if settings.on_hover_text("Settings").clicked() {
                    workspace.activity = Activity::Settings;
                    workspace.sidebar_open = true;
                }
                ui.add_space(SPACE_SM);
            });

            ui.allocate_rect(full, egui::Sense::hover());
        });
    action
}
