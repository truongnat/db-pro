//! Left activity rail (48px): Explorer / Files / Query / … settings.
use super::*;

impl DbProApp {
    pub(super) fn draw_activity_bar(&mut self, ctx: &egui::Context) {
        let theme = self.theme;
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
                        let active = activity.is_some_and(|value| self.activity == value)
                            || (hint == "Queries" && self.active_tab == WorkspaceTab::Query)
                            || (hint == "Agent (Copilot)" && self.agent_open);
                        let response = icon_button(ui, icon, active, self.theme);
                        if response.on_hover_text(hint).clicked() {
                            match (activity, hint) {
                                (Some(value), _) => {
                                    self.activity = value;
                                    self.sidebar_open = true;
                                    if value == Activity::Queries {
                                        self.active_tab = WorkspaceTab::Query;
                                    } else if value == Activity::Diagram {
                                        self.active_tab = WorkspaceTab::Diagram;
                                    } else if value == Activity::Schema {
                                        self.open_schema_workbench();
                                    } else if value == Activity::Compare {
                                        self.active_tab = WorkspaceTab::SchemaCompare;
                                    }
                                }
                                (None, "Agent (Copilot)") => self.set_agent_open(!self.agent_open, ctx),
                                _ => {}
                            }
                        }
                        ui.add_space(SPACE_XS);
                    }
                    ui.add_space((ui.available_height() - 44.0).max(0.0));
                    let settings = icon_button(ui, Icon::Settings2, self.activity == Activity::Settings, self.theme);
                    if settings.on_hover_text("Settings").clicked() {
                        self.activity = Activity::Settings;
                        self.sidebar_open = true;
                    }
                    ui.add_space(SPACE_SM);
                });

                ui.allocate_rect(full, egui::Sense::hover());
            });
    }
}
