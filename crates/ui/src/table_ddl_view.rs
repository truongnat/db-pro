use super::*;
use crate::components::badge::{Badge, BadgeVariant};
use crate::components::button::{Button, ButtonSize, ButtonVariant};
use crate::components::overlay::ToastPosition;
use egui::{Align, Layout};
use lucide_icons::Icon;

impl DbProApp {
    /// Draw the DDL / Schema tab.
    pub(super) fn draw_table_ddl_view(&mut self, ui: &mut egui::Ui, table_name: &str) {
        let Some(mut ddl) = self.table.state.table_ddl.clone() else {
            self.draw_table_ddl_placeholder(ui, table_name);
            return;
        };

        let schema = self.active_schema().to_owned();
        let can_mutate = self.can_mutate_active_connection();

        toolbar_frame(self.theme).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(icon_text(Icon::Code2, "DDL SCRIPT", self.theme.accent));
                Badge::new("CREATE TABLE", self.theme)
                    .variant(BadgeVariant::Default)
                    .compact(true)
                    .show(ui);

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if Button::new(self.theme)
                        .icon(Icon::RotateCcw)
                        .text("Refresh DDL")
                        .variant(ButtonVariant::Ghost)
                        .size(ButtonSize::Sm)
                        .tooltip("Re-generate DDL from latest database schema")
                        .show(ui)
                        .clicked()
                    {
                        self.table.state.table_ddl = None;
                        self.request_table_ddl();
                    }

                    if Button::new(self.theme)
                        .icon(Icon::Play)
                        .text("Open in Query")
                        .variant(ButtonVariant::Secondary)
                        .size(ButtonSize::Sm)
                        .tooltip("Open DDL in SQL query console")
                        .show(ui)
                        .clicked()
                    {
                        self.set_active_query_text(ddl.clone());
                        self.workspace.active_tab = WorkspaceTab::Query;
                        self.feedback.runtime_message = format!("Opened DDL for {schema}.{table_name} in Query editor");
                    }

                    if Button::new(self.theme)
                        .icon(Icon::Copy)
                        .text("Copy DDL")
                        .variant(ButtonVariant::Ghost)
                        .size(ButtonSize::Sm)
                        .tooltip("Copy DDL statement to clipboard")
                        .show(ui)
                        .clicked()
                    {
                        ui.ctx().copy_text(ddl.clone());
                        self.feedback
                            .toasts
                            .info("DDL copied to clipboard", ToastPosition::BottomRight);
                        self.feedback.runtime_message = "DDL copied to clipboard".to_owned();
                    }
                });
            });
        });

        ui.add_space(8.0);

        let content_changed = self.draw_ddl_script_card(ui, can_mutate, &mut ddl);
        if content_changed {
            self.table.state.table_ddl = Some(ddl);
        }

        if self.table.state.ddl_execute_confirmation {
            let impact = ddl_impact_summary(self.table.state.table_ddl.as_deref().unwrap_or(""), table_name);
            ui.add_space(8.0);
            self.draw_ddl_confirmation_card(ui, &impact);
        }
    }
}
