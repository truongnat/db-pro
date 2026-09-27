use crate::DbProTheme;
use egui::{Color32, Pos2, Rect, Vec2};
use lucide_icons::Icon;

use super::config::{
    CODE_BLOCK_COPY_BTN_INSET, CODE_BLOCK_DIGIT_CHAR_WIDTH, CODE_BLOCK_GUTTER_EXTRA_PAD, CODE_BLOCK_HEADER_LABEL_GAP,
    CODE_BLOCK_NO_GUTTER_WIDTH, INLINE_CODE_PAD,
};

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
mod tests {
    use super::*;

    #[test]
    fn copy_button_state_switches_to_success_when_recently_copied() {
        let theme = DbProTheme::light();
        let idle = copy_button_state(false, &theme);
        assert_eq!(char::from(idle.icon), char::from(Icon::Copy));
        assert_eq!(idle.label, "Copy");
        assert_eq!(idle.color, theme.text_secondary);

        let copied = copy_button_state(true, &theme);
        assert_eq!(char::from(copied.icon), char::from(Icon::Check));
        assert_eq!(copied.label, "Copied");
        assert_eq!(copied.color, theme.success);
    }

    #[test]
    fn is_copy_active_checks_time_interval() {
        assert!(is_copy_active(Some(10.0), 11.5, 2.0));
        assert!(!is_copy_active(Some(10.0), 12.5, 2.0));
        assert!(!is_copy_active(None, 10.0, 2.0));
        assert!(!is_copy_active(Some(12.0), 10.0, 2.0));
    }

    #[test]
    fn copy_feedback_remaining_expires_and_ignores_future_time() {
        assert_eq!(copy_feedback_remaining(Some(10.0), 11.25, 2.0), Some(0.75));
        assert_eq!(copy_feedback_remaining(Some(10.0), 12.0, 2.0), None);
        assert_eq!(copy_feedback_remaining(Some(12.0), 10.0, 2.0), None);
        assert_eq!(copy_feedback_remaining(None, 10.0, 2.0), None);
    }

    #[test]
    fn code_body_content_width_reserves_long_lines_and_min_width() {
        assert_eq!(code_body_content_width([20.0, 40.0], 28.0, 120.0), 120.0);
        assert_eq!(code_body_content_width([200.0], 28.0, 120.0), 244.0);
    }

    #[test]
    fn copy_button_rect_stays_inside_narrow_header() {
        let header = Rect::from_min_size(Pos2::ZERO, Vec2::new(24.0, 18.0));
        let copy = copy_button_rect(header, Vec2::new(68.0, 22.0));
        assert!(header.contains_rect(copy));
    }

    #[test]
    fn header_language_clip_stays_left_of_copy_button() {
        let header = Rect::from_min_size(Pos2::ZERO, Vec2::new(96.0, 32.0));
        let copy = copy_button_rect(header, Vec2::new(68.0, 22.0));
        let clip = header_language_clip_rect(header, copy);
        assert!(clip.right() <= copy.left());
        assert_eq!(clip.left(), header.left());
        assert_eq!(clip.bottom(), header.bottom());
    }

    #[test]
    fn calculate_gutter_width_accounts_for_digit_count() {
        assert_eq!(calculate_gutter_width(false, 100), CODE_BLOCK_NO_GUTTER_WIDTH);
        // 1-digit line numbers (1..9): 1 * 8.0 + 20.0 = 28.0
        assert_eq!(calculate_gutter_width(true, 5), 28.0);
        // 3-digit line numbers (1..100): 3 * 8.0 + 20.0 = 44.0
        assert_eq!(calculate_gutter_width(true, 100), 44.0);
    }

    #[test]
    fn inline_code_geometry_helpers() {
        let galley_size = Vec2::new(50.0, 14.0);
        let size = inline_code_size(galley_size);
        assert_eq!(size, Vec2::new(62.0, 18.0));

        let text_pos = inline_code_text_pos(Pos2::new(10.0, 20.0));
        assert_eq!(text_pos, Pos2::new(16.0, 22.0));
    }

    #[test]
    fn format_line_number_aligns_properly() {
        assert_eq!(format_line_number(1, 9), "1");
        assert_eq!(format_line_number(1, 100), "  1");
        assert_eq!(format_line_number(42, 100), " 42");
        assert_eq!(format_line_number(100, 100), "100");
    }
}
