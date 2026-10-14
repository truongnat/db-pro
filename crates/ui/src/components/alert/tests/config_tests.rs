//! Tests for `../config.rs` (moved out of the source file).
use super::*;

#[test]
fn dialog_outer_width_never_exceeds_viewport_budget() {
    let narrow = (280.0 - DIALOG_VIEWPORT_INSET * 2.0).max(0.0);
    assert!(narrow < DIALOG_MAX_WIDTH);
    assert_eq!(DIALOG_PADDING, 20.0);
}
