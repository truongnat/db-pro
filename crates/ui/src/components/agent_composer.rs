//! Agent Chat Composer, Toolbar, Model Selector, and Token Usage components.
//!
//! Implements AgentComposer, AgentModelSelector, AgentModeSelector,
//! and TokenUsage per `open-ai-refer.md`.

use crate::components::button::{Button, ButtonSize, ButtonVariant};
use crate::tokens::*;
use crate::DbProTheme;
use egui::{Color32, Pos2, Rect, RichText, Rounding, Stroke, Ui, Vec2};
use lucide_icons::Icon;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentMode {
    Chat,
    Plan,
    Code,
}

impl AgentMode {
    pub fn label(&self) -> &'static str {
        match self {
            AgentMode::Chat => "Chat",
            AgentMode::Plan => "Plan",
            AgentMode::Code => "SQL Agent",
        }
    }
}

pub struct AgentComposer<'a> {
    prompt: &'a mut String,
    is_generating: bool,
    selected_model: &'a str,
    selected_mode: AgentMode,
    token_usage: Option<usize>,
    theme: DbProTheme,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentComposerAction {
    Submit,
    Stop,
    Clear,
}

impl<'a> AgentComposer<'a> {
    pub fn new(prompt: &'a mut String, selected_model: &'a str, selected_mode: AgentMode, theme: DbProTheme) -> Self {
        Self {
            prompt,
            is_generating: false,
            selected_model,
            selected_mode,
            token_usage: None,
            theme,
        }
    }

    pub fn is_generating(mut self, generating: bool) -> Self {
        self.is_generating = generating;
        self
    }

    pub fn token_usage(mut self, tokens: usize) -> Self {
        self.token_usage = Some(tokens);
        self
    }

    pub fn show(self, ui: &mut Ui) -> Option<AgentComposerAction> {
        let mut triggered = None;

        let frame = egui::Frame::none()
            .fill(self.theme.surface_panel)
            .stroke(Stroke::new(STROKE_THIN, self.theme.border_default))
            .rounding(Rounding::same(RADIUS_COMPOSER))
            .inner_margin(egui::Margin::symmetric(SPACE_MD, SPACE_SM));

        frame.show(ui, |ui| {
            ui.set_width(ui.available_width());

            // Text input area
            let text_edit = egui::TextEdit::multiline(self.prompt)
                .desired_rows(2)
                .desired_width(ui.available_width())
                .frame(false)
                .hint_text("Ask AI to generate, optimize, or investigate SQL queries...");

            let edit_resp = ui.add(text_edit);

            if edit_resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter) && !i.modifiers.shift) {
                triggered = Some(AgentComposerAction::Submit);
            }

            ui.add_space(SPACE_XS);

            // Bottom toolbar inside composer: Model tag, mode, token count, send button
            ui.horizontal(|ui| {
                // Model badge
                let model_galley = ui.painter().layout_no_wrap(
                    self.selected_model.to_owned(),
                    font_caption(),
                    self.theme.text_secondary,
                );
                let model_rect = Rect::from_min_size(
                    Pos2::new(ui.cursor().min.x, ui.cursor().min.y + 2.0),
                    Vec2::new(model_galley.size().x + 12.0, 20.0),
                );
                ui.painter()
                    .rect_filled(model_rect, Rounding::same(RADIUS_XS), self.theme.surface_hover);
                ui.painter().galley(
                    Pos2::new(model_rect.left() + 6.0, model_rect.top() + 2.0),
                    model_galley,
                    Color32::PLACEHOLDER,
                );
                ui.add_space(model_rect.width() + SPACE_XS);

                // Mode badge
                let mode_galley = ui.painter().layout_no_wrap(
                    self.selected_mode.label().to_owned(),
                    font_caption(),
                    self.theme.text_tertiary,
                );
                let mode_rect = Rect::from_min_size(
                    Pos2::new(ui.cursor().min.x, ui.cursor().min.y + 2.0),
                    Vec2::new(mode_galley.size().x + 12.0, 20.0),
                );
                ui.painter()
                    .rect_filled(mode_rect, Rounding::same(RADIUS_XS), self.theme.surface_hover);
                ui.painter().galley(
                    Pos2::new(mode_rect.left() + 6.0, mode_rect.top() + 2.0),
                    mode_galley,
                    Color32::PLACEHOLDER,
                );
                ui.add_space(mode_rect.width());

                // Token usage
                if let Some(tokens) = self.token_usage {
                    ui.add_space(SPACE_SM);
                    ui.label(
                        RichText::new(format!("~{} tokens", tokens))
                            .size(FONT_SIZE_CAPTION)
                            .color(self.theme.text_disabled),
                    );
                }

                // Send / Stop button on the right
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if self.is_generating {
                        if Button::new(self.theme)
                            .icon(Icon::Square)
                            .access_label("Stop")
                            .variant(ButtonVariant::Destructive)
                            .size(ButtonSize::Icon)
                            .show(ui)
                            .clicked()
                        {
                            triggered = Some(AgentComposerAction::Stop);
                        }
                    } else {
                        let has_text = !self.prompt.trim().is_empty();
                        if Button::new(self.theme)
                            .icon(Icon::ArrowUp)
                            .access_label("Send")
                            .variant(if has_text {
                                ButtonVariant::Default
                            } else {
                                ButtonVariant::Ghost
                            })
                            .size(ButtonSize::Icon)
                            .enabled(has_text)
                            .show(ui)
                            .clicked()
                        {
                            triggered = Some(AgentComposerAction::Submit);
                        }
                    }
                });
            });
        });

        triggered
    }
}
