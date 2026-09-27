use crate::tokens::STROKE_THIN;
use crate::DbProTheme;
use egui::{Color32, FontId, Pos2, Response, Rounding, Sense, Stroke, Ui, Vec2, WidgetInfo, WidgetType};

use super::config::{
    DIFF_BOTTOM_PADDING_Y, DIFF_CONTENT_FONT_SIZE, DIFF_CONTENT_RIGHT_PADDING, DIFF_CORNER_RADIUS, DIFF_HEADER_HEIGHT,
    DIFF_LINE_HEIGHT, DIFF_LINE_NUM_FONT_SIZE, DIFF_MARKER_FONT_SIZE, DIFF_STATS_FONT_SIZE, DIFF_TITLE_FONT_SIZE,
};
use super::handler::{
    count_diff_changes, diff_geometry, diff_line_visual, format_diff_stats, format_line_num_col, DiffLine,
};

pub struct DiffViewer<'a> {
    title: &'a str,
    lines: &'a [DiffLine],
    theme: DbProTheme,
}

impl<'a> DiffViewer<'a> {
    pub fn new(title: &'a str, lines: &'a [DiffLine], theme: DbProTheme) -> Self {
        Self { title, lines, theme }
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        let frame = egui::Frame::none()
            .fill(self.theme.surface_editor)
            .stroke(Stroke::new(STROKE_THIN, self.theme.border_default))
            .rounding(Rounding::same(DIFF_CORNER_RADIUS))
            .inner_margin(egui::Margin::same(0.0));

        let (added_count, removed_count) = count_diff_changes(self.lines);
        let geometry = diff_geometry(self.lines);

        let response = frame
            .show(ui, |ui| {
                ui.set_width(ui.available_width());

                // ── Diff Header ─────────────────────────────────────────────
                let header_rect = ui.allocate_space(Vec2::new(ui.available_width(), DIFF_HEADER_HEIGHT)).1;
                ui.painter().rect_filled(
                    header_rect,
                    Rounding {
                        nw: DIFF_CORNER_RADIUS,
                        ne: DIFF_CORNER_RADIUS,
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

                // Added / Removed summary badges
                let stats_str = format_diff_stats(added_count, removed_count);
                let stats_galley = ui.painter().layout_no_wrap(
                    stats_str,
                    FontId::monospace(DIFF_STATS_FONT_SIZE),
                    self.theme.text_secondary,
                );
                let stats_pos = Pos2::new(
                    header_rect.right() - stats_galley.size().x - 14.0,
                    header_rect.center().y - 6.5,
                );

                // File / target title clips before the stats badge so narrow headers never overlap.
                let title_left = header_rect.left() + 12.0;
                let title_right = (stats_pos.x - 10.0).max(title_left);
                let title_clip = egui::Rect::from_min_max(
                    Pos2::new(title_left, header_rect.top()),
                    Pos2::new(title_right, header_rect.bottom()),
                );
                let title_galley = ui.painter().layout_no_wrap(
                    self.title.to_owned(),
                    FontId::proportional(DIFF_TITLE_FONT_SIZE),
                    self.theme.text_primary,
                );
                ui.painter().with_clip_rect(title_clip).galley(
                    Pos2::new(title_left, header_rect.center().y - 7.0),
                    title_galley,
                    Color32::PLACEHOLDER,
                );
                ui.painter()
                    .with_clip_rect(header_rect)
                    .galley(stats_pos, stats_galley, Color32::PLACEHOLDER);

                // Shape each line once; reuse the galleys for width measurement and painting.
                let line_galleys: Vec<_> = self
                    .lines
                    .iter()
                    .map(|line| {
                        ui.painter().layout_no_wrap(
                            line.content.clone(),
                            FontId::monospace(DIFF_CONTENT_FONT_SIZE),
                            self.theme.text_primary,
                        )
                    })
                    .collect();
                let content_width = diff_content_width(
                    line_galleys.iter().map(|galley| galley.size().x),
                    geometry.content_offset_x,
                );

                // ── Diff Lines ──────────────────────────────────────────────
                egui::ScrollArea::horizontal()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        let row_width = ui.available_width().max(content_width);
                        for (line, content_galley) in self.lines.iter().zip(&line_galleys) {
                            let (row_rect, row_resp) =
                                ui.allocate_exact_size(Vec2::new(row_width, DIFF_LINE_HEIGHT), Sense::hover());
                            row_resp.widget_info(|| WidgetInfo::labeled(WidgetType::Other, true, &line.content));

                            let visual = diff_line_visual(line.line_type, &self.theme);

                            if visual.bg_color != Color32::TRANSPARENT {
                                ui.painter().rect_filled(row_rect, Rounding::ZERO, visual.bg_color);
                            }

                            // Old line number column
                            let old_str = format_line_num_col(line.old_line_num, geometry.line_num_chars);
                            let old_galley = ui.painter().layout_no_wrap(
                                old_str,
                                FontId::monospace(DIFF_LINE_NUM_FONT_SIZE),
                                self.theme.text_tertiary,
                            );
                            ui.painter().galley(
                                Pos2::new(row_rect.left() + geometry.old_num_offset_x, row_rect.top() + 2.0),
                                old_galley,
                                Color32::PLACEHOLDER,
                            );

                            // New line number column
                            let new_str = format_line_num_col(line.new_line_num, geometry.line_num_chars);
                            let new_galley = ui.painter().layout_no_wrap(
                                new_str,
                                FontId::monospace(DIFF_LINE_NUM_FONT_SIZE),
                                self.theme.text_tertiary,
                            );
                            ui.painter().galley(
                                Pos2::new(row_rect.left() + geometry.new_num_offset_x, row_rect.top() + 2.0),
                                new_galley,
                                Color32::PLACEHOLDER,
                            );

                            // Marker prefix (+ / - / space)
                            let prefix_galley = ui.painter().layout_no_wrap(
                                visual.marker.to_owned(),
                                FontId::monospace(DIFF_MARKER_FONT_SIZE),
                                visual.marker_color,
                            );
                            ui.painter().galley(
                                Pos2::new(row_rect.left() + geometry.marker_offset_x, row_rect.top() + 2.0),
                                prefix_galley,
                                Color32::PLACEHOLDER,
                            );

                            // Line Content
                            ui.painter().galley(
                                Pos2::new(row_rect.left() + geometry.content_offset_x, row_rect.top() + 2.0),
                                content_galley.clone(),
                                Color32::PLACEHOLDER,
                            );
                        }

                        ui.add_space(DIFF_BOTTOM_PADDING_Y);
                    });
            })
            .response;

        response
    }
}

fn diff_content_width(line_widths: impl IntoIterator<Item = f32>, content_offset_x: f32) -> f32 {
    let widest_line = line_widths.into_iter().fold(0.0, f32::max);
    content_offset_x + widest_line + DIFF_CONTENT_RIGHT_PADDING
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DbProTheme;

    #[test]
    fn diff_content_width_reserves_the_longest_line_and_columns() {
        assert_eq!(diff_content_width([20.0, 80.0], 86.0), 174.0);
    }

    #[test]
    fn diff_viewer_renders_in_egui_context() {
        let theme = DbProTheme::light();
        let ctx = egui::Context::default();
        DbProTheme::install_fonts(&ctx);

        let lines = vec![
            DiffLine::context(1, 1, "SELECT * FROM users"),
            DiffLine::removed(2, "- WHERE active = false"),
            DiffLine::added(2, "+ WHERE active = true"),
        ];

        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let resp = DiffViewer::new("migration.sql", &lines, theme).show(ui);
                assert!(resp.rect.width() > 0.0);
                assert!(resp.rect.height() > 0.0);
            });
        });
    }
}
