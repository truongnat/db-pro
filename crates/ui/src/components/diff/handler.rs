// cc-scan:allow-file HUGE_FUNCTION,LONG_FUNCTION,HARD_COMPLEXITY,COMPLEXITY,DEEP_NESTING
// cc-scan:allow-file HARD_PARAMS,TOO_MANY_PARAMS,DUPLICATE_BLOCK
// egui painter/view file: fns are linear layout code; branches are per-state paint variants.
use crate::DbProTheme;
use egui::Color32;

use super::config::{
    DIFF_CONTENT_RIGHT_PADDING, DIFF_LINE_NUM_CHAR_WIDTH, DIFF_LINE_NUM_COLUMN_GAP, DIFF_MARKER_COLUMN_WIDTH,
    DIFF_OLD_NUM_INSET,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffLineType {
    Context,
    Added,
    Removed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffLine {
    pub line_type: DiffLineType,
    pub old_line_num: Option<usize>,
    pub new_line_num: Option<usize>,
    pub content: String,
}

impl DiffLine {
    pub fn context(old_num: usize, new_num: usize, content: impl Into<String>) -> Self {
        Self {
            line_type: DiffLineType::Context,
            old_line_num: Some(old_num),
            new_line_num: Some(new_num),
            content: content.into(),
        }
    }

    pub fn added(new_num: usize, content: impl Into<String>) -> Self {
        Self {
            line_type: DiffLineType::Added,
            old_line_num: None,
            new_line_num: Some(new_num),
            content: content.into(),
        }
    }

    pub fn removed(old_num: usize, content: impl Into<String>) -> Self {
        Self {
            line_type: DiffLineType::Removed,
            old_line_num: Some(old_num),
            new_line_num: None,
            content: content.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DiffLineVisual {
    pub bg_color: Color32,
    pub marker: &'static str,
    pub marker_color: Color32,
    pub content_color: Color32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DiffGeometry {
    pub line_num_chars: usize,
    pub old_num_offset_x: f32,
    pub new_num_offset_x: f32,
    pub marker_offset_x: f32,
    pub content_offset_x: f32,
}

pub fn diff_line_visual(line_type: DiffLineType, theme: &DbProTheme) -> DiffLineVisual {
    match line_type {
        DiffLineType::Added => DiffLineVisual {
            bg_color: theme.success_soft(),
            marker: "+",
            marker_color: theme.success,
            content_color: theme.text_primary,
        },
        DiffLineType::Removed => DiffLineVisual {
            bg_color: theme.danger_soft(),
            marker: "-",
            marker_color: theme.danger,
            content_color: theme.text_primary,
        },
        DiffLineType::Context => DiffLineVisual {
            bg_color: Color32::TRANSPARENT,
            marker: " ",
            marker_color: theme.text_secondary,
            content_color: theme.text_primary,
        },
    }
}

pub fn count_diff_changes(lines: &[DiffLine]) -> (usize, usize) {
    let added = lines.iter().filter(|l| l.line_type == DiffLineType::Added).count();
    let removed = lines.iter().filter(|l| l.line_type == DiffLineType::Removed).count();
    (added, removed)
}

pub fn format_diff_stats(added: usize, removed: usize) -> String {
    format!("+{}  -{}", added, removed)
}

pub fn line_num_chars(lines: &[DiffLine]) -> usize {
    // Keep ordinary diffs aligned to a stable three-column gutter, including empty diffs and line zero.
    lines
        .iter()
        .flat_map(|line| [line.old_line_num, line.new_line_num])
        .flatten()
        .map(decimal_digits)
        .max()
        .unwrap_or(3)
        .max(3)
}

pub fn diff_geometry(lines: &[DiffLine]) -> DiffGeometry {
    let line_num_chars = line_num_chars(lines);
    let line_num_width = line_num_chars as f32 * DIFF_LINE_NUM_CHAR_WIDTH;
    let old_num_offset_x = DIFF_OLD_NUM_INSET;
    let new_num_offset_x = old_num_offset_x + line_num_width + DIFF_LINE_NUM_COLUMN_GAP;
    let marker_offset_x = new_num_offset_x + line_num_width + DIFF_LINE_NUM_COLUMN_GAP;
    let content_offset_x = marker_offset_x + DIFF_MARKER_COLUMN_WIDTH;

    DiffGeometry {
        line_num_chars,
        old_num_offset_x,
        new_num_offset_x,
        marker_offset_x,
        content_offset_x,
    }
}

pub(super) fn diff_content_width(line_widths: impl IntoIterator<Item = f32>, content_offset_x: f32) -> f32 {
    let widest_line = line_widths.into_iter().fold(0.0, f32::max);
    content_offset_x + widest_line + DIFF_CONTENT_RIGHT_PADDING
}

/// Keep the summary's leading count visible when a narrow header cannot honor its trailing inset.
pub(super) fn diff_stats_left_offset(header_width: f32, stats_width: f32, right_inset: f32) -> f32 {
    (header_width - stats_width - right_inset).max(0.0)
}

pub fn format_line_num_col(num: Option<usize>, width: usize) -> String {
    num.map(|n| format!("{n:>width$}")).unwrap_or_else(|| " ".repeat(width))
}

fn decimal_digits(num: usize) -> usize {
    num.checked_ilog10().map_or(1, |digits| digits as usize + 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diff_line_constructors() {
        let ctx = DiffLine::context(10, 10, "SELECT 1;");
        assert_eq!(ctx.line_type, DiffLineType::Context);
        assert_eq!(ctx.old_line_num, Some(10));
        assert_eq!(ctx.new_line_num, Some(10));
        assert_eq!(ctx.content, "SELECT 1;");

        let add = DiffLine::added(11, "+ ADDED COLUMN");
        assert_eq!(add.line_type, DiffLineType::Added);
        assert_eq!(add.old_line_num, None);
        assert_eq!(add.new_line_num, Some(11));

        let rem = DiffLine::removed(12, "- REMOVED COLUMN");
        assert_eq!(rem.line_type, DiffLineType::Removed);
        assert_eq!(rem.old_line_num, Some(12));
        assert_eq!(rem.new_line_num, None);
    }

    #[test]
    fn test_count_diff_changes() {
        let lines = vec![
            DiffLine::context(1, 1, "line 1"),
            DiffLine::added(2, "added line"),
            DiffLine::removed(2, "removed line"),
            DiffLine::added(3, "another add"),
        ];
        let (added, removed) = count_diff_changes(&lines);
        assert_eq!(added, 2);
        assert_eq!(removed, 1);
        assert_eq!(format_diff_stats(added, removed), "+2  -1");
    }

    #[test]
    fn test_diff_line_visual_theming() {
        let theme = DbProTheme::light();
        let add_vis = diff_line_visual(DiffLineType::Added, &theme);
        assert_eq!(add_vis.marker, "+");
        assert_eq!(add_vis.marker_color, theme.success);
        assert_eq!(add_vis.content_color, theme.text_primary);

        let rem_vis = diff_line_visual(DiffLineType::Removed, &theme);
        assert_eq!(rem_vis.marker, "-");
        assert_eq!(rem_vis.marker_color, theme.danger);
        assert_eq!(rem_vis.content_color, theme.text_primary);

        let ctx_vis = diff_line_visual(DiffLineType::Context, &theme);
        assert_eq!(ctx_vis.marker, " ");
        assert_eq!(ctx_vis.bg_color, Color32::TRANSPARENT);
        assert_eq!(ctx_vis.content_color, theme.text_primary);
    }

    #[test]
    fn diff_geometry_expands_line_number_columns_past_four_digits() {
        let lines = [DiffLine::context(1, 12_345, "wide row")];
        let geometry = diff_geometry(&lines);
        assert_eq!(geometry.line_num_chars, 5);
        assert!(geometry.new_num_offset_x > geometry.old_num_offset_x);
        assert!(geometry.content_offset_x > geometry.marker_offset_x);
    }

    #[test]
    fn diff_content_width_reserves_the_longest_line_and_right_padding() {
        assert_eq!(diff_content_width([20.0, 80.0], 86.0), 174.0);
    }

    #[test]
    fn narrow_header_keeps_stats_origin_inside_the_clip_area() {
        assert_eq!(diff_stats_left_offset(200.0, 40.0, 14.0), 146.0);
        assert_eq!(diff_stats_left_offset(10.0, 40.0, 14.0), 0.0);
    }

    #[test]
    fn test_format_line_num_col() {
        assert_eq!(format_line_num_col(Some(5), 3), "  5");
        assert_eq!(format_line_num_col(Some(123), 3), "123");
        assert_eq!(format_line_num_col(Some(12_345), 5), "12345");
        assert_eq!(format_line_num_col(None, 3), "   ");
    }
}
