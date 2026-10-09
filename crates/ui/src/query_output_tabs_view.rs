//! Query output tab chrome and per-document output selection.
use super::*;
use egui::RichText;
use lucide_icons::Icon;

/// Minimum strip width that still fits every labelled tab plus the chrome
/// buttons; below this the tabs render icon-only.
const TAB_STRIP_LABEL_MIN_WIDTH: f32 = 520.0;

pub(super) struct QueryOutputTabsContext<'a> {
    pub(super) theme: DbProTheme,
    pub(super) output: &'a mut QueryOutputState,
    pub(super) session: &'a QuerySessionState,
    pub(super) editor: &'a mut QueryEditorState,
    pub(super) bottom_panel_open: &'a mut bool,
    pub(super) dock_position: Option<&'a mut OutputDockPosition>,
    pub(super) results_open: Option<&'a mut bool>,
    pub(super) active_tab: Option<&'a mut WorkspaceTab>,
}

/// Output tab strip. When `dock_chrome` is true, close/maximize and dock position toggle sit on the same row.
pub(super) fn draw_output_tabs(context: &mut QueryOutputTabsContext<'_>, ui: &mut egui::Ui, dock_chrome: bool) {
    ui.add_space(SPACE_XS);
    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
        if dock_chrome {
            if let (Some(results_open), Some(active_tab)) = (
                context.results_open.as_deref_mut(),
                context.active_tab.as_deref_mut(),
            ) {
                if compact_icon_button(ui, Icon::ExternalLink, context.theme)
                    .on_hover_text("Open results in a workspace tab")
                    .clicked()
                {
                    *results_open = true;
                    *active_tab = WorkspaceTab::Results;
                }
            }
            if Button::new(context.theme)
                .icon(Icon::X)
                .variant(ButtonVariant::Ghost)
                .size(ButtonSize::IconSm)
                .access_label("Close output")
                .tooltip("Close output")
                .show(ui)
                .clicked()
            {
                *context.bottom_panel_open = false;
                context.editor.query_output_dock_maximized = false;
            }
            // Maximize only exists in the docked surfaces — the plain bottom
            // panel resizes by drag and has nothing to maximize into.
            if context.dock_position.is_some() {
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
                    .access_label(max_tip)
                    .tooltip(max_tip)
                    .show(ui)
                    .clicked()
                {
                    context.editor.query_output_dock_maximized = !context.editor.query_output_dock_maximized;
                }
            }
            if let Some(pos) = context.dock_position.as_deref_mut() {
                let (toggle_tip, toggle_icon) = match *pos {
                    OutputDockPosition::Bottom => ("Dock to right", Icon::PanelRight),
                    OutputDockPosition::Right => ("Dock to bottom", Icon::PanelBottom),
                };
                if Button::new(context.theme)
                    .icon(toggle_icon)
                    .variant(ButtonVariant::Ghost)
                    .size(ButtonSize::IconSm)
                    .access_label(toggle_tip)
                    .tooltip(toggle_tip)
                    .show(ui)
                    .clicked()
                {
                    *pos = match *pos {
                        OutputDockPosition::Bottom => OutputDockPosition::Right,
                        OutputDockPosition::Right => OutputDockPosition::Bottom,
                    };
                    context.editor.query_output_dock_maximized = false;
                }
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
            // A docked-right panel is typically ~400px wide: full labels + the
            // chrome buttons do not fit, and egui draws overflowing widgets on
            // top of each other. Fall back to icon-only tabs with the label in
            // the tooltip when the strip gets narrow.
            let icon_only = ui.available_width() < TAB_STRIP_LABEL_MIN_WIDTH;
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

                let response = egui::Frame::NONE
                    .fill(bg_color)
                    .corner_radius(egui::CornerRadius::same(RADIUS_SM as u8))
                    .inner_margin(egui::Margin::symmetric(SPACE_SM as i8, SPACE_XS as i8))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(char::from(icon).to_string())
                                    .font(egui::FontId::new(12.0, egui::FontFamily::Name("lucide".into())))
                                    .color(icon_color),
                            );
                            if !icon_only {
                                ui.add_space(2.0);
                                ui.label(
                                    RichText::new(label).font(font_ui_label()).color(text_color),
                                );
                            }
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

                let response = response.response.interact(egui::Sense::click());
                response.widget_info(|| {
                    egui::WidgetInfo::selected(
                        egui::WidgetType::SelectableLabel,
                        ui.is_enabled(),
                        selected,
                        label,
                    )
                });
                let response = if icon_only {
                    response.on_hover_text(label)
                } else {
                    response
                };
                if response.clicked() {
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
