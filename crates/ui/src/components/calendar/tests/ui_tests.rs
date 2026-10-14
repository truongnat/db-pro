//! Tests for `../ui.rs` (moved out of the source file).
use super::*;

#[test]
fn calendar_frame_size_is_intrinsic_and_includes_symmetric_margin() {
    assert_eq!(calendar_content_width(), 248.0);
    assert_eq!(calendar_frame_size(false), Vec2::new(272.0, 292.0));
}

#[test]
fn calendar_layout_shrinks_cells_when_the_gallery_column_is_narrow() {
    let layout = calendar_layout(236.0);

    assert!(layout.cell_size < config::CELL_SIZE);
    assert!(layout.frame_width <= 236.0);
    assert_eq!(
        layout.content_width,
        (layout.cell_size * 7.0) + (config::CELL_GAP * 6.0)
    );
}

#[test]
fn calendar_layout_keeps_intrinsic_cells_when_space_is_available() {
    assert_eq!(
        calendar_layout(400.0),
        CalendarLayout {
            cell_size: config::CELL_SIZE,
            content_width: calendar_content_width(),
            frame_width: calendar_frame_size(false).x,
        }
    );
}
