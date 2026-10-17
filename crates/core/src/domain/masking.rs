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
    /// Profile rules that matched no column header — they mask nothing (#503).
    #[serde(default)]
    pub unmatched_rules: Vec<String>,
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

// cc-scan:allow LONG_FUNCTION — linear pipeline — one cohesive pass
pub fn mask_cell(value: Option<&str>, rule: &ColumnMask, keyed: bool, key_material: &str) -> String {
    // SQL NULL is preserved verbatim: masking must never rewrite a real NULL
    // into masked-looking content. Only `None` is NULL — the literal text
    // "null" is ordinary data and gets masked like any other value (#502).
    let Some(value) = value else {
        return "NULL".into();
    };
    match rule.rule {
        MaskRule::PreserveNull => value.to_owned(),
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

/// Profile rules whose column name matches no header (case-insensitive).
/// A rule that matches nothing masks nothing — surfacing it keeps a typo'd or
/// mis-cased column name from silently leaving sensitive data raw (#503).
pub fn unmatched_mask_rules(headers: &[String], profile: &MaskingProfile) -> Vec<String> {
    profile
        .columns
        .iter()
        .filter(|rule| !headers.iter().any(|header| header.eq_ignore_ascii_case(&rule.column)))
        .map(|rule| rule.column.clone())
        .collect()
}

pub fn mask_row(
    headers: &[String],
    cells: &[Option<&str>],
    profile: &MaskingProfile,
    key_material: &str,
) -> Vec<String> {
    cells
        .iter()
        .enumerate()
        .map(|(i, cell)| {
            let col = headers.get(i).map(String::as_str).unwrap_or("");
            if let Some(rule) = profile.columns.iter().find(|c| c.column.eq_ignore_ascii_case(col)) {
                mask_cell(*cell, rule, profile.keyed, key_material)
            } else {
                cell.map(str::to_owned).unwrap_or_else(|| "NULL".into())
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
        .map(|row| {
            let cells = row.iter().map(|cell| Some(cell.as_str())).collect::<Vec<_>>();
            mask_row(headers, &cells, profile, key_material)
        })
        .collect::<Vec<_>>();
    let unmatched_rules = unmatched_mask_rules(headers, profile);
    let mut message = format!(
        "preview {} rows · {} mask rule(s) · keyed={}",
        rows.len(),
        profile.columns.len(),
        profile.keyed
    );
    if !unmatched_rules.is_empty() {
        message.push_str(&format!(
            " · {} rule(s) match no column: {}",
            unmatched_rules.len(),
            unmatched_rules.join(", ")
        ));
    }
    MaskingPreview {
        columns: headers.to_vec(),
        original: rows.to_vec(),
        masked,
        unmatched_rules,
        message,
    }
}

/// Stream-friendly: transform a batch of transfer rows in place conceptually (returns new rows).
pub fn mask_transfer_batch(
    headers: &[String],
    batch: &[crate::domain::transfer::TransferRow],
    profile: &MaskingProfile,
    key_material: &str,
) -> Vec<crate::domain::transfer::TransferRow> {
    let unmatched = unmatched_mask_rules(headers, profile);
    if !unmatched.is_empty() {
        tracing::warn!(
            table = %profile.table,
            unmatched_rules = %unmatched.join(", "),
            "masking: profile rules match no column header; those columns export raw"
        );
    }
    batch
        .iter()
        .map(|row| {
            let cells = row.cells.iter().map(|cell| Some(cell.as_str())).collect::<Vec<_>>();
            crate::domain::transfer::TransferRow {
                cells: mask_row(headers, &cells, profile, key_material),
            }
        })
        .collect()
}

#[cfg(test)]
#[path = "tests/masking_tests.rs"]
mod tests;
