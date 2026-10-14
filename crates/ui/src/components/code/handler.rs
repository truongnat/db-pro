use egui::{Color32, Pos2, Rect, Vec2};
use lucide_icons::Icon;

use super::config::{
    CODE_BLOCK_COPY_BTN_INSET, CODE_BLOCK_DIGIT_CHAR_WIDTH, CODE_BLOCK_GUTTER_EXTRA_PAD, CODE_BLOCK_HEADER_LABEL_GAP,
    CODE_BLOCK_NO_GUTTER_WIDTH, INLINE_CODE_PAD,
};
use crate::DbProTheme;

#[derive(Debug, Clone, Copy)]
pub struct CopyButtonState {
    pub icon: Icon,
    pub label: &'static str,
    pub color: Color32,
}

pub fn copy_button_state(is_recently_copied: bool, theme: &DbProTheme) -> CopyButtonState {
    if is_recently_copied {
        CopyButtonState {
            icon: Icon::Check,
            label: "Copied",
            color: theme.success,
        }
    } else {
        CopyButtonState {
            icon: Icon::Copy,
            label: "Copy",
            color: theme.text_secondary,
        }
    }
}

pub fn is_copy_active(copied_time: Option<f64>, current_time: f64, duration_secs: f64) -> bool {
    copied_time.is_some_and(|t| t <= current_time && current_time - t < duration_secs)
}

pub fn copy_feedback_remaining(copied_time: Option<f64>, current_time: f64, duration_secs: f64) -> Option<f64> {
    let copied_time = copied_time?;
    if copied_time > current_time {
        return None;
    }

    let remaining = duration_secs - (current_time - copied_time);
    (remaining > 0.0).then_some(remaining)
}

pub fn code_body_content_width(line_widths: impl IntoIterator<Item = f32>, gutter_width: f32, min_width: f32) -> f32 {
    let widest_line = line_widths.into_iter().fold(0.0, f32::max);
    min_width.max(gutter_width + 4.0 + widest_line + 12.0)
}

pub fn header_language_clip_rect(header_rect: Rect, copy_button_rect: Rect) -> Rect {
    let right = (copy_button_rect.left() - CODE_BLOCK_HEADER_LABEL_GAP).max(header_rect.left());
    Rect::from_min_max(header_rect.min, Pos2::new(right, header_rect.bottom()))
}

pub fn calculate_gutter_width(show_line_numbers: bool, line_count: usize) -> f32 {
    if show_line_numbers {
        let digits = format!("{}", line_count.max(1)).len() as f32;
        digits * CODE_BLOCK_DIGIT_CHAR_WIDTH + CODE_BLOCK_GUTTER_EXTRA_PAD
    } else {
        CODE_BLOCK_NO_GUTTER_WIDTH
    }
}

pub fn inline_code_size(galley_size: Vec2) -> Vec2 {
    galley_size + INLINE_CODE_PAD * 2.0
}

pub fn inline_code_text_pos(rect_min: Pos2) -> Pos2 {
    Pos2::new(rect_min.x + INLINE_CODE_PAD.x, rect_min.y + INLINE_CODE_PAD.y)
}

pub fn copy_button_rect(header_rect: Rect, btn_size: Vec2) -> Rect {
    let size = Vec2::new(
        btn_size.x.min(header_rect.width()),
        btn_size.y.min(header_rect.height()),
    );
    let right = (header_rect.right() - CODE_BLOCK_COPY_BTN_INSET).max(header_rect.left() + size.x);
    Rect::from_min_size(Pos2::new(right - size.x, header_rect.center().y - size.y * 0.5), size)
}

pub fn format_line_number(line_num: usize, max_lines: usize) -> String {
    let width = format!("{}", max_lines.max(1)).len();
    format!("{:>width$}", line_num, width = width)
}

#[cfg(test)]
#[path = "tests/handler_tests.rs"]
mod tests;
