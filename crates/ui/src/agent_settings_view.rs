//! Agent provider settings presentation and user intents.

use super::*;
use crate::components::button::{Button, ButtonSize, ButtonVariant};
use lucide_icons::Icon;

pub(crate) const AI_EGRESS_DISCLOSURE: &str = "With a key configured, the AI features send data to that provider: your prompts, the SQL they reference and the schema names and types around them. When the agent runs a query, up to 20 sample result rows (50 columns, 256 characters per cell) are sent too.\nYour database and SSH connections are the app's only other outbound connections.";

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum AgentSettingsAction {
    Close,
    SaveKey(String),
    ForgetKey,
}

pub(crate) struct AgentSettingsContext<'a> {
    pub(crate) theme: DbProTheme,
    pub(crate) provider_label: &'a str,
    pub(crate) api_key_draft: &'a mut String,
    pub(crate) api_key_show_password: &'a mut bool,
    pub(crate) configure_request: Option<RequestId>,
    pub(crate) auto_run_read_only: &'a mut bool,
}

impl AgentSettingsContext<'_> {
    pub(crate) fn draw(&mut self, ui: &mut egui::Ui) -> Vec<AgentSettingsAction> {
        let mut actions = Vec::new();
        actions.extend(self.draw_header(ui));
        let save_shortcut = self.draw_key_input(ui);
        actions.extend(self.draw_key_actions(ui, save_shortcut));
        self.draw_footer(ui);
        actions
    }

    fn draw_header(&self, ui: &mut egui::Ui) -> Vec<AgentSettingsAction> {
        let actions = self.draw_toolbar(ui);
        self.draw_provider_disclosure(ui);
        actions
    }

    fn draw_toolbar(&self, ui: &mut egui::Ui) -> Vec<AgentSettingsAction> {
        let mut actions = Vec::new();
        let theme = self.theme;
        ui.add_space(8.0);
        toolbar_frame(theme).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(icon_text(Icon::KeyRound, "API Key", theme.text_primary));
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if Button::new(theme)
                        .icon(Icon::X)
                        .variant(ButtonVariant::Ghost)
                        .size(ButtonSize::IconSm)
                        .tooltip("Cancel")
                        .show(ui)
                        .clicked()
                    {
                        actions.push(AgentSettingsAction::Close);
                    }
                });
            });
        });
        actions
    }

    fn draw_provider_disclosure(&self, ui: &mut egui::Ui) {
        let theme = self.theme;
        let provider_label = self.provider_label;
        ui.add_space(4.0);
        ui.label(
            RichText::new("Enter a Groq or OpenAI API key to enable the AI provider.\nThe key is stored in DB Pro's secure secret store and never written to disk in plain text.")
                .font(font_caption())
                .color(theme.text_secondary),
        );
        ui.add_space(6.0);
        ui.label(
            RichText::new(AI_EGRESS_DISCLOSURE)
                .font(font_caption())
                .color(theme.text_muted),
        );
        ui.add_space(8.0);
        let current_label = if provider_label == "Offline draft" {
            "Not configured".to_owned()
        } else {
            format!("Active: {provider_label}")
        };
        ui.label(
            RichText::new(current_label)
                .font(font_caption())
                .color(if provider_label == "Offline draft" {
                    theme.text_muted
                } else {
                    theme.success
                }),
        );
        ui.add_space(6.0);
    }

    fn draw_key_input(&mut self, ui: &mut egui::Ui) -> bool {
        ui.add_space(6.0);

        let response = PasswordInput::new(
            self.api_key_draft,
            "gsk_… or sk-…",
            self.api_key_show_password,
            self.theme,
        )
        .id_salt("agent.api_key")
        .width(ui.available_width())
        .show(ui);
        response.has_focus() && ui.input(|input| input.key_pressed(egui::Key::Enter) && primary_modifier_pressed(input))
    }

    fn draw_key_actions(&mut self, ui: &mut egui::Ui, save_shortcut: bool) -> Vec<AgentSettingsAction> {
        let mut actions = Vec::new();
        let theme = self.theme;
        let provider_label = self.provider_label;
        ui.add_space(6.0);

        ui.horizontal(|ui| {
            let key_non_empty = !self.api_key_draft.trim().is_empty();
            let is_saving = self.configure_request.is_some();
            let save_button = Button::new(theme)
                .icon(if is_saving { Icon::Loader } else { Icon::Check })
                .text(if is_saving { "Saving…" } else { "Save key" })
                .variant(ButtonVariant::Default)
                .size(ButtonSize::Sm)
                .enabled(!is_saving && key_non_empty)
                .loading(is_saving)
                .show(ui);
            if (save_button.clicked() || save_shortcut) && key_non_empty && !is_saving {
                actions.push(AgentSettingsAction::SaveKey(self.api_key_draft.trim().to_owned()));
            }
            if !key_non_empty {
                ui.label(
                    RichText::new("Paste an API key above")
                        .font(font_caption())
                        .color(theme.text_muted),
                );
            }
            if provider_label != "Offline draft" && !is_saving && !key_non_empty {
                if Button::new(theme)
                    .icon(Icon::Trash2)
                    .text("Forget key")
                    .variant(ButtonVariant::Destructive)
                    .size(ButtonSize::Sm)
                    .show(ui)
                    .clicked()
                {
                    actions.push(AgentSettingsAction::ForgetKey);
                }
            }
        });
        actions
    }

    fn draw_footer(&mut self, ui: &mut egui::Ui) {
        ui.add_space(8.0);
        ui.checkbox(self.auto_run_read_only, "Auto-run read-only queries in Agent mode");
        ui.add_space(8.0);
        ui.separator();
        ui.add_space(4.0);
        ui.label(
            RichText::new("Supported providers:\n• Groq  — gsk_… key, model openai/gpt-oss-120b\n• OpenAI — sk-… key, model gpt-5.6\n\nThe provider is detected automatically from the key prefix.")
                .font(font_caption())
                .color(self.theme.text_muted),
        );
    }
}

fn primary_modifier_pressed(input: &egui::InputState) -> bool {
    input.modifiers.command || input.modifiers.ctrl || input.modifiers.mac_cmd
}
