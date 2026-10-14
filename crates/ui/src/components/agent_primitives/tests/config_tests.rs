//! Tests for `../config.rs` (moved out of the source file).
use super::*;

#[test]
// This focused assertion makes the local-vs-shared token boundary explicit.
fn local_geometry_constants_keep_the_legacy_chip_contract() {
    assert_eq!(CHIP_HEIGHT, 24.0);
    assert_eq!(CHIP_HORIZONTAL_PADDING, 8.0);
    assert_eq!(CHIP_REMOVABLE_WIDTH, 24.0);
}
