//! Tests for `../disclosure.rs` (moved out of the source file).
use super::*;

#[test]
fn reduced_motion_disclosure_progress_is_immediate() {
    let mut animated = false;
    assert_eq!(
        disclosure_progress(true, true, || {
            animated = true;
            0.2
        }),
        1.0
    );
    assert!(!animated);
    assert_eq!(disclosure_progress(false, true, || 0.8), 0.0);
    assert_eq!(disclosure_progress(true, false, || 0.2), 0.2);
}
