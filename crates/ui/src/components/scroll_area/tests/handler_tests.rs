//! Tests for `../handler.rs` (moved out of the source file).
use super::*;

#[test]
fn axis_order_matches_egui_scroll_area() {
    assert_eq!(scroll_axes(false, true), [false, true]);
    assert_eq!(scroll_axes(true, false), [true, false]);
    assert_eq!(scroll_axes(true, true), [true, true]);
}
