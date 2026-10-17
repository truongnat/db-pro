//! Tests for `../search.rs` (moved out of the source file).
use super::*;

#[test]
fn matches_ascii_case_insensitively() {
    assert_eq!(case_insensitive_matches("Select ID, id", "id"), &[(7, 9), (11, 13)]);
    assert_eq!(case_insensitive_matches("SELECT", "select"), &[(0, 6)]);
    assert!(case_insensitive_matches("SELECT", "from").is_empty());
}

#[test]
fn empty_or_long_queries_match_nothing() {
    assert!(case_insensitive_matches("abc", "").is_empty());
    assert!(case_insensitive_matches("", "a").is_empty());
    assert!(case_insensitive_matches("ab", "abc").is_empty());
}

#[test]
fn matches_are_non_overlapping() {
    assert_eq!(case_insensitive_matches("aaa", "aa"), &[(0, 2)]);
}

#[test]
fn ranges_are_original_byte_offsets_after_expanding_lowercase() {
    // Issue #509: 'İ' (U+0130) lowercases to "i\u{307}" — 2 bytes become 4,
    // so matching a lowercased copy produced ranges shifted off the original
    // text (and off char boundaries). The original text keeps 'İ' at 2 bytes.
    let text = "x İzmir y";
    let m = case_insensitive_matches(text, "zmir");
    assert_eq!(m.len(), 1);
    // The range must slice cleanly and equal "zmir" — in the lowered copy
    // 'İ' occupies 4 bytes and shifted every later offset by +2.
    assert_eq!(&text[m[0].0..m[0].1], "zmir");
}

#[test]
fn expansion_char_does_not_match_its_expansion() {
    // Documented per-char contract: 'İ' compares as "i\u{307}" — never equal
    // to a single query 'i', nor to a two-char "i\u{307}" query (1:1 pairing).
    // Text without a literal 'i' so 'İ' is the only candidate.
    assert!(case_insensitive_matches("x \u{0130}zmr", "i").is_empty());
    assert!(case_insensitive_matches("x \u{0130}zmr", "i\u{307}").is_empty());
    // It still matches itself.
    assert_eq!(case_insensitive_matches("x \u{0130}zmr", "\u{0130}"), &[(2, 4)]);
}

#[test]
fn shrinking_uppercase_keeps_offsets() {
    // Kelvin sign K (U+212A, 3 bytes) lowercases to 'k' (1 byte) — matching a
    // lowercased copy would place following matches 2 bytes too early.
    let text = "K x k";
    let m = case_insensitive_matches(text, "k");
    assert_eq!(m, &[(0, 1), (4, 5)]);
    assert_eq!(&text[m[0].0..m[0].1], "K");
    assert_eq!(&text[m[1].0..m[1].1], "k");
}

#[test]
fn range_end_never_splits_a_multibyte_char() {
    let text = "a\u{00E9}b\u{00E9}c"; // café bytes, é = 2 bytes
    for (s, e) in case_insensitive_matches(text, "\u{00E9}") {
        assert!(text.is_char_boundary(s) && text.is_char_boundary(e));
        assert_eq!(&text[s..e], "\u{00E9}");
    }
}
