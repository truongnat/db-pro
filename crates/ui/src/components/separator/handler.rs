use egui::{Pos2, Rect, Vec2};

use super::config::{MAX_VERTICAL_HEIGHT, MIN_HORIZONTAL_HEIGHT, VERTICAL_LINE_INSET};

/// Pure layout geometry for a horizontal separator with an optional centered text label.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LabeledSeparatorGeometry {
    /// Start and end coordinates for the left horizontal line segment.
    pub left_line: [Pos2; 2],
    /// Top-left position for positioning the laid-out label text galley.
    pub text_pos: Pos2,
    /// Start and end coordinates for the right horizontal line segment.
    pub right_line: [Pos2; 2],
}

/// Calculates the bounding allocation size for a horizontal separator.
/// Invalid negative or non-finite dimensions are treated as zero.
pub fn horizontal_size(available_width: f32, thickness: f32, margin: f32) -> Vec2 {
    let available_width = sanitize_non_negative(available_width, 0.0);
    let thickness = sanitize_non_negative(thickness, 0.0);
    let margin = sanitize_non_negative(margin, 0.0);
    let height = (thickness + margin * 2.0).max(MIN_HORIZONTAL_HEIGHT);
    Vec2::new(available_width, height)
}

/// Calculates the bounding allocation size for a vertical separator.
pub fn vertical_size(available_height: f32, thickness: f32, margin: f32) -> Vec2 {
    let thickness = sanitize_non_negative(thickness, 0.0);
    let margin = sanitize_non_negative(margin, 0.0);
    let height = sanitize_non_negative(available_height, 0.0).min(MAX_VERTICAL_HEIGHT);
    let width = thickness + margin * 2.0;
    Vec2::new(width, height)
}

/// Computes the start and end coordinates of a plain (unlabeled) horizontal divider line.
pub fn horizontal_line_segment(rect: Rect, center_y: f32) -> [Pos2; 2] {
    [Pos2::new(rect.left(), center_y), Pos2::new(rect.right(), center_y)]
}

/// Computes the left/right line segments and centered text position for a labeled horizontal separator.
pub fn calculate_labeled_separator_geometry(
    rect: Rect,
    center_y: f32,
    text_size: Vec2,
    text_pad: f32,
) -> LabeledSeparatorGeometry {
    let avail_w = sanitize_non_negative(rect.width(), 0.0);
    let text_w = sanitize_non_negative(text_size.x, 0.0);
    let text_h = sanitize_non_negative(text_size.y, 0.0);
    let text_pad = sanitize_non_negative(text_pad, 0.0);
    // Never preserve MIN_LINE_WIDTH when it would intrude into the label or outside the rect.
    let line_w = ((avail_w - text_w - text_pad * 2.0) * 0.5).max(0.0).min(avail_w * 0.5);

    let left_line = [
        Pos2::new(rect.left(), center_y),
        Pos2::new(rect.left() + line_w, center_y),
    ];

    let text_pos = Pos2::new(rect.center().x - text_w * 0.5, center_y - text_h * 0.5);

    let right_line = [
        Pos2::new(rect.right() - line_w, center_y),
        Pos2::new(rect.right(), center_y),
    ];

    LabeledSeparatorGeometry {
        left_line,
        text_pos,
        right_line,
    }
}

fn sanitize_non_negative(value: f32, fallback: f32) -> f32 {
    if value.is_finite() && value >= 0.0 {
        value
    } else {
        fallback
    }
}

/// Computes the start and end coordinates of a vertical divider line with vertical insets.
pub fn vertical_line_segment(center_x: f32, top: f32, bottom: f32) -> [Pos2; 2] {
    let min_y = top.min(bottom);
    let max_y = top.max(bottom);
    let inset = VERTICAL_LINE_INSET.min((max_y - min_y).max(0.0) * 0.5);
    [Pos2::new(center_x, min_y + inset), Pos2::new(center_x, max_y - inset)]
}

#[cfg(test)]
#[path = "tests/handler_tests.rs"]
mod tests;
