//! Tests for `../masking.rs` (moved out of the source file).
use super::*;

fn rule(name: &str, mask_rule: MaskRule, replacement: &str, keep_prefix: usize, keep_suffix: usize) -> ColumnMask {
    ColumnMask {
        column: name.into(),
        rule: mask_rule,
        replacement: replacement.into(),
        keep_prefix,
        keep_suffix,
    }
}

fn profile(column: &str, mask_rule: MaskRule, keyed: bool) -> MaskingProfile {
    MaskingProfile {
        name: "test".into(),
        schema: String::new(),
        table: String::new(),
        columns: vec![rule(column, mask_rule, "[masked]", 1, 1)],
        keyed,
        key_id: "k".into(),
    }
}

#[test]
fn redact_and_partial() {
    let redact = rule("email", MaskRule::Redact, "", 0, 0);
    assert_eq!(mask_cell(Some("a@b.com"), &redact, false, ""), "***");
    let partial = rule("phone", MaskRule::PartialReveal, "", 2, 2);
    assert_eq!(mask_cell(Some("1234567890"), &partial, false, ""), "12******90");
}

#[test]
fn keyed_hash_preserves_equality() {
    let rule = rule("id", MaskRule::Hash, "", 0, 0);
    let a = mask_cell(Some("42"), &rule, true, "secret-key");
    let b = mask_cell(Some("42"), &rule, true, "secret-key");
    let c = mask_cell(Some("43"), &rule, true, "secret-key");
    assert_eq!(a, b);
    assert_ne!(a, c);
    let d = mask_cell(Some("42"), &rule, true, "other-key");
    assert_ne!(a, d);
}

#[test]
fn unicode_partial_reveal() {
    let rule = rule("name", MaskRule::PartialReveal, "", 1, 1);
    assert_eq!(mask_cell(Some("東京太郎"), &rule, false, ""), "東**郎");
}

// #502 — a present text value that looks like NULL must still be masked.
#[test]
fn sql_null_is_rendered_not_masked() {
    for mask_rule in [
        MaskRule::Redact,
        MaskRule::Hash,
        MaskRule::Fixed,
        MaskRule::PartialReveal,
        MaskRule::PreserveNull,
        MaskRule::Synthetic,
    ] {
        let rule = rule("email", mask_rule, "[masked]", 2, 2);
        assert_eq!(mask_cell(None, &rule, false, ""), "NULL", "{mask_rule:?}");
    }
}

#[test]
fn literal_null_text_is_masked_by_every_rule() {
    assert_eq!(
        mask_cell(Some("null"), &rule("email", MaskRule::Redact, "", 0, 0), false, ""),
        "***"
    );
    assert_eq!(
        mask_cell(Some("NULL"), &rule("email", MaskRule::Redact, "", 0, 0), false, ""),
        "***"
    );
    assert_eq!(
        mask_cell(Some(""), &rule("email", MaskRule::Redact, "", 0, 0), false, ""),
        "***"
    );

    let fixed = rule("email", MaskRule::Fixed, "[masked]", 0, 0);
    assert_eq!(mask_cell(Some("null"), &fixed, false, ""), "[masked]");
    assert_eq!(mask_cell(Some("NULL"), &fixed, false, ""), "[masked]");

    let hashed = mask_cell(Some("null"), &rule("id", MaskRule::Hash, "", 0, 0), true, "secret-key");
    assert_ne!(hashed, "NULL");
    assert!(hashed.starts_with('h'), "{hashed}");

    let synthetic = rule("email", MaskRule::Synthetic, "", 0, 0);
    assert_eq!(mask_cell(Some("null"), &synthetic, false, ""), "sample");

    let partial = rule("email", MaskRule::PartialReveal, "", 1, 1);
    assert_eq!(mask_cell(Some("null"), &partial, false, ""), "n**l");
    assert_eq!(mask_cell(Some("NULL"), &partial, false, ""), "N**L");
}

#[test]
fn preserve_null_keeps_text_and_renders_nulls() {
    let preserve = rule("email", MaskRule::PreserveNull, "", 0, 0);
    assert_eq!(mask_cell(Some("null"), &preserve, false, ""), "null");
    assert_eq!(mask_cell(Some("NULL"), &preserve, false, ""), "NULL");
    assert_eq!(mask_cell(Some(""), &preserve, false, ""), "");
    assert_eq!(
        mask_cell(Some("ada@example.com"), &preserve, false, ""),
        "ada@example.com"
    );
    assert_eq!(mask_cell(None, &preserve, false, ""), "NULL");
}

#[test]
fn mask_row_distinguishes_sql_null_from_text_null() {
    let headers = vec!["email".into(), "note".into()];
    let profile = profile("email", MaskRule::Redact, false);

    let masked = mask_row(
        &headers,
        &[Some("null".to_owned()), Some("keep".to_owned())],
        &profile,
        "",
    );
    assert_eq!(masked, vec!["***", "keep"]);

    let masked = mask_row(&headers, &[None, Some("keep".to_owned())], &profile, "");
    assert_eq!(masked, vec!["NULL", "keep"]);

    let masked = mask_row(&headers, &[None, None], &profile, "");
    assert_eq!(masked, vec!["NULL", "NULL"]);
}

#[test]
fn preview_masking_masks_textual_null() {
    let headers = vec!["email".into()];
    let profile = profile("email", MaskRule::Redact, false);
    let rows = vec![vec![Some("null".to_owned())], vec![None], vec![Some("".to_owned())]];
    let preview = preview_masking(&headers, &rows, &profile, "");
    assert_eq!(preview.masked, vec![vec!["***"], vec!["NULL"], vec!["***"]]);
    assert_eq!(preview.original, vec![vec!["null"], vec!["NULL"], vec![""]]);
}

#[test]
fn transfer_batch_masks_textual_null_cells() {
    use crate::domain::transfer::TransferRow;
    let headers = vec!["email".into(), "note".into()];
    let profile = profile("email", MaskRule::Redact, false);
    let batch = vec![
        TransferRow {
            cells: vec!["null".into(), "keep".into()],
        },
        TransferRow {
            cells: vec!["NULL".into(), "".into()],
        },
    ];
    let masked = mask_transfer_batch(&headers, &batch, &profile, "");
    assert_eq!(masked[0].cells, vec!["***", "keep"]);
    assert_eq!(masked[1].cells, vec!["***", ""]);
}

// #503 — a rule whose column name matches no header must not silently pass.
#[test]
fn unmatched_rule_columns_are_case_sensitive() {
    let profile = profile("email", MaskRule::Redact, false);

    let exact = vec!["email".into(), "id".into()];
    assert!(unmatched_rule_columns(&exact, &profile).is_empty());

    let different_case = vec!["Email".into(), "id".into()];
    assert_eq!(unmatched_rule_columns(&different_case, &profile), vec!["email"]);

    let no_columns = Vec::new();
    assert_eq!(unmatched_rule_columns(&no_columns, &profile), vec!["email"]);
}

#[test]
fn preview_masking_warns_on_unmatched_rule() {
    let profile = profile("email", MaskRule::Redact, false);
    let rows = vec![vec![Some("Email".to_owned()), Some("1".to_owned())]];

    let mismatched = preview_masking(&vec!["Email".into(), "id".into()], &rows, &profile, "");
    assert!(
        mismatched.message.contains("WARNING: 1 rule(s) matched no column"),
        "{}",
        mismatched.message
    );
    assert!(mismatched.message.contains("email"), "{}", mismatched.message);
    // The unmatched rule is not applied — the warning is what makes it visible.
    assert_eq!(mismatched.masked[0][0], "Email");

    let matched = preview_masking(&vec!["email".into(), "id".into()], &rows, &profile, "");
    assert!(!matched.message.contains("WARNING"), "{}", matched.message);
    assert_eq!(matched.masked[0][0], "***");
}

#[test]
fn suggestions_are_hints_only() {
    let cols = vec!["id".into(), "email".into(), "created_at".into()];
    assert_eq!(suggest_sensitive_columns(&cols), vec!["email".to_owned()]);
}
