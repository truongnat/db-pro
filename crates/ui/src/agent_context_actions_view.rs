//! Quick actions for the Agent context surface.

use super::*;
use crate::components::button::{Button, ButtonSize, ButtonVariant};
use lucide_icons::Icon;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum AgentContextAction {
    Submit(&'static str),
}

pub(crate) struct AgentContextActionsContext<'a> {
    pub(crate) theme: DbProTheme,
    pub(crate) context: &'a AgentContext,
}

impl AgentContextActionsContext<'_> {
    pub(crate) fn draw(&self, ui: &mut egui::Ui) -> Vec<AgentContextAction> {
        if self.context.selected_table.is_none()
            && self.context.current_sql.trim().is_empty()
            && self.context.last_error.is_none()
        {
            return Vec::new();
        }

        let mut actions = Vec::new();
        toolbar_frame(self.theme).show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                actions.extend(self.draw_query_actions(ui));
                actions.extend(self.draw_table_action(ui));
                actions.extend(self.draw_error_action(ui));
            });
        });
        ui.add_space(8.0);
        actions
    }

    fn draw_query_actions(&self, ui: &mut egui::Ui) -> Vec<AgentContextAction> {
        if self.context.current_sql.trim().is_empty() {
            return Vec::new();
        }
        let mut actions = Vec::new();
        if Button::new(self.theme)
            .icon(Icon::ChartNoAxesCombined)
            .text("Explain query")
            .variant(ButtonVariant::Secondary)
            .size(ButtonSize::Sm)
            .show(ui)
            .clicked()
        {
            actions.push(AgentContextAction::Submit("Explain the current SQL and its query plan"));
        }
        if Button::new(self.theme)
            .icon(Icon::Gauge)
            .text("Optimize")
            .variant(ButtonVariant::Secondary)
            .size(ButtonSize::Sm)
            .show(ui)
            .clicked()
        {
            actions.push(AgentContextAction::Submit(
                "Optimize the current SQL and explain the trade-offs",
            ));
        }
        actions
    }

    fn draw_table_action(&self, ui: &mut egui::Ui) -> Vec<AgentContextAction> {
        if self.context.selected_table.is_none() {
            return Vec::new();
        }
        if Button::new(self.theme)
            .icon(Icon::Table2)
            .text("Explain table")
            .variant(ButtonVariant::Secondary)
            .size(ButtonSize::Sm)
            .show(ui)
            .clicked()
        {
            return vec![AgentContextAction::Submit(
                "Explain the selected table and suggest useful read-only queries",
            )];
        }
        Vec::new()
    }

    fn draw_error_action(&self, ui: &mut egui::Ui) -> Vec<AgentContextAction> {
        if self.context.last_error.is_none() {
            return Vec::new();
        }
        if Button::new(self.theme)
            .icon(Icon::TriangleAlert)
            .text("Investigate error")
            .variant(ButtonVariant::Secondary)
            .size(ButtonSize::Sm)
            .show(ui)
            .clicked()
        {
            return vec![AgentContextAction::Submit(
                "Investigate the current database error and propose a safe fix",
            )];
        }
        Vec::new()
    }
}
