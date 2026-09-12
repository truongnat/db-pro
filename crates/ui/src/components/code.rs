use crate::DbProTheme;
use egui::{Align2, Color32, FontFamily, FontId, Pos2, Rect, Response, Rounding, Sense, Stroke, Ui, Vec2};
use lucide_icons::Icon;

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
        let font_id = FontId::monospace(12.0);
        let galley = ui
            .painter()
            .layout_no_wrap(self.text.to_owned(), font_id, self.theme.text_primary);
        let padding = Vec2::new(6.0, 2.0);
        let size = galley.size() + padding * 2.0;

        let (rect, response) = ui.allocate_exact_size(size, Sense::hover());
        ui.painter()
            .rect_filled(rect, Rounding::same(4.0), self.theme.surface_hover);
        ui.painter()
            .rect_stroke(rect, Rounding::same(4.0), Stroke::new(1.0, self.theme.border_subtle));

        let text_pos = Pos2::new(rect.left() + padding.x, rect.top() + padding.y);
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
        let is_recently_copied = ui.data(|d| {
            d.get_temp::<f64>(copied_key)
                .map(|t| ui.input(|i| i.time) - t < 2.0)
                .unwrap_or(false)
        });

        let frame = egui::Frame::none()
            .fill(self.theme.surface_editor)
            .stroke(Stroke::new(1.0, self.theme.border_default))
            .rounding(Rounding::same(8.0))
            .inner_margin(egui::Margin::same(0.0));

        frame
            .show(ui, |ui| {
                ui.set_width(ui.available_width());

                // ── Header Bar ──────────────────────────────────────────────
                let header_rect = ui.allocate_space(Vec2::new(ui.available_width(), 32.0)).1;
                ui.painter().rect_filled(
                    header_rect,
                    Rounding {
                        nw: 8.0,
                        ne: 8.0,
                        sw: 0.0,
                        se: 0.0,
                    },
                    self.theme.surface_panel,
                );
                ui.painter().hline(
                    header_rect.x_range(),
                    header_rect.bottom(),
                    Stroke::new(1.0, self.theme.border_subtle),
                );

                // Language label
                let lang_str = self.language.unwrap_or("SQL");
                let lang_galley = ui.painter().layout_no_wrap(
                    lang_str.to_uppercase(),
                    FontId::monospace(11.0),
                    self.theme.text_secondary,
                );
                ui.painter().galley(
                    Pos2::new(header_rect.left() + 12.0, header_rect.center().y - 6.0),
                    lang_galley,
                    Color32::PLACEHOLDER,
                );

                // Copy button on top right
                let copy_icon = if is_recently_copied { Icon::Check } else { Icon::Copy };
                let copy_label = if is_recently_copied { "Copied" } else { "Copy" };
                let copy_color = if is_recently_copied {
                    self.theme.success
                } else {
                    self.theme.text_secondary
                };

                let copy_btn_rect = Rect::from_min_size(
                    Pos2::new(header_rect.right() - 80.0, header_rect.center().y - 11.0),
                    Vec2::new(68.0, 22.0),
                );
                let copy_resp = ui.interact(copy_btn_rect, block_id.with("copy_btn"), Sense::click());
                if copy_resp.hovered() {
                    ui.painter()
                        .rect_filled(copy_btn_rect, Rounding::same(4.0), self.theme.surface_hover);
                }

                // Icon
                let icon_x = copy_btn_rect.left() + 8.0;
                ui.painter().text(
                    Pos2::new(icon_x, copy_btn_rect.center().y),
                    Align2::LEFT_CENTER,
                    char::from(copy_icon).to_string(),
                    FontId::new(11.0, FontFamily::Name("lucide".into())),
                    copy_color,
                );

                // Label
                let label_x = icon_x + 16.0;
                ui.painter().text(
                    Pos2::new(label_x, copy_btn_rect.center().y),
                    Align2::LEFT_CENTER,
                    copy_label,
                    FontId::proportional(11.5),
                    copy_color,
                );

                if copy_resp.clicked() {
                    ui.output_mut(|o| o.copied_text = self.code.to_owned());
                    ui.data_mut(|d| d.insert_temp(copied_key, ui.input(|i| i.time)));
                }

                // ── Code Lines ──────────────────────────────────────────────
                ui.add_space(8.0);
                let lines: Vec<&str> = self.code.lines().collect();
                let line_count = lines.len().max(1);
                let gutter_w = if self.show_line_numbers {
                    format!("{}", line_count).len() as f32 * 8.0 + 20.0
                } else {
                    12.0
                };

                for (idx, line) in lines.iter().enumerate() {
                    let line_num = idx + 1;
                    let (row_rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 20.0), Sense::hover());

                    if self.show_line_numbers {
                        let num_galley = ui.painter().layout_no_wrap(
                            format!("{:>width$}", line_num, width = format!("{}", line_count).len()),
                            FontId::monospace(12.0),
                            self.theme.text_tertiary,
                        );
                        ui.painter().galley(
                            Pos2::new(row_rect.left() + 10.0, row_rect.top() + 2.0),
                            num_galley,
                            Color32::PLACEHOLDER,
                        );
                    }

                    let code_galley = ui.painter().layout_no_wrap(
                        (*line).to_owned(),
                        FontId::monospace(12.5),
                        self.theme.text_primary,
                    );
                    ui.painter().galley(
                        Pos2::new(row_rect.left() + gutter_w + 4.0, row_rect.top() + 2.0),
                        code_galley,
                        Color32::PLACEHOLDER,
                    );
                }

                ui.add_space(8.0);
            })
            .response
    }
}
