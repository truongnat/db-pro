//! Agent header presentation and user intents.

use super::*;
use crate::components::button::{Button, ButtonSize, ButtonVariant};
use lucide_icons::Icon;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum AgentHeaderAction {
    ClearConversation,
    Close,
    ToggleSettings,
}

pub(crate) struct AgentHeaderContext<'a> {
    pub(crate) theme: DbProTheme,
    pub(crate) mode: Option<&'a mut db_pro_core::domain::agent::AgentMode>,
    pub(crate) mode_disabled: bool,
    pub(crate) can_clear_conversation: bool,
    pub(crate) has_messages: bool,
}

impl AgentHeaderContext<'_> {
    pub(crate) fn draw(&mut self, ui: &mut egui::Ui) -> Vec<AgentHeaderAction> {
        let mut actions = Vec::new();
        ui.horizontal(|ui| {
            actions.extend(self.draw_mode(ui));
            actions.extend(self.draw_controls(ui));
        });
        actions
    }

    fn draw_mode(&mut self, ui: &mut egui::Ui) -> Vec<AgentHeaderAction> {
        let Some(mode) = self.mode.as_deref_mut() else {
            return Vec::new();
        };
        ui.label(icon_text(Icon::Sparkles, "Agent", self.theme.accent));
        ui.add_enabled_ui(!self.mode_disabled, |ui| {
            egui::ComboBox::from_id_salt("agent-workflow-mode")
                .selected_text(mode_label(*mode))
                .show_ui(ui, |ui| {
                    ui.selectable_value(mode, db_pro_core::domain::agent::AgentMode::Ask, "Ask");
                    ui.selectable_value(mode, db_pro_core::domain::agent::AgentMode::Edit, "Edit");
                    ui.selectable_value(mode, db_pro_core::domain::agent::AgentMode::Agent, "Agent");
                });
        });
        Vec::new()
    }

    fn draw_controls(&self, ui: &mut egui::Ui) -> Vec<AgentHeaderAction> {
        let mut actions = Vec::new();
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if self.can_clear_conversation
                && self.has_messages
                && Button::new(self.theme)
                    .icon(Icon::RotateCcw)
                    .variant(ButtonVariant::Ghost)
                    .size(ButtonSize::IconSm)
                    .tooltip("Clear conversation")
                    .show(ui)
                    .clicked()
            {
                actions.push(AgentHeaderAction::ClearConversation);
            }
            if Button::new(self.theme)
                .icon(Icon::X)
                .variant(ButtonVariant::Ghost)
                .size(ButtonSize::IconSm)
                .tooltip("Close Agent")
                .show(ui)
                .clicked()
            {
                actions.push(AgentHeaderAction::Close);
            }
            if Button::new(self.theme)
                .icon(Icon::Settings)
                .variant(ButtonVariant::Ghost)
                .size(ButtonSize::IconSm)
                .tooltip("Agent settings (API key)")
                .show(ui)
                .clicked()
            {
                actions.push(AgentHeaderAction::ToggleSettings);
            }
        });
        actions
    }
}

fn mode_label(mode: db_pro_core::domain::agent::AgentMode) -> &'static str {
    match mode {
        db_pro_core::domain::agent::AgentMode::Ask => "Ask",
        db_pro_core::domain::agent::AgentMode::Edit => "Edit",
        db_pro_core::domain::agent::AgentMode::Agent => "Agent",
    }
}
