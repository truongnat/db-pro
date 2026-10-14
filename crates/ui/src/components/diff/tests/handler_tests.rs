//! Tests for `../handler.rs` (moved out of the source file).
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
