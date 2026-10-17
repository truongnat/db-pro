//! Tests for `../masking.rs` (moved out of the source file).
use super::*;

fn rule(column: &str, rule: MaskRule) -> ColumnMask {
    ColumnMask {
        column: column.into(),
        rule,
        replacement: String::new(),
        keep_prefix: 0,
        keep_suffix: 0,
    }
}

#[test]
fn redact_and_partial() {
    let redact = rule("email", MaskRule::Redact);
    assert_eq!(mask_cell(Some("a@b.com"), &redact, false, ""), "***");
    let partial = ColumnMask {
        keep_prefix: 2,
        keep_suffix: 2,
        ..rule("phone", MaskRule::PartialReveal)
    };
    assert_eq!(mask_cell(Some("1234567890"), &partial, false, ""), "12******90");
}

#[test]
fn keyed_hash_preserves_equality() {
    let rule = rule("id", MaskRule::Hash);
    let a = mask_cell(Some("42"), &rule, true, "secret-key");
    let b = mask_cell(Some("42"), &rule, true, "secret-key");
    let c = mask_cell(Some("43"), &rule, true, "secret-key");
    assert_eq!(a, b);
    assert_ne!(a, c);
    let d = mask_cell(Some("42"), &rule, true, "other-key");
    assert_ne!(a, d);
}

#[test]
fn literal_null_text_is_masked_like_data() {
    // #502: the string "null" is data, not SQL NULL — it must not bypass rules.
    let redact = rule("secret", MaskRule::Redact);
    assert_eq!(mask_cell(Some("null"), &redact, false, ""), "***");
    assert_eq!(mask_cell(Some("NULL"), &redact, false, ""), "***");

    let preserve = rule("secret", MaskRule::PreserveNull);
    assert_eq!(mask_cell(Some("null"), &preserve, false, ""), "null");
}

#[test]
fn real_null_is_preserved_for_every_rule() {
    // #502: SQL NULL (None) round-trips as NULL regardless of the rule.
    for mask_rule in [
        MaskRule::Redact,
        MaskRule::Hash,
        MaskRule::Fixed,
        MaskRule::PartialReveal,
        MaskRule::PreserveNull,
        MaskRule::Synthetic,
    ] {
        let rule = rule("anything", mask_rule);
        assert_eq!(mask_cell(None, &rule, true, "key"), "NULL", "{mask_rule:?}");
    }
}

#[test]
fn null_and_unicode() {
    let rule = ColumnMask {
        keep_prefix: 1,
        keep_suffix: 1,
        ..rule("name", MaskRule::PartialReveal)
    };
    assert_eq!(mask_cell(None, &rule, false, ""), "NULL");
    assert_eq!(mask_cell(Some("東京太郎"), &rule, false, ""), "東**郎");
}

#[test]
fn row_rule_matching_is_case_insensitive() {
    // #503: a DB header "Email" must match a profile rule for "email".
    let headers = vec!["Id".into(), "Email".into()];
    let profile = MaskingProfile {
        columns: vec![rule("email", MaskRule::Redact)],
        ..Default::default()
    };
    let cells = vec![Some("7"), Some("ada@example.com")];
    assert_eq!(mask_row(&headers, &cells, &profile, ""), vec!["7", "***"]);
}

#[test]
fn row_passes_real_null_through() {
    let headers = vec!["id".into(), "email".into()];
    let profile = MaskingProfile {
        columns: vec![rule("email", MaskRule::Redact)],
        ..Default::default()
    };
    let cells = vec![Some("7"), None];
    assert_eq!(mask_row(&headers, &cells, &profile, ""), vec!["7", "NULL"]);
}

#[test]
fn unmatched_rules_are_surfaced() {
    // #503: rules that match no header must not be silent.
    let headers = vec!["id".into(), "email".into()];
    let profile = MaskingProfile {
        columns: vec![rule("ssn", MaskRule::Redact), rule("Email", MaskRule::Fixed)],
        ..Default::default()
    };
    assert_eq!(unmatched_mask_rules(&headers, &profile), vec!["ssn".to_owned()]);

    let rows = vec![vec!["1".into(), "ada@example.com".into()]];
    let preview = preview_masking(&headers, &rows, &profile, "");
    assert_eq!(preview.unmatched_rules, vec!["ssn".to_owned()]);
    assert!(preview.message.contains("ssn"));
    // The matching rule still applied despite the case difference.
    assert_eq!(preview.masked[0][1], "[masked]");
}

#[test]
fn suggestions_are_hints_only() {
    let cols = vec!["id".into(), "email".into(), "created_at".into()];
    assert_eq!(suggest_sensitive_columns(&cols), vec!["email".to_owned()]);
}
