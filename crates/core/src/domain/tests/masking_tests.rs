//! Tests for `../masking.rs` (moved out of the source file).
use super::*;

#[test]
fn redact_and_partial() {
    let redact = ColumnMask {
        column: "email".into(),
        rule: MaskRule::Redact,
        replacement: String::new(),
        keep_prefix: 0,
        keep_suffix: 0,
    };
    assert_eq!(mask_cell("a@b.com", &redact, false, ""), "***");
    let partial = ColumnMask {
        column: "phone".into(),
        rule: MaskRule::PartialReveal,
        replacement: String::new(),
        keep_prefix: 2,
        keep_suffix: 2,
    };
    assert_eq!(mask_cell("1234567890", &partial, false, ""), "12******90");
}

#[test]
fn keyed_hash_preserves_equality() {
    let rule = ColumnMask {
        column: "id".into(),
        rule: MaskRule::Hash,
        replacement: String::new(),
        keep_prefix: 0,
        keep_suffix: 0,
    };
    let a = mask_cell("42", &rule, true, "secret-key");
    let b = mask_cell("42", &rule, true, "secret-key");
    let c = mask_cell("43", &rule, true, "secret-key");
    assert_eq!(a, b);
    assert_ne!(a, c);
    let d = mask_cell("42", &rule, true, "other-key");
    assert_ne!(a, d);
}

#[test]
fn null_and_unicode() {
    let rule = ColumnMask {
        column: "name".into(),
        rule: MaskRule::PartialReveal,
        replacement: String::new(),
        keep_prefix: 1,
        keep_suffix: 1,
    };
    assert_eq!(mask_cell("NULL", &rule, false, ""), "NULL");
    assert_eq!(mask_cell("東京太郎", &rule, false, ""), "東**郎");
}

#[test]
fn suggestions_are_hints_only() {
    let cols = vec!["id".into(), "email".into(), "created_at".into()];
    assert_eq!(suggest_sensitive_columns(&cols), vec!["email".to_owned()]);
}
