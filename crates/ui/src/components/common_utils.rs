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

/// Safely truncates a string to at most `max_chars` characters, appending `…` if truncated.
///
/// This avoids splitting unicode multi-byte character boundaries and provides consistent
/// ellipsis formatting across all UI views and widgets.
pub fn truncate_ellipsis(text: &str, max_chars: usize) -> String {
    let char_count = text.chars().count();
    if char_count <= max_chars {
        return text.to_owned();
    }
    let keep_chars = max_chars.saturating_sub(1);
    let truncated: String = text.chars().take(keep_chars).collect();
    format!("{truncated}…")
}

/// Formats a byte size into human-readable representation (e.g. `120 B`, `4.2 KB`, `18.5 MB`, `1.2 GB`).
pub fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * KB;
    const GB: u64 = 1024 * MB;
    const TB: u64 = 1024 * GB;

    if bytes < KB {
        format!("{bytes} B")
    } else if bytes < MB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else if bytes < GB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes < TB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else {
        format!("{:.2} TB", bytes as f64 / TB as f64)
    }
}

/// Formats execution or latency durations in milliseconds into readable text (`15ms`, `1.24s`, `2m 14s`).
pub fn format_duration_millis(ms: u64) -> String {
    if ms < 1_000 {
        format!("{ms}ms")
    } else if ms < 60_000 {
        format!("{:.2}s", ms as f64 / 1_000.0)
    } else {
        let minutes = ms / 60_000;
        let seconds = (ms % 60_000) / 1_000;
        format!("{minutes}m {seconds}s")
    }
}

/// Formats an item count with singular/plural suffix (e.g. `1 row`, `42 rows`, `0 tables`).
pub fn format_count_with_suffix(count: usize, singular: &str, plural: &str) -> String {
    if count == 1 {
        format!("1 {singular}")
    } else {
        format!("{count} {plural}")
    }
}

/// Formats a table pagination range description (e.g. `Rows 1–50 of 200`, `Rows 1–25`, `0 rows`).
pub fn format_page_range(offset: u64, count: u64, total_rows: Option<u64>) -> String {
    if let Some(total) = total_rows {
        if total > 0 && count > 0 {
            let start = offset + 1;
            let end = (offset + count).min(total);
            format!("Rows {start}–{end} of {total}")
        } else if total > 0 {
            format!("0 of {total} rows")
        } else {
            "0 rows".to_owned()
        }
    } else if count > 0 {
        let start = offset + 1;
        let end = offset + count;
        format!("Rows {start}–{end}")
    } else {
        "0 rows".to_owned()
    }
}

/// Formats a percentage ratio with one decimal precision (e.g. `98.5%`, `100.0%`).
pub fn format_percentage(numerator: u64, denominator: u64) -> String {
    if denominator == 0 {
        return "0.0%".to_owned();
    }
    let ratio = (numerator as f64 / denominator as f64) * 100.0;
    format!("{ratio:.1}%")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_dialog_layout_centers_correctly() {
        let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(1280.0, 800.0));
        let layout = calculate_dialog_layout(screen, 420.0, Some(300.0), 16.0, 24.0, 220.0);

        assert_eq!(layout.width, 420.0);
        assert_eq!(layout.target_pos.x, (1280.0 - 420.0) / 2.0);
        assert_eq!(layout.target_pos.y, (800.0 - 300.0) / 2.0);
        assert_eq!(layout.max_body_height, 800.0 - 48.0 - 220.0);
    }

    #[test]
    fn test_calculate_dialog_layout_handles_small_screens() {
        // Small laptop screen (e.g. 1024x600)
        let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(1024.0, 600.0));
        let layout = calculate_dialog_layout(screen, 880.0, Some(700.0), 16.0, 24.0, 220.0);

        assert_eq!(layout.width, 880.0);
        // Because height 700 exceeds screen height 600, target_y should clamp to vertical_margin
        assert_eq!(layout.target_pos.y, 24.0);
        assert_eq!(layout.max_body_height, 600.0 - 48.0 - 220.0);
    }

    #[test]
    fn test_calculate_sheet_layout() {
        let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(1280.0, 800.0));
        let (width, x) = calculate_sheet_layout(screen, 360.0, 1.0, 16.0, 16.0);
        assert_eq!(width, 360.0);
        assert_eq!(x, 1280.0 - 360.0);
    }

    #[test]
    fn test_clamp_popup_to_screen() {
        let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(1000.0, 800.0));
        let pos = clamp_popup_to_screen(Pos2::new(950.0, 780.0), Vec2::new(200.0, 100.0), screen, 10.0);
        assert!(pos.x + 200.0 <= 990.0);
        assert!(pos.y >= 10.0);
    }

    #[test]
    fn test_truncate_ellipsis() {
        assert_eq!(truncate_ellipsis("short", 10), "short");
        assert_eq!(truncate_ellipsis("hello world", 5), "hell…");
        assert_eq!(truncate_ellipsis("tiếng việt unicode", 6), "tiếng…");
        assert_eq!(truncate_ellipsis("", 5), "");
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(500), "500 B");
        assert_eq!(format_bytes(1024), "1.0 KB");
        assert_eq!(format_bytes(1536), "1.5 KB");
        assert_eq!(format_bytes(1024 * 1024 * 5), "5.0 MB");
    }

    #[test]
    fn test_format_duration_millis() {
        assert_eq!(format_duration_millis(45), "45ms");
        assert_eq!(format_duration_millis(1500), "1.50s");
        assert_eq!(format_duration_millis(65000), "1m 5s");
    }

    #[test]
    fn test_format_count_with_suffix() {
        assert_eq!(format_count_with_suffix(1, "row", "rows"), "1 row");
        assert_eq!(format_count_with_suffix(0, "row", "rows"), "0 rows");
        assert_eq!(format_count_with_suffix(5, "table", "tables"), "5 tables");
    }

    #[test]
    fn test_format_page_range() {
        assert_eq!(format_page_range(0, 50, Some(200)), "Rows 1–50 of 200");
        assert_eq!(format_page_range(50, 50, Some(200)), "Rows 51–100 of 200");
        assert_eq!(format_page_range(180, 50, Some(200)), "Rows 181–200 of 200");
        assert_eq!(format_page_range(0, 25, None), "Rows 1–25");
        assert_eq!(format_page_range(0, 0, Some(0)), "0 rows");
    }

    #[test]
    fn test_format_percentage() {
        assert_eq!(format_percentage(985, 1000), "98.5%");
        assert_eq!(format_percentage(1, 1), "100.0%");
        assert_eq!(format_percentage(0, 0), "0.0%");
    }
}
