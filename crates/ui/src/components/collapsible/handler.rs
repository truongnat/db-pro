use egui::{Pos2, Rect, Vec2};

use super::config::{BADGE_HEIGHT, BADGE_PAD_H, BADGE_RIGHT_MARGIN};

/// Toggles the collapsible state if clicked and not disabled.
pub fn apply_header_click(open: &mut bool, clicked: bool, disabled: bool) {
    if clicked && !disabled {
        *open = !*open;
    }
}

/// Computes the layout bounding rectangle for the trailing badge pill.
pub fn calculate_badge_rect(header_rect: Rect, center_y: f32, badge_text_width: f32) -> Rect {
    let available_width = (header_rect.width() - BADGE_RIGHT_MARGIN).max(0.0);
    let badge_width = (badge_text_width + BADGE_PAD_H).clamp(0.0, available_width);
    let right_edge = (header_rect.right() - BADGE_RIGHT_MARGIN).max(header_rect.left());
    Rect::from_center_size(
        Pos2::new(right_edge - badge_width * 0.5, center_y),
        Vec2::new(badge_width, BADGE_HEIGHT),
    )
}

#[cfg(test)]
#[path = "tests/handler_tests.rs"]
mod tests;
