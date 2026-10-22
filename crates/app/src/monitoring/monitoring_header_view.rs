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
        Card::new(self.theme).frame().show(ui, |ui| {
            ui.set_max_width(ui.available_width());
            // A right-to-left group in this narrow card paints over the status
            // label. Wrap instead so Connected and Auto-refresh stay readable.
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing = egui::vec2(SPACE_SM, SPACE_XS);
                self.draw_status(ui);
                // Reserve the control's real width so the label is not squeezed
                // into "Auto-" / "refresh" on the status row.
                ui.allocate_ui_with_layout(
                    egui::vec2(132.0, crate::tokens::component::input::INPUT_HEIGHT_DEFAULT),
                    egui::Layout::left_to_right(egui::Align::Center),
                    |ui| {
                        Checkbox::new(self.poll, "Auto-refresh", self.theme).show(ui);
                    },
                );
                if self.connected
                    && Button::new(self.theme)
                        .icon(Icon::RefreshCw)
                        .text("Refresh")
                        .variant(ButtonVariant::Secondary)
                        .show(ui)
                        .clicked()
                {
                    actions.push(MonitoringHeaderAction::Refresh);
                }
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

#[cfg(test)]
#[path = "tests/monitoring_header_view_tests.rs"]
mod tests;
