// cc-scan:allow-file HUGE_FUNCTION,LONG_FUNCTION,HARD_COMPLEXITY,COMPLEXITY,DEEP_NESTING
// cc-scan:allow-file HARD_PARAMS,TOO_MANY_PARAMS,DUPLICATE_BLOCK
// egui painter/view file: fns are linear layout code; branches are per-state paint variants.
use crate::tokens::STROKE_THIN;
use crate::DbProTheme;
use egui::{Color32, CornerRadius, FontId, Pos2, Response, RichText, Sense, Stroke, Ui, Vec2, WidgetInfo, WidgetType};

use super::config::{
    DIFF_BOTTOM_PADDING_Y, DIFF_CONTENT_FONT_SIZE, DIFF_CORNER_RADIUS, DIFF_HEADER_HEIGHT,
    DIFF_HEADER_STATS_RIGHT_INSET, DIFF_HEADER_STATS_TEXT_Y_OFFSET, DIFF_HEADER_TITLE_LEFT_INSET,
    DIFF_HEADER_TITLE_STATS_GAP, DIFF_HEADER_TITLE_TEXT_Y_OFFSET, DIFF_LINE_HEIGHT, DIFF_LINE_NUM_FONT_SIZE,
    DIFF_MARKER_FONT_SIZE, DIFF_ROW_TEXT_TOP_OFFSET, DIFF_STATS_FONT_SIZE, DIFF_TITLE_FONT_SIZE,
};
use super::handler::{
    count_diff_changes, diff_content_width, diff_geometry, diff_line_visual, diff_stats_left_offset, format_diff_stats,
    format_line_num_col, DiffLine,
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
        let frame = egui::Frame::NONE
            .fill(self.theme.surface_editor)
            .stroke(Stroke::new(STROKE_THIN, self.theme.border_default))
            .corner_radius(CornerRadius::same(DIFF_CORNER_RADIUS as u8))
            .inner_margin(egui::Margin::same(0.0 as i8));

        let (added_count, removed_count) = count_diff_changes(self.lines);
        let geometry = diff_geometry(self.lines);

        let response = frame
            .show(ui, |ui| {
                ui.set_width(ui.available_width());

                // ── Diff Header ─────────────────────────────────────────────
                let header_rect = ui.allocate_space(Vec2::new(ui.available_width(), DIFF_HEADER_HEIGHT)).1;
                ui.painter().rect_filled(
                    header_rect,
                    CornerRadius {
                        nw: (DIFF_CORNER_RADIUS) as u8,
                        ne: (DIFF_CORNER_RADIUS) as u8,
                        sw: 0.0 as u8,
                        se: 0.0 as u8,
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
                let stats_left = header_rect.left()
                    + diff_stats_left_offset(
                        header_rect.width(),
                        stats_galley.size().x,
                        DIFF_HEADER_STATS_RIGHT_INSET,
                    );
                let stats_pos = Pos2::new(stats_left, header_rect.center().y - DIFF_HEADER_STATS_TEXT_Y_OFFSET);

                // File / target title clips before the stats badge so narrow headers never overlap.
                let title_left = header_rect.left() + DIFF_HEADER_TITLE_LEFT_INSET;
                let title_right = (stats_pos.x - DIFF_HEADER_TITLE_STATS_GAP).max(title_left);
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
                    Pos2::new(title_left, header_rect.center().y - DIFF_HEADER_TITLE_TEXT_Y_OFFSET),
                    title_galley,
                    Color32::PLACEHOLDER,
                );
                ui.painter()
                    .with_clip_rect(header_rect)
                    .galley(stats_pos, stats_galley, Color32::PLACEHOLDER);

                // Shape each line once; reuse its theme-aware visual and galley for width measurement and painting.
                let line_rows: Vec<_> = self
                    .lines
                    .iter()
                    .map(|line| {
                        let visual = diff_line_visual(line.line_type, &self.theme);
                        let galley = ui.painter().layout_no_wrap(
                            line.content.clone(),
                            FontId::monospace(DIFF_CONTENT_FONT_SIZE),
                            visual.content_color,
                        );
                        (galley, visual)
                    })
                    .collect();
                let content_width = diff_content_width(
                    line_rows.iter().map(|(galley, _)| galley.size().x),
                    geometry.content_offset_x,
                );

                // ── Diff Lines ──────────────────────────────────────────────
                // Columns children share stable ids, so the scroll state must be salted per instance.
                egui::ScrollArea::horizontal()
                    .id_salt(ui.auto_id_with("diff_viewer_scroll"))
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        let row_width = ui.available_width().max(content_width);
                        if self.lines.is_empty() {
                            ui.allocate_ui_with_layout(
                                Vec2::new(row_width, DIFF_LINE_HEIGHT * 2.0),
                                egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
                                |ui| {
                                    ui.label(
                                        RichText::new("No changes to display.")
                                            .font(FontId::proportional(DIFF_CONTENT_FONT_SIZE))
                                            .color(self.theme.text_muted),
                                    );
                                },
                            );
                            return;
                        }

                        for (line, (content_galley, visual)) in self.lines.iter().zip(&line_rows) {
                            let (row_rect, row_resp) =
                                ui.allocate_exact_size(Vec2::new(row_width, DIFF_LINE_HEIGHT), Sense::hover());
                            row_resp.widget_info(|| WidgetInfo::labeled(WidgetType::Other, true, &line.content));

                            if visual.bg_color != Color32::TRANSPARENT {
                                ui.painter().rect_filled(row_rect, CornerRadius::ZERO, visual.bg_color);
                            }

                            // Old line number column
                            let old_str = format_line_num_col(line.old_line_num, geometry.line_num_chars);
                            let old_galley = ui.painter().layout_no_wrap(
                                old_str,
                                FontId::monospace(DIFF_LINE_NUM_FONT_SIZE),
                                self.theme.text_tertiary,
                            );
                            ui.painter().galley(
                                Pos2::new(
                                    row_rect.left() + geometry.old_num_offset_x,
                                    row_rect.top() + DIFF_ROW_TEXT_TOP_OFFSET,
                                ),
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
                                Pos2::new(
                                    row_rect.left() + geometry.new_num_offset_x,
                                    row_rect.top() + DIFF_ROW_TEXT_TOP_OFFSET,
                                ),
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
                                Pos2::new(
                                    row_rect.left() + geometry.marker_offset_x,
                                    row_rect.top() + DIFF_ROW_TEXT_TOP_OFFSET,
                                ),
                                prefix_galley,
                                Color32::PLACEHOLDER,
                            );

                            // Line Content
                            ui.painter().galley(
                                Pos2::new(
                                    row_rect.left() + geometry.content_offset_x,
                                    row_rect.top() + DIFF_ROW_TEXT_TOP_OFFSET,
                                ),
                                content_galley.clone(),
                                visual.content_color,
                            );
                        }

                        ui.add_space(DIFF_BOTTOM_PADDING_Y);
                    });
            })
            .response;

        response.widget_info(|| WidgetInfo::labeled(WidgetType::Other, true, self.title));
        response
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DbProTheme;

    #[test]
    fn empty_diff_renders_a_labeled_state() {
        let theme = DbProTheme::light();
        let ctx = egui::Context::default();
        DbProTheme::install_fonts(&ctx);

        let _ = crate::test_frame::frame(&ctx, Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let response = DiffViewer::new("migration.sql", &[], theme).show(ui);
                assert!(response.rect.width() > 0.0);
                assert!(response.rect.height() > 0.0);
            });
        });
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

        let _ = crate::test_frame::frame(&ctx, Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let resp = DiffViewer::new("migration.sql", &lines, theme).show(ui);
                assert!(resp.rect.width() > 0.0);
                assert!(resp.rect.height() > 0.0);
            });
        });
    }
}
