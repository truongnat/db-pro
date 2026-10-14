//! Tests for `../common_utils.rs` (moved out of the source file).
use super::{format_page_range, truncate_ellipsis};

#[test]
fn compatibility_facade_reexports_common_support() {
    assert_eq!(format_page_range(0, 2, Some(2)), "Rows 1–2 of 2");
    assert_eq!(truncate_ellipsis("abcdef", 4), "abc…");
}
