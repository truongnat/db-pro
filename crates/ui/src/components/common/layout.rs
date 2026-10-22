use egui::{Pos2, Rect, Vec2};

use crate::tokens::SPACE_2XL;

/// Layout parameters calculated for a modal dialog.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DialogLayout {
    pub width: f32,
    pub max_body_height: f32,
    pub target_pos: Pos2,
}

/// Calculates the layout dimensions and centered viewport coordinates for a modal dialog.
///
/// Handles small-screen viewports (such as MacBook 13"/14" at 1280x800 or 1512x982 logical points),
/// clamping the dialog inside safety margins and estimating smooth initial vertical position
/// before the first frame's height measurement to prevent visual snapping.
pub fn calculate_dialog_layout(
    screen: Rect,
    requested_width: f32,
    measured_height: Option<f32>,
    horizontal_margin: f32,
    vertical_margin: f32,
    chrome_height: f32,
) -> DialogLayout {
    let horizontal_padding = horizontal_margin * 2.0;
    let width = requested_width.min((screen.width() - horizontal_padding).max(80.0));
    let max_body_height = dialog_body_cap(screen.height(), vertical_margin, chrome_height);

    let target_x = (screen.center().x - width * 0.5).clamp(
        screen.left() + horizontal_margin,
        (screen.right() - width - horizontal_margin).max(screen.left() + horizontal_margin),
    );

    let target_y = if let Some(h) = measured_height {
        (screen.center().y - h * 0.5).clamp(
            screen.top() + vertical_margin,
            (screen.bottom() - h - vertical_margin).max(screen.top() + vertical_margin),
        )
    } else {
        let estimated_h = (screen.height() * 0.45).clamp(160.0, 480.0);
        (screen.center().y - estimated_h * 0.5).clamp(
            screen.top() + vertical_margin,
            (screen.bottom() - estimated_h - vertical_margin).max(screen.top() + vertical_margin),
        )
    };

    DialogLayout {
        width,
        max_body_height,
        target_pos: Pos2::new(target_x, target_y),
    }
}

/// Tallest scrollable body a dialog can claim without leaving its margins.
pub fn dialog_body_cap(screen_height: f32, vertical_margin: f32, chrome_height: f32) -> f32 {
    (screen_height - vertical_margin * 2.0 - chrome_height).max(120.0)
}

/// Calculates the layout dimensions and horizontal slide-in position for a drawer sheet.
pub fn calculate_sheet_layout(
    screen: Rect,
    requested_width: f32,
    progress: f32,
    translate_px: f32,
    margin: f32,
) -> (f32, f32) {
    let width = requested_width.min((screen.width() - margin).max(80.0));
    let x = screen.right() - width + crate::components::animation::small_translate(progress, translate_px);
    (width, x)
}

/// Clamps a popup or completion menu's position so it remains fully visible within the screen bounds.
pub fn clamp_popup_to_screen(mut desired_pos: Pos2, popup_size: Vec2, screen: Rect, margin: f32) -> Pos2 {
    if desired_pos.y + popup_size.y > screen.max.y - margin {
        desired_pos.y = (desired_pos.y - popup_size.y - SPACE_2XL).max(screen.min.y + margin);
    }
    desired_pos.x = desired_pos.x.clamp(
        screen.min.x + margin,
        (screen.max.x - popup_size.x - margin).max(screen.min.x + margin),
    );
    desired_pos
}

#[cfg(test)]
#[path = "tests/layout_tests.rs"]
mod tests;
