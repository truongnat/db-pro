use crate::DbProTheme;
use egui::{Color32, FontId, Pos2, Response, Sense, Stroke, Ui, Vec2};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeparatorOrientation {
    Horizontal,
    Vertical,
}

pub struct Separator<'a> {
    orientation: SeparatorOrientation,
    label: Option<&'a str>,
    thickness: f32,
    margin: f32,
    theme: DbProTheme,
}

impl<'a> Separator<'a> {
    pub fn horizontal(theme: DbProTheme) -> Self {
        Self {
            orientation: SeparatorOrientation::Horizontal,
            label: None,
            thickness: 1.0,
            margin: 8.0,
            theme,
        }
    }

    pub fn vertical(theme: DbProTheme) -> Self {
        Self {
            orientation: SeparatorOrientation::Vertical,
            label: None,
            thickness: 1.0,
            margin: 6.0,
            theme,
        }
    }

    pub fn label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }

    pub fn thickness(mut self, thickness: f32) -> Self {
        self.thickness = thickness;
        self
    }

    pub fn margin(mut self, margin: f32) -> Self {
        self.margin = margin;
        self
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        match self.orientation {
            SeparatorOrientation::Horizontal => {
                let avail_w = ui.available_width();
                let height = (self.thickness + self.margin * 2.0).max(14.0);
                let (rect, resp) = ui.allocate_exact_size(Vec2::new(avail_w, height), Sense::hover());
                let center_y = rect.center().y;

                if let Some(lbl) = self.label {
                    let galley =
                        ui.painter()
                            .layout_no_wrap(lbl.to_string(), FontId::proportional(11.0), self.theme.text_muted);
                    let text_w = galley.size().x;
                    let text_pad = 8.0;
                    let line_w = ((avail_w - text_w - text_pad * 2.0) * 0.5).max(4.0);

                    // Left line
                    ui.painter().line_segment(
                        [
                            Pos2::new(rect.left(), center_y),
                            Pos2::new(rect.left() + line_w, center_y),
                        ],
                        Stroke::new(self.thickness, self.theme.border_subtle),
                    );

                    // Centered label
                    let text_x = rect.left() + line_w + text_pad;
                    ui.painter().galley(
                        Pos2::new(text_x, center_y - galley.size().y * 0.5),
                        galley,
                        Color32::PLACEHOLDER,
                    );

                    // Right line
                    ui.painter().line_segment(
                        [
                            Pos2::new(rect.right() - line_w, center_y),
                            Pos2::new(rect.right(), center_y),
                        ],
                        Stroke::new(self.thickness, self.theme.border_subtle),
                    );
                } else {
                    ui.painter().line_segment(
                        [Pos2::new(rect.left(), center_y), Pos2::new(rect.right(), center_y)],
                        Stroke::new(self.thickness, self.theme.border_subtle),
                    );
                }
                resp
            }
            SeparatorOrientation::Vertical => {
                let avail_h = ui.available_height().min(24.0);
                let width = self.thickness + self.margin * 2.0;
                let (rect, resp) = ui.allocate_exact_size(Vec2::new(width, avail_h), Sense::hover());
                let center_x = rect.center().x;

                ui.painter().line_segment(
                    [
                        Pos2::new(center_x, rect.top() + 2.0),
                        Pos2::new(center_x, rect.bottom() - 2.0),
                    ],
                    Stroke::new(self.thickness, self.theme.border_subtle),
                );
                resp
            }
        }
    }
}
