use crate::DbProTheme;
use egui::{Color32, FontId, Pos2, Response, Rounding, Sense, Stroke, Ui, Vec2};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffLineType {
    Context,
    Added,
    Removed,
}

#[derive(Debug, Clone)]
pub struct DiffLine {
    pub line_type: DiffLineType,
    pub old_line_num: Option<usize>,
    pub new_line_num: Option<usize>,
    pub content: String,
}

impl DiffLine {
    pub fn context(old_num: usize, new_num: usize, content: impl Into<String>) -> Self {
        Self {
            line_type: DiffLineType::Context,
            old_line_num: Some(old_num),
            new_line_num: Some(new_num),
            content: content.into(),
        }
    }

    pub fn added(new_num: usize, content: impl Into<String>) -> Self {
        Self {
            line_type: DiffLineType::Added,
            old_line_num: None,
            new_line_num: Some(new_num),
            content: content.into(),
        }
    }

    pub fn removed(old_num: usize, content: impl Into<String>) -> Self {
        Self {
            line_type: DiffLineType::Removed,
            old_line_num: Some(old_num),
            new_line_num: None,
            content: content.into(),
        }
    }
}

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
            .stroke(Stroke::new(1.0, self.theme.border_default))
            .rounding(Rounding::same(8.0))
            .inner_margin(egui::Margin::same(0.0));

        let added_count = self.lines.iter().filter(|l| l.line_type == DiffLineType::Added).count();
        let removed_count = self
            .lines
            .iter()
            .filter(|l| l.line_type == DiffLineType::Removed)
            .count();

        frame
            .show(ui, |ui| {
                ui.set_width(ui.available_width());

                // ── Diff Header ─────────────────────────────────────────────
                let header_rect = ui.allocate_space(Vec2::new(ui.available_width(), 34.0)).1;
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

                // File / target title
                let title_galley = ui.painter().layout_no_wrap(
                    self.title.to_owned(),
                    FontId::proportional(12.5),
                    self.theme.text_primary,
                );
                ui.painter().galley(
                    Pos2::new(header_rect.left() + 12.0, header_rect.center().y - 7.0),
                    title_galley,
                    Color32::PLACEHOLDER,
                );

                // Added / Removed summary badges
                let stats_str = format!("+{}  -{}", added_count, removed_count);
                let stats_galley =
                    ui.painter()
                        .layout_no_wrap(stats_str, FontId::monospace(11.5), self.theme.text_secondary);
                ui.painter().galley(
                    Pos2::new(
                        header_rect.right() - stats_galley.size().x - 14.0,
                        header_rect.center().y - 6.5,
                    ),
                    stats_galley,
                    Color32::PLACEHOLDER,
                );

                // ── Diff Lines ──────────────────────────────────────────────
                for line in self.lines {
                    let (row_rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 20.0), Sense::hover());

                    let (bg_color, prefix_char, text_color) = match line.line_type {
                        DiffLineType::Added => (
                            if self.theme.dark_mode {
                                Color32::from_rgba_premultiplied(34, 197, 94, 25)
                            } else {
                                Color32::from_rgba_premultiplied(22, 163, 74, 20)
                            },
                            "+",
                            self.theme.success,
                        ),
                        DiffLineType::Removed => (
                            if self.theme.dark_mode {
                                Color32::from_rgba_premultiplied(239, 68, 68, 25)
                            } else {
                                Color32::from_rgba_premultiplied(220, 38, 38, 20)
                            },
                            "-",
                            self.theme.danger,
                        ),
                        DiffLineType::Context => (Color32::TRANSPARENT, " ", self.theme.text_secondary),
                    };

                    if bg_color != Color32::TRANSPARENT {
                        ui.painter().rect_filled(row_rect, Rounding::ZERO, bg_color);
                    }

                    // Old line number column
                    let old_str = line.old_line_num.map(|n| n.to_string()).unwrap_or_default();
                    let old_galley = ui.painter().layout_no_wrap(
                        format!("{:>3}", old_str),
                        FontId::monospace(11.0),
                        self.theme.text_tertiary,
                    );
                    ui.painter().galley(
                        Pos2::new(row_rect.left() + 8.0, row_rect.top() + 2.0),
                        old_galley,
                        Color32::PLACEHOLDER,
                    );

                    // New line number column
                    let new_str = line.new_line_num.map(|n| n.to_string()).unwrap_or_default();
                    let new_galley = ui.painter().layout_no_wrap(
                        format!("{:>3}", new_str),
                        FontId::monospace(11.0),
                        self.theme.text_tertiary,
                    );
                    ui.painter().galley(
                        Pos2::new(row_rect.left() + 38.0, row_rect.top() + 2.0),
                        new_galley,
                        Color32::PLACEHOLDER,
                    );

                    // Marker prefix (+ / - / space)
                    let prefix_galley =
                        ui.painter()
                            .layout_no_wrap(prefix_char.to_owned(), FontId::monospace(12.0), text_color);
                    ui.painter().galley(
                        Pos2::new(row_rect.left() + 68.0, row_rect.top() + 2.0),
                        prefix_galley,
                        Color32::PLACEHOLDER,
                    );

                    // Line Content
                    let content_galley = ui.painter().layout_no_wrap(
                        line.content.clone(),
                        FontId::monospace(12.0),
                        if line.line_type == DiffLineType::Context {
                            self.theme.text_primary
                        } else {
                            text_color
                        },
                    );
                    ui.painter().galley(
                        Pos2::new(row_rect.left() + 82.0, row_rect.top() + 2.0),
                        content_galley,
                        Color32::PLACEHOLDER,
                    );
                }

                ui.add_space(6.0);
            })
            .response
    }
}
