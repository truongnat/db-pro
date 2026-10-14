mod palette;
mod size;

use egui::Pos2;
pub use palette::ButtonPalette;
pub use size::SizeTokens;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonVariant {
    Default,
    Secondary,
    Outline,
    Ghost,
    Destructive,
    Link,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonSize {
    Sm,
    Default,
    Lg,
    Icon,
    IconSm,
}

pub fn leading_content_x(
    left_aligned: bool,
    rect_left: f32,
    rect_center_x: f32,
    content_width: f32,
    padding_x: f32,
) -> f32 {
    if left_aligned {
        rect_left + padding_x
    } else {
        rect_center_x - content_width * 0.5
    }
}

pub fn centered_content_pos(start_x: f32, center_y: f32, content_height: f32) -> Pos2 {
    Pos2::new(start_x, center_y - content_height * 0.5)
}

#[cfg(test)]
#[path = "tests/mod_tests.rs"]
mod tests;
