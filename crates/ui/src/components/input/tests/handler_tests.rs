//! Tests for `../handler.rs` (moved out of the source file).
use super::{accessible_label, character_count};

#[test]
fn counts_unicode_characters_for_textarea_feedback() {
    assert_eq!(character_count("café", 8), "4/8");
}

#[test]
fn accessible_label_falls_back_in_expected_order() {
    assert_eq!(accessible_label(Some("name"), Some("Label"), "Hint"), "name");
    assert_eq!(accessible_label(None, Some("Label"), "Hint"), "Label");
    assert_eq!(accessible_label(None, None, "Hint"), "Hint");
}
