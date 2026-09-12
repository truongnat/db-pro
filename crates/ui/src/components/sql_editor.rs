//! SQL Editor controls and status components.
//!
//! Implements SqlEditorToolbar, SqlEditorStatusBar, RunQueryButton,
//! ExplainQueryButton, and FormatterButton per `open-ai-refer.md`.

use crate::components::button::{Button, ButtonSize, ButtonVariant};
use crate::tokens::*;
use crate::DbProTheme;
use egui::{Rounding, Stroke, Ui};
use lucide_icons::Icon;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SqlEditorAction {
    RunQuery,
    RunSelection,
    ExplainQuery,
    FormatSql,
    CancelQuery,
    AskAi,
}

pub struct SqlEditorToolbar<'a> {
    is_running: bool,
    has_selection: bool,
    theme: DbProTheme,
    _marker: std::marker::PhantomData<&'a ()>,
}

impl<'a> SqlEditorToolbar<'a> {
    pub fn new(is_running: bool, has_selection: bool, theme: DbProTheme) -> Self {
        Self {
            is_running,
            has_selection,
            theme,
            _marker: std::marker::PhantomData,
        }
    }

    pub fn show(self, ui: &mut Ui) -> Option<SqlEditorAction> {
        let mut triggered = None;

        let frame = egui::Frame::none()
            .fill(self.theme.surface_panel)
            .stroke(Stroke::new(STROKE_THIN, self.theme.border_subtle))
            .rounding(Rounding {
                nw: RADIUS_MD,
                ne: RADIUS_MD,
                sw: 0.0,
                se: 0.0,
            })
            .inner_margin(egui::Margin::symmetric(SPACE_MD, SPACE_SM));

        frame.show(ui, |ui| {
            ui.horizontal(|ui| {
                if self.is_running {
                    if Button::new(self.theme)
                        .text("Cancel")
                        .icon(Icon::Square)
                        .variant(ButtonVariant::Destructive)
                        .size(ButtonSize::Sm)
                        .show(ui)
                        .clicked()
                    {
                        triggered = Some(SqlEditorAction::CancelQuery);
                    }
                } else {
                    if Button::new(self.theme)
                        .text("Run")
                        .icon(Icon::Play)
                        .variant(ButtonVariant::Default)
                        .size(ButtonSize::Sm)
                        .show(ui)
                        .clicked()
                    {
                        triggered = Some(SqlEditorAction::RunQuery);
                    }

                    if self.has_selection {
                        ui.add_space(SPACE_XXS);
                        if Button::new(self.theme)
                            .text("Run Selection")
                            .variant(ButtonVariant::Outline)
                            .size(ButtonSize::Sm)
                            .show(ui)
                            .clicked()
                        {
                            triggered = Some(SqlEditorAction::RunSelection);
                        }
                    }

                    ui.add_space(SPACE_XXS);
                    if Button::new(self.theme)
                        .text("Explain")
                        .icon(Icon::ChartNoAxesCombined)
                        .variant(ButtonVariant::Outline)
                        .size(ButtonSize::Sm)
                        .show(ui)
                        .clicked()
                    {
                        triggered = Some(SqlEditorAction::ExplainQuery);
                    }

                    ui.add_space(SPACE_XXS);
                    if Button::new(self.theme)
                        .text("Format")
                        .icon(Icon::AlignLeft)
                        .variant(ButtonVariant::Ghost)
                        .size(ButtonSize::Sm)
                        .show(ui)
                        .clicked()
                    {
                        triggered = Some(SqlEditorAction::FormatSql);
                    }
                }

                // AI Action trigger on the right
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if Button::new(self.theme)
                        .text("Ask AI")
                        .icon(Icon::Sparkles)
                        .variant(ButtonVariant::Outline)
                        .size(ButtonSize::Sm)
                        .show(ui)
                        .clicked()
                    {
                        triggered = Some(SqlEditorAction::AskAi);
                    }
                });
            });
        });

        triggered
    }
}
