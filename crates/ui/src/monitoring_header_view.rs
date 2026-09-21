//! Monitoring connection header presentation and user intents.

use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum MonitoringHeaderAction {
    Refresh,
}

pub(super) struct MonitoringHeaderContext<'a> {
    pub(super) theme: DbProTheme,
    pub(super) connected: bool,
    pub(super) driver: &'a str,
    pub(super) connection_name: &'a str,
    pub(super) poll: &'a mut bool,
}

impl MonitoringHeaderContext<'_> {
    pub(super) fn draw(&mut self, ui: &mut egui::Ui) -> Vec<MonitoringHeaderAction> {
        let mut actions = Vec::new();
        egui::Frame {
            fill: self.theme.surface_elevated,
            inner_margin: egui::Margin::same(SPACE_MD),
            rounding: egui::Rounding::same(RADIUS_MD),
            stroke: egui::Stroke::new(STROKE_THIN, self.theme.border_subtle),
            ..Default::default()
        }
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                self.draw_status(ui);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if self.connected
                        && secondary_button_with_icon(ui, Icon::RefreshCw, "Refresh", self.theme).clicked()
                    {
                        actions.push(MonitoringHeaderAction::Refresh);
                    }
                    ui.checkbox(self.poll, "Auto-refresh");
                });
            });
            ui.add_space(SPACE_SM);
            if self.connected {
                ui.label(
                    RichText::new(format!("{} · {}", self.connection_name, self.driver))
                        .small()
                        .color(self.theme.text_secondary),
                );
            } else {
                ui.label(
                    RichText::new("Connect from Explorer to monitor sessions.")
                        .small()
                        .color(self.theme.text_muted),
                );
            }
        });
        actions
    }

    fn draw_status(&self, ui: &mut egui::Ui) {
        let (dot, label) = if self.connected {
            (self.theme.success, "Connected")
        } else {
            (self.theme.text_muted, "Disconnected")
        };
        status_dot(ui, dot, self.connected, false, self.theme);
        ui.label(RichText::new(label).strong().color(self.theme.text_primary));
    }
}
