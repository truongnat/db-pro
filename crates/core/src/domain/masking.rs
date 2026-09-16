//! Column masking / anonymization for safe sample export (#253).
//!
//! Preview-only by default. In-place destructive masking is out of scope.

use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MaskRule {
    Redact,
    Hash,
    Fixed,
    PartialReveal,
    PreserveNull,
    Synthetic,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ColumnMask {
    pub column: String,
    pub rule: MaskRule,
    /// Used by Fixed and as salt input for keyed Hash (never log the raw key).
    pub replacement: String,
    /// Characters to keep at start/end for PartialReveal.
    pub keep_prefix: usize,
    pub keep_suffix: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct MaskingProfile {
    pub name: String,
    pub schema: String,
    pub table: String,
    pub columns: Vec<ColumnMask>,
    /// Deterministic keyed hashing for join-preserving IDs.
    pub keyed: bool,
    pub key_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MaskingPreview {
    pub columns: Vec<String>,
    pub original: Vec<Vec<String>>,
    pub masked: Vec<Vec<String>>,
    pub message: String,
}

pub fn suggest_sensitive_columns(column_names: &[String]) -> Vec<String> {
    column_names
        .iter()
        .filter(|name| {
            let n = name.to_ascii_lowercase();
            n.contains("email")
                || n.contains("phone")
                || n.contains("ssn")
                || n.contains("password")
                || n.contains("secret")
                || n.contains("token")
                || n.contains("address")
                || n.contains("dob")
                || n.contains("birth")
        })
        .cloned()
        .collect()
}

pub fn mask_cell(value: &str, rule: &ColumnMask, keyed: bool, key_material: &str) -> String {
    if value.eq_ignore_ascii_case("null") {
        return "NULL".into();
    }
    if value.is_empty() && matches!(rule.rule, MaskRule::PreserveNull) {
        return value.to_owned();
    }
    match rule.rule {
        MaskRule::PreserveNull => {
            if value.eq_ignore_ascii_case("null") {
                "NULL".into()
            } else {
                value.to_owned()
            }
        }
        MaskRule::Redact => "***".into(),
        MaskRule::Fixed => {
            if rule.replacement.is_empty() {
                "[masked]".into()
            } else {
                rule.replacement.clone()
            }
        }
        MaskRule::Hash => {
            let mut hasher = DefaultHasher::new();
            if keyed {
                key_material.hash(&mut hasher);
            }
            value.hash(&mut hasher);
            format!("h{:016x}", hasher.finish())
        }
        MaskRule::PartialReveal => {
            let chars: Vec<char> = value.chars().collect();
            if chars.len() <= rule.keep_prefix + rule.keep_suffix {
                return "*".repeat(chars.len().max(1));
            }
            let prefix: String = chars.iter().take(rule.keep_prefix).collect();
            let suffix: String = chars.iter().rev().take(rule.keep_suffix).rev().collect();
            let middle = chars.len() - rule.keep_prefix - rule.keep_suffix;
            format!("{prefix}{}{suffix}", "*".repeat(middle))
        }
        MaskRule::Synthetic => {
            if value.parse::<i64>().is_ok() {
                "0".into()
            } else if value.contains('@') {
                "user@example.test".into()
            } else {
                "sample".into()
            }
        }
    }
}

pub fn mask_row(headers: &[String], cells: &[String], profile: &MaskingProfile, key_material: &str) -> Vec<String> {
    cells
        .iter()
        .enumerate()
        .map(|(i, cell)| {
            let col = headers.get(i).map(String::as_str).unwrap_or("");
            if let Some(rule) = profile.columns.iter().find(|c| c.column == col) {
                mask_cell(cell, rule, profile.keyed, key_material)
            } else {
                cell.clone()
            }
        })
        .collect()
}

pub fn preview_masking(
    headers: &[String],
    rows: &[Vec<String>],
    profile: &MaskingProfile,
    key_material: &str,
) -> MaskingPreview {
    let masked = rows
        .iter()
        .map(|row| mask_row(headers, row, profile, key_material))
        .collect::<Vec<_>>();
    MaskingPreview {
        columns: headers.to_vec(),
        original: rows.to_vec(),
        masked,
        message: format!(
            "preview {} rows · {} mask rule(s) · keyed={}",
            rows.len(),
            profile.columns.len(),
            profile.keyed
        ),
    }
}

/// Stream-friendly: transform a batch of transfer rows in place conceptually (returns new rows).
pub fn mask_transfer_batch(
    headers: &[String],
    batch: &[crate::domain::transfer::TransferRow],
    profile: &MaskingProfile,
    key_material: &str,
) -> Vec<crate::domain::transfer::TransferRow> {
    batch
        .iter()
        .map(|row| crate::domain::transfer::TransferRow {
            cells: mask_row(headers, &row.cells, profile, key_material),
        })
        .collect()
}

#[cfg(test)]
mod tests {
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
}
