use crate::DbProTheme;
use egui::{FontId, Response, Sense, Stroke, Ui, WidgetInfo, WidgetType};

use super::config::{DEFAULT_THICKNESS, DEFAULT_VERTICAL_MARGIN, LABEL_FONT_SIZE};
use super::handler::{
    calculate_labeled_separator_geometry, horizontal_line_segment, horizontal_size, vertical_line_segment,
    vertical_size,
};
use crate::tokens::SPACE_SM;

/// Orientation of the separator line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeparatorOrientation {
    Horizontal,
    Vertical,
}

/// A subtle visual divider for grouping UI sections or toolbar items.
///
/// Separator supports both horizontal (full-width, with optional text label)
/// and vertical (bounded-height for toolbars/headers) orientations. Labels render only
/// for horizontal separators; vertical labels would expand the behavior/API contract.
pub struct Separator<'a> {
    orientation: SeparatorOrientation,
    label: Option<&'a str>,
    thickness: f32,
    margin: f32,
    theme: DbProTheme,
}

impl<'a> Separator<'a> {
    /// Creates a new horizontal separator with standard margin.
    pub fn horizontal(theme: DbProTheme) -> Self {
        Self {
            orientation: SeparatorOrientation::Horizontal,
            label: None,
            thickness: DEFAULT_THICKNESS,
            margin: SPACE_SM,
            theme,
        }
    }

    /// Creates a new vertical separator with standard margin.
    pub fn vertical(theme: DbProTheme) -> Self {
        Self {
            orientation: SeparatorOrientation::Vertical,
            label: None,
            thickness: DEFAULT_THICKNESS,
            margin: DEFAULT_VERTICAL_MARGIN,
            theme,
        }
    }

    /// Sets an optional centered label for horizontal separators.
    pub fn label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }

    /// Overrides the line thickness in points.
    pub fn thickness(mut self, thickness: f32) -> Self {
        self.thickness = if thickness.is_finite() && thickness >= 0.0 {
            thickness
        } else {
            DEFAULT_THICKNESS
        };
        self
    }

    /// Overrides the margin spacing around the separator.
    pub fn margin(mut self, margin: f32) -> Self {
        self.margin = if margin.is_finite() && margin >= 0.0 {
            margin
        } else {
            match self.orientation {
                SeparatorOrientation::Horizontal => SPACE_SM,
                SeparatorOrientation::Vertical => DEFAULT_VERTICAL_MARGIN,
            }
        };
        self
    }

    /// Allocates layout geometry and renders the separator into the provided egui UI.
    pub fn show(self, ui: &mut Ui) -> Response {
        let stroke = Stroke::new(self.thickness, self.theme.border_subtle);

        match self.orientation {
            SeparatorOrientation::Horizontal => {
                // UI Flow:
                // 1. Query available width from egui layout.
                // 2. Compute minimum bounding size using handler geometry logic.
                // 3. Allocate exact size in egui with hover sense.
                let avail_w = ui.available_width();
                let size = horizontal_size(avail_w, self.thickness, self.margin);
                let (rect, resp) = ui.allocate_exact_size(
                    size,
                    Sense {
                        click: false,
                        drag: false,
                        focusable: false,
                    },
                );
                let center_y = rect.center().y;

                if let Some(lbl) = self.label.filter(|label| !label.is_empty()) {
                    resp.widget_info(|| WidgetInfo::labeled(WidgetType::Label, true, lbl));
                    // 4a. If labeled, measure label galley text size.
                    let galley = ui.painter().layout_no_wrap(
                        lbl.to_string(),
                        FontId::proportional(LABEL_FONT_SIZE),
                        self.theme.text_muted,
                    );

                    // 4b. Pure geometric calculation determines left line, text pos, and right line.
                    let geom = calculate_labeled_separator_geometry(rect, center_y, galley.size(), SPACE_SM);

                    // 4c. Clip overlong text to the allocation so narrow layouts never paint into neighbors.
                    let painter = ui.painter().with_clip_rect(rect);
                    painter.line_segment(geom.left_line, stroke);
                    painter.galley(geom.text_pos, galley, self.theme.text_muted);
                    painter.line_segment(geom.right_line, stroke);
                } else {
                    // 4d. If unlabeled, paint single continuous horizontal line across allocation.
                    let line = horizontal_line_segment(rect, center_y);
                    ui.painter().line_segment(line, stroke);
                }

                resp
            }
            SeparatorOrientation::Vertical => {
                // UI Flow:
                // 1. Query available height from egui layout.
                // 2. Compute vertical bounding size clamped to max height using handler logic.
                // 3. Allocate exact size in egui with hover sense.
                let avail_h = ui.available_height();
                let size = vertical_size(avail_h, self.thickness, self.margin);
                let (rect, resp) = ui.allocate_exact_size(
                    size,
                    Sense {
                        click: false,
                        drag: false,
                        focusable: false,
                    },
                );
                let center_x = rect.center().x;

                // 4. Compute vertical line coordinates with top/bottom insets and paint line.
                let line = vertical_line_segment(center_x, rect.top(), rect.bottom());
                ui.painter().line_segment(line, stroke);

                resp
            }
        }
    }
}
