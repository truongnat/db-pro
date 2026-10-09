// cc-scan:allow-file HUGE_FUNCTION,LONG_FUNCTION,HARD_COMPLEXITY,COMPLEXITY,DEEP_NESTING
// cc-scan:allow-file HARD_PARAMS,TOO_MANY_PARAMS,DUPLICATE_BLOCK
// egui painter/view file: fns are linear layout code; branches are per-state paint variants.
use crate::components::{Button, ButtonSize, ButtonVariant};
use crate::DbProTheme;
use egui::{CornerRadius, RichText, Stroke, Ui};

use super::{config, handler, ExecutionApprovalAction, StatusBadge};

pub struct ExecutionApproval<'a> {
    title: &'a str,
    impact: &'a str,
    sql_preview: &'a str,
    risk: super::RiskLevel,
    theme: DbProTheme,
}

impl<'a> ExecutionApproval<'a> {
    pub fn new(
        title: &'a str,
        impact: &'a str,
        sql_preview: &'a str,
        risk: super::RiskLevel,
        theme: DbProTheme,
    ) -> Self {
        Self {
            title,
            impact,
            sql_preview,
            risk,
            theme,
        }
    }

    pub fn show(self, ui: &mut Ui) -> Option<ExecutionApprovalAction> {
        let frame = egui::Frame::NONE
            .fill(self.theme.surface_panel)
            .stroke(Stroke::new(config::FRAME_BORDER_WIDTH, self.theme.border_default))
            .corner_radius(CornerRadius::same(config::APPROVAL_RADIUS as u8))
            .inner_margin(egui::Margin::same(config::APPROVAL_PADDING as i8));
        let mut button_actions = [false; 3];

        frame.show(ui, |ui| {
            ui.set_width(ui.available_width());

            // Risk mapping is decided before painting so every risk level follows one
            // typed policy; the header only renders the returned label and badge variant.
            ui.horizontal_wrapped(|ui| {
                ui.label(
                    RichText::new(self.title)
                        .size(config::APPROVAL_TITLE_SIZE)
                        .strong()
                        .color(self.theme.text_primary),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let (risk_text, risk_variant) = handler::risk_badge(self.risk);
                    StatusBadge::new(risk_text, risk_variant, self.theme).show(ui);
                });
            });
            ui.add_space(config::APPROVAL_TITLE_GAP);

            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("Impact:")
                        .size(config::APPROVAL_BODY_SIZE)
                        .strong()
                        .color(self.theme.text_secondary),
                );
                ui.label(
                    RichText::new(self.impact)
                        .size(config::APPROVAL_BODY_SIZE)
                        .color(self.theme.text_primary),
                );
            });
            ui.add_space(config::APPROVAL_CODE_GAP);

            let code_frame = egui::Frame::NONE
                .fill(self.theme.surface_editor)
                .stroke(Stroke::new(config::FRAME_BORDER_WIDTH, self.theme.border_subtle))
                .corner_radius(CornerRadius::same(config::APPROVAL_CODE_RADIUS as u8))
                .inner_margin(egui::Margin::same(config::APPROVAL_CODE_PADDING as i8));
            code_frame.show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.label(
                    RichText::new(self.sql_preview)
                        .monospace()
                        .size(config::APPROVAL_BODY_SIZE)
                        .color(self.theme.text_primary),
                );
            });
            ui.add_space(config::APPROVAL_ACTION_GAP);

            let (run_variant, preview_variant) = handler::approval_button_variants(self.risk);
            ui.horizontal_wrapped(|ui| {
                button_actions[0] = Button::new(self.theme)
                    .text("Run Changes")
                    .variant(run_variant)
                    .size(ButtonSize::Sm)
                    .show(ui)
                    .clicked();
                ui.add_space(config::APPROVAL_BUTTON_GAP);
                button_actions[1] = Button::new(self.theme)
                    .text("Preview SQL")
                    .variant(preview_variant)
                    .size(ButtonSize::Sm)
                    .show(ui)
                    .clicked();
                ui.add_space(config::APPROVAL_BUTTON_GAP);
                button_actions[2] = Button::new(self.theme)
                    .text("Cancel")
                    .variant(ButtonVariant::Ghost)
                    .size(ButtonSize::Sm)
                    .show(ui)
                    .clicked();
            });
        });

        handler::approval_action(button_actions[0], button_actions[1], button_actions[2])
    }
}
