use egui::{Pos2, Rect, Vec2};

use super::config::{CHEVRON_SLOT, ICON_OFFSET, ICON_SLOT, INDENT_STEP, LEFT_INSET};

pub(crate) fn row_height() -> f32 {
    super::config::ROW_HEIGHT
}

pub(crate) fn node_origin(rect: Rect, depth: usize) -> (f32, f32) {
    (rect.left() + depth as f32 * INDENT_STEP + LEFT_INSET, rect.center().y)
}

pub(crate) fn chevron_position(x: f32, center_y: f32) -> Pos2 {
    Pos2::new(x + ICON_OFFSET - 1.0, center_y)
}

pub(crate) fn icon_position(x: f32, center_y: f32) -> Pos2 {
    Pos2::new(x + ICON_OFFSET, center_y)
}

pub(crate) fn after_chevron(x: f32) -> f32 {
    x + CHEVRON_SLOT
}

pub(crate) fn after_icon(x: f32) -> f32 {
    x + ICON_SLOT
}

pub(crate) fn clip_rect(rect: Rect, width: f32, height: f32) -> Rect {
    Rect::from_min_size(rect.min, Vec2::new(width, height))
}
