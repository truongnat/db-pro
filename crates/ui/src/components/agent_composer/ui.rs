// cc-scan:allow-file HUGE_FUNCTION,LONG_FUNCTION,HARD_COMPLEXITY,COMPLEXITY,DEEP_NESTING
// cc-scan:allow-file HARD_PARAMS,TOO_MANY_PARAMS,DUPLICATE_BLOCK
// egui painter/view file: fns are linear layout code; branches are per-state paint variants.
use crate::components::button::{Button, ButtonSize, ButtonVariant};
use crate::tokens::{
    font_caption, FONT_SIZE_CAPTION, RADIUS_COMPOSER, RADIUS_XS, SPACE_MD, SPACE_SM, SPACE_XS, STROKE_THIN,
};
use crate::DbProTheme;
use egui::{Color32, Rect, RichText, Rounding, Stroke, Ui, WidgetInfo, WidgetType};
use lucide_icons::Icon;

use super::{handler, AgentComposerAction, AgentMode};

pub struct AgentComposer<'a> {
    prompt: &'a mut String,
    is_generating: bool,
    selected_model: &'a str,
    selected_mode: AgentMode,
    token_usage: Option<usize>,
    theme: DbProTheme,
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

            ui.label(
                RichText::new("Prompt")
                    .size(FONT_SIZE_CAPTION)
                    .color(self.theme.text_secondary),
            );

            // The editor remains multiline: the handler only submits when egui reports
            // lost focus plus an unshifted Enter, while Shift+Enter remains a newline.
            let text_edit = egui::TextEdit::multiline(self.prompt)
                .desired_rows(2)
                .desired_width(ui.available_width())
                .frame(false)
                .hint_text("Ask AI to generate, optimize, or investigate SQL queries...");
            let edit_resp = ui.add(text_edit);
            edit_resp.widget_info(|| WidgetInfo::labeled(WidgetType::TextEdit, true, "Prompt"));
            let enter_pressed = ui.input(|input| input.key_pressed(egui::Key::Enter));
            let shift_held = ui.input(|input| input.modifiers.shift);
            if let Some(action) =
                handler::keyboard_action(edit_resp.lost_focus(), enter_pressed, shift_held, self.prompt)
            {
                triggered = Some(action);
            }

            ui.add_space(SPACE_XS);

            // Egui measures each badge's label here because only the presentation layer
            // knows the active font. Handler geometry then applies the shared badge shape
            // without making rendering or layout decisions inside the behavior layer.
            ui.horizontal_wrapped(|ui| {
                let model_galley = ui.painter().layout_no_wrap(
                    self.selected_model.to_owned(),
                    font_caption(),
                    self.theme.text_secondary,
                );
                let model_rect = handler::badge_rect(ui.cursor().min, model_galley.size().x);
                paint_badge(ui, model_rect, model_galley, self.theme.surface_hover);
                ui.add_space(model_rect.width() + SPACE_XS);

                let mode_galley = ui.painter().layout_no_wrap(
                    self.selected_mode.label().to_owned(),
                    font_caption(),
                    self.theme.text_tertiary,
                );
                let mode_rect = handler::badge_rect(ui.cursor().min, mode_galley.size().x);
                paint_badge(ui, mode_rect, mode_galley, self.theme.surface_hover);
                ui.add_space(mode_rect.width());

                if let Some(tokens) = self.token_usage {
                    ui.add_space(SPACE_SM);
                    ui.label(
                        RichText::new(format!("~{} tokens", tokens))
                            .size(FONT_SIZE_CAPTION)
                            .color(self.theme.text_disabled),
                    );
                }

                // The button owns the visual enabled/disabled state, but the handler
                // owns the action contract. This prevents blank prompts from submitting
                // even if a future caller accidentally emits a click signal directly.
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let clicked = if self.is_generating {
                        Button::new(self.theme)
                            .icon(Icon::Square)
                            .access_label("Stop")
                            .variant(ButtonVariant::Destructive)
                            .size(ButtonSize::Icon)
                            .show(ui)
                            .clicked()
                    } else {
                        let has_text = handler::has_prompt_text(self.prompt);
                        Button::new(self.theme)
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
                    };
                    if let Some(action) = handler::button_action(self.is_generating, self.prompt, clicked) {
                        triggered = Some(action);
                    }
                });
            });
        });

        triggered
    }
}

fn paint_badge(ui: &Ui, rect: Rect, galley: std::sync::Arc<egui::Galley>, fill_color: Color32) {
    ui.painter().rect_filled(rect, Rounding::same(RADIUS_XS), fill_color);
    ui.painter()
        .galley(handler::badge_text_pos(rect), galley, Color32::PLACEHOLDER);
}
