// cc-scan:allow-file HUGE_FUNCTION,LONG_FUNCTION,HARD_COMPLEXITY,COMPLEXITY,DEEP_NESTING
// cc-scan:allow-file HARD_PARAMS,TOO_MANY_PARAMS,DUPLICATE_BLOCK
// egui painter/view file: fns are linear layout code; branches are per-state paint variants.
use crate::tokens::{font_icon, RADIUS_MD, RADIUS_SM};
use crate::DbProTheme;
use egui::{
    Align2, Color32, CornerRadius, FontId, Pos2, Rect, Response, RichText, Sense, Stroke, Ui, Vec2, WidgetInfo,
    WidgetType,
};
use lucide_icons::Icon;

use super::{config, handler, ContextChipKind, StatusBadgeVariant, ToolCallStatus};

pub struct ContextChip<'a> {
    kind: ContextChipKind,
    label: &'a str,
    removable: bool,
    theme: DbProTheme,
}

impl<'a> ContextChip<'a> {
    pub fn new(kind: ContextChipKind, label: &'a str, theme: DbProTheme) -> Self {
        Self {
            kind,
            label,
            removable: false,
            theme,
        }
    }

    pub fn removable(mut self, removable: bool) -> Self {
        self.removable = removable;
        self
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        let text_font = FontId::proportional(11.5);
        let icon_font = font_icon(config::ICON_CONTEXT_SIZE);
        let icon_char = char::from(self.kind.icon()).to_string();
        let icon_galley = ui
            .painter()
            .layout_no_wrap(icon_char, icon_font, self.theme.text_secondary);
        let text_galley = ui
            .painter()
            .layout_no_wrap(self.label.to_owned(), text_font, self.theme.text_primary);
        let icon_size = icon_galley.size();
        let text_size = text_galley.size();
        let size = handler::context_chip_size(icon_size, text_size, self.removable);

        let (rect, response) = ui.allocate_exact_size(size, Sense::hover());
        let is_hovered = response.hovered();
        let fill = if is_hovered {
            self.theme.surface_hover
        } else {
            self.theme.surface_editor
        };
        let stroke = if is_hovered {
            Stroke::new(1.0, self.theme.border_strong)
        } else {
            Stroke::new(1.0, self.theme.border_default)
        };

        ui.painter()
            .rect_filled(rect, CornerRadius::same(config::CHIP_RADIUS as u8), fill);
        ui.painter().rect_stroke(
            rect,
            CornerRadius::same(config::CHIP_RADIUS as u8),
            stroke,
            egui::StrokeKind::Inside,
        );
        ui.painter().galley(
            handler::context_chip_icon_pos(rect, icon_size),
            icon_galley,
            Color32::PLACEHOLDER,
        );
        ui.painter().galley(
            handler::context_chip_text_pos(rect, icon_size, text_size),
            text_galley,
            Color32::PLACEHOLDER,
        );

        if !self.removable {
            let chip_response = ui.interact(rect, response.id, Sense::click());
            chip_response.widget_info(|| WidgetInfo::labeled(WidgetType::Button, true, self.label));
            return chip_response;
        }

        // A removable chip exposes the close glyph as the sole click target so its
        // returned Response is an unambiguous removal request for the caller.
        let remove_rect = handler::context_chip_remove_rect(rect);
        let remove_label = format!("Remove context: {}", self.label);
        let remove_response = ui
            .interact(remove_rect, response.id.with("remove"), Sense::click())
            .on_hover_text(&remove_label);
        remove_response.widget_info(|| WidgetInfo::labeled(WidgetType::Button, true, &remove_label));
        let remove_color = if remove_response.hovered() {
            self.theme.text_primary
        } else {
            self.theme.text_tertiary
        };
        ui.painter().text(
            remove_rect.center(),
            Align2::CENTER_CENTER,
            char::from(Icon::X).to_string(),
            font_icon(10.5),
            remove_color,
        );

        remove_response
    }
}

pub struct StatusBadge<'a> {
    text: &'a str,
    variant: StatusBadgeVariant,
    theme: DbProTheme,
}

impl<'a> StatusBadge<'a> {
    pub fn new(text: &'a str, variant: StatusBadgeVariant, theme: DbProTheme) -> Self {
        Self { text, variant, theme }
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        let palette = handler::status_badge_palette(self.theme, self.variant);
        let text_galley =
            ui.painter()
                .layout_no_wrap(self.text.to_owned(), DbProTheme::ui_medium_font(12.0), palette.text);
        let size = handler::status_badge_size(text_galley.size());
        let (rect, response) = ui.allocate_exact_size(size, Sense::hover());

        ui.painter().rect_filled(
            rect,
            CornerRadius::same((rect.height() * 0.5) as u8),
            palette.background,
        );
        ui.painter().rect_stroke(
            rect,
            CornerRadius::same((rect.height() * 0.5) as u8),
            Stroke::new(1.0, palette.dot.linear_multiply(0.35)),
            egui::StrokeKind::Inside,
        );
        ui.painter().circle_filled(
            handler::status_badge_dot_pos(rect),
            config::STATUS_DOT_RADIUS,
            palette.dot,
        );
        ui.painter().galley(
            handler::status_badge_text_pos(rect, text_galley.size()),
            text_galley,
            Color32::PLACEHOLDER,
        );

        response.widget_info(|| WidgetInfo::labeled(WidgetType::Label, true, self.text));
        response
    }
}

pub struct ToolCall<'a> {
    tool_name: &'a str,
    duration: Option<&'a str>,
    status: ToolCallStatus,
    input_preview: &'a str,
    output_preview: Option<&'a str>,
    expanded: &'a mut bool,
    theme: DbProTheme,
}

impl<'a> ToolCall<'a> {
    pub fn new(
        tool_name: &'a str,
        status: ToolCallStatus,
        input_preview: &'a str,
        expanded: &'a mut bool,
        theme: DbProTheme,
    ) -> Self {
        Self {
            tool_name,
            duration: None,
            status,
            input_preview,
            output_preview: None,
            expanded,
            theme,
        }
    }

    pub fn duration(mut self, dur: &'a str) -> Self {
        self.duration = Some(dur);
        self
    }

    pub fn output_preview(mut self, out: &'a str) -> Self {
        self.output_preview = Some(out);
        self
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        let is_expanded = *self.expanded;
        let frame = egui::Frame::NONE
            .fill(self.theme.surface_panel)
            .stroke(Stroke::new(config::FRAME_BORDER_WIDTH, self.theme.border_default))
            .corner_radius(CornerRadius::same(RADIUS_MD as u8))
            .inner_margin(egui::Margin::same(0.0 as i8));

        frame
            .show(ui, |ui| {
                ui.set_width(ui.available_width());

                // The header first captures one full-width click target. The handler owns
                // the expansion transition, then the UI paints the old or new body state.
                let (header_rect, header_resp) = ui.allocate_exact_size(
                    Vec2::new(ui.available_width(), config::TOOL_HEADER_HEIGHT),
                    Sense::click(),
                );
                let keyboard_toggle = if header_resp.has_focus() {
                    ui.input_mut(|input| {
                        [egui::Key::Space, egui::Key::Enter]
                            .into_iter()
                            .any(|key| input.consume_key(egui::Modifiers::NONE, key))
                    })
                } else {
                    false
                };
                header_resp.widget_info(|| {
                    WidgetInfo::selected(WidgetType::CollapsingHeader, true, is_expanded, self.tool_name)
                });
                if header_resp.has_focus() {
                    ui.painter().rect_stroke(
                        header_rect,
                        CornerRadius::same(RADIUS_MD as u8),
                        Stroke::new(1.5, self.theme.border_strong),
                        egui::StrokeKind::Inside,
                    );
                }
                if header_resp.hovered() {
                    ui.painter().rect_filled(
                        header_rect,
                        CornerRadius::same(RADIUS_MD as u8),
                        self.theme.surface_hover,
                    );
                }

                let chevron_icon = if is_expanded {
                    Icon::ChevronDown
                } else {
                    Icon::ChevronRight
                };
                ui.painter().text(
                    Pos2::new(
                        header_rect.left() + config::TOOL_HEADER_ICON_OFFSET_X,
                        header_rect.center().y,
                    ),
                    Align2::CENTER_CENTER,
                    char::from(chevron_icon).to_string(),
                    font_icon(config::ICON_TOOL_CHEVRON_SIZE),
                    self.theme.text_secondary,
                );
                ui.painter().text(
                    Pos2::new(
                        header_rect.left() + config::TOOL_NAME_ICON_OFFSET_X,
                        header_rect.center().y,
                    ),
                    Align2::CENTER_CENTER,
                    char::from(Icon::Terminal).to_string(),
                    font_icon(config::ICON_TOOL_SIZE),
                    self.theme.text_secondary,
                );
                let status_text = handler::tool_status_label(self.status);
                let status_galley = ui.painter().layout_no_wrap(
                    status_text.to_owned(),
                    FontId::proportional(10.5),
                    self.theme.text_secondary,
                );
                let status_rect = handler::tool_badge_rect(header_rect, handler::tool_badge_size(status_galley.size()));
                ui.painter().rect_filled(
                    status_rect,
                    CornerRadius::same(config::TOOL_BADGE_RADIUS as u8),
                    handler::tool_status_fill(self.theme, self.status),
                );
                ui.painter().galley(
                    handler::tool_badge_text_pos(status_rect),
                    status_galley,
                    Color32::PLACEHOLDER,
                );

                let mut right_cursor = status_rect.left();
                if let Some(duration) = self.duration {
                    let duration_galley = ui.painter().layout_no_wrap(
                        duration.to_owned(),
                        FontId::monospace(11.0),
                        self.theme.text_tertiary,
                    );
                    let position = handler::tool_duration_pos(header_rect, &mut right_cursor, duration_galley.size());
                    ui.painter().galley(position, duration_galley, Color32::PLACEHOLDER);
                }
                let title_left = header_rect.left() + config::TOOL_NAME_TEXT_OFFSET_X;
                let title_right = (right_cursor - config::TOOL_BADGE_HORIZONTAL_PADDING).max(title_left);
                let title_rect = Rect::from_min_max(
                    Pos2::new(title_left, header_rect.top()),
                    Pos2::new(title_right, header_rect.bottom()),
                );
                ui.painter().with_clip_rect(title_rect).text(
                    Pos2::new(title_rect.left(), header_rect.center().y),
                    Align2::LEFT_CENTER,
                    self.tool_name,
                    FontId::monospace(12.0),
                    self.theme.text_primary,
                );
                header_resp.clone().on_hover_text(self.tool_name);

                let should_toggle =
                    handler::disclosure_activation(header_resp.clicked(), header_resp.has_focus(), keyboard_toggle);
                handler::toggle_expanded(self.expanded, should_toggle);

                if is_expanded {
                    ui.painter().hline(
                        header_rect.x_range(),
                        header_rect.bottom(),
                        Stroke::new(config::FRAME_BORDER_WIDTH, self.theme.border_subtle),
                    );
                    ui.add_space(config::TOOL_SECTION_TOP_SPACE);
                    ui.horizontal(|ui| {
                        ui.add_space(config::TOOL_SECTION_INSET);
                        ui.vertical(|ui| {
                            ui.label(
                                RichText::new("INPUT PARAMETERS")
                                    .size(config::TEXT_LABEL_SIZE)
                                    .strong()
                                    .color(self.theme.text_tertiary),
                            );
                            ui.add_space(config::TOOL_SECTION_LABEL_GAP);
                            let code_frame = egui::Frame::NONE
                                .fill(self.theme.surface_editor)
                                .corner_radius(CornerRadius::same(RADIUS_SM as u8))
                                .stroke(Stroke::new(config::FRAME_BORDER_WIDTH, self.theme.border_subtle))
                                .inner_margin(egui::Margin::same(config::TOOL_CODE_PADDING as i8));
                            code_frame.show(ui, |ui| {
                                ui.set_width(ui.available_width() - config::TOOL_CODE_HORIZONTAL_INSET);
                                ui.label(
                                    RichText::new(self.input_preview)
                                        .monospace()
                                        .size(config::TEXT_BODY_MONO_SIZE)
                                        .color(self.theme.text_primary),
                                );
                            });

                            if let Some(output) = self.output_preview {
                                ui.add_space(config::TOOL_OUTPUT_GAP);
                                ui.label(
                                    RichText::new("OUTPUT RESULT")
                                        .size(config::TEXT_LABEL_SIZE)
                                        .strong()
                                        .color(self.theme.text_tertiary),
                                );
                                ui.add_space(config::TOOL_SECTION_LABEL_GAP);
                                let output_frame = egui::Frame::NONE
                                    .fill(self.theme.surface_editor)
                                    .corner_radius(CornerRadius::same(RADIUS_SM as u8))
                                    .stroke(Stroke::new(config::FRAME_BORDER_WIDTH, self.theme.border_subtle))
                                    .inner_margin(egui::Margin::same(config::TOOL_CODE_PADDING as i8));
                                output_frame.show(ui, |ui| {
                                    ui.set_width(ui.available_width() - config::TOOL_CODE_HORIZONTAL_INSET);
                                    ui.label(
                                        RichText::new(output)
                                            .monospace()
                                            .size(config::TEXT_BODY_MONO_SIZE)
                                            .color(self.theme.text_secondary),
                                    );
                                });
                            }
                        });
                        ui.add_space(config::TOOL_SECTION_INSET);
                    });
                    ui.add_space(config::TOOL_SECTION_BOTTOM_SPACE);
                }
            })
            .response
    }
}
