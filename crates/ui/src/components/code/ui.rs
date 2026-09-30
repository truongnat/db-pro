use crate::components::interact::paint_focus_ring;
use crate::tokens::{RADIUS_CODE_BLOCK, RADIUS_XS, STROKE_THIN};
use crate::DbProTheme;
use egui::{
    Align2, Color32, FontFamily, FontId, Pos2, Response, Rounding, Sense, Stroke, Ui, Vec2, WidgetInfo, WidgetType,
};
use std::time::Duration;

use super::config::{
    CODE_BLOCK_BODY_PADDING_Y, CODE_BLOCK_CODE_FONT_SIZE, CODE_BLOCK_COPIED_FEEDBACK_SECS, CODE_BLOCK_COPY_BTN_SIZE,
    CODE_BLOCK_COPY_ICON_SIZE, CODE_BLOCK_COPY_LABEL_SIZE, CODE_BLOCK_CORNER_RADIUS, CODE_BLOCK_HEADER_HEIGHT,
    CODE_BLOCK_LANG_FONT_SIZE, CODE_BLOCK_LINE_HEIGHT, CODE_BLOCK_LINE_NUM_FONT_SIZE, INLINE_CODE_FONT_SIZE,
};
use super::handler::{
    calculate_gutter_width, code_body_content_width, copy_button_rect, copy_button_state, copy_feedback_remaining,
    format_line_number, header_language_clip_rect, inline_code_size, inline_code_text_pos, is_copy_active,
};

/// Clean inline monospace code snippet.
pub struct InlineCode<'a> {
    text: &'a str,
    theme: DbProTheme,
}

impl<'a> InlineCode<'a> {
    pub fn new(text: &'a str, theme: DbProTheme) -> Self {
        Self { text, theme }
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        let font_id = FontId::monospace(INLINE_CODE_FONT_SIZE);
        let galley = ui
            .painter()
            .layout_no_wrap(self.text.to_owned(), font_id, self.theme.text_primary);
        let size = inline_code_size(galley.size());

        let (rect, response) = ui.allocate_exact_size(size, Sense::hover());
        response.widget_info(|| WidgetInfo::labeled(WidgetType::Other, true, self.text));

        ui.painter()
            .rect_filled(rect, Rounding::same(RADIUS_XS), self.theme.surface_hover);
        ui.painter().rect_stroke(
            rect,
            Rounding::same(RADIUS_XS),
            Stroke::new(STROKE_THIN, self.theme.border_subtle),
        );

        let text_pos = inline_code_text_pos(rect.min);
        ui.painter().galley(text_pos, galley, Color32::PLACEHOLDER);

        response
    }
}

/// Developer-oriented CodeBlock with language badge, copy action, and line numbers.
pub struct CodeBlock<'a> {
    code: &'a str,
    language: Option<&'a str>,
    show_line_numbers: bool,
    theme: DbProTheme,
}

impl<'a> CodeBlock<'a> {
    pub fn new(code: &'a str, theme: DbProTheme) -> Self {
        Self {
            code,
            language: None,
            show_line_numbers: true,
            theme,
        }
    }

    pub fn language(mut self, lang: &'a str) -> Self {
        self.language = Some(lang);
        self
    }

    pub fn show_line_numbers(mut self, show: bool) -> Self {
        self.show_line_numbers = show;
        self
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        let block_id = ui.id().with("code_block");
        let copied_key = block_id.with("copied_time");
        let copied_time = ui.data(|d| d.get_temp::<f64>(copied_key));
        let current_time = ui.input(|i| i.time);
        let is_recently_copied = is_copy_active(copied_time, current_time, CODE_BLOCK_COPIED_FEEDBACK_SECS);
        if let Some(remaining) = copy_feedback_remaining(copied_time, current_time, CODE_BLOCK_COPIED_FEEDBACK_SECS) {
            ui.ctx().request_repaint_after(Duration::from_secs_f64(remaining));
        }

        let frame = egui::Frame::none()
            .fill(self.theme.surface_editor)
            .stroke(Stroke::new(STROKE_THIN, self.theme.border_default))
            .rounding(Rounding::same(CODE_BLOCK_CORNER_RADIUS))
            .inner_margin(egui::Margin::same(0.0));

        let resp = frame
            .show(ui, |ui| {
                ui.set_width(ui.available_width());

                // ── Header Bar ──────────────────────────────────────────────
                let header_rect = ui
                    .allocate_space(Vec2::new(ui.available_width(), CODE_BLOCK_HEADER_HEIGHT))
                    .1;
                ui.painter().rect_filled(
                    header_rect,
                    Rounding {
                        nw: CODE_BLOCK_CORNER_RADIUS,
                        ne: CODE_BLOCK_CORNER_RADIUS,
                        sw: 0.0,
                        se: 0.0,
                    },
                    self.theme.surface_panel,
                );
                ui.painter().hline(
                    header_rect.x_range(),
                    header_rect.bottom(),
                    Stroke::new(STROKE_THIN, self.theme.border_subtle),
                );

                // Copy button on top right
                let btn_state = copy_button_state(is_recently_copied, &self.theme);
                let copy_btn_rect = copy_button_rect(header_rect, CODE_BLOCK_COPY_BTN_SIZE);

                // Language label, clipped so narrow headers never paint under the copy control.
                let lang_str = self.language.unwrap_or("SQL");
                let lang_galley = ui.painter().layout_no_wrap(
                    lang_str.to_uppercase(),
                    FontId::monospace(CODE_BLOCK_LANG_FONT_SIZE),
                    self.theme.text_secondary,
                );
                ui.painter()
                    .with_clip_rect(header_language_clip_rect(header_rect, copy_btn_rect))
                    .galley(
                        Pos2::new(header_rect.left() + 12.0, header_rect.center().y - 6.0),
                        lang_galley,
                        Color32::PLACEHOLDER,
                    );

                let copy_resp = ui.put(copy_btn_rect, egui::Button::new("").frame(false));
                copy_resp.widget_info(|| WidgetInfo::labeled(WidgetType::Button, true, btn_state.label));

                if copy_resp.hovered() || copy_resp.has_focus() {
                    ui.painter().rect_filled(
                        copy_btn_rect,
                        Rounding::same(RADIUS_CODE_BLOCK),
                        self.theme.surface_hover,
                    );
                }
                if copy_resp.has_focus() {
                    paint_focus_ring(ui, copy_btn_rect, RADIUS_CODE_BLOCK, self.theme);
                }

                // Icon
                let icon_x = copy_btn_rect.left() + 8.0;
                ui.painter().text(
                    Pos2::new(icon_x, copy_btn_rect.center().y),
                    Align2::LEFT_CENTER,
                    char::from(btn_state.icon).to_string(),
                    FontId::new(CODE_BLOCK_COPY_ICON_SIZE, FontFamily::Name("lucide".into())),
                    btn_state.color,
                );

                // Label
                let label_x = icon_x + 16.0;
                ui.painter().text(
                    Pos2::new(label_x, copy_btn_rect.center().y),
                    Align2::LEFT_CENTER,
                    btn_state.label,
                    FontId::proportional(CODE_BLOCK_COPY_LABEL_SIZE),
                    btn_state.color,
                );

                if copy_resp.clicked() {
                    ui.output_mut(|o| o.copied_text = self.code.to_owned());
                    ui.data_mut(|d| d.insert_temp(copied_key, ui.input(|i| i.time)));
                    ui.ctx()
                        .request_repaint_after(Duration::from_secs_f64(CODE_BLOCK_COPIED_FEEDBACK_SECS));
                }

                // ── Code Lines ──────────────────────────────────────────────
                ui.add_space(CODE_BLOCK_BODY_PADDING_Y);
                let lines: Vec<&str> = self.code.lines().collect();
                let display_lines: Vec<&str> = if lines.is_empty() { vec![""] } else { lines };
                let line_count = display_lines.len().max(1);
                let gutter_w = calculate_gutter_width(self.show_line_numbers, line_count);
                let code_font_id = FontId::monospace(CODE_BLOCK_CODE_FONT_SIZE);
                let code_galleys: Vec<_> = display_lines
                    .iter()
                    .map(|line| {
                        ui.painter()
                            .layout_no_wrap((*line).to_owned(), code_font_id.clone(), self.theme.text_primary)
                    })
                    .collect();
                let content_width = code_body_content_width(
                    code_galleys.iter().map(|galley| galley.size().x),
                    gutter_w,
                    ui.available_width(),
                );

                egui::ScrollArea::horizontal()
                    .id_salt(block_id.with("code_scroll"))
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        for (idx, code_galley) in code_galleys.into_iter().enumerate() {
                            let line_num = idx + 1;
                            let (row_rect, _) = ui
                                .allocate_exact_size(Vec2::new(content_width, CODE_BLOCK_LINE_HEIGHT), Sense::hover());

                            if self.show_line_numbers {
                                let num_galley = ui.painter().layout_no_wrap(
                                    format_line_number(line_num, line_count),
                                    FontId::monospace(CODE_BLOCK_LINE_NUM_FONT_SIZE),
                                    self.theme.text_tertiary,
                                );
                                ui.painter().galley(
                                    Pos2::new(row_rect.left() + 10.0, row_rect.top() + 2.0),
                                    num_galley,
                                    Color32::PLACEHOLDER,
                                );
                            }

                            ui.painter().galley(
                                Pos2::new(row_rect.left() + gutter_w + 4.0, row_rect.top() + 2.0),
                                code_galley,
                                Color32::PLACEHOLDER,
                            );
                        }
                    });

                ui.add_space(CODE_BLOCK_BODY_PADDING_Y);
            })
            .response;

        resp
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DbProTheme;

    #[test]
    fn inline_code_renders_accessible_response() {
        let theme = DbProTheme::light();
        let ctx = egui::Context::default();
        DbProTheme::install_fonts(&ctx);
        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let resp = InlineCode::new("SELECT 1;", theme).show(ui);
                assert!(resp.rect.width() > 0.0);
                assert!(resp.rect.height() > 0.0);
            });
        });
    }

    #[test]
    fn code_block_renders_with_and_without_line_numbers() {
        let theme = DbProTheme::dark();
        let ctx = egui::Context::default();
        DbProTheme::install_fonts(&ctx);
        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let resp1 = CodeBlock::new("SELECT * FROM users;\nWHERE id = 1;", theme)
                    .language("sql")
                    .show(ui);
                assert!(resp1.rect.height() > 32.0);

                let resp2 = CodeBlock::new("const x = 42;", theme)
                    .language("ts")
                    .show_line_numbers(false)
                    .show(ui);
                assert!(resp2.rect.height() > 32.0);
            });
        });
    }
}
