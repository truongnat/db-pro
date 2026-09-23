//! Query output tab chrome and per-document output selection.
use super::*;
use egui::RichText;
use lucide_icons::Icon;

pub(super) struct QueryOutputTabsContext<'a> {
    pub(super) theme: DbProTheme,
    pub(super) output: &'a mut QueryOutputState,
    pub(super) session: &'a QuerySessionState,
    pub(super) editor: &'a mut QueryEditorState,
    pub(super) bottom_panel_open: &'a mut bool,
}

/// Output tab strip. When `dock_chrome` is true, close/maximize sit on the same row.
pub(super) fn draw_output_tabs(context: &mut QueryOutputTabsContext<'_>, ui: &mut egui::Ui, dock_chrome: bool) {
    ui.add_space(SPACE_XS);
    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
        if dock_chrome {
            if Button::new(context.theme)
                .icon(Icon::X)
                .variant(ButtonVariant::Ghost)
                .size(ButtonSize::IconSm)
                .tooltip("Close output")
                .show(ui)
                .clicked()
            {
                *context.bottom_panel_open = false;
                context.editor.query_output_dock_maximized = false;
            }
            let max_tip = if context.editor.query_output_dock_maximized {
                "Restore output"
            } else {
                "Maximize output"
            };
            let max_icon = if context.editor.query_output_dock_maximized {
                Icon::Minimize2
            } else {
                Icon::Maximize2
            };
            if Button::new(context.theme)
                .icon(max_icon)
                .variant(ButtonVariant::Ghost)
                .size(ButtonSize::IconSm)
                .tooltip(max_tip)
                .show(ui)
                .clicked()
            {
                context.editor.query_output_dock_maximized = !context.editor.query_output_dock_maximized;
            }
        }
        if let Some(request_id) = context.session.active_explain_request() {
            ui.label(
                RichText::new(format!("Explain request {}…", request_id.0))
                    .font(font_caption())
                    .color(context.theme.text_muted),
            );
        }

        ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
            ui.spacing_mut().item_spacing = egui::vec2(4.0, 0.0);
            for (tab, icon, label) in [
                (OutputTab::Results, Icon::Table2, "Results"),
                (OutputTab::Chart, Icon::BarChart3, "Chart"),
                (OutputTab::Messages, Icon::MessageSquareText, "Messages"),
                (OutputTab::Explain, Icon::ChartNoAxesCombined, "Explain"),
                (OutputTab::History, Icon::History, "History"),
            ] {
                let selected = context
                    .output
                    .active_tab_for_document(context.session.active_document().map(|document| document.id.as_str()))
                    == tab;
                let bg_color = if selected {
                    context.theme.surface_active
                } else {
                    egui::Color32::TRANSPARENT
                };
                let text_color = if selected {
                    context.theme.text_primary
                } else {
                    context.theme.text_secondary
                };
                let icon_color = if selected {
                    context.theme.accent
                } else {
                    context.theme.text_muted
                };

                let response = egui::Frame::none()
                    .fill(bg_color)
                    .rounding(egui::Rounding::same(RADIUS_SM))
                    .inner_margin(egui::Margin::symmetric(SPACE_SM, SPACE_XS))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(char::from(icon).to_string())
                                    .font(egui::FontId::new(12.0, egui::FontFamily::Name("lucide".into())))
                                    .color(icon_color),
                            );
                            ui.add_space(2.0);
                            ui.label(RichText::new(label).font(font_ui_label()).color(text_color));
                            if tab == OutputTab::Results {
                                if let Some(result) = context.session.active_result() {
                                    badge(
                                        ui,
                                        &result.row_count.to_string(),
                                        context.theme.accent_soft,
                                        context.theme.accent,
                                    );
                                }
                            } else if tab == OutputTab::Messages && !context.session.active_messages().is_empty() {
                                badge(
                                    ui,
                                    &context.session.active_messages().len().to_string(),
                                    context.theme.surface_hover,
                                    context.theme.text_muted,
                                );
                            }
                        });
                    });

                if response.response.interact(egui::Sense::click()).clicked() {
                    context.output.set_active_for_optional_document(
                        context.session.active_document().map(|document| document.id.as_str()),
                        tab,
                    );
                }
            }
        });
    });
    ui.add_space(SPACE_XS);
}
