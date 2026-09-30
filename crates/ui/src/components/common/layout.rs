use egui::{Pos2, Rect, Vec2};

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
    let max_body_height = (screen.height() - vertical_margin * 2.0 - chrome_height).max(120.0);

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
        desired_pos.y = (desired_pos.y - popup_size.y - 24.0).max(screen.min.y + margin);
    }
    desired_pos.x = desired_pos.x.clamp(
        screen.min.x + margin,
        (screen.max.x - popup_size.x - margin).max(screen.min.x + margin),
    );
    desired_pos
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calculate_dialog_layout_centers_correctly() {
        let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(1280.0, 800.0));
        let layout = calculate_dialog_layout(screen, 420.0, Some(300.0), 16.0, 24.0, 220.0);

        assert_eq!(layout.width, 420.0);
        assert_eq!(layout.target_pos.x, (1280.0 - 420.0) / 2.0);
        assert_eq!(layout.target_pos.y, (800.0 - 300.0) / 2.0);
        assert_eq!(layout.max_body_height, 800.0 - 48.0 - 220.0);
    }

    #[test]
    fn calculate_dialog_layout_handles_small_screens() {
        let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(1024.0, 600.0));
        let layout = calculate_dialog_layout(screen, 880.0, Some(700.0), 16.0, 24.0, 220.0);

        assert_eq!(layout.width, 880.0);
        assert_eq!(layout.target_pos.y, 24.0);
        assert_eq!(layout.max_body_height, 600.0 - 48.0 - 220.0);
    }

    #[test]
    fn calculate_sheet_layout() {
        let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(1280.0, 800.0));
        let (width, x) = super::calculate_sheet_layout(screen, 360.0, 1.0, 16.0, 16.0);
        assert_eq!(width, 360.0);
        assert_eq!(x, 1280.0 - 360.0);
    }

    #[test]
    fn clamp_popup_to_screen() {
        let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(1000.0, 800.0));
        let pos = super::clamp_popup_to_screen(Pos2::new(950.0, 780.0), Vec2::new(200.0, 100.0), screen, 10.0);
        assert!(pos.x + 200.0 <= 990.0);
        assert!(pos.y >= 10.0);
    }
}
